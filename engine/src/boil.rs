//! Boil 30.0: ai 490e40's variant-0 path (shared with Gut 30.1 and Sack
//! 30.2, not admitted). A Boil sits on its anchor and grows from 7 HP to its
//! maximum, a real hit restarting the growth; at full size a 1-in-50 roll
//! either shakes it or, mostly when the player is near, makes it lob three
//! or four shots. The AI never draws the NPC's own RNG.
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
    kinds: &[30],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

/// Gurdy 36: its Boils grow at half the rate and never aim.
const GURDY: u32 = 36;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BoilState {
    /// +f78: the growth rate.
    pub rate: f32,
    /// +f68: Shoot updates.
    pub counter: i32,
}

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 30
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Boil requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(30, 0);
    let mut npc = Npc {
        // Config47f573 SetFrame(default "Appear", 0): the smallest body.
        animation: "Appear".into(),
        actor: config.actor.into(),
        boil: Some(BoilState::default()),
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

fn state(npc: &mut Npc) -> &mut BoilState {
    npc.boil.as_mut().expect("Boil owns its state")
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.state == 0 {
        // 490e60: start small on the anchor. A Gurdy summon first kills
        // every other Boil within 20 px, so repeated summons replace them.
        npc.hp = 7.0;
        if npc.spawner_type == GURDY {
            for other in world
                .peers
                .before
                .iter_mut()
                .chain(world.peers.after.iter_mut())
                .filter(|other| other.kind == 30 && !other.removed && !other.dead)
            {
                let (dx, dy) = (
                    other.position.x - npc.position.x,
                    other.position.y - npc.position.y,
                );
                if dx * dx + dy * dy < 400.0 {
                    // PORT GAP: Kill's gib burst runs on the victim's next
                    // update rather than inside this INIT.
                    other.dead = true;
                }
            }
        }
        state(npc).rate = 0.0;
        npc.target = npc.position;
        npc.state = 3;
        return Ok(());
    }
    *friction *= 0.8;
    // 490ef0: enemy-owned explosions do not hurt a Boil; any other hit
    // restarts its growth.
    let mut hit = 0.0;
    npc.damage.zero_events(|event| {
        let enemy_blast = event.kind.flags() & 4 != 0
            && event
                .copied_source
                .is_some_and(|source| !matches!(source.spawner_type, 0 | 1));
        if !enemy_blast {
            hit += event.amount;
        }
        enemy_blast
    });
    if hit > 0.0 {
        state(npc).rate = 0.0;
    }
    let delta = WorldVector {
        x: world.player.x - npc.position.x,
        y: world.player.y - npc.position.y,
    };
    let distance = (delta.x * delta.x + delta.y * delta.y).sqrt();
    if npc.hp < npc.max_hp {
        // 491090: grow with the old rate, then step it.
        let rate = state(npc).rate;
        state(npc).counter = 0;
        npc.hp = npc.max_hp.min(1.2 * rate + npc.hp);
        let step = if npc.spawner_type == GURDY {
            0.01
        } else {
            0.02
        };
        state(npc).rate = (rate + step) * 0.9;
        let fraction = (npc.hp / npc.max_hp).max(0.0);
        plain_npc::set_frame(npc, "Shrink", (13 - (fraction * 13.0) as i32) as u32);
        npc.state = 3;
    } else {
        match npc.state {
            // PORT GAP: fear and confusion (0xa00) only shake; the port
            // admits neither status.
            3 => {
                if random.bounded(50) == 0 {
                    npc.state = if distance < 230.0 || random.bounded(4) == 0 {
                        8
                    } else {
                        9
                    };
                }
                plain_npc::set_frame(npc, "Shrink", 0);
                state(npc).counter = 0;
            }
            8 => {
                plain_npc::play(npc, "Shoot", false);
                let counter = state(npc).counter;
                state(npc).counter = counter.wrapping_add(1);
                if counter == 10 {
                    fire(npc, random, world, births, distance);
                }
                if plain_npc::finished(npc) {
                    npc.state = 3;
                }
            }
            _ => {
                plain_npc::play(npc, "Shake", false);
                if plain_npc::finished(npc) {
                    npc.state = 3;
                }
            }
        }
    }
    // 49117a: pinned to the anchor.
    npc.velocity = WorldVector {
        x: npc.target.x - npc.position.x,
        y: npc.target.y - npc.position.y,
    };
    Ok(())
}

/// 49158f: SFX 178, the range roll, default ProjectileParams, then 48d5c0
/// lobs three or four shots with h = 10 at the player when near, otherwise
/// in random directions.
fn fire(
    npc: &Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    distance: f32,
) {
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
    let near = 250.0 - random.uniform() * 20.0 > distance && npc.spawner_type != GURDY;
    let parameter_angle = (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
    let count = 3 + random.bounded(2);
    const HEIGHT: f32 = 10.0;
    for _ in 0..count {
        let velocity = if near {
            let radius = random.uniform();
            let jitter = (direction(random) * radius) * 3.5;
            let delta = WorldVector {
                x: world.player.x - npc.position.x,
                y: world.player.y - npc.position.y,
            };
            let length = (delta.x * delta.x + delta.y * delta.y).sqrt();
            let aim = if length > 0.0 {
                delta * (1.0 / length)
            } else {
                delta
            };
            aim * 7.0 + jitter
        } else {
            direction(random) * 7.0
        };
        let spawn_seed = random.next_u32();
        let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
        let falling_speed = (((-random.uniform() * 30.0) * 0.8) + 5.0) - HEIGHT;
        let scale_choice = random.bounded(2);
        let scale_jitter = random.uniform() * 0.05;
        births.push(Birth::Projectile(ProjectileBirth {
            position: npc.position,
            velocity,
            spawn_seed,
            initial_falling_speed,
            parameter_angle,
            falling_speed,
            falling_acceleration: 0.1 * HEIGHT + 0.32,
            scale: (scale_choice as f32 * 0.4 + 0.9) + scale_jitter,
            height: -23.0,
        }));
    }
}

/// 42e8d0: one MT.float turned into a unit vector.
fn direction(random: &mut Mt19937) -> WorldVector {
    let angle = (f64::from(random.uniform()) * std::f64::consts::PI * 2.0) as f32;
    WorldVector {
        x: crate::fmath::cos(angle),
        y: crate::fmath::sin(angle),
    }
}
