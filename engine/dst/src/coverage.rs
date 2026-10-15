//! A census of what the `game` workload reached, written to every report.
//!
//! Must-hit properties bind the branches that matter; the census shows the
//! rest, so a coverage gap (a room type never entered, an implemented NPC
//! family never simulated) is visible in every run rather than only to a
//! probe. It is a measurement, not a property: nothing here passes or fails.
use rebirth_cleanroom::{implemented_npc_kinds, Screen, State};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

struct Census {
    cases: u64,
    /// Cases that reached each key.
    reached: BTreeMap<String, u64>,
    /// Keys the running case reached.
    current: BTreeSet<String>,
    /// Room types the running case's floors contained.
    floor_rooms: BTreeSet<u32>,
    /// Cases whose floors contained each room type.
    floor_reached: BTreeMap<u32, u64>,
}

static CENSUS: Mutex<Census> = Mutex::new(Census {
    cases: 0,
    reached: BTreeMap::new(),
    current: BTreeSet::new(),
    floor_rooms: BTreeSet::new(),
    floor_reached: BTreeMap::new(),
});

fn census() -> std::sync::MutexGuard<'static, Census> {
    CENSUS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn begin_case() {
    let mut census = census();
    census.current.clear();
    census.floor_rooms.clear();
}

pub fn end_case() {
    let mut census = census();
    census.cases += 1;
    let current = std::mem::take(&mut census.current);
    for key in current {
        *census.reached.entry(key).or_default() += 1;
    }
    let floor_rooms = std::mem::take(&mut census.floor_rooms);
    for room_type in floor_rooms {
        *census.floor_reached.entry(room_type).or_default() += 1;
    }
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
    let mut census = census();
    if state.screen == Screen::Playing {
        let rooms: Vec<u32> = state
            .floor
            .descriptors
            .iter()
            .map(|room| room.template.room_type)
            .collect();
        census.floor_rooms.extend(rooms);
    }
    census.current.extend(keys);
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
    json!({
        "gameCases": census.cases,
        "reached": census.reached,
        "floorRoomTypes": census.floor_reached,
        "unreachedNpcKinds": unreached_npcs,
        "unenteredRoomTypes": unentered_rooms,
    })
}
