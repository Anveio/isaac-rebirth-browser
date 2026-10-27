//! Actual Player3044/3048 and304C/3050 records. Pool selection remains in
//! the canonical pickup Init; capacity, insertion and presentation are callers.
use crate::pickups::PlayerContactDecision;
use crate::{
    actors::{ActorKind, ActorRef},
    player_items, State,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PocketSlot {
    pub id: u32,
    pub card: bool,
}
impl PocketSlot {
    const EMPTY: Self = Self { id: 0, card: true };
    fn variant(self) -> u32 {
        if self.card {
            300
        } else {
            70
        }
    }
}

///These records retain the last actually owned contents. An unimplemented
///writer invalidates current ownership explicitly; it never consumes a pocket
///or manufactures the result of a Card/Pill callback.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum PocketFrontier {
    CardEffect { card_id: u32 },
    PillEffect { pill_color: u32 },
    ModifierConversion { collectible_id: u32 },
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeldPockets {
    pub slots: [PocketSlot; 2],
    pub frontier: Option<PocketFrontier>,
}
impl HeldPockets {
    pub(crate) fn fresh() -> Self {
        //5197C0/CA and5197D1/DB: ID0 and Card tag1 in both records.
        Self {
            slots: [PocketSlot::EMPTY; 2],
            frontier: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum PocketOverlay {
    Pills,
    CardFronts,
    Runes,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PocketPresentation {
    pub overlay: PocketOverlay,
    pub overlay_frame: u32,
}
impl PocketPresentation {
    fn for_slot(slot: PocketSlot) -> Self {
        let (overlay, requested_frame) = if !slot.card {
            (PocketOverlay::Pills, slot.id - 1)
        } else if (28..=35).contains(&slot.id) {
            (PocketOverlay::Runes, u32::from(slot.id > 31))
        } else {
            (
                PocketOverlay::CardFronts,
                if slot.id > 35 {
                    slot.id - 8
                } else {
                    slot.id - 1
                },
            )
        };
        //4103E0→40EB30 clamps the stored overlay clock even for Card40's
        //actual requested32. Keep its independent Sparkle main animation.
        let last = match overlay {
            PocketOverlay::Pills => 8,
            PocketOverlay::CardFronts => 31,
            PocketOverlay::Runes => 1,
        };
        Self {
            overlay,
            overlay_frame: requested_frame.min(last),
        }
    }
}

fn capacity(owner: &player_items::PlayerItems) -> Option<usize> {
    //50ADC0 short-circuits: Have251 true skips the Have252 operand.
    if *owner.owned_counts.get(251)? > 0 {
        Some(2)
    } else {
        Some(if *owner.owned_counts.get(252)? > 0 {
            2
        } else {
            1
        })
    }
}

///52DDA4/52DEF1 convert only a nonempty record of the opposite kind. The
///worldspawn/item-RNG conversion callback is tracked until implemented; empty
///or already-matching records remain owned, without inferring new contents.
pub(crate) fn note_modifier_grant(owner: &mut player_items::PlayerItems, id: u32) {
    let Some(held) = owner
        .held_pockets
        .as_mut()
        .filter(|held| held.frontier.is_none())
    else {
        return;
    };
    if held
        .slots
        .iter()
        .any(|slot| slot.id != 0 && ((id == 251 && !slot.card) || (id == 252 && slot.card)))
    {
        held.frontier = Some(PocketFrontier::ModifierConversion { collectible_id: id });
    }
}

///Unimplemented reached use owners remain separate from known held slots,
///the canonical Pill words and completed scalar callbacks. These flags never
///certify progression, time effects, player color or private callback entry.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PillUseFrontiers {
    pub player_be8: bool,
    pub persistent_counter5: bool,
    pub persistent_use43: bool,
    pub game_time_551dd0: bool,
    pub identified_color: bool,
    pub unresolved_name_mode: bool,
    pub range_up_color: bool,
}

///5282B0's known11/12 arm. Resolution did not create input ownership: an
///unknown prior counter remains unknown even though the actual use consumes
///the Pill. Cache completion is still the reached Range plan's own decision.
fn use_range_pill(state: &mut State, effect: u32) -> Result<(), String> {
    //5283D9 ->5283EE precede the effect/cache. Real Persistent callbacks now
    //own these words; every new award retains its actual inline dependencies.
    crate::persistent_effects::add_counter(state, 5, 1)?;
    crate::persistent_effects::mark_collected(state, 43)?;
    let displayed_name = match state
        .run_seeds
        .as_ref()
        .and_then(|seeds| seeds.source_effect_mode)
    {
        Some(29) => Some("???"),
        Some(9) | None => None,
        Some(_) => Some(
            crate::pickups::pill_definition(effect)
                .ok_or("Resolved Pill config is absent")?
                .name,
        ),
    };
    //528446's identification byte is an independent unimplemented owner.
    //Do not invent a fresh global store without its foreign-writer history.
    if let Some(name) = displayed_name {
        state.run_introduction = crate::RunIntroduction::pickup(name, "");
    }
    let owner = state.player_items.as_mut().expect("resolved Pill owner");
    let frontiers = owner
        .pill_use_frontiers
        .get_or_insert_with(Default::default);
    frontiers.player_be8 = true;
    frontiers.persistent_counter5 = false;
    frontiers.persistent_use43 = false;
    frontiers.game_time_551dd0 = true;
    frontiers.identified_color = true;
    frontiers.unresolved_name_mode |= displayed_name.is_none();
    frontiers.range_up_color |= effect == 12;
    //5284A3 increments before529003/528EC7 request the independent cache8.
    if let Some(history) = owner.pill_effect_history.as_mut() {
        history.increment_resolved(effect)?;
    }
    owner.unevaluated_cache_flags |= 8;
    crate::player_animation::pill_response(state, effect == 12);
    if effect == 12 {
        //528EF6 follows Happy268. Color13F8/timed tint remains explicit above.
        state.audio.admit(
            &mut state.boot_lifecycle.shared,
            state.system_clock.system_counter.wrapping_sub(1),
            crate::audio::SoundRequest {
                id: 132,
                volume: 1.0,
                delay: 2,
                looping: false,
                pitch: 1.0,
            },
        );
    }
    //5286D2 evaluates CURRENT inputs before5286D7 reads CURRENT slot1.
    //No cached number or zero pending endpoint is a callback-entry witness.
    crate::player_stats::evaluate_cache(state)?;
    let held = state
        .player_items
        .as_mut()
        .and_then(|owner| owner.held_pockets.as_mut())
        .ok_or("Pill effect lost its held-slot owner before promotion")?;
    held.slots[0] = held.slots[1];
    //5286DF clears only the ID dword; the second record's tag is retained.
    held.slots[1].id = 0;
    Ok(())
}

///UseCard526DB0's Jera arm 527641. 455680 snapshots the room's type5 list
///in manager order (spawns made below are not revisited). It skips
///collectibles100, trinkets350 and Jera itself (300.29), 5080F0's
///340/370/380 and the chests 50/51/60/360. Every other pickup is copied with
///Game.Spawn550A40(5, variant, FindFreePosition5C7F30(position, 40, EDX0),
///zero velocity, Player spawner, subtype, its own spawn seed+7D0); then
///5280BE requests GiantBook 21, which freezes the game while it plays.
///Returns false, changing nothing, when a source family, its Init or the
///placement readset is not owned here; the caller keeps the frontier path.
fn duplicate_room_pickups(state: &mut State) -> Result<bool, String> {
    let Some(grid) = state.room.grid.clone() else {
        return Ok(false);
    };
    let grid = crate::grid::Grid::new(grid)?;
    let Some(costs) = state.room.path_costs.clone() else {
        return Ok(false);
    };
    let sources: Vec<u32> = state
        .actor_scheduler
        .full_phase()
        .into_iter()
        .filter(|actor| actor.kind == ActorKind::Pickup)
        .map(|actor| actor.id)
        .collect();
    for source in sources {
        let Some(pickup) = state.room.pickups.iter().find(|pickup| pickup.id == source) else {
            return Ok(false);
        };
        let (variant, subtype, anchor, seed) = (
            pickup.variant,
            pickup.subtype,
            pickup.position,
            pickup.spawn_seed,
        );
        if matches!(variant, 100 | 350 | 340 | 370 | 380 | 50 | 51 | 60 | 360)
            || (variant, subtype) == (300, 29)
        {
            continue;
        }
        if !matches!(variant, 10 | 20 | 30 | 40 | 70 | 90 | 300) || seed == 0 {
            return Ok(false);
        }
        let Ok(occupied) = crate::ordinary_placement::positions(state) else {
            return Ok(false);
        };
        let mut shared = state.boot_lifecycle.shared.clone();
        let position = crate::roundworm::find_free_position(
            anchor,
            40.0,
            &mut shared,
            &grid,
            &costs,
            &occupied,
        )?;
        state.boot_lifecycle.shared = shared;
        let handle = state.next_pickup;
        let copy = if matches!(variant, 70 | 300) {
            crate::Pickup::from_pocket_parts(
                handle,
                variant,
                subtype,
                seed,
                position,
                true,
                crate::pickups::PocketSubtypeInputs::from_state(state),
            )?
        } else {
            let selector = crate::room_spawns::SpawnSelector {
                group_index: 0,
                grid_index: 0,
                category: crate::room_spawns::SpawnCategory::Entity,
                entity_type: 5,
                variant: variant as u16,
                subtype: subtype as u16,
                position,
                group_seed: 0,
                spawn_seed: seed,
            };
            crate::Pickup::from_source_selector(handle, &selector, true, state)?
        };
        let Some(mut copy) = copy else {
            return Ok(false);
        };
        copy.birth_game_update = state.game_update_count;
        // Game.Spawn copies the Player spawner's type/variant (550ACD..ADA).
        copy.source_spawner = Some(crate::pickups::SourceSpawner {
            entity_type: 1,
            variant: 0,
        });
        copy.morph_links = Some(crate::pickups::MorphLinks {
            parent498: None,
            link4a0: None,
            link4a8: None,
        });
        state.next_pickup = handle
            .checked_add(1)
            .ok_or("Jera pickup identity exhausted")?;
        state.room.pickups.push(copy);
        let actor = ActorRef {
            kind: ActorKind::Pickup,
            id: handle,
        };
        state.room.authored_actor_order.push(actor);
        state.actor_scheduler.admit(actor)?;
    }
    if !state.information_animation.request_authored(21, 0) {
        return Ok(false);
    }
    Ok(true)
}

///UseCard526DB0 for 2 of Clubs (23), 2 of Diamonds (24) and Jera (29),
///shared by the pocket arm below and Blank Card's leaf 523611. 50AA80 puts
///the card face into the copied Sprite+7E0 and 50A5C0 force-plays UseItem.
///The jump-table arms 527DE3/52752F add the current bombs/coins, or 2 when
///there are none (AddBombs5086F0 clamps to 0..99; AddCoins518AB0); 527641
///is Jera above. Returns false, with nothing changed, for any other card or
///when Jera's duplicates are not owned here.
pub(crate) fn use_card_effect(state: &mut State, slot: PocketSlot) -> Result<bool, String> {
    if !slot.card || !matches!(slot.id, 23 | 24 | 29) {
        return Ok(false);
    }
    if slot.id == 29 {
        // Plan the whole room copy first; adopt it only when every source,
        // Init and placement read is owned.
        let mut trial = state.clone();
        if !duplicate_room_pickups(&mut trial)? {
            return Ok(false);
        }
        *state = trial;
    }
    let owner = state
        .player_items
        .as_mut()
        .ok_or("Card use lost its Player owner")?;
    owner.pickup_presentation = Some(player_items::PickupPresentation {
        item_id: slot.id,
        animation_frame: 0.0,
        animation_playing: true,
        pocket: Some(PocketPresentation::for_slot(slot)),
        regular: None,
    });
    crate::player_animation::force_play(state, crate::player_animation::Clip::UseItem);
    match slot.id {
        23 => {
            let bombs = state.inventory.bombs;
            let added = if bombs == 0 { 2 } else { bombs };
            state.inventory.bombs = (bombs + added).min(99);
        }
        24 => {
            let coins = state.inventory.coins as i32;
            player_items::add_coins(state, if coins == 0 { 2 } else { coins })?;
        }
        29 => {}
        _ => unreachable!("checked card"),
    }
    Ok(true)
}

///5282B0's Card arm. UseCard (above) runs first; the independent 52837C
///Player+BE8 store (an unowned word, kept as a frontier) and 528399's
///persistent 61 mark follow only when it is owned. 5283AE then promotes
///slot1 and clears its ID dword. False leaves the frontier path to the caller.
fn use_card(state: &mut State, slot: PocketSlot) -> Result<bool, String> {
    if !use_card_effect(state, slot)? {
        return Ok(false);
    }
    state
        .player_items
        .as_mut()
        .ok_or("Card use lost its Player owner")?
        .pill_use_frontiers
        .get_or_insert_with(Default::default)
        .player_be8 = true;
    crate::persistent_effects::mark_collected(state, 61)?;
    let held = state
        .player_items
        .as_mut()
        .and_then(|owner| owner.held_pockets.as_mut())
        .ok_or("Card effect lost its held-slot owner before promotion")?;
    held.slots[0] = held.slots[1];
    held.slots[1].id = 0;
    Ok(true)
}

///5282B0 invokes the effect before reading CURRENT slot1 and shifting it.
///Only the source-closed11/12 scalar arms consume current slots here; other
///effects preserve the existing typed frontier and conservative invalidation.
pub(crate) fn use_pocket(state: &mut State, controls_enabled: bool) -> Result<(), String> {
    if !controls_enabled
        || state.player.input_cooldown > 0
        || !state.public_actions.pocket_triggered
        || (state.player_animation.current == Some(crate::player_animation::Clip::UseItem)
            && state.player_animation.playing)
    {
        return Ok(());
    }
    let resolved_pill = crate::pill_effect_history::resolve_held_pill(state);
    if let Some(effect @ (11 | 12)) = resolved_pill {
        return use_range_pill(state, effect);
    }
    if let Some(slot) = state
        .player_items
        .as_ref()
        .and_then(|owner| owner.held_pockets.as_ref())
        .filter(|held| held.frontier.is_none())
        .map(|held| held.slots[0])
        .filter(|slot| slot.card && matches!(slot.id, 23 | 24 | 29))
    {
        if use_card(state, slot)? {
            return Ok(());
        }
    }
    let Some(owner) = state.player_items.as_mut() else {
        return Ok(());
    };
    let Some(held) = owner.held_pockets.as_mut() else {
        //An admitted request with legacy unknown slots may run a
        //Pill writer. Blocked requests returned before this branch.
        crate::pill_effect_history::forget(owner);
        crate::active_charges::invalidate_owner(
            owner,
            &[
                crate::active_charges::Word::ActiveId,
                crate::active_charges::Word::Charge,
                crate::active_charges::Word::Subordinate,
                crate::active_charges::Word::Reserve,
            ],
            crate::active_charges::Frontier::PocketEffect {
                source_pc: 0x5282b0,
            },
        );
        return Ok(());
    };
    if held.frontier.is_none() {
        let slot = held.slots[0];
        if slot.id == 0 {
            return Ok(());
        }
        held.frontier = Some(if slot.card {
            PocketFrontier::CardEffect { card_id: slot.id }
        } else {
            PocketFrontier::PillEffect {
                pill_color: slot.id,
            }
        });
    }
    crate::active_charges::invalidate_owner(
        owner,
        &[
            crate::active_charges::Word::ActiveId,
            crate::active_charges::Word::Charge,
            crate::active_charges::Word::Subordinate,
            crate::active_charges::Word::Reserve,
        ],
        crate::active_charges::Frontier::PocketEffect {
            source_pc: 0x5282b0,
        },
    );
    if let Some(coverage) = &mut state.room.ordinary_placement {
        coverage.invalidate(crate::ordinary_placement::Frontier::PocketEffects);
    }
    crate::ordinary_placement::remember_owner(
        owner,
        crate::ordinary_placement::Frontier::PocketEffects,
    );
    //5284A3 commits the actual final-effect increment before dispatch.
    //The unimplemented effect tail cannot claim its final word history;
    //forget that ownership afterward, preserving the existing frontier.
    if let (Some(effect), Some(history)) = (resolved_pill, owner.pill_effect_history.as_mut()) {
        history
            .increment_resolved(effect)
            .expect("bounded actual Pill-effect resolver");
    }
    crate::pill_effect_history::forget(owner);
    //An existing frontier contains historical records, not known-empty
    //current slots. Preserve it, but never retain new input ownership on a
    //later admitted request whose actual effect may still be unknown.
    if let Some(history) = &mut owner.query_history {
        history.forget_unowned_effect();
    }
    state.attack_runtime.aim_rotation_c70 = None;
    owner.move_speed_inputs = None;
    Ok(())
}

///504820→505520 adds the pocket before copying7E0, starting UseItem and
///requesting Collect audio. The guarded main clip is50C780, not BF4 alone.
pub(crate) fn collect(state: &mut State, index: usize) -> Result<PlayerContactDecision, String> {
    let pickup = &state.room.pickups[index];
    if !matches!(pickup.variant, 70 | 300)
        || pickup.removed
        || pickup.collected
        || pickup
            .regular_collision
            .as_ref()
            .is_none_or(|collision| collision.entity_collision_class != 4)
        || !pickup.touches(&state.player)
    {
        return Ok(PlayerContactDecision::NotReached);
    }
    //504950 precedes ordinary collection and resets this distinct AD0 word
    //on an actual touching Player. ACC is the independent admission delay.
    let Some(delay) = pickup.pocket_contact_delay else {
        return Ok(PlayerContactDecision::Unowned);
    };
    if delay > 0 {
        state.room.pickups[index].pocket_contact_delay = Some(10);
        return Ok(PlayerContactDecision::VetoPhysicalResponse);
    }
    let pickup = &state.room.pickups[index];
    //Only the currently admitted free normal1/0 caller closes the AL0
    //postlude. Other Player variants/priced contexts keep their own owner.
    if state
        .player
        .entity_source
        .is_none_or(|source| source.entity_type != 1 || source.variant != 0)
    {
        return Ok(PlayerContactDecision::Unowned);
    }
    if pickup.collection_delay > 0 || !crate::pickups::player_can_collect(state) {
        return Ok(PlayerContactDecision::AllowPhysicalResponse);
    }
    if !pickup.can_collect {
        return Ok(PlayerContactDecision::Unowned);
    }
    //50C780 tests the actual main clip name without requiring held inputs.
    if state
        .player_animation
        .current
        .is_some_and(|clip| matches!(clip.name(), "Pickup" | "LiftItem" | "HideItem" | "UseItem"))
    {
        return Ok(PlayerContactDecision::AllowPhysicalResponse);
    }
    //Preserve the free callback's AL0 physical response for the legacy guard.
    if state
        .player_items
        .as_ref()
        .is_some_and(|owner| owner.pickup_animation_guard)
    {
        return Ok(PlayerContactDecision::AllowPhysicalResponse);
    }
    if !grant(state, index)? {
        return Ok(PlayerContactDecision::Unowned);
    }
    crate::pickups::collect_sound(state, index);
    state.room.pickups[index].finish_collect();
    Ok(PlayerContactDecision::VetoPhysicalResponse)
}

///50AE00/50AF00 and50A8A0/50AA80 are the shared family grant prefix.
///The contact owner has already checked AD0, ACC, CanCollect and item clips.
///Paid callers retain these writes before50A830 replaces the copied Sprite.
pub(crate) fn grant(state: &mut State, index: usize) -> Result<bool, String> {
    let pickup = &state.room.pickups[index];
    if !matches!(pickup.variant, 70 | 300) || pickup.collected || pickup.removed {
        return Ok(false);
    }
    let Some(owner) = state.player_items.as_ref() else {
        return Ok(false);
    };
    if owner.pickup_animation_guard {
        return Ok(false);
    }
    let Some(held) = owner
        .held_pockets
        .as_ref()
        .filter(|held| held.frontier.is_none())
    else {
        return Ok(false);
    };
    let Some(capacity) = capacity(owner) else {
        return Ok(false);
    };
    let slots = held.slots;
    let slot = PocketSlot {
        id: pickup.subtype,
        card: pickup.variant == 300,
    };
    if !matches!((slot.card, slot.id), (true, 1..=40) | (false, 1..=9)) {
        return Ok(false);
    }
    let spill = slots[0].id != 0 && slots[capacity - 1].id != 0;
    if spill {
        //50AE90/50AF90: exactly one shared seed before overwriting slot0;
        //Game.Spawn receives old tag/ID, Player448, zero velocity/no parent.
        let seed = state.boot_lifecycle.shared.next_u32();
        let id = state.next_pickup;
        let mut dropped = crate::Pickup::from_pocket_parts(
            id,
            slots[0].variant(),
            slots[0].id,
            seed,
            crate::bombs::player_world(state),
            true,
            crate::pickups::PocketSubtypeInputs::from_state(state),
        )?
        .ok_or("capacity ejection reached an unowned Pocket Init")?;
        dropped.birth_game_update = state.game_update_count;
        //50AE90/50AF90 passes null to Game.Spawn, not weak498.
        dropped.source_spawner = Some(crate::pickups::SourceSpawner {
            entity_type: 0,
            variant: 0,
        });
        dropped.pocket_contact_delay = Some(4);
        state.next_pickup = id
            .checked_add(1)
            .ok_or("pocket pickup identity exhausted")?;
        state.room.pickups.push(dropped);
        let actor = ActorRef {
            kind: ActorKind::Pickup,
            id,
        };
        state.room.authored_actor_order.push(actor);
        state.actor_scheduler.admit(actor)?;
    }
    let held = state
        .player_items
        .as_mut()
        .expect("checked Player owner")
        .held_pockets
        .as_mut()
        .expect("checked pocket owner");
    if held.slots[capacity - 1].id == 0 && capacity == 2 {
        held.slots[1] = held.slots[0];
    }
    held.slots[0] = slot;
    let owner = state.player_items.as_mut().expect("checked Player owner");
    owner.pickup_presentation = Some(player_items::PickupPresentation {
        item_id: slot.id,
        animation_frame: 0.0,
        animation_playing: true,
        pocket: Some(PocketPresentation::for_slot(slot)),
        regular: None,
    });
    crate::player_animation::force_play(state, crate::player_animation::Clip::UseItem);
    if slot.card && state.challenge_id != 9 {
        let (name, description) = if state.challenge_id == 30 {
            ("???", "")
        } else {
            let config = crate::pickups::card_definition(slot.id).ok_or("card config absent")?;
            (config.name, config.description)
        };
        state.run_introduction = crate::RunIntroduction::pickup(name, description);
    }
    Ok(true)
}

///445710 captures the overlap before the two callbacks, then reads current
///masses/velocities after an AL0 result. The owned refusal branches above
///leave positions/radii unchanged; AD0 and successful collection veto it.
///Use the same normal Player/grid geometry and kernel as chests/empty altars.
pub(crate) fn player_contact(state: &mut State, index: usize) -> Result<(), String> {
    //4453A0 vetoes callbacks for the actual dead31F Player before AD0.
    if state.player.dead {
        return Ok(());
    }
    let player_position = crate::bombs::player_world(state);
    let pickup_position = state.room.pickups[index].position;
    let pickup_radius = state.room.pickups[index].collision_radius;
    if collect(state, index)? != PlayerContactDecision::AllowPhysicalResponse {
        return Ok(());
    }
    crate::pickups::player_physical_response(
        state,
        index,
        player_position,
        pickup_position,
        pickup_radius,
    )
}
