//! Skinny 226.0 and Bony 227.0: ai 4b7f40 (shared with Rotty 226.1, not
//! admitted). A Skinny walks at the player with FindGridPath under its
//! looping Head overlay, moans now and then, and every seventh frame leaves
//! a small red creep pool. A Bony wanders with MoveRandomly and, lined up
//! with the player's predicted position, throws one bone. Neither draws its
//! own RNG.
use crate::{
    audio::SoundRequest,
    content,
    enemy_projectiles::{ProjectileBirth, BONE},
    entities::{Birth, Family, NpcWorld, SpawnContext},
    motion::WorldVector,
    mt::Mt19937,
    npcs::{normalized, Npc, Overlay},
    pathfinder::{self, Pathfinder},
    phase::Phase,
    plain_npc,
    room_spawns::{SpawnCategory, SpawnSelector},
};

pub(crate) const FAMILY: Family = Family {
    kinds: &[226, 227],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    // Rotty 226.1 first appears in Depths and stays out.
    if selector.category != SpawnCategory::Entity
        || !matches!(selector.entity_type, 226 | 227)
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Skinny requires bounded authored position and nonzero spawn seed".into());
    }
    let kind = u32::from(selector.entity_type);
    let config = content::entity(kind, 0);
    let mut npc = Npc {
        // Config47f573 SetFrame(default, 0) leaves the default clip stopped.
        animation: if kind == 227 { "Appear" } else { "WalkHori" }.into(),
        actor: config.actor.into(),
        pathfinder: Some(Pathfinder::new()),
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

/// The walk body also runs on the dead-AI update (4b7f40 has no dead test for
/// 226.0), so a dying Skinny may still moan and leave a last pool.
fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.kind == 227 {
        return bony(npc, random, world, births, friction);
    }
    if npc.state == 0 {
        // 4b7f63: state 4 and PlayOverlay("Head"); no tail.
        npc.state = 4;
        npc.overlay = Some(Overlay {
            animation: "Head".into(),
            frame: 0,
            clock: 0.0,
            playing: true,
        });
        return Ok(());
    }
    if npc.state == 4 {
        if plain_npc::on_frame(npc, world, 20)? && random.bounded(3) == 0 {
            world.audio.admit(
                random,
                world.system_counter,
                SoundRequest {
                    id: 167,
                    volume: 0.8,
                    delay: 120,
                    looping: false,
                    pitch: 1.0,
                },
            );
        }
        pathfinder::find_grid_path(npc, random, world, world.player, 0.5, 900, friction)?;
        // After the path: the walk clip follows this update's velocity.
        plain_npc::walk_frame(npc, "WalkHori", "WalkVert", 0.1);
        if plain_npc::on_frame(npc, world, 7)? {
            // 4b858b: a spawnerless 1000.22 at scale 0.1, size 0.5, 30/30,
            // first updated by the scheduler.
            let seed = random.next_u32();
            world
                .effects
                .admit_sized_creep(seed, npc.position, 0.1, 0.5, (30, 30))?;
        }
    }
    *friction *= 0.8;
    Ok(())
}

fn sound(world: &mut NpcWorld, random: &mut Mt19937, id: u32, volume: f32, delay: i32) {
    world.audio.admit(
        random,
        world.system_counter,
        SoundRequest {
            id,
            volume,
            delay,
            looping: false,
            pitch: 1.0,
        },
    );
}

/// 4b800d and 4b8380: the Bony branch.
fn bony(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.state == 0 {
        // 4b8320: state 4. PORT GAP: the +318 bit 0x10000 it also sets is
        // not admitted.
        npc.state = 4;
        return Ok(());
    }
    match npc.state {
        4 => {
            if plain_npc::on_frame(npc, world, 30)? && random.bounded(3) == 0 {
                sound(world, random, 116, 0.8, 120);
            }
            // 5b40d0: the cell is taken before MoveRandomly moves it.
            let cell = world
                .grid
                .index(npc.position)
                .and_then(|index| world.grid.position(index))
                .ok_or("Bony requires a position on the room lattice")?;
            pathfinder::move_randomly(npc, random, world, 1.25, *friction)?;
            let velocity = npc.velocity;
            if 1.0 > velocity.x * velocity.x + velocity.y * velocity.y {
                plain_npc::set_frame(npc, "WalkDown", 0);
            } else if velocity.x.abs() > velocity.y.abs() {
                plain_npc::play(npc, "WalkHori", false);
                npc.flip_x = velocity.x < 0.0;
            } else if velocity.y > 0.0 {
                plain_npc::play(npc, "WalkDown", false);
            } else {
                plain_npc::play(npc, "WalkUp", false);
            }
            // PORT GAP: confusion and fear (+318 & 0xa00) skip the check;
            // the port admits neither status.
            if plain_npc::on_frame(npc, world, 3)? {
                let lead = random.uniform() * 16.0;
                let aim = world.player + world.player_velocity * lead;
                let delta = WorldVector {
                    x: cell.x - aim.x,
                    y: cell.y - aim.y,
                };
                let costs = world
                    .path_costs
                    .as_deref()
                    .ok_or("Bony line check requires source Room.Init path costs")?;
                if (20.0 > delta.x.abs() || 20.0 > delta.y.abs())
                    && 200.0 > (delta.x * delta.x + delta.y * delta.y).sqrt()
                    && world.grid.projectile_line_clear(cell, aim, costs)
                {
                    npc.state = 8;
                    let toward = WorldVector {
                        x: aim.x - npc.position.x,
                        y: aim.y - npc.position.y,
                    };
                    if toward.x.abs() > toward.y.abs() {
                        plain_npc::play(npc, "AttackHori", false);
                        npc.flip_x = toward.x < 0.0;
                    } else if toward.y > 0.0 {
                        plain_npc::play(npc, "AttackDown", false);
                    } else {
                        plain_npc::play(npc, "AttackUp", false);
                    }
                }
            }
        }
        8 => {
            if plain_npc::event_triggered(npc, "Shoot") {
                throw(npc, random, world, births);
            }
            if plain_npc::finished(npc) {
                npc.state = 4;
            }
            *friction *= 0.3;
        }
        _ => {}
    }
    *friction *= 0.8;
    Ok(())
}

/// 4b8380 Shoot: default ProjectileParams with the bone variant, SFX 249,
/// then FireProjectiles mode 0 at the live player position at speed 10.
fn throw(npc: &Npc, random: &mut Mt19937, world: &mut NpcWorld, births: &mut Vec<Birth>) {
    let parameter_angle = (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
    sound(world, random, 249, 1.0, 2);
    let velocity = normalized(WorldVector {
        x: world.player.x - npc.position.x,
        y: world.player.y - npc.position.y,
    }) * 10.0;
    let spawn_seed = random.next_u32();
    let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
    births.push(Birth::VariantProjectile(
        ProjectileBirth {
            position: npc.position,
            velocity,
            spawn_seed,
            initial_falling_speed,
            parameter_angle,
            falling_speed: -3.0,
            falling_acceleration: 0.1,
            scale: 1.0,
            height: -23.0,
        },
        BONE,
    ));
}
