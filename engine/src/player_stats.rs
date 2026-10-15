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
    /// Actual Player D64. Legacy or unadmitted output remains absent.
    #[serde(default)]
    pub luck: Option<f32>,
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
            luck: None,
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
            || self.luck.is_some_and(|luck| !luck.is_finite())
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
    let mut completed = 0;
    if let Some(source_type) = state.weapon_admission.source_player_type {
        let prior_primary = state.weapon_cache.primary;
        state.weapon_cache.primary = crate::weapon_plan::resolve_primary(
            item_pipeline::ItemInputs::new(owner)?,
            source_type,
            state.weapon_cache.primary,
            requested,
        )?;
        if requested.0 & 0x100 != 0 {
            if crate::attack_runtime::complete_primary_cleanup(state, prior_primary)? {
                completed |= 0x100;
            } else if state.weapon_cache.primary != prior_primary {
                state
                    .weapon_admission
                    .pending_primary_cleanup
                    .get_or_insert(crate::weapon_plan::PrimaryCleanupIntent {
                        from_primary: prior_primary,
                        to_primary: state.weapon_cache.primary,
                    });
            }
        }
    }
    let inputs =
        item_pipeline::ItemInputs::new(state.player_items.as_ref().expect("checked cache owner"))?;
    let resolved = item_pipeline::resolve_scalars(
        inputs,
        state.weapon_cache.primary,
        state.weapon_admission.source_player_type,
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
    if requested.0 & 128 != 0 {
        if let Some((flying, grid_class)) = item_pipeline::resolve_neutral_flight(
            inputs,
            state.weapon_admission.source_player_type,
            state.player.transformation_counts,
        ) {
            state.player.flying = Some(flying);
            state.player.grid_collision_class = Some(grid_class);
            completed |= 128;
        }
    }
    if requested.0 & 512 != 0 {
        // 511F10 evaluates Familiar after Flight and before Luck. Retain the
        // sole manager/RNG consumer; its pending result controls completion.
        crate::familiars::evaluate_cache(state)?;
        if state
            .player_items
            .as_ref()
            .expect("checked Player cache owner")
            .unevaluated_cache_flags
            & 512
            == 0
        {
            completed |= 512;
        }
    }
    if requested.0 & 1024 != 0 {
        let inputs = item_pipeline::ItemInputs::new(
            state.player_items.as_ref().expect("checked cache owner"),
        )?;
        if let Some(luck) = item_pipeline::resolve_luck(
            inputs,
            state.weapon_admission.source_player_type,
            state.source_cache_run_flags,
        )? {
            state.player_stats.luck = Some(luck);
            completed |= 1024;
        }
    }
    state.player_stats.validate()?;
    let pending = resolved.pending(requested).0 & !completed;
    let tail_admitted = state.weapon_admission.source_player_type == Some(0)
        && state.weapon_admission.pending_primary_cleanup.is_none()
        && state.player.transformation_counts == Some([0, 0])
        && state.source_cache_run_flags.is_some()
        && item_pipeline::ItemInputs::new(
            state.player_items.as_ref().expect("checked cache owner"),
        )?
        .admits_complete_selected_tail()
        && pending & 0x7ff == 0;
    state
        .player_items
        .as_mut()
        .expect("checked Player cache owner")
        // 512286 clears all 32 D6C bits after the actual requested callbacks.
        // Reserved bits have no independent callback at this admitted tail.
        // Unimplemented callbacks or history retain their pending requests.
        .unevaluated_cache_flags = if tail_admitted { 0 } else { pending };
    Ok(())
}
