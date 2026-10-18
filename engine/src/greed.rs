//! Greed 50.0 and Super Greed 50.1: ai 4c85f0. Greed wanders on the
//! pathfinder and, on a 1-in-50 roll, shoots a fan along the player's axis
//! or summons two Hoppers (Super Greed: two Keepers) while at most one other
//! countable enemy lives. The AI never draws the NPC's own RNG.
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
    sprite_color::Color,
};
use serde::{Deserialize, Serialize};

pub(crate) const FAMILY: Family = Family {
    kinds: &[50],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::Items,
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GreedState {
    /// +f58.
    pub counter: i32,
    /// +f78: the shot velocity, frozen at counter 1 of SHOOT.
    pub aim: WorldVector,
}

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 50
        || selector.variant > 1
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Greed requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(50, selector.variant.into());
    let mut npc = Npc {
        animation: "WalkVert".into(),
        actor: config.actor.into(),
        pathfinder: Some(crate::pathfinder::Pathfinder::new()),
        greed: Some(GreedState {
            counter: 0,
            aim: WorldVector { x: 0.0, y: 0.0 },
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

fn state(npc: &mut Npc) -> &mut GreedState {
    npc.greed.as_mut().expect("Greed owns its state")
}

/// 42e930: the dominant axis's unit vector; a tie goes vertical, 0 counts +.
fn axis(vector: WorldVector) -> WorldVector {
    let sign = |value: f32| if value >= 0.0 { 1.0 } else { -1.0 };
    if vector.x.abs() > vector.y.abs() {
        WorldVector {
            x: sign(vector.x),
            y: 0.0,
        }
    } else {
        WorldVector {
            x: 0.0,
            y: sign(vector.y),
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
        // INIT 4c86c0: brownish blood, no friction and no counter step.
        npc.state = 4;
        npc.splat_color = Color {
            tint: [0.4, 0.2, 0.2, 1.0],
            ..Color::WHITE
        };
        return Ok(());
    }
    *friction *= 0.75;
    let delta = WorldVector {
        x: world.player.x - npc.position.x,
        y: world.player.y - npc.position.y,
    };
    let counter = state(npc).counter;
    match npc.state {
        4 => {
            plain_npc::walk_frame(npc, "WalkHori", "WalkVert", 0.1);
            crate::pathfinder::move_randomly(npc, random, world, 1.0, *friction)?;
            if random.bounded(50) == 0 {
                state(npc).counter = 0;
                npc.state = if random.bounded(3) != 0 || world.countable_total > 2 {
                    8
                } else {
                    12
                };
            }
        }
        8 => match counter {
            1 => {
                npc.flip_x = false;
                // 4448c0's direction index picks the clip.
                let clip = if delta.x.abs() > delta.y.abs() {
                    npc.flip_x = delta.x < 0.0;
                    "Attack01Hori"
                } else if delta.y >= 0.0 {
                    "Attack01Down"
                } else {
                    "Attack01Up"
                };
                plain_npc::play(npc, clip, true);
                state(npc).aim = axis(delta) * 10.0;
            }
            5 => fire(npc, random, births),
            counter if counter > 24 => npc.state = 4,
            _ => {}
        },
        12 => match counter {
            1 => {
                npc.flip_x = false;
                plain_npc::play(npc, "Attack02", true);
            }
            5 => summon(npc, random, world, births)?,
            counter if counter > 26 => npc.state = 4,
            _ => {}
        },
        _ => {}
    }
    let state = state(npc);
    state.counter = state.counter.wrapping_add(1);
    Ok(())
}

/// FireProjectiles 49ebc0 from the frozen aim: Greed mode 2 (0° and ±16° at
/// 0.9), Super Greed mode 4 (±8°, and ±24° at 0.9; no centre shot), each then
/// ×1.2 plus 0.4 of Greed's velocity, as plain 9.0 shots.
fn fire(npc: &Npc, random: &mut Mt19937, births: &mut Vec<Birth>) {
    let parameter_angle = (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
    let aim = npc.greed.as_ref().expect("Greed state").aim;
    let (x, y) = (aim.x, aim.y);
    let spread = 1.0_f32;
    let rotate = |cos: f32, sin: f32, scale: f32, side: f32| WorldVector {
        x: (x * cos) * scale - side * (((y * sin) * scale) * spread),
        y: side * (((sin * x) * scale) * spread) + (cos * y) * scale,
    };
    let velocities: Vec<WorldVector> = if npc.variant == 1 {
        vec![
            WorldVector {
                x: x * 0.99 - (y * 0.139) * spread,
                y: (0.139 * x) * spread + 0.99 * y,
            },
            WorldVector {
                x: 0.99 * x + (0.139 * y) * spread,
                y: 0.99 * y - (0.139 * x) * spread,
            },
            rotate(0.914, 0.407, 0.9, 1.0),
            rotate(0.914, 0.407, 0.9, -1.0),
        ]
    } else {
        vec![
            aim,
            rotate(0.961, 0.276, 0.9, 1.0),
            rotate(0.961, 0.276, 0.9, -1.0),
        ]
    };
    for velocity in velocities {
        let spawn_seed = random.next_u32();
        let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
        births.push(Birth::Projectile(ProjectileBirth {
            position: npc.position,
            velocity: velocity * 1.2 + npc.velocity * 0.4,
            spawn_seed,
            initial_falling_speed,
            parameter_angle,
            falling_speed: -4.0,
            falling_acceleration: 0.1,
            scale: 1.0,
            height: -23.0,
        }));
    }
}

/// SUMMON at counter 5 (4c8af0): SFX 265, two free positions 50 px either
/// side (side by side when Greed is alone, else above and below), then a
/// seed and a spawn each.
fn summon(
    npc: &Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    world.audio.admit(
        random,
        world.system_counter,
        SoundRequest {
            id: 265,
            volume: 1.2,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
    let offset = if world.countable_total == 1 {
        WorldVector { x: 50.0, y: 0.0 }
    } else {
        WorldVector { x: 0.0, y: 50.0 }
    };
    let costs = world
        .path_costs
        .as_deref()
        .ok_or("Greed summon requires source Room.Init path costs")?;
    let mut positions = [WorldVector { x: 0.0, y: 0.0 }; 2];
    for (position, side) in positions.iter_mut().zip([-1.0_f32, 1.0]) {
        *position = crate::roundworm::find_free_position(
            npc.position + offset * side,
            0.0,
            random,
            world.grid,
            costs,
            world.occupied,
        )?;
    }
    for position in positions {
        let spawn_seed = random.next_u32();
        // PORT GAP: Super Greed summons Keeper 86.0, which the port does not
        // construct; its seeds are drawn and no Keeper spawns.
        if npc.variant == 1 {
            continue;
        }
        births.push(Birth::Npc(SpawnSelector {
            group_index: 0,
            grid_index: 0,
            category: SpawnCategory::Entity,
            entity_type: 29,
            variant: 0,
            subtype: 0,
            position,
            group_seed: spawn_seed,
            spawn_seed,
        }));
    }
    Ok(())
}
