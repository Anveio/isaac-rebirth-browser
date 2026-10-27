//! Player52F000: action11 swaps whole pocket records and counts actual
//! Player callbacks on both presented phases. It is independent of action10.
use crate::{
    actors::{ActorKind, ActorRef},
    motion::WorldVector,
    State,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Frontier {
    PlayerVariant,
    InputDevice,
    HeldCounter,
    HeldRecords,
    Capacity,
    TrinketRemoval,
    QueuedFlush,
    PendingCache,
    Placement {
        frontier: crate::ordinary_placement::Frontier,
    },
    MissingGrid,
    PocketInit,
}

///Source words, not a timer derived from elapsed frames. In particular,
///519350 has no established3054 initializer: only52F220's released callback
///owns its first zero. A legacy restore never receives a fresh history.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct History {
    pub source_d98: Option<i32>,
    pub held_callbacks_3054: Option<u32>,
    pub source_30d0: Option<u32>,
    pub frontier: Option<Frontier>,
    ///A reached52F137 store, not an actual current BE8 value. Later Player
    ///writers and Gnawed Leaf210's53191C consumer retain their own owners.
    pub player_be8_store_reached: bool,
    pub gnawed_leaf_frontier: bool,
}
impl History {
    pub(crate) fn fresh(source_d98: Option<i32>) -> Self {
        Self {
            source_d98,
            held_callbacks_3054: None,
            //519BAB independently initializes this word, not3054.
            source_30d0: Some(0),
            frontier: None,
            player_be8_store_reached: false,
            gnawed_leaf_frontier: false,
        }
    }
}

fn refuse(state: &mut State, frontier: Frontier, possible_drop: bool) {
    if let Some(history) = state.player.pocket_actions.as_mut() {
        history.frontier.get_or_insert(frontier);
    }
    if possible_drop {
        //An unclosed helper may grant/convert/drop records before returning.
        //Keep their last values, but never advertise them as current proof.
        if let Some(held) = state
            .player_items
            .as_mut()
            .and_then(|owner| owner.held_pockets.as_mut())
        {
            held.frontier
                .get_or_insert(crate::pockets::PocketFrontier::DropAction);
        }
        crate::ordinary_placement::invalidate(
            state,
            crate::ordinary_placement::Frontier::PocketEffects,
        );
    }
    if !state.controls_menu.unavailable_actions.contains(&11) {
        state.controls_menu.unavailable_actions.push(11);
    }
}

///Readsets for the admitted52EEC0 arm. Positive Trinkets (including53),
///unknown slots, a real queued config and pending511F10 work stay explicit.
///This gate runs before any speculative shared draw or world mutation.
fn empty_trinket_readset(state: &State) -> Result<(), Frontier> {
    let owner = state
        .player_items
        .as_ref()
        .ok_or(Frontier::TrinketRemoval)?;
    if !owner
        .query_history
        .as_ref()
        .is_some_and(|history| history.trinket_slots)
        || owner.trinket_slots != [0, 0]
    {
        return Err(Frontier::TrinketRemoval);
    }
    if !owner.queue_is_empty() {
        return Err(Frontier::QueuedFlush);
    }
    if owner.unevaluated_cache_flags != 0 {
        return Err(Frontier::PendingCache);
    }
    Ok(())
}

fn free_position(state: &mut State, anchor: WorldVector) -> Result<WorldVector, Frontier> {
    let occupied = crate::ordinary_placement::positions(state)
        .map_err(|frontier| Frontier::Placement { frontier })?;
    let grid = crate::grid::Grid::new(state.room.grid.clone().ok_or(Frontier::MissingGrid)?)
        .map_err(|_| Frontier::MissingGrid)?;
    let costs = state
        .room
        .path_costs
        .as_ref()
        .ok_or(Frontier::MissingGrid)?;
    crate::roundworm::find_free_position(
        anchor,
        0.0,
        &mut state.boot_lifecycle.shared,
        &grid,
        costs,
        &occupied,
    )
    .map_err(|_| Frontier::MissingGrid)
}

///The actual52EEC0 empty-slot arm still calls52EE40. Only after that
///callback does it test E14/E18 and D6C and take its no-removal return.
fn remove_empty_trinket(state: &mut State) -> Result<(), String> {
    crate::player_items::flush_queued(state)?;
    empty_trinket_readset(state)
        .map_err(|frontier| format!("pocket drop lost its preflight: {frontier:?}"))
}

fn drop_pockets(state: &mut State, anchor: WorldVector) -> Result<(), String> {
    let Some(owner) = state.player_items.as_ref() else {
        refuse(state, Frontier::HeldRecords, true);
        return Ok(());
    };
    let Some(held) = owner
        .held_pockets
        .as_ref()
        .filter(|held| held.frontier.is_none())
    else {
        refuse(state, Frontier::HeldRecords, true);
        return Ok(());
    };
    let Some(capacity) = crate::pockets::capacity(state) else {
        refuse(state, Frontier::Capacity, true);
        return Ok(());
    };
    if !owner
        .query_history
        .as_ref()
        .is_some_and(|history| history.trinket_slots)
    {
        refuse(state, Frontier::TrinketRemoval, true);
        return Ok(());
    }
    //52F0CE..127 tests actual held Trinkets and capacity-visible pocket IDs.
    //A known empty hold resets3054 but performs no placement/helper calls.
    let trinket_drop = owner.trinket_slots.iter().any(|id| *id != 0 && *id != 53);
    if !trinket_drop && held.slots[..capacity].iter().all(|slot| slot.id == 0) {
        return Ok(());
    }
    let gnawed_leaf = crate::item_queries::AppearanceQueries::new(
        owner,
        state.weapon_cache.primary,
        state.player.entity_source,
        None,
    )
    .have(210);
    let history = state.player.pocket_actions.as_mut().expect("drop history");
    //52F137 precedes BOTH free-position/removal passes, even when a later
    //helper cannot be completed. This marker certifies no current BE8 byte.
    history.player_be8_store_reached = true;
    history.gnawed_leaf_frontier |= gnawed_leaf != Some(false);
    if let Err(frontier) = empty_trinket_readset(state) {
        //An unimplemented queued grant/removal can change the later Have210
        //readset; its prehelper negative cannot qualify the consumer.
        state
            .player
            .pocket_actions
            .as_mut()
            .expect("drop history")
            .gnawed_leaf_frontier = true;
        refuse(state, frontier, true);
        return Ok(());
    }
    //Prepare on a clone: a missing placement or Init contract cannot consume
    //a prefix of the shared stream and then pretend the rest completed.
    let draw_log = crate::mt::DrawLogTransaction::begin();
    let mut planned = state.clone();
    for _ in 0..2 {
        if let Err(frontier) = free_position(&mut planned, anchor) {
            refuse(state, frontier, true);
            return Ok(());
        }
        remove_empty_trinket(&mut planned)?;
    }
    for slot_index in 0..capacity {
        let slot = planned
            .player_items
            .as_ref()
            .expect("drop owner")
            .held_pockets
            .as_ref()
            .expect("drop records")
            .slots[slot_index];
        if slot.id != 0 {
            //52F280 consumes the seed BEFORE this third/later FindFree call.
            let seed = planned.boot_lifecycle.shared.next_u32();
            let position = match free_position(&mut planned, anchor) {
                Ok(position) => position,
                Err(frontier) => {
                    refuse(state, frontier, true);
                    return Ok(());
                }
            };
            let id = planned.next_pickup;
            let Some(mut pickup) = crate::Pickup::from_pocket_parts(
                id,
                if slot.card { 300 } else { 70 },
                slot.id,
                seed,
                position,
                true,
                crate::pickups::PocketSubtypeInputs::from_state(&planned),
            )?
            else {
                refuse(state, Frontier::PocketInit, true);
                return Ok(());
            };
            pickup.birth_game_update = planned.game_update_count;
            pickup.source_spawner = Some(crate::pickups::SourceSpawner {
                entity_type: 0,
                variant: 0,
            });
            pickup.pocket_contact_delay = Some(4);
            planned.next_pickup = id.checked_add(1).ok_or("pocket drop identity exhausted")?;
            planned.room.pickups.push(pickup);
            let actor = ActorRef {
                kind: ActorKind::Pickup,
                id,
            };
            planned.room.authored_actor_order.push(actor);
            planned.actor_scheduler.admit(actor)?;
        }
        //52F25A clears only ID. The Card/Pill tag survives an empty record.
        planned
            .player_items
            .as_mut()
            .expect("drop owner")
            .held_pockets
            .as_mut()
            .expect("drop records")
            .slots[slot_index]
            .id = 0;
    }
    *state = planned;
    draw_log.commit();
    Ok(())
}

///Caller5318EE follows the actual shooting/DB4 callback. The position is the
///post-Base, pre-grid position; later movement in the port cannot relocate its
///source FindFreePosition anchor. No controls/cooldown/action10 gate exists
///inside52F000 itself, and phase interpolation does not halve this counter.
pub(crate) fn advance(state: &mut State, anchor: WorldVector) -> Result<(), String> {
    let held = state.public_actions.held[11];
    let triggered = state.public_actions.triggered[11];
    match state.player.entity_source {
        Some(source) if source.variant != 0 => return Ok(()),
        Some(source) if source.entity_type == 1 => {}
        _ => {
            if held || triggered {
                refuse(state, Frontier::PlayerVariant, true);
            }
            return Ok(());
        }
    }
    let Some(history) = state.player.pocket_actions.as_ref() else {
        if held || triggered {
            refuse(state, Frontier::InputDevice, true);
        }
        return Ok(());
    };
    if history.source_d98 != Some(0) {
        if held || triggered {
            refuse(state, Frontier::InputDevice, true);
        }
        return Ok(());
    }
    if triggered {
        let records = state
            .player_items
            .as_mut()
            .and_then(|owner| owner.held_pockets.as_mut());
        match records.filter(|records| records.frontier.is_none()) {
            Some(records) if records.slots[1].id != 0 => records.slots.swap(0, 1),
            Some(_) => {}
            None => refuse(state, Frontier::HeldRecords, true),
        }
    }
    let history = state
        .player
        .pocket_actions
        .as_mut()
        .expect("action history");
    if !held {
        history.held_callbacks_3054 = Some(0);
        return Ok(());
    }
    let Some(old) = history.held_callbacks_3054 else {
        refuse(state, Frontier::HeldCounter, true);
        return Ok(());
    };
    let next = old.wrapping_add(1);
    history.held_callbacks_3054 = Some(next);
    if next <= 120 {
        history.source_30d0 = Some(0);
        return Ok(());
    }
    history.held_callbacks_3054 = Some(0);
    if history.frontier.is_some() {
        return Ok(());
    }
    drop_pockets(state, anchor)
}
