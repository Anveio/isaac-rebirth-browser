//! Crazy Long Legs 207.0 and Small Crazy Long Legs 207.1: ai 4b8e60's
//! type-207 path (shared with Baby Long Legs 206, not admitted). It walks at
//! the player with FindGridPath and, on a 1-in-50 roll while the path is
//! direct, plays Attack; the Shoot event lobs eight shots around it (the
//! small variant four diagonals) at speed 8..10.
use crate::{
    audio::SoundRequest,
    content,
    enemy_projectiles::ProjectileBirth,
    entities::{Birth, Family, NpcWorld, SpawnContext},
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    pathfinder::{self, Pathfinder},
    phase::Phase,
    plain_npc,
    room_spawns::{SpawnCategory, SpawnSelector},
};

pub(crate) const FAMILY: Family = Family {
    kinds: &[207],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 207
        || selector.variant > 1
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err(
            "Crazy Long Legs requires bounded authored position and nonzero spawn seed".into(),
        );
    }
    let config = content::entity(207, selector.variant.into());
    let mut npc = Npc {
        // Config47f573 SetFrame(default "Idle", 0) leaves the clip stopped.
        animation: "Idle".into(),
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

/// 4b8e60 never tests the dead flag, so a dying Crazy Long Legs runs its
/// state body once more.
fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    match npc.state {
        0 => npc.state = 4,
        4 => {
            // PORT GAP: fear (0x800) evades and confusion (0x200) wanders
            // instead; the port admits neither status.
            pathfinder::find_grid_path(npc, random, world, world.player, 0.65, 900, friction)?;
            let direct = npc.pathfinder.as_ref().is_some_and(|path| path.direct);
            if direct && random.bounded(50) == 0 {
                npc.state = 8;
            }
            plain_npc::walk_frame(npc, "Walk", "Walk", 0.1);
            // 4b8f18 assigns the friction rather than scaling it.
            *friction = 0.8;
        }
        8 => {
            plain_npc::play(npc, "Attack", false);
            if plain_npc::event_triggered(npc, "Shoot") {
                world.audio.admit(
                    random,
                    world.system_counter,
                    SoundRequest {
                        id: 178,
                        volume: 1.0,
                        delay: 2,
                        looping: false,
                        pitch: 1.0,
                    },
                );
                lob(npc, random, births);
            }
            if plain_npc::finished(npc) {
                npc.state = 4;
            }
            *friction *= 0.5;
        }
        _ => {}
    }
    Ok(())
}

/// 4b9340: default ProjectileParams draw their angle and get height +10,
/// falling speed -10 and acceleration +0.5; the speed draw follows, then
/// FireProjectiles mode 8 (207.0) or mode 7 (207.1). Modes 6..9 keep each
/// shot's Init falling speed and add the modifiers, so every shot arcs.
fn lob(npc: &Npc, random: &mut Mt19937, births: &mut Vec<Birth>) {
    let parameter_angle = (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
    let speed = random.uniform() * 2.0 + 8.0;
    let diagonal = speed * 0.707_106_77;
    let diagonals = [
        (diagonal, diagonal),
        (-diagonal, diagonal),
        (diagonal, -diagonal),
        (-diagonal, -diagonal),
    ];
    let straight = [(speed, 0.0), (-speed, 0.0), (0.0, speed), (0.0, -speed)];
    let velocities: Vec<(f32, f32)> = if npc.variant == 0 {
        straight.into_iter().chain(diagonals).collect()
    } else {
        diagonals.to_vec()
    };
    for (x, y) in velocities {
        let spawn_seed = random.next_u32();
        let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
        births.push(Birth::Projectile(ProjectileBirth {
            position: npc.position,
            velocity: WorldVector { x, y },
            spawn_seed,
            initial_falling_speed,
            parameter_angle,
            falling_speed: initial_falling_speed - 10.0,
            falling_acceleration: 0.5,
            scale: 1.0,
            height: -13.0,
        }));
    }
}
