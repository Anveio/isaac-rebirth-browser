//! Gluttony49.0: ai_vis_gluttony4c91e0 on the plain NPC base. It walks the
//! grid with NPCAI_Pathfinder, spits a FireProjectiles mode8 burst when the
//! player is near and fires an axis-aligned laser when lined up.
use crate::{
    content,
    enemy_projectiles::ProjectileBirth,
    entities::{Birth, Family, NpcWorld, SpawnContext},
    lasers::LaserBirth,
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    pathfinder::{self, Pathfinder},
    phase::Phase,
    plain_npc,
    room_spawns::{SpawnCategory, SpawnSelector},
};
use serde::{Deserialize, Serialize};

pub(crate) const FAMILY: Family = Family {
    kinds: &[49],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GluttonyState {
    /// Entity_NPC+f58 state frame counter.
    pub counter: i32,
    /// Entity_NPC+f78 axis-aligned laser direction.
    pub direction: WorldVector,
}

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    // Super Gluttony49.1 adds extra shots to both attacks; not yet admitted.
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 49
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Gluttony requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(49, 0);
    let mut npc = Npc {
        // Config47f573 SetFrame(default, 0) leaves the default clip stopped.
        animation: "WalkVert".into(),
        actor: config.actor.into(),
        pathfinder: Some(Pathfinder::new()),
        gluttony: Some(GluttonyState {
            counter: 0,
            direction: WorldVector { x: 0.0, y: 0.0 },
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
    plain_npc::advance(npc, phase, random, world, births, ai)
}

fn length(vector: WorldVector) -> f32 {
    (vector.x * vector.x + vector.y * vector.y).sqrt()
}

/// 42e930: the dominant-axis unit vector.
pub(crate) fn axis(vector: WorldVector) -> WorldVector {
    if vector.x.abs() > vector.y.abs() {
        WorldVector {
            x: if 0.0 > vector.x { -1.0 } else { 1.0 },
            y: 0.0,
        }
    } else {
        WorldVector {
            x: 0.0,
            y: if 0.0 > vector.y { -1.0 } else { 1.0 },
        }
    }
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.state == 0 {
        npc.state = 4;
        return Ok(());
    }
    // 4c93a8 marks Gluttony's cell before every state.
    let index = world
        .grid
        .index(npc.position)
        .ok_or("Gluttony stands outside the room grid")?;
    world
        .path_costs
        .as_deref_mut()
        .ok_or("Gluttony requires source Room.Init path costs")?
        .store(index, 900)?;
    let delta = WorldVector {
        x: world.player.x - npc.position.x,
        y: world.player.y - npc.position.y,
    };
    let mut state = npc
        .gluttony
        .take()
        .ok_or("Gluttony requires its AI state")?;
    let result = step(npc, &mut state, random, world, births, friction, delta);
    npc.gluttony = Some(state);
    result
}

fn step(
    npc: &mut Npc,
    state: &mut GluttonyState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
    delta: WorldVector,
) -> Result<(), String> {
    match npc.state {
        4 => {
            plain_npc::walk_frame(npc, "WalkHori", "WalkVert", 0.1);
            pathfinder::move_randomly(npc, random, world, 0.7, *friction)?;
            if plain_npc::on_frame(npc, world, 3)? {
                let lazy = random.bounded(10);
                let spit = random.bounded(7) == 0;
                // 4c9f1e: a near player draws the mode8 burst.
                let near = spit && random.uniform() * 130.0 + 70.0 > length(delta);
                if near {
                    npc.state = 8;
                    state.counter = 0;
                } else if (spit || lazy == 0) && lined_up(npc, delta) {
                    state.direction = axis(delta);
                    npc.state = 9;
                    state.counter = 1;
                    *friction *= 0.8;
                    return Ok(());
                }
            }
        }
        8 => {
            plain_npc::play(npc, "FatAttack", false);
            if state.counter == 14 {
                spit_burst(npc, random, births)?;
            } else if state.counter > 44 {
                npc.state = 4;
            }
        }
        9 => {
            if state.counter == 1 {
                // 4c95fe: Attack%02u{Horiz,Up,Down} numbered from variant+1.
                let clip = if state.direction.x.abs() > 0.0 {
                    npc.flip_x = 0.0 > state.direction.x;
                    "Attack01Horiz"
                } else if 0.0 > state.direction.y {
                    "Attack01Up"
                } else {
                    "Attack01Down"
                };
                plain_npc::play(npc, clip, false);
            }
            if state.counter == 30 {
                fire_laser(npc, random, state, births)?;
            } else if state.counter > 79 {
                npc.state = 4;
            }
        }
        // 4c9408: any other state only runs the tail.
        _ => {}
    }
    state.counter += 1;
    *friction *= 0.8;
    Ok(())
}

/// 4c97ce: within 20 units on either axis, closer than 400, and on screen
/// (42c5c0 always holds inside a fixed-camera 1x1 room).
fn lined_up(_: &Npc, delta: WorldVector) -> bool {
    (delta.x.abs() < 20.0 || delta.y.abs() < 20.0) && length(delta) < 400.0
}

/// 4c9a71: default ProjectileParams draw their angle, then FireProjectiles
/// mode8 (4a04b0) spawns eight shots around Gluttony. Modes 6..9 skip the
/// mode0 falling overrides, so each keeps its type9.Init falling speed.
fn spit_burst(npc: &mut Npc, random: &mut Mt19937, births: &mut Vec<Birth>) -> Result<(), String> {
    let parameter_angle = (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
    let straight = 8.0_f32;
    let diagonal = straight * 0.707_106_77;
    let velocities = [
        (straight, 0.0),
        (-straight, 0.0),
        (0.0, straight),
        (0.0, -straight),
        (diagonal, diagonal),
        (-diagonal, diagonal),
        (diagonal, -diagonal),
        (-diagonal, -diagonal),
    ];
    for (x, y) in velocities {
        let spawn_seed = random.next_u32();
        let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
        births.push(Birth::Projectile(ProjectileBirth {
            position: npc.position,
            velocity: WorldVector { x, y },
            spawn_seed,
            initial_falling_speed,
            parameter_angle,
            falling_speed: initial_falling_speed,
            falling_acceleration: 0.0,
            scale: 1.0,
            height: -23.0,
        }));
    }
    Ok(())
}

/// 4c94e0: ShootAngle476120(variant1, pos + (6*dx, 0), angle, timeout32,
/// offset (0,-20)). The laser seed is drawn here; Game admits the beam and
/// runs its spawn-frame Update before the next actor.
fn fire_laser(
    npc: &mut Npc,
    random: &mut Mt19937,
    state: &GluttonyState,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    let heading = state.direction;
    let depth = if heading.y < -0.5 { -10.0 } else { 3000.0 };
    let angle =
        (crate::fmath::atan2(f64::from(heading.y), f64::from(heading.x)) as f32) * 57.295_776;
    let spawn_seed = random.next_u32();
    births.push(Birth::Laser(LaserBirth {
        variant: 1,
        parent_position: npc.position,
        position: npc.position
            + WorldVector {
                x: 6.0 * heading.x,
                y: 0.0,
            },
        angle,
        timeout: 32,
        position_offset: WorldVector { x: 0.0, y: -20.0 },
        depth,
        spawn_seed,
    }));
    Ok(())
}
