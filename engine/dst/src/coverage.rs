//! A census of what the `game` workload reached, written to every report.
//!
//! Must-hit properties bind the branches that matter; the census shows the
//! rest, so a coverage gap (a room type never entered, an implemented NPC
//! family never simulated) is visible in every run rather than only to a
//! probe. It is a measurement, not a property: nothing here passes or fails.
//!
//! Each case also records the command at which it first reached every key,
//! so `tools/dst/cover.py` can cut the shortest tape prefixes that bring a
//! missing key into the corpus, and `tools/dst/chase.py` can branch from
//! where a case first reached one. A property the case met is the key
//! `property:<message>`, so a must-hit a corpus no longer meets is cut back
//! the same way.
use rebirth_cleanroom::{implemented_npc_kinds, items, room_assets, Screen, State};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::Mutex;

struct Census {
    cases: u64,
    /// Cases that reached each key.
    reached: BTreeMap<String, u64>,
    /// Cases whose floors contained each room type.
    floor_reached: BTreeMap<u32, u64>,
    /// Each case's label with the command index that first reached each key.
    first_steps: Vec<(String, BTreeMap<String, u64>)>,
}

/// What the case running on this thread reached, and at which command.
#[derive(Default)]
struct Current {
    keys: BTreeMap<String, u64>,
    /// Room types the case's floors contained.
    floor_rooms: BTreeSet<u32>,
    /// The live NPCs (id to kind) of the room the last observation saw, by
    /// floor and room.
    live: Option<((u32, usize), BTreeMap<u32, u32>)>,
    /// Whether a `game` case is running; other units' properties are not
    /// part of the census.
    active: bool,
    /// Property sites (by address) already met, checked before formatting.
    met: HashSet<usize>,
    /// The highest floor stage the case has played.
    deepest: u32,
    /// Whether the case reached the menus after playing a deeper stage.
    menus_after_descent: bool,
}

static CENSUS: Mutex<Census> = Mutex::new(Census {
    cases: 0,
    reached: BTreeMap::new(),
    floor_reached: BTreeMap::new(),
    first_steps: Vec::new(),
});

thread_local! {
    static CURRENT: RefCell<Current> = RefCell::new(Current::default());
}

fn census() -> std::sync::MutexGuard<'static, Census> {
    CENSUS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn begin_case() {
    CURRENT.with(|current| {
        *current.borrow_mut() = Current {
            active: true,
            ..Current::default()
        }
    });
}

/// A property call site the running case met (see `props::Site::record`).
pub fn met(site: usize, message: &'static str) {
    CURRENT.with(|current| {
        let mut current = current.borrow_mut();
        if current.active && current.met.insert(site) {
            let step = crate::props::step();
            current
                .keys
                .entry(format!("property:{message}"))
                .or_insert(step);
        }
    });
}

/// Folds the case into the census under `label`, its retained tape's name.
pub fn end_case(label: &str) {
    let current = CURRENT.with(|current| std::mem::take(&mut *current.borrow_mut()));
    let mut census = census();
    census.cases += 1;
    for key in current.keys.keys() {
        *census.reached.entry(key.clone()).or_default() += 1;
    }
    for room_type in current.floor_rooms {
        *census.floor_reached.entry(room_type).or_default() += 1;
    }
    census.first_steps.push((label.to_owned(), current.keys));
}

/// Records what this published state shows the workload reaching.
pub fn observe(state: &State) {
    let mut keys = vec![format!("screen:{:?}", state.screen)];
    if state.screen == Screen::Playing {
        // The floor's stage: a descent shows as `stage:2`, and a run started
        // after one as `new run after a descent`, from the menus also as
        // `new run from the menus after a descent`.
        keys.push(format!("stage:{}", state.floor.stage));
        let (deepest, menus) = CURRENT.with(|current| {
            let mut current = current.borrow_mut();
            current.deepest = current.deepest.max(state.floor.stage);
            (current.deepest, current.menus_after_descent)
        });
        if state.floor.stage == 1 && deepest > 1 {
            keys.push("new run after a descent".to_owned());
            if menus {
                keys.push("new run from the menus after a descent".to_owned());
            }
        }
        if let Some(room) = state.floor.descriptors.get(state.current_room) {
            let template = &room.template;
            keys.push(if (template.width, template.height) == (13, 7) {
                format!("room:{}", template.room_type)
            } else {
                format!(
                    "room:{}({}x{})",
                    template.room_type, template.width, template.height
                )
            });
            if state.room.combat_clear && matches!(template.room_type, 5 | 6) {
                keys.push(format!("cleared:{}", template.room_type));
            }
        }
        for npc in state.room.npcs.iter().filter(|npc| !npc.removed) {
            keys.push(format!("npc:{}", npc.kind));
            // A Fireplace dies by going out (state 3, NoFire) and stays put.
            if npc.kind == 33 && npc.state == 3 {
                keys.push("extinguished:33".to_owned());
            }
        }
        // Removal: an NPC left a room the player stayed in, killed or retired
        // by its family. Removed bodies are pruned within the update that
        // removes them, so a batched tick hides the death but not the
        // absence.
        let room = (state.floor.stage_seed, state.current_room);
        let live: BTreeMap<u32, u32> = state
            .room
            .npcs
            .iter()
            .filter(|npc| !npc.removed)
            .map(|npc| (npc.id, npc.kind))
            .collect();
        CURRENT.with(|current| {
            let mut current = current.borrow_mut();
            if state.room_transition.is_some() {
                // A transition swaps the room's NPCs; compare after it.
                current.live = None;
                return;
            }
            if let Some((seen, before)) = &current.live {
                if *seen == room {
                    keys.extend(
                        before
                            .iter()
                            .filter(|(id, _)| !live.contains_key(id))
                            .map(|(_, kind)| format!("removed:{kind}")),
                    );
                }
            }
            current.live = Some((room, live));
        });
        for pickup in state.room.pickups.iter().filter(|pickup| !pickup.removed) {
            keys.push(format!("pickup:{}", pickup.variant));
        }
        // Owned collectibles that re-resolve the primary weapon (cache 0x100).
        if let Some(items) = &state.player_items {
            for (id, count) in items.owned_counts.iter().enumerate() {
                let weapon = items::definition(id as u32)
                    .is_some_and(|config| config.cache_flags & 0x100 != 0);
                if *count > 0 && weapon {
                    keys.push(format!("weapon:{id}"));
                }
            }
        }
        let features = [
            (
                "collectible owned",
                state
                    .player_items
                    .as_ref()
                    .is_some_and(|items| items.owned_counts.iter().any(|count| *count > 0)),
            ),
            (
                "active item held",
                state
                    .player_items
                    .as_ref()
                    .is_some_and(|items| items.active_id != 0),
            ),
            (
                "secret door revealed",
                !state.revealed_secret_connections.is_empty(),
            ),
            (
                "boss below half health",
                state
                    .room
                    .npcs
                    .iter()
                    .any(|npc| npc.boss && !npc.removed && npc.hp < npc.max_hp / 2.0),
            ),
            ("laser", !state.lasers.is_empty()),
            ("knife", !state.knives.is_empty()),
            ("familiar", !state.familiars.is_empty()),
            ("explosion", !state.explosions.is_empty()),
            ("coins held", state.inventory.coins > 0),
            ("keys held", state.inventory.keys > 0),
            ("paused", state.paused),
            ("game over", state.player_death.game_over.is_some()),
        ];
        keys.extend(
            features
                .iter()
                .filter(|(_, reached)| *reached)
                .map(|(key, _)| (*key).to_owned()),
        );
    }
    CURRENT.with(|current| {
        let mut current = current.borrow_mut();
        if state.screen != Screen::Playing {
            current.live = None;
            current.menus_after_descent |= current.deepest > 1;
        }
        if state.screen == Screen::Playing {
            let rooms = state.floor.descriptors.iter();
            current
                .floor_rooms
                .extend(rooms.map(|room| room.template.room_type));
        }
        let step = crate::props::step();
        for key in keys {
            current.keys.entry(key).or_insert(step);
        }
    });
}

/// Entity types some authored Basement or special room spawns: what a
/// floor can place directly. Other kinds appear only when spawned at runtime.
fn authored_entity_types() -> BTreeSet<u32> {
    room_assets::basement()
        .iter()
        .chain(room_assets::special().iter())
        .flat_map(|asset| asset.spawns.iter())
        .flat_map(|group| group.entries.iter())
        .map(|entry| u32::from(entry.entity_type))
        .collect()
}

/// The census for the report, and what the workload never reached: NPC kinds
/// the engine implements (split by whether any authored room places them)
/// and room types its floors contained.
pub fn report() -> Value {
    let census = census();
    let authored = authored_entity_types();
    let (unreached_npcs, unplaced_npcs): (Vec<u32>, Vec<u32>) = implemented_npc_kinds()
        .filter(|kind| !census.reached.contains_key(&format!("npc:{kind}")))
        .partition(|kind| authored.contains(kind));
    // Simulated kinds whose death or retirement never ran.
    let unremoved_npcs: Vec<u32> = implemented_npc_kinds()
        .filter(|kind| {
            census.reached.contains_key(&format!("npc:{kind}"))
                && !census.reached.contains_key(&format!("removed:{kind}"))
                && !census.reached.contains_key(&format!("extinguished:{kind}"))
        })
        .collect();
    let unentered_rooms: Vec<u32> = census
        .floor_reached
        .keys()
        .copied()
        .filter(|room_type| {
            !census.reached.keys().any(|key| {
                key == &format!("room:{room_type}")
                    || key.starts_with(&format!("room:{room_type}("))
            })
        })
        .collect();
    let mut first_steps = census.first_steps.clone();
    first_steps.sort();
    let first_steps: BTreeMap<String, BTreeMap<String, u64>> = first_steps.into_iter().collect();
    json!({
        "gameCases": census.cases,
        "firstSteps": first_steps,
        "reached": census.reached,
        "floorRoomTypes": census.floor_reached,
        "unreachedNpcKinds": unreached_npcs,
        "unplacedNpcKinds": unplaced_npcs,
        "unremovedNpcKinds": unremoved_npcs,
        "unenteredRoomTypes": unentered_rooms,
    })
}
