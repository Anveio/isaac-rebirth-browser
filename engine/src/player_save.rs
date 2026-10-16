//! Concrete Player518350/5116b0 owned item stores, separate from live
//! presentation, pending grants, temporary effects and evaluated stats.
use crate::{items::Rng, player_items::PlayerItems, State};
use serde::{Deserialize, Serialize};

const CHILD_SHIFTS: [u32; 3] = [5, 9, 7];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedPlayerItems {
    /// Actual Saved88D0/88EC/95EC/95F0, separate from evaluated MoveSpeed.
    /// Legacy omission never inherits a fresh constructor's neutral terms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub move_speed_inputs: Option<crate::player_items::MoveSpeedInputs>,
    /// Actual Saved+88D8, copied only from an already owned Player+D30.
    /// Legacy saved data never inherits a fresh constructor's known zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_inputs: Option<crate::player_items::DamageInputs>,
    ///Actual Saved88DC; absence does not inherit a constructor's zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range_inputs: Option<crate::player_items::RangeInputs>,
    ///Actual Saved88D4. Legacy omission remains unknown on restore.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fire_delay_inputs: Option<crate::player_items::FireDelayInputs>,
    /// Actual Saved+88E0, copied only from an already owned Player+D38.
    /// A legacy/incomplete saved table never inherits constructor zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shot_speed_inputs: Option<crate::player_items::ShotSpeedInputs>,
    ///518577 copies347 slots beginning Player+e1c into Saved+893c.
    pub owned_counts: Vec<i32>,
    ///5186b0/5186e0/518708 copy current states, not initializer seeds.
    pub item_rng_states: Vec<u32>,
    pub trinket_rng_states: Vec<u32>,
    pub pocket_rng_states: Vec<u32>,
    ///518570/590 copy both actual records;511B25/33 restores them whole.
    ///Omitted legacy history stays unknown, never constructor-empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub held_pockets: Option<crate::pockets::HeldPockets>,
    /// Saved+8908/890c/8904/8910; D80/D84 are not saved fields.
    pub active_id: u32,
    pub active_charge: i32,
    pub save_word_d88: i32,
    pub reserve_charge: i32,
}

fn current_states(children: &[Rng], count: usize) -> Result<Vec<u32>, String> {
    if children.len() != count
        || children
            .iter()
            .any(|rng| rng.state == 0 || [rng.right_a, rng.left, rng.right_b] != CHILD_SHIFTS)
    {
        return Err("Player save requires its actual initialized RNG35 children".into());
    }
    Ok(children.iter().map(|rng| rng.state).collect())
}

impl SavedPlayerItems {
    fn copy_from(owner: &PlayerItems) -> Result<Self, String> {
        if owner.owned_counts.len() != 347 {
            return Err("Player save requires all347 owned count slots".into());
        }
        Ok(Self {
            move_speed_inputs: owner.move_speed_inputs,
            damage_inputs: owner.damage_inputs,
            range_inputs: owner.range_inputs,
            fire_delay_inputs: owner.fire_delay_inputs,
            shot_speed_inputs: owner.shot_speed_inputs,
            owned_counts: owner.owned_counts.clone(),
            item_rng_states: current_states(&owner.item_rngs, 347)?,
            trinket_rng_states: current_states(&owner.trinket_rngs, 62)?,
            pocket_rng_states: current_states(&owner.pocket_rngs, 41)?,
            held_pockets: owner.held_pockets.clone(),
            active_id: owner.active_id,
            active_charge: owner.active_charge,
            save_word_d88: owner.save_word_d88,
            reserve_charge: owner.reserve_charge,
        })
    }

    /// Actual5116b0 destination is the already constructed first Player.
    /// This store slice does not construct transient presentation/effect
    /// owners or run515f20's later full cache/familiar evaluation.
    pub fn restore_into(&self, owner: &mut PlayerItems) -> Result<(), String> {
        if self.owned_counts.len() != 347
            || owner.owned_counts.len() != 347
            || owner.item_rngs.len() != 347
            || owner.trinket_rngs.len() != 62
            || owner.pocket_rngs.len() != 41
            || owner.queued_id.is_some()
            || !owner.temporary_collectibles.is_empty()
            || owner.active_state_d80 != 0
            || owner.active_state_d84 != 0
        {
            return Err("Player restore requires its complete saved and live owned tables".into());
        }
        for (states, count) in [
            (&self.item_rng_states, 347),
            (&self.trinket_rng_states, 62),
            (&self.pocket_rng_states, 41),
        ] {
            if states.len() != count || states.contains(&0) {
                return Err("Saved Player RNG35 states are incomplete or reserved zero".into());
            }
        }
        //511710 skips dummy ID0 and copies346 dwords from Saved+8940.
        owner.owned_counts[1..].copy_from_slice(&self.owned_counts[1..]);
        for (children, states) in [
            (&mut owner.item_rngs, &self.item_rng_states),
            (&mut owner.trinket_rngs, &self.trinket_rng_states),
            (&mut owner.pocket_rngs, &self.pocket_rng_states),
        ] {
            for (rng, state) in children.iter_mut().zip(states) {
                //5c2d40(savedCurrentState,35), with no Next/initializer draw.
                *rng = Rng::new(*state, CHILD_SHIFTS);
            }
        }
        owner.save_word_d88 = self.save_word_d88;
        owner.active_id = self.active_id;
        owner.active_charge = self.active_charge;
        owner.reserve_charge = self.reserve_charge;
        owner.held_pockets = self.held_pockets.clone();
        //51849A/5184A0 stores Saved88D8;511A25/511A2B restores D30.
        //None overwrites constructor Some(0) rather than inventing history.
        owner.damage_inputs = self.damage_inputs;
        //5184A6/5184AC saves88DC;511A31/511A37 restores the word.
        owner.range_inputs = self.range_inputs;
        //518482/488 and5184D6/4DC save D28/D44;51877E/785 and
        //51878C/793 include 13BC/13C0 in the saved112-byte block.
        //511820/82E/1A13/1A67 restore these same four words. None must
        //overwrite constructor neutrality when the saved owner is absent.
        owner.move_speed_inputs = self.move_speed_inputs;
        //51848E/518494 saves88D4;511A19/511A1F restores the actual word.
        owner.fire_delay_inputs = self.fire_delay_inputs;
        //5184B2/5184B8 writes Saved88E0;511A43 restores PlayerD38.
        owner.shot_speed_inputs = self.shot_speed_inputs;
        Ok(())
    }
}

/// Actual518120 is shared by Player Save518350 and stage request5508d0.
/// Its caller separately owns queued-grant admission. This bounded callback
/// clears mapped temporary records and neutral costume-free active state;
/// other removal, appearance andBF5 owners remain explicit source gaps.
pub(crate) fn normalize_player(state: &mut State) -> Result<(), String> {
    let owner = state
        .player_items
        .as_mut()
        .ok_or("Player save requires the actual first Player item owner")?;
    if !matches!(owner.active_id, 0 | 45)
        || owner.active_state_d80 != 0
        || owner.active_state_d84 != 0
    {
        return Err("Player save normalization requires an unmapped temporary/active owner".into());
    }
    if !owner.temporary_collectibles.is_empty() {
        owner.clear_save_temporary_effects()?;
        //5180d0 tests the OR of removed config flags and existingD6C.
        //Real supported cache/familiar callbacks own any requested work.
        if owner.unevaluated_cache_flags != 0 {
            crate::player_stats::evaluate_cache(state)?;
        }
    }
    let owner = state
        .player_items
        .as_mut()
        .expect("checked first Player owner");
    //5182c3/5182cd store zero after cancellation. This supported owner is
    //already neutral; no transient owner or missing BF5 value is invented.
    owner.active_state_d80 = 0;
    owner.active_state_d84 = 0;
    Ok(())
}

///54f373 flushes52ee40 before Manager5bf400 calls Player518350. A stage
///request must not inherit this save-specific queue flush from normalization.
pub(crate) fn capture_at_manager_save(state: &mut State) -> Result<SavedPlayerItems, String> {
    crate::player_items::flush_queued(state)?;
    normalize_player(state)?;
    SavedPlayerItems::copy_from(
        state
            .player_items
            .as_ref()
            .expect("checked first Player owner"),
    )
}
