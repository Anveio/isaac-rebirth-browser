//! Concrete Player518350/5116b0 owned item stores, separate from live
//! presentation, pending grants, temporary effects and evaluated stats.
use crate::{items::Rng, player_items::PlayerItems, State};
use serde::{Deserialize, Serialize};

const CHILD_SHIFTS: [u32; 3] = [5, 9, 7];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedPlayerItems {
    ///518577 copies347 slots beginning Player+e1c into Saved+893c.
    pub owned_counts: Vec<i32>,
    ///5186b0/5186e0/518708 copy current states, not initializer seeds.
    pub item_rng_states: Vec<u32>,
    pub trinket_rng_states: Vec<u32>,
    pub pocket_rng_states: Vec<u32>,
    /// Saved+8908/890c/8904/8910; D80/D84 are not saved fields.
    pub active_id: u32,
    pub active_charge: i32,
    pub active_subordinate_charge: i32,
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
            owned_counts: owner.owned_counts.clone(),
            item_rng_states: current_states(&owner.item_rngs, 347)?,
            trinket_rng_states: current_states(&owner.trinket_rngs, 62)?,
            pocket_rng_states: current_states(&owner.pocket_rngs, 41)?,
            active_id: owner.active_id,
            active_charge: owner.active_charge,
            active_subordinate_charge: owner.active_subordinate_charge,
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
        owner.active_subordinate_charge = self.active_subordinate_charge;
        owner.active_id = self.active_id;
        owner.active_charge = self.active_charge;
        owner.reserve_charge = self.reserve_charge;
        Ok(())
    }
}

///54f373 flushes52ee40 before Manager5bf400 calls Player518350. Normal
///518120 has no temporary/costume mutations when those owners are absent.
/// Their real clear/evaluation callbacks remain required before wider saves.
pub(crate) fn capture_at_manager_save(state: &mut State) -> Result<SavedPlayerItems, String> {
    crate::player_items::flush_queued(state)?;
    let owner = state
        .player_items
        .as_mut()
        .ok_or("Player save requires the actual first Player item owner")?;
    if !owner.temporary_collectibles.is_empty()
        || !matches!(owner.active_id, 0 | 45)
        || owner.active_state_d80 != 0
        || owner.active_state_d84 != 0
    {
        return Err("Player save normalization requires an unmapped temporary/active owner".into());
    }
    //5182c3/5182cd store zero after cancellation. This supported owner is
    //already neutral; no transient owner or missing BF5 value is invented.
    owner.active_state_d80 = 0;
    owner.active_state_d84 = 0;
    SavedPlayerItems::copy_from(owner)
}
