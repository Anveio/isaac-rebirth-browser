//! Mulligan 16.0 and Mulligoon 16.1: ai 4e7d50. They wander until the
//! player comes in range, then flee with EvadeTarget, whimpering, with the
//! head overlay's eyes open. Bumping a wall may start the Explode overlay
//! (state 5); when it ends the blast, four cardinal shots and the death
//! spawns follow (`Birth::MulliganExplode`). A dead Mulligan scatters four
//! Flies, Pooters or Attack Flies unless overkilled by 10; a dead Mulligoon
//! fires four diagonal shots and drops a Troll Bomb. Mulliboom 16.2 is not
//! admitted.
use crate::{
    audio::SoundRequest,
    content,
    enemy_projectiles::ProjectileBirth,
    entities::{Birth, Family, NpcWorld, SpawnContext},
    motion::WorldVector,
    mt::Mt19937,
    npcs::{Npc, Overlay},
    pathfinder::{self, Pathfinder},
    phase::Phase,
    plain_npc,
    room_spawns::{SpawnCategory, SpawnSelector},
};

pub(crate) const FAMILY: Family = Family {
    kinds: &[16],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 16
        || selector.variant > 1
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Mulligan requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(16, selector.variant.into());
    let mut npc = Npc {
        // Config47f573 SetFrame(default, 0) leaves the default clip stopped.
        animation: "Appear".into(),
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

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.state == 0 {
        // INIT: no draws, no SetFrame, no overlay.
        npc.state = 4;
        return Ok(());
    }
    let target = world.player;
    if npc.state == 4 {
        walk(npc, random, world, target, friction)?;
    } else {
        *friction *= 0.8;
        // IsOverlayFinished(""): no overlay, or one that stopped.
        if npc.overlay.as_ref().is_none_or(|overlay| !overlay.playing) {
            // EXPLODE 4e8200 needs the room: the blast, effect seeds, shots,
            // Kill and the death spawns run as one birth right after this
            // update, which draws nothing more.
            births.push(Birth::MulliganExplode);
            return Ok(());
        }
    }
    if npc.dead {
        death(npc, random, births);
    }
    Ok(())
}

/// WALK 4e7f80 for 16.0/16.1. Authored Mulligans spend 20 updates appearing,
/// so the +42c age test (`> 10` game frames) always passes here.
fn walk(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    target: WorldVector,
    friction: &mut f32,
) -> Result<(), String> {
    pathfinder::evade_target(npc, random, world, target, friction)?;
    let panicking = npc.pathfinder.as_ref().is_some_and(|path| path.panic > 0);
    if panicking {
        let delay = random.bounded(100) as i32 + 45;
        world.audio.admit(
            random,
            world.system_counter,
            SoundRequest {
                id: 143,
                volume: 1.0,
                delay,
                looping: false,
                pitch: 1.0,
            },
        );
    }
    *friction *= 0.8;
    plain_npc::walk_frame(npc, "WalkHori", "WalkVert", 0.1);
    let name = if panicking { "WalkOpenEye" } else { "Walk" };
    if npc.animation_playing {
        play_overlay(npc, name)?;
    } else {
        // SetOverlayFrame(name, 0): selected and stopped at frame 0.
        npc.overlay = Some(Overlay {
            animation: name.into(),
            frame: 0,
            clock: 0.0,
            playing: false,
        });
    }
    if npc.grid_collided && random.bounded(300) == 0 {
        play_overlay(npc, "Explode")?;
        npc.state = 5;
    }
    Ok(())
}

/// PlayOverlay 4102a0 unforced: restart at frame 0 without an overlay, on a
/// different name, or on the same looping clip once stopped.
fn play_overlay(npc: &mut Npc, name: &str) -> Result<(), String> {
    if let Some(overlay) = npc.overlay.as_ref().filter(|overlay| overlay.animation == name) {
        let looping = content::clip(&npc.actor, name)
            .ok_or_else(|| format!("{} has no authored overlay {name}", npc.actor))?
            .looping;
        if overlay.playing || !looping {
            return Ok(());
        }
    }
    npc.overlay = Some(Overlay {
        animation: name.into(),
        frame: 0,
        clock: 0.0,
        playing: true,
    });
    Ok(())
}

/// The dead branch (4e7e32): Mulligan 4e7e54 flies unless overkilled by 10,
/// Mulligoon 4e8520 a diagonal burst and a Troll Bomb.
pub(crate) fn death(npc: &Npc, random: &mut Mt19937, births: &mut Vec<Birth>) {
    if npc.variant == 0 {
        if npc.hp <= -10.0 {
            return;
        }
        for _ in 0..4 {
            let roll = random.bounded(11);
            let kind = match roll {
                8.. => 18,
                7 => 14,
                _ => 13,
            };
            // 4e7e9c: (u·π) + (u·π) in f64 from one float.
            let unit = f64::from(random.uniform());
            let angle = ((unit * std::f64::consts::PI) + (unit * std::f64::consts::PI)) as f32;
            let distance = random.uniform() * 6.0;
            let offset = WorldVector {
                x: crate::fmath::cos(angle) * distance,
                y: crate::fmath::sin(angle) * distance,
            };
            let spawn_seed = random.next_u32();
            births.push(Birth::FlungNpc(
                SpawnSelector {
                    group_index: 0,
                    grid_index: 0,
                    category: SpawnCategory::Entity,
                    entity_type: kind,
                    variant: 0,
                    subtype: 0,
                    position: npc.position + offset,
                    group_seed: spawn_seed,
                    spawn_seed,
                },
                ZERO,
            ));
        }
    } else {
        let diagonal = 7.0_f32 * 0.707_106_77;
        burst(
            npc.position,
            random,
            births,
            &[
                (diagonal, diagonal),
                (-diagonal, diagonal),
                (diagonal, -diagonal),
                (-diagonal, -diagonal),
            ],
        );
        let seed = random.next_u32();
        births.push(Birth::Bomb {
            variant: 3,
            seed,
            position: npc.position,
        });
    }
}

/// FireProjectiles 49ebc0 modes 6/7 after a default ProjectileParams (one
/// float): each shot draws its seed and Init falling speed; modes 6..9 keep
/// Init's falling speed and acceleration.
pub(crate) fn burst(position: WorldVector, random: &mut Mt19937, births: &mut Vec<Birth>, velocities: &[(f32, f32)]) {
    let parameter_angle = (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
    for &(x, y) in velocities {
        let spawn_seed = random.next_u32();
        let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
        births.push(Birth::Projectile(ProjectileBirth {
            position,
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
}

/// EXPLODE 4e8200 after the blast: the 1000.1 seed and its Init FlipX roll,
/// the 1000.18 seed and its Init alpha float (neither effect actor is
/// admitted, so 1000.1's first-update SFX 182 draw is missing), then the
/// mode-6 cardinal burst at speed 7.
pub(crate) fn explode_shots(position: WorldVector, random: &mut Mt19937, births: &mut Vec<Birth>) {
    random.next_u32();
    random.bounded(2);
    random.next_u32();
    random.uniform();
    burst(position, random, births, &[(7.0, 0.0), (-7.0, 0.0), (0.0, 7.0), (0.0, -7.0)]);
}
