//! Wall Hugger218.0: ai_wallhugger4856a0. An invincible crawler that keeps a
//! solid cell on one side and circles the room's walls and obstacles,
//! harmless for its first 30 game frames ("No-Spikes").
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
    kinds: &[218],
    spawn,
    advance,
    post_contact_grid: crate::entities::clamp_walls,
    entity_flags,
    occupancy: crate::entities::Occupancy::None,
};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };
/// 66ebe0: down, right, up, left (world y grows downward).
const DIRECTIONS: [(f32, f32); 4] = [(0.0, 1.0), (1.0, 0.0), (0.0, -1.0), (-1.0, 0.0)];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WallHuggerState {
    /// Entity_NPC+f88 travel direction index (-1 until chosen).
    pub direction: i32,
    /// +f8c: turned around an outer corner and has not yet taken a target.
    pub outer_turned: bool,
    /// Entity+42c: Game+ce9f4 at Init.
    pub init_counter: u32,
    /// +318 after state0: NO_STATUS_EFFECTS set, APPEAR cleared.
    pub entity_flags: u32,
}

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    // Authored subtypes 0..4 pick the starting direction.
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 218
        || selector.variant != 0
        || selector.subtype > 4
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Wall Hugger requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(218, 0);
    let mut npc = Npc {
        // Config47f573 SetFrame(default "Spikes", 0) leaves the clip stopped.
        animation: "Spikes".into(),
        actor: config.actor.into(),
        wall_hugger: Some(WallHuggerState {
            direction: -1,
            outer_turned: false,
            init_counter: 0,
            entity_flags: 4,
        }),
        ..Npc::authored(id, config, selector)
    };
    npc.max_hp = content::initial_hp(config, context.stage);
    npc.hp = npc.max_hp;
    Ok(Some(npc))
}

fn entity_flags(npc: &Npc) -> u32 {
    npc.wall_hugger.as_ref().map_or(4, |state| state.entity_flags)
}

fn advance(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    // State0 clears APPEAR, so Entity_NPC::Update skips the hide/Poof and runs
    // the crawl AI again in the same age-0 update.
    plain_npc::advance_with(npc, phase, random, world, births, ai, false)
}

fn step(direction: i32) -> WorldVector {
    let (x, y) = DIRECTIONS[direction.rem_euclid(4) as usize];
    WorldVector { x: x * 40.0, y: y * 40.0 }
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    _: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    let mut state = npc
        .wall_hugger
        .take()
        .ok_or("Wall Hugger requires its AI state")?;
    let result = crawl(npc, &mut state, random, world, friction);
    npc.wall_hugger = Some(state);
    result
}

fn crawl(
    npc: &mut Npc,
    state: &mut WallHuggerState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.state == 0 {
        // 485770: subtype k > 0 starts in direction 4 - min(k, 4).
        state.direction = if npc.subtype == 0 {
            -1
        } else {
            4 - npc.subtype.min(4) as i32
        };
        state.outer_turned = false;
        state.init_counter = world.game_counter;
        state.entity_flags = (state.entity_flags | 1) & !4;
        npc.target = ZERO;
        npc.flip_x = true;
        npc.invincible = true;
        npc.state = 4;
        return Ok(());
    }
    // 485c00: harmless "No-Spikes" for 30 game frames, then "Spikes".
    let age = world.game_counter.wrapping_sub(state.init_counter);
    if age <= 29 {
        plain_npc::play(npc, "No-Spikes", false);
    } else {
        if age == 30 {
            world.audio.admit(
                random,
                world.system_counter,
                SoundRequest {
                    id: 205,
                    volume: 1.5,
                    delay: 2,
                    looping: false,
                    pitch: 1.0,
                },
            );
        }
        plain_npc::play(npc, "Spikes", false);
    }
    if state.direction < 0 {
        // 485aa0: the first solid orthogonal neighbour, else a random side.
        state.direction = (0..4)
            .find(|&side| world.grid.collision_at(npc.position + step(side)) != 0)
            .unwrap_or_else(|| random.bounded(4) as i32);
    }
    *friction *= 0.6;
    if npc.target == ZERO {
        choose(npc, state, world);
    }
    if npc.target != ZERO {
        // 48572c: push toward the target at 3/friction, then centre on the
        // cross axis.
        let delta = WorldVector {
            x: npc.position.x - npc.target.x,
            y: npc.position.y - npc.target.y,
        };
        let length = (delta.x * delta.x + delta.y * delta.y).sqrt();
        if 10.0 > length {
            npc.target = ZERO;
        }
        let k = 3.0 / length;
        let push = WorldVector {
            x: (delta.x * k) / *friction,
            y: (delta.y * k) / *friction,
        };
        npc.velocity = WorldVector {
            x: npc.velocity.x - push.x,
            y: npc.velocity.y - push.y,
        };
        if npc.target != ZERO {
            if delta.y.abs() > delta.x.abs() {
                npc.velocity.x = 0.6 * npc.velocity.x + (npc.target.x - npc.position.x) * 0.4;
            } else {
                npc.velocity.y = 0.6 * npc.velocity.y + (npc.target.y - npc.position.y) * 0.4;
            }
        }
    }
    Ok(())
}

/// 4857e0: keep the wall on side direction-1. An outer corner turns toward
/// the wall once; a blocked cell ahead turns away; otherwise target the next
/// cell centre.
fn choose(npc: &mut Npc, state: &mut WallHuggerState, world: &NpcWorld) {
    let ahead = step(state.direction);
    let side = step(state.direction - 1);
    let behind_side = world.grid.collision_at(npc.position + side + ahead * -1.0);
    let beside = world.grid.collision_at(npc.position + side);
    if beside == 0 && behind_side != 0 && !state.outer_turned {
        state.outer_turned = true;
        state.direction -= 1;
    } else {
        let index = world.grid.index(npc.position + ahead);
        let blocked = index.map_or(4, |at| world.grid.collision(at)) != 0;
        if blocked {
            state.direction += 1;
        } else if let Some(center) = index.and_then(|at| world.grid.position(at)) {
            npc.target = center;
            state.outer_turned = false;
        }
    }
    state.direction = state.direction.rem_euclid(4);
}
