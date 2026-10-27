//! Reached Add194/251/252 operators share ordinary placement and Pickup Init.
//! Current bodies and positions remain owned by the scheduler and room.
use crate::{
    actors::{ActorKind, ActorRef},
    motion::WorldVector,
    State,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Frontier {
    MissingHistory,
    Input {
        field: String,
    },
    Placement {
        frontier: crate::ordinary_placement::Frontier,
    },
    Operator {
        message: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UnownedEffect {
    pub collectible_id: u32,
    pub frontier: Frontier,
}

/// This is provenance of actual unclosed effects and conditional RNG writes.
/// It stores no inventory, entity roster, positions, or derived occupancy.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct History {
    pub first_unowned_effect: Option<UnownedEffect>,
    pub item_rng_frontiers: BTreeMap<u32, Frontier>,
}

impl History {
    pub(crate) fn fresh() -> Self {
        Self {
            first_unowned_effect: None,
            item_rng_frontiers: BTreeMap::new(),
        }
    }
}

struct PlacementPlan {
    position: WorldVector,
    shared: crate::mt::Mt19937,
}

struct AdmissionPlan {
    pickup: crate::Pickup,
    actor: ActorRef,
    next_pickup: u32,
    scheduler: crate::actors::Scheduler,
}

fn input(field: &str) -> Frontier {
    Frontier::Input {
        field: field.into(),
    }
}
fn operator(message: String) -> Frontier {
    Frontier::Operator { message }
}

fn prepare_placement(state: &State) -> Result<PlacementPlan, Frontier> {
    let occupied = crate::ordinary_placement::positions(state)
        .map_err(|frontier| Frontier::Placement { frontier })?;
    let grid = crate::grid::Grid::new(state.room.grid.clone().ok_or_else(|| input("Room grid"))?)
        .map_err(operator)?;
    let costs = state
        .room
        .path_costs
        .as_ref()
        .ok_or_else(|| input("Room E98 costs"))?;
    let anchor = crate::bombs::player_world(state);
    if !anchor.is_finite() {
        return Err(input("finite Player448"));
    }
    let mut shared = state.boot_lifecycle.shared.clone();
    let position =
        crate::roundworm::find_free_position(anchor, 40.0, &mut shared, &grid, costs, &occupied)
            .map_err(operator)?;
    Ok(PlacementPlan { position, shared })
}

pub(crate) fn next_item_seed(state: &mut State, id: u32) -> Result<u32, Frontier> {
    let owner = state
        .player_items
        .as_mut()
        .ok_or_else(|| input("Player items"))?;
    let history = owner
        .pickup_grant_history
        .as_ref()
        .ok_or(Frontier::MissingHistory)?;
    if let Some(frontier) = history.item_rng_frontiers.get(&id) {
        return Err(frontier.clone());
    }
    Ok(owner
        .item_rngs
        .get_mut(id as usize)
        .ok_or_else(|| input("collectible RNG"))?
        .raw())
}

fn get_card(state: &State, seed: u32, runes: bool) -> Result<u32, Frontier> {
    let profile = state
        .profiles
        .get(usize::from(state.profile).wrapping_sub(1))
        .ok_or_else(|| input("actual profile achievements"))?;
    crate::pickups::random_card(seed, &profile.achievements, true, runes).map_err(operator)
}

fn prepare_admission(
    state: &State,
    variant: u32,
    subtype: u32,
    seed: u32,
    position: WorldVector,
    spawner: crate::pickups::SourceSpawner,
) -> Result<AdmissionPlan, Frontier> {
    // This guard belongs to reached world Init, after the source seed/search
    // prefix. It must not erase independently known preceding RNG advances.
    if state.challenge_id == 33 {
        return Err(input("Challenge33 pickup pricing"));
    }
    let handle = state.next_pickup;
    let next_pickup = handle
        .checked_add(1)
        .ok_or_else(|| input("fresh pickup handle"))?;
    if state.room.pickups.iter().any(|pickup| pickup.id == handle) {
        return Err(input("unused pickup handle"));
    }
    let actor = ActorRef {
        kind: ActorKind::Pickup,
        id: handle,
    };
    let mut pickup = crate::Pickup::from_pocket_parts(
        handle,
        variant,
        subtype,
        seed,
        position,
        true,
        crate::pickups::PocketSubtypeInputs::from_state(state),
    )
    .map_err(operator)?
    .ok_or_else(|| input("canonical world Pickup Init"))?;
    pickup.birth_game_update = state.game_update_count;
    // Game.Spawn copies source1C/20 only. Weak parent498 is a separate owner.
    pickup.source_spawner = Some(spawner);
    pickup.morph_links = Some(crate::pickups::MorphLinks {
        parent498: None,
        link4a0: None,
        link4a8: None,
    });
    let mut scheduler = state.actor_scheduler.clone();
    scheduler.admit(actor).map_err(operator)?;
    Ok(AdmissionPlan {
        pickup,
        actor,
        next_pickup,
        scheduler,
    })
}

fn commit_admission(state: &mut State, plan: AdmissionPlan) {
    state.next_pickup = plan.next_pickup;
    state.room.pickups.push(plan.pickup);
    state.room.authored_actor_order.push(plan.actor);
    state.actor_scheduler = plan.scheduler;
}

fn remember_unowned(
    state: &mut State,
    id: u32,
    frontier: Frontier,
    local_rng_complete: bool,
    world_complete: bool,
) {
    if let Some(history) = state
        .player_items
        .as_mut()
        .and_then(|owner| owner.pickup_grant_history.as_mut())
    {
        //An unclosed local suffix makes the canonical final state unknown.
        //A later world Init gap does not erase completed local RNG calls.
        if !local_rng_complete {
            history
                .item_rng_frontiers
                .entry(id)
                .or_insert_with(|| frontier.clone());
        }
        history.first_unowned_effect.get_or_insert(UnownedEffect {
            collectible_id: id,
            frontier,
        });
    }
    if !world_complete {
        crate::ordinary_placement::invalidate(
            state,
            crate::ordinary_placement::Frontier::CollectibleEffects { collectible_id: id },
        );
    }
}

/// Source52DF3F searches first. The two local Next calls are52DF50/52DF5B;
/// only the second seed selects the explicit Card. No held record is read.
fn card194(state: &mut State, local_rng_complete: &mut bool) -> Result<(), Frontier> {
    let placement = prepare_placement(state)?;
    state.boot_lifecycle.shared = placement.shared;
    let world_seed = next_item_seed(state, 194)?;
    let card_seed = next_item_seed(state, 194)?;
    *local_rng_complete = true;
    let card = get_card(state, card_seed, false)?;
    let admission = prepare_admission(
        state,
        300,
        card,
        world_seed,
        placement.position,
        crate::pickups::SourceSpawner {
            entity_type: 0,
            variant: 0,
        },
    )?;
    commit_admission(state, admission);
    Ok(())
}

///194's search precedes its local draws;252 below has the reverse prefix.
///251's world Card and held-Pill conversion remain a separate consumer.
pub(crate) fn on_card194_added(state: &mut State) {
    match state.player.entity_source {
        Some(source) if source.entity_type == 1 && source.variant != 0 => return,
        Some(source) if source.entity_type == 1 => {}
        _ => {
            remember_unowned(state, 194, input("Player entity source"), false, false);
            return;
        }
    }
    let mut local_rng_complete = false;
    if let Err(frontier) = card194(state, &mut local_rng_complete) {
        remember_unowned(state, 194, frontier, local_rng_complete, false);
    }
}

///52DD33..52DDCB: Next252, free position, world Pill with a Player spawner,
///then52E132 converts each nonempty held Card with the next item252 seed.
fn little_baggy(
    state: &mut State,
    local_rng_complete: &mut bool,
    world_complete: &mut bool,
) -> Result<(), Frontier> {
    let world_seed = next_item_seed(state, 252)?;
    let placement = prepare_placement(state)?;
    state.boot_lifecycle.shared = placement.shared;
    let admission = (|| {
        let owner = state.player_items.as_ref().ok_or_else(|| input("Player items"))?;
        //Pill Init reads Have251 before selecting its family. Its actual
        //query owner is required even though the canonical Init takes counts.
        crate::item_queries::AppearanceQueries::new(
            owner,
            state.weapon_cache.primary,
            state.player.entity_source,
            None,
        )
        .have(251)
        .ok_or_else(|| input("Pill Init Have251"))?;
        prepare_admission(
            state,
            70,
            0,
            world_seed,
            placement.position,
            crate::pickups::SourceSpawner { entity_type: 1, variant: 0 },
        )
    })();
    let world_result = match admission {
        Ok(plan) => {
            commit_admission(state, plan);
            *world_complete = true;
            Ok(())
        }
        Err(frontier) => Err(frontier),
    };
    //World Init never mutates the Player's held records or item252 RNG.
    //Its unavailable result therefore does not erase a closed held suffix.
    let slots = state.player_items.as_ref()
        .and_then(|owner| owner.held_pockets.as_ref())
        .filter(|held| held.frontier.is_none())
        .ok_or_else(|| input("actual held pocket records"))?.slots;
    for (index, slot) in slots.into_iter().enumerate() {
        if slot.id == 0 || !slot.card {
            continue;
        }
        let seed = next_item_seed(state, 252)?;
        let held = state.player_items.as_mut()
            .and_then(|owner| owner.held_pockets.as_mut())
            .expect("checked held pocket records");
        //52E135 stores the Pill tag, then581950(seed) is unsigned %9+1.
        held.slots[index] = crate::pockets::PocketSlot { id: seed % 9 + 1, card: false };
    }
    *local_rng_complete = true;
    world_result
}

pub(crate) fn on_little_baggy_added(state: &mut State) {
    match state.player.entity_source {
        Some(source) if source.entity_type == 1 && source.variant != 0 => return,
        Some(source) if source.entity_type == 1 => {}
        _ => {
            if let Some(owner) = state.player_items.as_mut() {
                crate::pockets::note_modifier_grant(owner, 252);
            }
            remember_unowned(state, 252, input("Player entity source"), false, false);
            return;
        }
    }
    let (mut local_rng_complete, mut world_complete) = (false, false);
    if let Err(frontier) = little_baggy(state, &mut local_rng_complete, &mut world_complete) {
        if let Some(owner) = state.player_items.as_mut() {
            crate::pockets::note_modifier_grant(owner, 252);
        }
        remember_unowned(state, 252, frontier, local_rng_complete, world_complete);
    }
}
