//! A census of what the `game` workload reached, written to every report.
//!
//! Must-hit properties bind the branches that matter; the census shows the
//! rest, so a coverage gap (a room type never entered, an implemented NPC
//! family never simulated) is visible in every run rather than only to a
//! probe. It is a measurement, not a property: nothing here passes or fails.
//!
//! Each case also records the command at which it first reached every key,
//! so `tools/dst/cover.py` can cut the shortest tape prefixes that bring a
//! missing key into the corpus.
use rebirth_cleanroom::{implemented_npc_kinds, items, Screen, State};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
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
    CURRENT.with(|current| *current.borrow_mut() = Current::default());
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
        }
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

/// The census for the report, and what the workload never reached: NPC kinds
/// the engine implements and room types its floors contained.
pub fn report() -> Value {
    let census = census();
    let unreached_npcs: Vec<u32> = implemented_npc_kinds()
        .filter(|kind| !census.reached.contains_key(&format!("npc:{kind}")))
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
        "unenteredRoomTypes": unentered_rooms,
    })
}
