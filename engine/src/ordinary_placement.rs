//! Source5C7F30 EDX0 reads ordinary manager positions for types4/5/6.
//! Completeness is history, independent of the current actor roster.
use crate::{actors::{ActorKind, ActorRef}, motion::WorldVector, State};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase", deny_unknown_fields)]
pub enum Frontier {
    Factory { entity_type: u32, variant: u32, subtype: u32 },
    ActorEffects { actor: ActorRef },
    CollectibleEffects { collectible_id: u32 },
    PocketEffects,
    ActiveEffects,
    PlayerDamageEffects,
    GridEffects,
    ChestEffects,
    Restore,
    MissingBody { actor: ActorRef },
    NonfinitePosition { actor: ActorRef },
    PlayerUpdate { field: String },
    MissingHistory,
}

///Room coverage and retained Player producer history have separate stores.
///Only their owned combination admits a complete ordinary4/5/6 query.
///Omitted legacy state is unknown in either domain.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Coverage {
    pub frontier: Option<Frontier>,
}
impl Coverage {
    ///The actual fresh Player begins before any item/effect producer history.
    pub(crate) fn fresh_player() -> Self { Self { frontier: None } }
    ///4524F0's retained pool admits only1/3/8sub0.453EC0 clears ordinary74
    ///and copies that pool, proving an empty relevant list at actual reset.
    pub(crate) fn after_fresh_reset() -> Self { Self { frontier: None } }
    pub(crate) fn invalidate(&mut self, frontier: Frontier) {
        self.frontier.get_or_insert(frontier);
    }
}

pub(crate) fn invalidate(state: &mut State, frontier: Frontier) {
    if let Some(coverage) = &mut state.room.ordinary_placement {
        coverage.invalidate(frontier.clone());
    }
    if let Some(owner) = &mut state.player_items { remember_owner(owner, frontier); }
}

///An unknown effect footprint may leave a retained producer after the room
///itself clears. Only a source-proven room-only footprint may omit this owner.
pub(crate) fn remember_owner(owner: &mut crate::player_items::PlayerItems, frontier: Frontier) {
    if let Some(history) = &mut owner.ordinary_producer_history { history.invalidate(frontier); }
}

pub(crate) fn begin_fresh_room(state: &mut State) {
    let mut coverage = Coverage::after_fresh_reset();
    match state.player_items.as_ref().and_then(|owner| owner.ordinary_producer_history.as_ref()) {
        Some(history) => {
            if let Some(frontier) = &history.frontier { coverage.invalidate(frontier.clone()); }
        }
        None => coverage.invalidate(Frontier::MissingHistory),
    }
    state.room.ordinary_placement = Some(coverage);
}

pub(crate) fn remember_room_frontier(state: &mut State) {
    if let Some(frontier) = state.room.ordinary_placement.as_ref().and_then(|coverage| coverage.frontier.clone()) {
        if let Some(owner) = &mut state.player_items { remember_owner(owner, frontier); }
    }
}

///An admitted gameplay pass through a legacy/unclosed Room cannot later
///become retained-producer proof merely because the next Room resets.
pub(crate) fn observe_room_boundary(state: &mut State) {
    if state.room.loaded && state.room.ordinary_placement.is_none() {
        if let Some(owner) = &mut state.player_items { remember_owner(owner, Frontier::MissingHistory); }
    }
}

///No31F/dead, active, collision-class or current-position prefilter is owned
///by this source scan. Background actors are already absent from this array.
pub fn positions(state: &State) -> Result<Vec<WorldVector>, Frontier> {
    let retained = state.player_items.as_ref()
        .and_then(|owner| owner.ordinary_producer_history.as_ref())
        .ok_or(Frontier::MissingHistory)?;
    if let Some(frontier) = &retained.frontier { return Err(frontier.clone()) }
    let coverage = state.room.ordinary_placement.as_ref().ok_or(Frontier::MissingHistory)?;
    if let Some(frontier) = &coverage.frontier { return Err(frontier.clone()) }
    let mut positions = Vec::new();
    for actor in state.actor_scheduler.full_phase() {
        let position = match actor.kind {
            ActorKind::Bomb => Some(state.bombs.iter().find(|body| body.id == actor.id)
                .map(|body| body.motion.position).ok_or(Frontier::MissingBody { actor })?),
            ActorKind::Pickup => Some(state.room.pickups.iter().find(|body| body.id == actor.id)
                .map(|body| body.position).ok_or(Frontier::MissingBody { actor })?),
            _ => None,
        };
        if let Some(position) = position {
            if !position.is_finite() { return Err(Frontier::NonfinitePosition { actor }) }
            positions.push(position);
        }
    }
    Ok(positions)
}

/// A reached Player update needs the precise producer readset, not a list of
/// neutral collectible IDs. This preserves only existing list/history proof.
/// Constructor status defaults are covered by that retained proof; actual
/// damage, unowned effects, and unclosed factories invalidate it at their
/// writer boundary instead of reconstructing a clean state from this query.
pub(crate) fn player_update_frontier(
    state: &State,
    full: bool,
    shoot_held: bool,
    bomb_callback_owned: bool,
) -> Option<Frontier> {
    let unknown = |field: &str| Some(Frontier::PlayerUpdate { field: field.into() });
    let Some(owner) = state.player_items.as_ref() else {
        return unknown("actual Player items");
    };
    if !state.player.entity_source.is_some_and(|source|
        source.entity_type == 1 && source.variant == 0) {
        return unknown("normal alive Player source");
    }
    if state.player.dead { return unknown("reached Player death/status callback"); }
    if owner.owned_counts.len() != 347 { return unknown("actual permanent347 counts"); }
    if owner.active_state_d80 != 0 {
        return unknown("released Player weapon/movement D80=0 subtree");
    }
    if state.weapon_cache.primary != crate::weapon_plan::PrimaryWeaponC08::RegularTears {
        return unknown("released ordinary Tear C08 subtree");
    }
    if shoot_held {
        return unknown("reached weapon producer");
    }
    //529750 and52F370 test record PRESENCE. A zero-count record is present.
    for id in [93, 77, 298, 299, 58, 313] {
        if owner.temporary_collectibles.iter().any(|record| record.item_id == id) {
            return unknown("reached TEMP movement/aim producer subtree");
        }
    }
    //The source BL=false519FC0 path merges both Have217 outcomes before
    //519FEF. With Have69/316/222 false it never reaches projectile creation.
    for id in [69, 316, 222, 243] {
        if owner.owned_counts[id] > 0 {
            return unknown("reached released weapon producer subtree");
        }
    }
    //509540 returns before RNG/Spawn4 for an untriggered actual action8.
    //A positive branch consumes its actual Bomb owner's outcome, passed by
    //the caller; this query does not synthesize another placement receipt.
    if !bomb_callback_owned {
        return unknown("unclosed admitted Bomb placement");
    }
    if !full { return None; }
    //5E3EF0 calls517580 only on an actual cooldown1→0 transition. Records
    //with cooldown<=0 (including unrelated persistent TEMP45) do not expire.
    if owner.temporary_collectibles.iter().any(|record| record.cooldown == 1) {
        return unknown("reached TEMP expiry callback517580");
    }
    //515F40's selected source domain. This is an operator readset, not an
    //inventory restriction; unrelated actual permanent/TEMP entries are fine.
    for id in [0, 65, 160, 77, 299, 93, 298, 214, 326] {
        if owner.temporary_collectibles.iter().any(|record| record.item_id == id) {
            return unknown("reached TEMP status producer subtree");
        }
    }
    for id in [326, 218, 231, 299, 302, 308] {
        if owner.owned_counts[id] > 0 {
            return unknown("reached permanent status producer subtree");
        }
    }
    //Empty canonical E14/E18 plus an actually nontrinket/empty queue makes
    //Have25/Have16 false without reaching Paper21's independent age owner.
    if owner.trinket_slots != [0, 0] || owner.queued_id.is_some_and(|id|
        crate::items::definition(id).is_none_or(|config| config.kind == 2)) {
        return unknown("actual HaveTrinket25/16 source context");
    }
    //531ED5 calls an effect helper only on an actual C70 expiry1→0.
    if state.attack_runtime.aim_rotation_c70.is_none_or(|counter| counter == 1) {
        return unknown("reached C70 expiry predicate");
    }
    //This is Seeds+48, not ChallengeID/systemCounter. The canonical lookup
    //owner comes from5DFB70 at5E1498→5E149D during actual seed admission.
    match state.run_seeds.as_ref().and_then(|seeds| seeds.source_effect_mode) {
        Some(1 | 2 | 3 | 6 | 37 | 14 | 53 | 24 | 26 | 27 | 28) | None =>
            return unknown("reached source seed-effect Player tail"),
        Some(_) => {}
    }
    None
}
