//! Fatty 208.0 and Pale Fatty 208.1: ai 4e9330's type-208 path (shared with
//! the Gapers). A Fatty walks slowly at the player with FindGridPath and,
//! once its counter reaches 100, farts with a 1-in-120 roll per update while
//! the player is within 200 px: Game::Fart 5522d0 knocks back the player and
//! nearby enemies. A Pale Fatty also fires a six-shot ring.
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
    State,
};
use serde::{Deserialize, Serialize};

pub(crate) const FAMILY: Family = Family {
    kinds: &[208],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

/// Game::Fart radius for a Fatty.
const FART_RADIUS: f32 = 120.0;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FattyState {
    /// +f58: counts up to 100 in every alive update.
    pub counter: i32,
}

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 208
        || selector.variant > 1
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Fatty requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(208, selector.variant.into());
    let mut npc = Npc {
        // Config47f573 SetFrame(default, 0) leaves the default clip stopped.
        animation: "WalkHori".into(),
        actor: config.actor.into(),
        pathfinder: Some(Pathfinder::new()),
        fatty: Some(FattyState::default()),
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

fn sound(world: &mut NpcWorld, random: &mut Mt19937, id: u32, delay: i32) {
    world.audio.admit(
        random,
        world.system_counter,
        SoundRequest {
            id,
            volume: 1.0,
            delay,
            looping: false,
            pitch: 1.0,
        },
    );
}

fn counter(npc: &mut Npc) -> &mut i32 {
    &mut npc.fatty.as_mut().expect("Fatty owns its state").counter
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.dead {
        // 4e9418: with overkill under 25, bounded(2) on both variants; a Pale
        // Fatty's 0 Morphs into Blubber 210.0 at full HP.
        // PORT GAP: Blubber 210 is not constructed, so a Pale Fatty that
        // rolls the morph dies instead of becoming a Blubber.
        if npc.hp > -25.0 {
            random.bounded(2);
        }
        return Ok(());
    }
    if npc.state == 0 {
        npc.state = 4;
        *counter(npc) = 0;
        plain_npc::set_frame(npc, "WalkVert", 0);
        return Ok(());
    }
    if *counter(npc) <= 99 {
        *counter(npc) += 1;
    }
    match npc.state {
        4 => {
            let delta = WorldVector {
                x: world.player.x - npc.position.x,
                y: world.player.y - npc.position.y,
            };
            if delta.x * delta.x + delta.y * delta.y < 40000.0
                && random.bounded(120) == 0
                && *counter(npc) > 99
            {
                sound(world, random, 167, 5);
                npc.state = 8;
                if npc.velocity.x.abs() > npc.velocity.y.abs() {
                    plain_npc::play(npc, "AttackHori", false);
                    npc.flip_x = npc.velocity.x < 0.0;
                } else {
                    plain_npc::play(npc, "AttackVert", false);
                }
                return Ok(());
            }
        }
        8 => {
            if plain_npc::event_triggered(npc, "Shoot") {
                // PORT GAP: Effect 1000.34 Fart (cosmetic) is not admitted;
                // its seed, Init FlipX roll and sound 37 still run.
                random.next_u32();
                random.bounded(2);
                sound(world, random, 37, 2);
                births.push(Birth::Fart {
                    position: npc.position,
                    radius: FART_RADIUS,
                    source: npc.id,
                });
                if npc.variant == 1 {
                    ring(npc, random, births);
                }
            }
            if plain_npc::finished(npc) {
                npc.state = 4;
                *counter(npc) = 0;
            }
            *friction *= 0.4;
            return Ok(());
        }
        _ => {}
    }
    plain_npc::walk_frame(npc, "WalkHori", "WalkVert", 0.1);
    pathfinder::find_grid_path(npc, random, world, world.player, 0.3, 900, friction)?;
    let direct = npc.pathfinder.as_ref().is_some_and(|path| path.direct);
    if direct && random.bounded(3) == 0 {
        sound(world, random, 116, 60);
    }
    Ok(())
}

/// FireProjectiles mode 9 (4a09a8): six shots at speed 8 around the circle,
/// the first one step past the params angle, keeping their Init fall.
fn ring(npc: &Npc, random: &mut Mt19937, births: &mut Vec<Birth>) {
    let parameter_angle = (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
    let step = (2.0 * std::f64::consts::PI / 6.0) as f32;
    let mut angle = parameter_angle;
    for _ in 0..6 {
        angle += step;
        let velocity = WorldVector {
            x: 8.0 * crate::fmath::cos(angle),
            y: 8.0 * crate::fmath::sin(angle),
        };
        let spawn_seed = random.next_u32();
        let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
        births.push(Birth::Projectile(ProjectileBirth {
            position: npc.position,
            velocity,
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

/// Game::Fart 5522d0's push over the actors within `radius` of `position`:
/// the player (unless a player farted) and every vulnerable NPC but the
/// source gain `vel·0.5 + (d/L)·min(2x, 14)` with `x = radius + size − L`,
/// halved above mass 20. No damage. The source's own spawn draws ran in its
/// AI; the push itself draws nothing.
pub(crate) fn fart(state: &mut State, position: WorldVector, radius: f32, source: u32) {
    let push = |at: WorldVector, size: f32, mass: f32, velocity: WorldVector| {
        let delta = WorldVector {
            x: at.x - position.x,
            y: at.y - position.y,
        };
        let length = (delta.x * delta.x + delta.y * delta.y).sqrt();
        if mass == 0.0 || length <= 0.0 || (radius + size) * (radius + size) <= length * length {
            return None;
        }
        let x = (radius + size) - length;
        let mut strength = if x > 7.0 { 14.0 } else { x + x };
        if mass > 20.0 {
            strength *= 0.5;
        }
        let direction = WorldVector {
            x: delta.x / length,
            y: delta.y / length,
        };
        Some(velocity * 0.5 + direction * strength)
    };
    for npc in state.room.npcs.iter_mut().filter(|npc| {
        npc.id != source
            && (10..=999).contains(&npc.kind)
            && !matches!(npc.entity_collision_class, 0 | 1)
            && !npc.invincible
            && !npc.dead
            && !npc.removed
    }) {
        if let Some(velocity) = push(
            npc.position,
            npc.collision_radius,
            npc.collision_mass,
            npc.velocity,
        ) {
            npc.velocity = velocity;
        }
    }
    if !state.player.dead {
        let player = crate::bombs::player_world(state);
        let velocity = crate::bombs::player_velocity(state);
        if let Some(velocity) = push(
            player,
            content::PLAYER_COLLISION_RADIUS_WORLD as f32,
            5.0,
            velocity,
        ) {
            state.player.vx = f64::from(velocity.x) * crate::motion::WORLD_SCALE;
            state.player.vy = f64::from(velocity.y) * crate::motion::WORLD_SCALE;
        }
    }
}
