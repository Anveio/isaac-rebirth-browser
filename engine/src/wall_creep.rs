//! Wall Creep 240.0: ai 4b5980's type-240 path (shared with Rage Creep 241
//! and Blind Creep 242, not admitted). A Wall Creep snaps to the nearest
//! wall of the room rectangle, crawls along it toward the player's line at
//! speed 5 and, once its countdown runs out, stops to fire three shots off
//! the wall on Attack frames 10, 13 and 16. The AI owns its velocity and
//! grid class 1/2 only clamps it along its wall.
use crate::{
    audio::SoundRequest,
    content,
    enemy_projectiles::ProjectileBirth,
    entities::{Birth, Family, NpcWorld, SpawnContext},
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    phase::Phase,
    plain_npc,
    room_spawns::{SpawnCategory, SpawnSelector},
};
use serde::{Deserialize, Serialize};

pub(crate) const FAMILY: Family = Family {
    kinds: &[240],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WallCreepState {
    /// +f88: 1 top, 2 bottom, 3 left, 4 right (0 before the first update).
    pub wall: i32,
    /// +f78: the crawl vector, ±5 along the wall.
    pub crawl: WorldVector,
    /// +f58: updates until the next attack check.
    pub countdown: i32,
    /// Sprite rotation +2cc in degrees.
    pub rotation: f32,
    /// Sprite +2fd, the vertical flip on a side wall.
    pub flip_y: bool,
}

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 240
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Wall Creep requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(240, 0);
    let mut npc = Npc {
        // Config47f573 SetFrame(default "Walk", 0) leaves the clip stopped.
        animation: "Walk".into(),
        actor: config.actor.into(),
        wall_creep: Some(WallCreepState {
            wall: 0,
            crawl: WorldVector { x: 0.0, y: 0.0 },
            countdown: 0,
            rotation: 0.0,
            flip_y: false,
        }),
        ..Npc::authored(id, config, selector)
    };
    npc.max_hp = content::initial_hp(config, context.stage);
    npc.hp = npc.max_hp;
    Ok(Some(npc))
}

fn advance(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    plain_npc::advance_dead_ai(npc, phase, random, world, births, ai)
}

/// The room rectangle (Room+1c..+28) inset by 5: left, top, right, bottom.
fn inset(world: &NpcWorld) -> (f32, f32, f32, f32) {
    let low = world.grid.clamp_room(
        WorldVector {
            x: f32::MIN,
            y: f32::MIN,
        },
        5.0,
    );
    let high = world.grid.clamp_room(
        WorldVector {
            x: f32::MAX,
            y: f32::MAX,
        },
        5.0,
    );
    (low.x, low.y, high.x, high.y)
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    _: &mut f32,
) -> Result<(), String> {
    let mut state = npc
        .wall_creep
        .take()
        .ok_or("Wall Creep requires its AI state")?;
    let result = step(npc, &mut state, random, world, births);
    npc.wall_creep = Some(state);
    result
}

fn step(
    npc: &mut Npc,
    state: &mut WallCreepState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    // 4b5980: every update starts at rest; the AI sets the whole velocity.
    npc.velocity = WorldVector { x: 0.0, y: 0.0 };
    let (left, top, right, bottom) = inset(world);
    match npc.state {
        0 => {
            // 4b5a82: the nearest wall; strict comparisons keep the earlier
            // of left, right, top, bottom on a tie.
            let position = npc.position;
            let mut best = 10_000.0_f32;
            let mut target = position;
            let walls = [
                ((left - position.x).abs(), 3, 2, -90.0, (0.0, 5.0)),
                ((right - position.x).abs(), 4, 2, 90.0, (0.0, 5.0)),
                ((top - position.y).abs(), 1, 1, 0.0, (5.0, 0.0)),
                ((bottom - position.y).abs(), 2, 1, 180.0, (5.0, 0.0)),
            ];
            for (distance, wall, class, rotation, (x, y)) in walls {
                if best > distance {
                    best = distance;
                    target = match wall {
                        3 => WorldVector {
                            x: left,
                            ..position
                        },
                        4 => WorldVector {
                            x: right,
                            ..position
                        },
                        1 => WorldVector { y: top, ..position },
                        _ => WorldVector {
                            y: bottom,
                            ..position
                        },
                    };
                    state.wall = wall;
                    npc.grid_collision_class = class;
                    state.rotation = rotation;
                    state.crawl = WorldVector { x, y };
                }
            }
            npc.position = target;
            plain_npc::play(npc, "Walk", false);
            npc.state = 4;
        }
        4 => crawl(npc, state, random, world, (left, top, right, bottom)),
        8 => {
            let frame = npc.animation_clock.floor() as i32;
            if matches!(frame, 10 | 13 | 16) {
                shoot(npc, state, random, world, births);
            }
            if frame == 10 {
                world.audio.admit(
                    random,
                    world.system_counter,
                    SoundRequest {
                        id: 317,
                        volume: 1.0,
                        delay: 2,
                        looping: false,
                        pitch: 1.0,
                    },
                );
            }
            if plain_npc::finished(npc) {
                npc.state = 4;
            }
        }
        _ => {}
    }
    Ok(())
}

/// State 4 (4b5ce0): snap onto the wall line, turn toward the player's line
/// and crawl unless within 10 px of it; attack once the countdown is spent.
fn crawl(
    npc: &mut Npc,
    state: &mut WallCreepState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    (left, top, right, bottom): (f32, f32, f32, f32),
) {
    match state.wall {
        1 => npc.position.y = top,
        2 => npc.position.y = bottom,
        3 => npc.position.x = left,
        4 => npc.position.x = right,
        _ => {}
    }
    // PORT GAP: a feared creep chases the point 50 px away from the player
    // instead; the port admits no fear (0x800) or confusion (0x200).
    let target = world.player;
    if state.wall <= 2 {
        if npc.position.x > target.x && state.crawl.x > 0.0
            || target.x > npc.position.x && 0.0 > state.crawl.x
        {
            state.crawl.x = -state.crawl.x;
        }
        if (target.x - npc.position.x).abs() > 10.0 {
            npc.velocity = state.crawl;
        }
        npc.flip_x = if state.wall == 1 {
            state.crawl.x > 0.0
        } else {
            0.0 > state.crawl.x
        };
    } else {
        if npc.position.y > target.y && state.crawl.y > 0.0
            || target.y > npc.position.y && 0.0 > state.crawl.y
        {
            state.crawl.y = -state.crawl.y;
        }
        if (target.y - npc.position.y).abs() > 10.0 {
            npc.velocity = state.crawl;
        }
        state.flip_y = if state.wall == 3 {
            state.crawl.y > 0.0
        } else {
            0.0 > state.crawl.y
        };
    }
    let speed = npc.velocity.x * npc.velocity.x + npc.velocity.y * npc.velocity.y;
    if speed > 1e-4 {
        plain_npc::play(npc, "Walk", false);
    } else {
        plain_npc::set_frame(npc, "Walk", 0);
    }
    let countdown = state.countdown;
    state.countdown = countdown.wrapping_sub(1);
    // PORT GAP: Camera::IsPositionOnScreen 42c5c0 is taken as true, which
    // holds in a fixed-camera 1x1 room; larger rooms can scroll it off.
    if countdown >= 0 {
        return;
    }
    npc.state = 8;
    plain_npc::play(npc, "Attack", false);
    state.countdown = random.bounded(45) as i32 + 15;
}

/// 4b6161: default ProjectileParams draw their angle, then FireProjectiles
/// mode 0 fires one shot off the wall from the room-clamped offset point.
fn shoot(
    npc: &Npc,
    state: &WallCreepState,
    random: &mut Mt19937,
    world: &NpcWorld,
    births: &mut Vec<Birth>,
) {
    let ((vx, vy), (ox, oy)) = match state.wall {
        1 => ((0.0, 8.0), (0.0, 4.0)),
        2 => ((0.0, -8.0), (0.0, -12.0)),
        3 => ((8.0, 0.0), (0.0, 23.0)),
        4 => ((-8.0, 0.0), (4.0, 23.0)),
        _ => ((0.0, 0.0), (0.0, 0.0)),
    };
    let parameter_angle = (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
    let position = world
        .grid
        .clamp_room(npc.position + WorldVector { x: ox, y: oy }, 5.0);
    let spawn_seed = random.next_u32();
    let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
    births.push(Birth::Projectile(ProjectileBirth {
        position,
        velocity: WorldVector { x: vx, y: vy },
        spawn_seed,
        initial_falling_speed,
        parameter_angle,
        falling_speed: -3.0,
        falling_acceleration: 0.1,
        scale: 1.0,
        height: -23.0,
    }));
}
