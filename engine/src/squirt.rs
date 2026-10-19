//! Squirt 220.0: ai 494eb0. A Squirt idles, then on a 1-in-5 roll at a
//! fifth frame winds up Attack01 and, on its Slide event, lunges at the
//! player; it slides on, pushing along its own velocity for 15 updates, until
//! it slows to a stop. A brown Blood7 splat trails it on a 1-in-10 roll and it
//! bursts into two Dips 217.0 when it dies.
use crate::{
    audio::SoundRequest,
    content,
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
    kinds: &[220],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SquirtState {
    /// +f58: updates spent sliding.
    pub counter: i32,
    /// +f88: grid hits this slide.
    pub hits: i32,
}

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 220
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Squirt requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(220, 0);
    let mut npc = Npc {
        // Config47f573 SetFrame(default "Idle", 0) leaves the clip stopped.
        animation: "Idle".into(),
        actor: config.actor.into(),
        squirt: Some(SquirtState::default()),
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

fn state(npc: &mut Npc) -> &mut SquirtState {
    npc.squirt.as_mut().expect("Squirt owns its state")
}

/// The 6128e0 resize: `v·(n/|v|)`, a zero vector unchanged.
fn resized(vector: WorldVector, length: f32) -> WorldVector {
    let current = (vector.x * vector.x + vector.y * vector.y).sqrt();
    if current > 0.0 {
        vector * (length / current)
    } else {
        vector
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
        // 494f40: state 3 and the brown SplatColor (built in the tail);
        // no tail.
        npc.state = 3;
        return Ok(());
    }
    if npc.dead {
        // 495100: two Dips 10 px either side, at rest; no tail.
        for x in [10.0, -10.0] {
            let spawn_seed = random.next_u32();
            births.push(Birth::Npc(SpawnSelector {
                group_index: 0,
                grid_index: 0,
                category: SpawnCategory::Entity,
                entity_type: 217,
                variant: 0,
                subtype: 0,
                position: npc.position + WorldVector { x, y: 0.0 },
                group_seed: spawn_seed,
                spawn_seed,
            }));
        }
        return Ok(());
    }
    *friction *= 0.95;
    match npc.state {
        3 => {
            plain_npc::play(npc, "Idle", false);
            // PORT GAP: with fear (0x800) or confusion (0x200) the roll
            // slides away from the player or in a random direction instead;
            // the port admits neither status.
            if random.bounded(5) == 0 && plain_npc::on_frame(npc, world, 5)? {
                npc.state = 8;
                state(npc).hits = 0;
            }
            *friction *= 0.7;
        }
        4 => {
            plain_npc::play(npc, "Slide", false);
            let counter = state(npc).counter;
            state(npc).counter = counter.wrapping_add(1);
            let speed = npc.velocity.x * npc.velocity.x + npc.velocity.y * npc.velocity.y;
            if counter <= 14 {
                let push = resized(npc.velocity, 0.5);
                if *friction != 0.0 {
                    npc.velocity = npc.velocity
                        + WorldVector {
                            x: push.x / *friction,
                            y: push.y / *friction,
                        };
                }
            } else if speed < 1.0 {
                npc.state = 3;
            } else if speed < 9.0 {
                *friction *= 0.8;
            }
            if npc.grid_collided {
                state(npc).hits += 1;
            }
            if state(npc).hits > 4 {
                *friction *= 0.6;
            }
        }
        8 => {
            plain_npc::play(npc, "Attack01", false);
            if plain_npc::event_triggered(npc, "Slide") {
                let delta = WorldVector {
                    x: world.player.x - npc.position.x,
                    y: world.player.y - npc.position.y,
                };
                let length = (delta.x * delta.x + delta.y * delta.y).sqrt();
                let toward = if length > 0.0 {
                    delta * (1.0 / length)
                } else {
                    delta
                } * 20.0;
                if *friction != 0.0 {
                    npc.velocity = npc.velocity
                        + WorldVector {
                            x: toward.x / *friction,
                            y: toward.y / *friction,
                        };
                }
                world.audio.admit(
                    random,
                    world.system_counter,
                    SoundRequest {
                        id: 57,
                        volume: 1.0,
                        delay: 100,
                        looping: false,
                        pitch: 1.0,
                    },
                );
            }
            if plain_npc::finished(npc) {
                state(npc).counter = 0;
                npc.state = 4;
            }
        }
        _ => {}
    }
    // Tail 494f0f: the brown trail, then FlipX from the velocity.
    if random.bounded(10) == 0 {
        let unit = f32::from_bits(0x3b80_8081);
        crate::dingle::brown_splat(
            npc.position,
            [0.0, 0.0, 0.0, 255.0 * unit],
            [65.0 * unit, 41.0 * unit, 35.0 * unit],
            random,
            world,
        )?;
    }
    npc.flip_x = npc.velocity.x < 0.0;
    Ok(())
}
