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

///5282B0 invokes the effect before reading CURRENT slot1 and shifting it.
///Until that effect is owned, retain diagnostic history and refuse to treat
///it as current source slots. No unsupported effect or early shift is faked.
pub(crate) fn note_unowned_use(state: &mut State, controls_enabled: bool) {
    if !controls_enabled
        || state.player.input_cooldown > 0
        || !state.public_actions.pocket_triggered
        || (state.player_animation.current == Some(crate::player_animation::Clip::UseItem)
            && state.player_animation.playing)
    {
        return;
    }
    let Some(owner) = state.player_items.as_mut() else {
        return;
    };
    let Some(held) = owner.held_pockets.as_mut() else {
        return;
    };
    if held.frontier.is_none() {
        let slot = held.slots[0];
        if slot.id == 0 {
            return;
        }
        held.frontier = Some(if slot.card {
            PocketFrontier::CardEffect { card_id: slot.id }
        } else {
            PocketFrontier::PillEffect {
                pill_color: slot.id,
            }
        });
    }
    //An existing frontier contains historical records, not known-empty
    //current slots. Preserve it, but never retain new input ownership on a
    //later admitted request whose actual effect may still be unknown.
    state.attack_runtime.aim_rotation_c70 = None;
    owner.move_speed_inputs = None;
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
    let Some(owner) = state.player_items.as_ref() else {
        return Ok(PlayerContactDecision::Unowned);
    };
    if owner.pickup_animation_guard {
        return Ok(PlayerContactDecision::AllowPhysicalResponse);
    }
    let Some(held) = owner
        .held_pockets
        .as_ref()
        .filter(|held| held.frontier.is_none())
    else {
        return Ok(PlayerContactDecision::Unowned);
    };
    let Some(capacity) = capacity(owner) else {
        return Ok(PlayerContactDecision::Unowned);
    };
    let slots = held.slots;
    let slot = PocketSlot {
        id: pickup.subtype,
        card: pickup.variant == 300,
    };
    if !matches!((slot.card, slot.id), (true, 1..=40) | (false, 1..=9)) {
        return Ok(PlayerContactDecision::Unowned);
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
    state.audio.admit(
        &mut state.boot_lifecycle.shared,
        state.system_clock.system_counter.wrapping_sub(1),
        crate::audio::SoundRequest {
            id: if slot.card { 8 } else { 252 },
            volume: 1.0,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
    state.room.pickups[index].finish_collect();
    Ok(PlayerContactDecision::VetoPhysicalResponse)
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
    //A missing legacy grid/flying/class owner cannot become ground geometry.
    if state.player.flying != Some(false) || state.player.grid_collision_class != Some(5) {
        return Ok(());
    }
    let Some(frame) = state.room.grid.clone() else {
        return Ok(());
    };
    let grid = crate::grid::Grid::new(frame)?;
    static PLAYER_COLLIDER: std::sync::OnceLock<crate::grid::Collider> = std::sync::OnceLock::new();
    let collider = PLAYER_COLLIDER.get_or_init(|| {
        crate::grid::Collider::new(crate::grid::CollisionGeometry {
            radius: crate::content::PLAYER_COLLISION_RADIUS_WORLD as f32,
            scale: crate::motion::WorldVector { x: 1.0, y: 1.0 },
            points: 40,
        })
        .expect("authored normal Isaac geometry")
    });
    let mut player = crate::combat::ContactBody {
        id: 0,
        entity_type: 1,
        position: player_position,
        velocity: crate::bombs::player_velocity(state),
        radius: crate::content::PLAYER_COLLISION_RADIUS_WORLD as f32,
        mass: crate::content::PLAYER_COLLISION_MASS_WORLD as f32,
    };
    let pickup = &state.room.pickups[index];
    let mut body = crate::combat::ContactBody {
        id: pickup.id,
        entity_type: 5,
        position: pickup_position,
        velocity: pickup.velocity,
        radius: pickup_radius,
        mass: pickup.collision_mass,
    };
    crate::combat::resolve_player_pickup_contact(&mut player, &mut body, |shift| {
        collider.blocks_player_shift(&grid, player_position, shift)
    })?;
    state.player.x = f64::from(player.position.x) * 0.65 + 32.0;
    state.player.y = f64::from(player.position.y) * 0.65 - 46.0;
    state.player.vx = f64::from(player.velocity.x) * 0.65;
    state.player.vy = f64::from(player.velocity.y) * 0.65;
    let pickup = &mut state.room.pickups[index];
    pickup.position = body.position;
    pickup.presented_position = body.position;
    pickup.velocity = body.velocity;
    pickup.project();
    Ok(())
}
