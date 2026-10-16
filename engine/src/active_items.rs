//! Source Player52a640 active input and the normal Yum Heart45 dispatch leaf.
//! Input edges belong to the released System poll; the Player callback runs
//! in both presentations, while its main sprite advances only on full updates.
use crate::{audio::SoundRequest, items, player_animation::Clip, player_items, State};

/// 509e10 clamps to the authored maximum; a rising normal charge requests
/// partial171 or full218 sound before storing the new charge. The actual
/// charge is signed. Reserve/costume/passive charging leaves retain their
/// separate owners and are not reconstructed from observed charge deltas.
pub(crate) fn set_charge(state: &mut State, requested: i32) -> Result<(), String> {
    let Some(owner) = state.player_items.as_ref() else {
        return Ok(());
    };
    if owner.active_id == 0 {
        return Ok(());
    }
    let config = items::definition(owner.active_id).ok_or("active ItemConfig absent")?;
    let charge = requested.min(config.max_charges);
    if charge > owner.active_charge && !matches!(owner.active_id, 164 | 289) {
        state.audio.admit(
            &mut state.boot_lifecycle.shared,
            state.system_clock.system_counter - 1,
            SoundRequest {
                id: if requested >= config.max_charges {
                    218
                } else {
                    171
                },
                volume: 1.0,
                delay: 2,
                looping: false,
                pitch: 1.0,
            },
        );
    }
    state
        .player_items
        .as_mut()
        .expect("checked Player owner")
        .active_charge = charge;
    Ok(())
}

/// The plain5295c0 branch spends the current charge and transfers the owned
/// reserve. Battery/trinket modifiers and their real effect births are later
/// leaves; the ordinary no-modifier use admitted here has no extra SFX or RNG.
fn discharge(state: &mut State) -> Result<(), String> {
    let reserve = state
        .player_items
        .as_ref()
        .expect("checked owner")
        .reserve_charge;
    set_charge(state, reserve)?;
    let owner = state.player_items.as_mut().expect("checked owner");
    owner.reserve_charge = 0;
    owner.active_flag_3076 = false;
    Ok(())
}

pub(crate) fn advance(state: &mut State, controls_enabled: bool) -> Result<(), String> {
    if state.public_actions.active_triggered
        && controls_enabled
        && state
            .player_items
            .as_ref()
            .is_none_or(|owner| !matches!(owner.active_id, 0 | 45))
    {
        crate::ordinary_placement::invalidate(
            state,
            crate::ordinary_placement::Frontier::ActiveEffects,
        );
    }
    //Only empty active0 and the closed45 leaf exclude the D30 reset/random
    //writers in522250 here. A requested untracked active loses this input
    //proof before dispatch; no claim of actual callback execution is made.
    if state.public_actions.active_triggered && controls_enabled {
        if let Some(owner) = state.player_items.as_mut() {
            if !matches!(owner.active_id, 0 | 45) {
                if let Some(history) = &mut owner.query_history {
                    //Unimplemented requested effect may change counts,
                    //CF8, held trinkets or retained C08. This is an
                    //explicit unknown frontier, not invocation credit.
                    history.forget_unowned_effect();
                }
                crate::pill_effect_history::forget(owner);
                owner.damage_inputs = None;
                owner.range_inputs = None;
                owner.move_speed_inputs = None;
                owner.fire_delay_inputs = None;
            }
        }
    }
    //Empty active0 and the closed45 leaf cannot run the D38 writers in
    //522250. An untracked active-effect request loses only this input proof;
    //a later release or cache evaluation cannot reconstruct it.
    if state.public_actions.active_triggered && controls_enabled {
        if let Some(owner) = state.player_items.as_mut() {
            if !matches!(owner.active_id, 0 | 45) {
                owner.shot_speed_inputs = None;
            }
        }
    }
    let Some(owner) = state.player_items.as_ref() else {
        return Ok(());
    };
    //52a640 reads held/triggered action9 even on the Entity+318 word bit4000
    // fast path (the same little-endian byte+319 bit40 predicate).
    // Normal45 has no held-use or frame-recharge leaf; an actual new edge is
    // required. Its guards are7d4/7d8, not the bf4 main-animation guard.
    if !state.public_actions.active_triggered
        || !controls_enabled
        || owner.active_id != 45
        || owner.active_state_d80 != 0
        || owner.active_state_d84 != 0
    {
        return Ok(());
    }
    let config = items::definition(45).expect("authored Yum Heart config");
    if owner.active_charge != config.max_charges {
        return Ok(());
    }
    //522349 costume admission is empty for this actual config.5e4140 owns
    // a temporary record before the dispatch leaf; a zero authored cooldown
    // is persistent, not an immediately expired effect.
    //529600 can dispatch another active through positiveTrinket4.
    //The actual known-empty held/queue query closes that producer;
    //unknown/positive trinket history does not create a CF8 proof.
    let query_stores_preserved = state
        .player
        .entity_source
        .is_some_and(|source| source.entity_type == 1 && source.variant == 0)
        && crate::item_queries::AppearanceQueries::new(
            owner,
            state.weapon_cache.primary,
            state.player.entity_source,
            None,
        )
        .has_trinket(4)
            == Some(false);
    let owner = state.player_items.as_mut().expect("checked owner");
    if !query_stores_preserved {
        //The admitted529600 suffix can dispatch another active through
        //Trinket4. Its unclosed writer may change canonical Pill words,
        //independently of whether query history is still available.
        crate::pill_effect_history::forget(owner);
        if let Some(history) = &mut owner.query_history {
            history.forget_unowned_effect();
        }
    }
    owner.add_temporary_collectible(45)?;
    //524084->52598b requests157 before508230 heals two native half-hearts.
    // This succeeds and discharges even when red health is already full.
    state.audio.admit(
        &mut state.boot_lifecycle.shared,
        state.system_clock.system_counter - 1,
        SoundRequest {
            id: 157,
            volume: 1.0,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
    player_items::add_red_hearts(&mut state.player, 2);
    let owner = state.player_items.as_mut().expect("checked owner");
    owner.pickup_presentation = Some(player_items::PickupPresentation {
        item_id: 45,
        animation_frame: 0.0,
        animation_playing: true,
        pocket: None,
    });
    owner.unevaluated_cache_flags |= config.cache_flags;
    owner.active_flag_3076 = true;
    crate::player_animation::force_play(state, Clip::UseItem);
    discharge(state)
}
