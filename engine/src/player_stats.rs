//! Actual Player cache fields and mapped common passive callback branches.
//! Unmapped item/cache owners remain requested and observable.
use crate::{item_pipeline, projectiles::Parameters, State};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayerStats {
    /// Source Player+c7c.
    pub shot_speed: f32,
    /// Source Player+c88, copied into regular tears at actual birth.
    pub damage: f32,
    /// Source Player+c8c/c90/c94, used by actual regular tear birth.
    pub tear_height: f32,
    pub tear_falling_speed: f32,
    pub tear_falling_acceleration: f32,
    /// Source Player+d60, consumed before both-phase input acceleration.
    pub move_speed: f32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            shot_speed: 1.0,
            damage: 3.5,
            tear_height: -23.75,
            tear_falling_speed: 0.0,
            tear_falling_acceleration: 0.0,
            move_speed: 1.0,
        }
    }
}

impl PlayerStats {
    pub fn parameters(self) -> Parameters {
        Parameters {
            height: self.tear_height,
            falling_speed: self.tear_falling_speed,
            falling_acceleration: self.tear_falling_acceleration,
        }
    }

    pub fn validate(self) -> Result<(), String> {
        if !self.shot_speed.is_finite()
            || self.shot_speed < 0.6
            || !self.damage.is_finite()
            || self.damage < 0.5
            || !self.tear_height.is_finite()
            || self.tear_height > -7.0
            || !self.tear_falling_speed.is_finite()
            || !self.tear_falling_acceleration.is_finite()
            || !self.move_speed.is_finite()
            || !(0.1..=2.0).contains(&self.move_speed)
        {
            return Err("Player cached stats violate finite source clamps".into());
        }
        Ok(())
    }
}

/// Requested511f10 callbacks preserve source order and partial completion.
/// The fresh empty C18 source domain admits selection; unknown prior actor
/// effects remain requested. Other cache callbacks retain their existing owner.
pub(crate) fn evaluate_cache(state: &mut State) -> Result<(), String> {
    let Some(owner) = state.player_items.as_ref() else {
        return Ok(());
    };
    let requested = item_pipeline::CacheMask(owner.unevaluated_cache_flags);
    let inputs = item_pipeline::ItemInputs::new(owner)?;
    let mut completed = 0;
    if let Some(source_type) = state.weapon_admission.source_player_type {
        let prior_primary = state.weapon_cache.primary;
        state.weapon_cache.primary = crate::weapon_plan::resolve_primary(
            inputs,
            source_type,
            state.weapon_cache.primary,
            requested,
        )?;
        if requested.0 & 0x100 != 0 {
            if state.weapon_cache.primary != prior_primary
                && !state.weapon_admission.c18_is_known_empty
            {
                state
                    .weapon_admission
                    .pending_primary_cleanup
                    .get_or_insert(crate::weapon_plan::PrimaryCleanupIntent {
                        from_primary: prior_primary,
                        to_primary: state.weapon_cache.primary,
                    });
            }
            //513AF7 equal primary skips this invocation's C18 effect only.
            //A previously unperformed cleanup persists independently of the
            //current C08, including later equality or a new NULL assumption.
            if state.weapon_admission.pending_primary_cleanup.is_none()
                && (state.weapon_cache.primary == prior_primary
                    || state.weapon_admission.c18_is_known_empty)
            {
                completed |= 0x100;
            }
        }
    }
    let resolved = item_pipeline::resolve_scalars(
        inputs,
        state.weapon_cache.primary,
        item_pipeline::CachedScalars {
            stats: state.player_stats,
            max_fire_delay: state.max_fire_delay,
        },
        requested,
    )?;
    state.player_stats = resolved.values.stats;
    state.max_fire_delay = resolved.values.max_fire_delay;
    let (cache, appearance) = crate::weapon_plan::resolve_appearance(
        inputs,
        state.weapon_admission.source_player_type,
        state.weapon_cache,
        requested,
    );
    state.weapon_cache = cache;
    completed |= appearance.0;
    state
        .player_items
        .as_mut()
        .expect("checked Player cache owner")
        .unevaluated_cache_flags = resolved.pending(requested).0 & !completed;
    Ok(())
}
