//! Gurdy 36.0: ai 4c55f0. Gurdy sits pinned to its anchor and alternates an
//! idle clip with an attack: two Attack Flies, a Pooter or two Boils when the
//! room is not crowded, or a five-shot fan down or to the side toward a
//! player below it. Its spawned children are pushed out of its body. The AI
//! never draws the NPC's own RNG.
use crate::{
    audio::SoundRequest,
    content,
    enemy_projectiles::ProjectileBirth,
    entities::{Birth, Family, NpcWorld, SpawnContext, SpawnedNpc},
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    phase::Phase,
    plain_npc,
    room_spawns::{SpawnCategory, SpawnSelector},
};
use serde::{Deserialize, Serialize};

pub(crate) const FAMILY: Family = Family {
    kinds: &[36],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GurdyState {
    /// +f58: IDLE 0 starts an idle clip, 1 waits; ATTACK counts updates.
    pub counter: i32,
    /// +f88: the Attack clip number, 1..6.
    pub attack: i32,
}

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 36
        || selector.variant != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Gurdy requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(36, 0);
    let mut npc = Npc {
        animation: "Idle".into(),
        actor: config.actor.into(),
        gurdy: Some(GurdyState::default()),
        ..Npc::authored(id, config, selector)
    };
    // stageHP 0: 595 on every floor.
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

fn state(npc: &mut Npc) -> &mut GurdyState {
    npc.gurdy.as_mut().expect("Gurdy owns its state")
}

fn sound(world: &mut NpcWorld, random: &mut Mt19937, id: u32, volume: f32) {
    world.audio.admit(
        random,
        world.system_counter,
        SoundRequest {
            id,
            volume,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.state == 0 {
        // 4c561b: down 25 onto its anchor; the RoomConfig fallback that
        // moves the player only fires without a room shape.
        npc.position.y += 25.0;
        npc.target = npc.position;
        npc.state = 3;
        return Ok(());
    }
    // 4c56d0: pinned to the anchor, then push its children out.
    npc.velocity = WorldVector {
        x: npc.target.x - npc.position.x,
        y: npc.target.y - npc.position.y,
    };
    *friction *= 0.8;
    for child in world
        .peers
        .before
        .iter_mut()
        .chain(world.peers.after.iter_mut())
        .filter(|child| !child.removed && child.spawner_type == npc.kind && child.kind != 30)
    {
        let delta = WorldVector {
            x: child.position.x - npc.position.x,
            y: child.position.y - npc.position.y,
        };
        let length = (delta.x * delta.x + delta.y * delta.y).sqrt();
        if length > 0.01 && length < 80.0 {
            let push = delta * (((80.0 - length) * 0.03) / length);
            child.velocity = child.velocity
                + WorldVector {
                    x: push.x / *friction,
                    y: push.y / *friction,
                };
        }
    }
    match npc.state {
        3 => idle(npc, random, world),
        8 => attack(npc, random, world, births),
        _ => {}
    }
    Ok(())
}

/// IDLE 4c589a: play an idle clip, then decide when it finishes.
fn idle(npc: &mut Npc, random: &mut Mt19937, world: &NpcWorld) {
    if state(npc).counter == 0 {
        // PORT GAP: Idle2 plays once the player's +484 drops below the
        // snapshot +f78, which starts at 0 and never triggers in play.
        let id = if random.bounded(2) == 0 { 1 } else { 3 };
        plain_npc::play(npc, &format!("Idle{id}"), true);
        state(npc).counter += 1;
        return;
    }
    if !plain_npc::finished(npc) {
        return;
    }
    let total = world.countable_total;
    let directional =
        random.bounded(3) != 0 && world.player.y > 200.0 && total <= 7 && npc.subtype != 1;
    let id = if directional {
        let delta = WorldVector {
            x: npc.position.x - world.player.x,
            y: npc.position.y - world.player.y,
        };
        if delta.x.abs() > delta.y.abs() || delta.y > 0.0 {
            if delta.x <= 0.0 {
                5
            } else {
                4
            }
        } else {
            2
        }
    } else if total > 13 {
        state(npc).counter = 0;
        return;
    } else {
        [3, 1, 6][random.bounded(3) as usize]
    };
    state(npc).counter = 0;
    npc.state = 8;
    plain_npc::play(npc, &format!("Attack{id}"), false);
    state(npc).attack = id;
}

/// ATTACK 4c5afd: each update draws the default ProjectileParams angle; the
/// attack fires on its counter value.
fn attack(npc: &mut Npc, random: &mut Mt19937, world: &mut NpcWorld, births: &mut Vec<Birth>) {
    if plain_npc::finished(npc) {
        state(npc).counter = 0;
        npc.state = 3;
        return;
    }
    state(npc).counter += 1;
    let parameter_angle = (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
    let GurdyState { counter, attack } = *state(npc);
    match (attack, counter) {
        (1, 16) => {
            sound(world, random, 265, 1.0);
            sound(world, random, 13, 1.0);
            for x in [40.0, -40.0] {
                summon(
                    npc,
                    random,
                    births,
                    18,
                    (x, -65.0),
                    (0.0, -20.0),
                    true,
                    -2.0,
                );
            }
        }
        (3, 9) => {
            sound(world, random, 265, 0.7);
            sound(world, random, 9, 1.0);
            summon(
                npc,
                random,
                births,
                14,
                (60.0, -25.0),
                (10.0, 0.0),
                false,
                0.0,
            );
        }
        (6, 16) => {
            sound(world, random, 265, 1.2);
            for x in [42.0, -42.0] {
                summon(npc, random, births, 30, (x, 45.0), (0.0, 0.0), false, 0.0);
            }
        }
        (2, 15) | (4, 15) | (5, 16) => {
            sound(world, random, if attack == 2 { 12 } else { 11 }, 1.0);
            let jitter = random.uniform() * 30.0 - 15.0;
            let spread = random.uniform() * 3.0;
            let (muzzle, velocity) = match attack {
                2 => ((jitter, -20.0), (spread - 1.5, 10.0)),
                4 => ((13.0, -18.0 - jitter), (-10.0, (spread + 1.0) - 1.5)),
                _ => ((-13.0, -18.0 - jitter), (10.0, (spread + 1.0) - 1.5)),
            };
            fan(
                npc.position
                    + WorldVector {
                        x: muzzle.0,
                        y: muzzle.1,
                    },
                WorldVector {
                    x: velocity.0,
                    y: velocity.1,
                },
                parameter_angle,
                random,
                births,
            );
        }
        _ => {}
    }
}

/// Spawn550a40 with Gurdy as the spawner: the child keeps its appear; the
/// Attack Flies are marked boss-spawned (+320) and lose 2 HP.
#[expect(
    clippy::too_many_arguments,
    reason = "Spawn550a40's type, offset, velocity and the caller's +320/HP stores"
)]
fn summon(
    npc: &Npc,
    random: &mut Mt19937,
    births: &mut Vec<Birth>,
    entity_type: u16,
    (x, y): (f32, f32),
    (vx, vy): (f32, f32),
    boss_spawned: bool,
    health_delta: f32,
) {
    let spawn_seed = random.next_u32();
    births.push(Birth::Spawned(SpawnedNpc {
        selector: SpawnSelector {
            group_index: 0,
            grid_index: 0,
            category: SpawnCategory::Entity,
            entity_type,
            variant: 0,
            subtype: 0,
            position: npc.position + WorldVector { x, y },
            group_seed: spawn_seed,
            spawn_seed,
        },
        velocity: WorldVector { x: vx, y: vy },
        spawner_type: npc.kind,
        boss_spawned,
        scale: 1.0,
        health_scale: 1.0,
        max_health_scale: 1.0,
        health_delta,
        appear: true,
    }));
}

/// FireProjectiles mode 5 (49fbb8): the aim and ±12°/±24° turns, each a
/// plain 9.0 shot with the default params (height -23, FS -3, FA 0.1).
fn fan(
    position: WorldVector,
    velocity: WorldVector,
    parameter_angle: f32,
    random: &mut Mt19937,
    births: &mut Vec<Birth>,
) {
    let (x, y) = (velocity.x, velocity.y);
    let turn = |c: f32, s: f32, side: f32| WorldVector {
        x: x * c - side * (y * s),
        y: side * (s * x) + c * y,
    };
    for shot in [
        velocity,
        turn(0.978, 0.208, 1.0),
        turn(0.978, 0.208, -1.0),
        turn(0.914, 0.407, 1.0),
        turn(0.914, 0.407, -1.0),
    ] {
        let spawn_seed = random.next_u32();
        let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
        births.push(Birth::Projectile(ProjectileBirth {
            position,
            velocity: shot,
            spawn_seed,
            initial_falling_speed,
            parameter_angle,
            falling_speed: -3.0,
            falling_acceleration: 0.1,
            scale: 1.0,
            height: -23.0,
        }));
    }
}
