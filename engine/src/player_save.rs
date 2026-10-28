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
    pub move_speed_inputs: Option<crate::pill_effect_history::MoveSpeedView>,
    ///New non-Pill terms survive independently of missing Pill words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub move_speed_terms: Option<crate::player_items::MoveSpeedInputs>,
    ///Only truly omitted legacy data may seed the old copied13/14 pair.
    #[serde(
        default,
        skip_serializing_if = "crate::pill_effect_history::SavedPillEffectHistory::is_legacy_missing"
    )]
    pub pill_effect_history: crate::pill_effect_history::SavedPillEffectHistory,
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
    ///5184BE/CA/518512 copy D3C/D40/D58;511A49/55/9D restore
    ///the same words. Missing legacy data never inherits constructor Luck.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub luck_inputs: Option<SavedLuckInputs>,
    ///5185FD/518615 copy E14/E18, restored at51172A/51173C.
    ///Only the actual owned held-slot domain can issue this saved value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trinket_slots: Option<[u32; 2]>,
    ///51861C..6A1 saves actual registration configs, not collectible counts.
    ///The first Continue caller admits only an actually owned empty list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub costume_registrations:
        Option<crate::player_costume_registration::CostumeRegistrationHistory>,
    ///518577 copies347 slots beginning Player+e1c into Saved+893c.
    pub owned_counts: Vec<i32>,
    ///Only the actual copied count-table domain is saved here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_history: Option<crate::item_queries::ItemQueryHistory>,
    ///5186b0/5186e0/518708 copy current states, not initializer seeds.
    pub item_rng_states: Vec<u32>,
    pub trinket_rng_states: Vec<u32>,
    pub pocket_rng_states: Vec<u32>,
    ///518570/590 copy both actual records;511B25/33 restores them whole.
    ///Omitted legacy history stays unknown, never constructor-empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub held_pockets: Option<crate::pockets::HeldPockets>,
    #[serde(default)]
    pub ordinary_producer_history: Option<crate::ordinary_placement::Coverage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pickup_grant_history: Option<crate::pickup_world_grants::History>,
    /// Saved+8908/890c/8904/8910; D80/D84 are not saved fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_charge_history: Option<crate::active_charges::History>,
    pub active_id: u32,
    pub active_charge: i32,
    pub save_word_d88: i32,
    pub reserve_charge: i32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedLuckInputs {
    pub source_d3c: i32,
    pub source_d40: i32,
    pub source_d58_bits: u32,
}

impl SavedLuckInputs {
    fn capture(value: crate::player_items::LuckInputs) -> Self {
        Self {
            source_d3c: value.source_d3c,
            source_d40: value.source_d40,
            source_d58_bits: value.source_d58.to_bits(),
        }
    }
    fn restore(self) -> Result<crate::player_items::LuckInputs, String> {
        let value = f32::from_bits(self.source_d58_bits);
        if !value.is_finite() {
            return Err("saved D58 Luck input is nonfinite".into());
        }
        Ok(crate::player_items::LuckInputs {
            source_d3c: self.source_d3c,
            source_d40: self.source_d40,
            source_d58: value,
        })
    }
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
            move_speed_inputs: crate::pill_effect_history::MoveSpeedView::from_owner(owner),
            move_speed_terms: owner.move_speed_inputs,
            pill_effect_history: crate::pill_effect_history::SavedPillEffectHistory::capture(owner),
            damage_inputs: owner.damage_inputs,
            range_inputs: owner.range_inputs,
            fire_delay_inputs: owner.fire_delay_inputs,
            shot_speed_inputs: owner.shot_speed_inputs,
            luck_inputs: owner.luck_inputs.map(SavedLuckInputs::capture),
            trinket_slots: owner
                .query_history
                .as_ref()
                .filter(|history| history.trinket_slots)
                .map(|_| owner.trinket_slots),
            costume_registrations: owner.costume_registrations.clone(),
            owned_counts: owner.owned_counts.clone(),
            query_history: owner
                .query_history
                .as_ref()
                .map(|history| history.saved_counts()),
            item_rng_states: current_states(&owner.item_rngs, 347)?,
            trinket_rng_states: current_states(&owner.trinket_rngs, 62)?,
            pocket_rng_states: current_states(&owner.pocket_rngs, 41)?,
            held_pockets: owner.held_pockets.clone(),
            ordinary_producer_history: owner.ordinary_producer_history.clone(),
            pickup_grant_history: owner.pickup_grant_history.clone(),
            active_charge_history: owner
                .active_charge_history
                .as_ref()
                .map(crate::active_charges::History::saved_words),
            active_id: owner.active_id,
            active_charge: owner.active_charge,
            save_word_d88: owner.save_word_d88,
            reserve_charge: owner.reserve_charge,
        })
    }

    ///Explicit519350 constructor→5116B0 load seam. Constructor-owned TEMP
    ///and C08 stores are distinct from the copied saved count provenance.
    ///This method never restores an old live history or infers a costume list.
    pub(crate) fn restore_after_constructor(&self, owner: &mut PlayerItems) -> Result<(), String> {
        //The saved costume history may hold the items' configurations; the
        //destination constructor must still be the fresh empty Player.
        if self.costume_registrations.is_none()
            || owner
                .costume_registrations
                .as_ref()
                .is_none_or(|history| !history.configuration_ids.is_empty())
            || !owner.temporary_collectibles.is_empty()
            || owner.owned_counts.iter().any(|count| *count != 0)
        {
            return Err(
                "Continue destination requires its actual fresh empty Player constructor".into(),
            );
        }
        self.restore_into(owner)?;
        if let Some(history) = &mut owner.query_history {
            //5194A1→5E3F80 empties records, writes CF8=0 and D18=1.
            //51958F independently initializes C08=1.5116B0 does not
            //overwrite these owners;515F20 later evaluates all callbacks.
            history.temporary_records = true;
            history.temporary_source_cache = Some(crate::item_queries::TemporarySourceCache {
                mask_cf8: Some(0),
                pending_d18: Some(true),
            });
            history.primary_c08 = true;
            history.cached_callbacks = 0;
        }
        //Only the positive saved-empty list plus constructor-empty vector
        //admits this path. Nonempty511060 registration replay is separate.
        owner.costume_registrations = self.costume_registrations.clone();
        Ok(())
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
            || !owner.queue_is_empty()
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
        //Validate wire/history before changing any live restored field.
        let luck_inputs = self.luck_inputs.map(SavedLuckInputs::restore).transpose()?;
        if self
            .trinket_slots
            .is_some_and(|slots| slots.into_iter().any(|id| id >= 62))
        {
            return Err("saved trinket slots are outside the base Rebirth catalogue".into());
        }
        let legacy_pill = self.pill_effect_history.is_legacy_missing();
        if legacy_pill && self.move_speed_terms.is_some() {
            return Err(
                "Current saved movement terms require the explicit Pill history field".into(),
            );
        }
        let pill_history = self
            .pill_effect_history
            .restore(self.move_speed_inputs, None)?;
        let movement = if legacy_pill {
            self.move_speed_inputs.map(|view| view.terms())
        } else {
            self.move_speed_terms
        };
        //511710 skips dummy ID0 and copies346 dwords from Saved+8940.
        owner.owned_counts[1..].copy_from_slice(&self.owned_counts[1..]);
        //Legacy None overwrites constructor knowledge. Even a current
        //DTO cannot transfer C08/TEMP/trinket facts absent their stores.
        owner.query_history = self
            .query_history
            .as_ref()
            .map(|history| history.saved_counts());
        if let Some(slots) = self.trinket_slots {
            owner.trinket_slots = slots;
            if let Some(history) = &mut owner.query_history {
                history.trinket_slots = true;
            }
        }
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
        owner.active_charge_history = self
            .active_charge_history
            .as_ref()
            .map(crate::active_charges::History::saved_words);
        owner.active_id = self.active_id;
        owner.active_charge = self.active_charge;
        owner.reserve_charge = self.reserve_charge;
        owner.held_pockets = self.held_pockets.clone();
        owner.ordinary_producer_history = self.ordinary_producer_history.clone();
        owner.pickup_grant_history = self.pickup_grant_history.clone();
        //51849A/5184A0 stores Saved88D8;511A25/511A2B restores D30.
        //None overwrites constructor Some(0) rather than inventing history.
        owner.damage_inputs = self.damage_inputs;
        //5184A6/5184AC saves88DC;511A31/511A37 restores the word.
        owner.range_inputs = self.range_inputs;
        //518482/488 and5184D6/4DC save D28/D44;51877E/785 and
        //51878C/793 include 13BC/13C0 in the saved112-byte block.
        //511820/82E/1A13/1A67 restore these same four words. None must
        //overwrite constructor neutrality when the saved owner is absent.
        owner.move_speed_inputs = movement;
        owner.pill_effect_history = pill_history;
        //A genuinely omitted legacy owner overwrites fresh constructor data.
        owner.luck_inputs = luck_inputs;
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
///
/// 518120 has no active-item gate: the held active (Player+D74, which the
/// grant 52cc90 writes at 52d1b7), its charges and reserve pass through to
/// the save. Its active work is 510420's costume removal for that item and
/// the D80/D84/BF5 stores.
pub(crate) fn normalize_player(state: &mut State) -> Result<(), String> {
    let owner = state
        .player_items
        .as_mut()
        .ok_or("Player save requires the actual first Player item owner")?;
    // No port writer sets D80/D84; a nonzero value would be an active use
    // whose transient owner cancellation (5182c3) is not ported.
    if owner.active_state_d80 != 0 || owner.active_state_d84 != 0 {
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
    //51828B clears temporary effects before51829B removes the active costume.
    // 510420(D74, 0): an item under a temporary effect keeps its costume
    // (Shoop da Whoop 49 excepted), as do 181/130/86/40/299; otherwise
    // 5102f0 erases the item config's costume entries. A known history only
    // holds configs whose grants registered there, so this never invents a
    // removal; an unknown history stays unknown. The null costume2 that
    // 518120 also removes (57bee0) is not a collectible registration.
    let active = owner.active_id;
    let kept = active != 49
        && owner
            .temporary_collectibles
            .iter()
            .any(|record| record.item_id == active);
    if active != 0 && !kept && !matches!(active, 181 | 130 | 86 | 40 | 299) {
        if let Some(history) = &mut owner.costume_registrations {
            history.configuration_ids.retain(|id| *id != active);
        }
    }
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
