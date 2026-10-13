//! Actual Player cache fields and mapped common passive callback branches.
//! Unmapped item/cache owners remain requested and observable.
use crate::{items, player_items::PlayerItems, projectiles::Parameters, State};
use serde::{Deserialize, Serialize};

const SHOT_SPEED: u32 = 4;
const RANGE: u32 = 8;
const MOVE_SPEED: u32 = 16;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayerStats {
    /// Source Player+c7c.
    pub shot_speed: f32,
    /// Source Player+c88. Damage callbacks remain a following slice.
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

fn count(owner: &PlayerItems, id: usize) -> i32 {
    owner.owned_counts[id]
}

fn has_unmapped_owner(owner: &PlayerItems, flag: u32, supported: &[u32]) -> bool {
    owner.owned_counts.iter().enumerate().any(|(id, &count)| {
        count != 0
            && !supported.contains(&(id as u32))
            && items::definition(id as u32).is_some_and(|config| config.cache_flags & flag != 0)
    }) || owner.temporary_collectibles.iter().any(|entry| {
        entry.count != 0
            && !supported.contains(&entry.item_id)
            && items::definition(entry.item_id).is_some_and(|config| config.cache_flags & flag != 0)
    })
}

/// AddCollectible52d173 invokes511f10 before clearing the deferred queue.
/// These selected callbacks follow source order4 ->8 ->16 before Familiar bit0x200.
/// The real partial result is retained when another owner is unmapped; its
/// pending bit is not presented as completed native cache evaluation.
pub(crate) fn evaluate_cache(state: &mut State) -> Result<(), String> {
    let Some(owner) = state.player_items.as_ref() else {
        return Ok(());
    };
    if owner.owned_counts.len() != 347 {
        return Err("Player cache requires all347 actual owned counts".into());
    }
    let flags = owner.unevaluated_cache_flags;
    let mut handled = 0;
    if flags & SHOT_SPEED != 0 {
        // 511f68/513e48 select1.2 on permanent or temporary SpeedBall143
        // presence, otherwise1.0. Multiple copies do not repeat this override.
        state.player_stats.shot_speed = if count(owner, 143) != 0 || owner.temporary_count(143) != 0
        {
            1.2
        } else {
            1.0
        };
        state.player_stats.shot_speed = state.player_stats.shot_speed.max(0.6);
        if !has_unmapped_owner(owner, SHOT_SPEED, &[143]) {
            handled |= SHOT_SPEED;
        }
    }
    if flags & RANGE != 0 {
        // 5127c8 resets all three actual tear fields;5128d3/5128f3
        // subtract5 per permanent RoidRage14 from the cached height.
        state.player_stats.tear_height = (-23.75_f32 - count(owner, 14) as f32 * 5.0).min(-7.0);
        state.player_stats.tear_falling_speed = 0.0;
        state.player_stats.tear_falling_acceleration = 0.0;
        if !has_unmapped_owner(owner, RANGE, &[14]) {
            handled |= RANGE;
        }
    }
    if flags & MOVE_SPEED != 0 {
        // 512c59..512df5 combine27/28 and143 before one f32*.3;
        // RoidRage14 contributes*.6 after min(count,1). Isaac
        // modifier table8e6820[0] is0;512e86..512e9d takes min(sum,1)
        // then adds1. Source final speed clamps are .1..2.
        let additive = count(owner, 27)
            .wrapping_add(count(owner, 28))
            .wrapping_add(count(owner, 143))
            .wrapping_add(owner.temporary_count(143));
        let roid = (count(owner, 14) as f32).min(1.0);
        let speed = additive as f32 * 0.3_f32 + roid * 0.6_f32;
        state.player_stats.move_speed = (speed.min(1.0) + 1.0).clamp(0.1, 2.0);
        if !has_unmapped_owner(owner, MOVE_SPEED, &[14, 27, 28, 143]) {
            handled |= MOVE_SPEED;
        }
    }
    state.player_stats.validate()?;
    state
        .player_items
        .as_mut()
        .expect("checked Player cache owner")
        .unevaluated_cache_flags &= !handled;
    Ok(())
}
