//! Validity of canonical Player active words after actual unported writers.
//! This stores no charge, inventory, entity roster, timer or replay value.
use crate::{player_items::PlayerItems, State};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum Word {
    ActiveId,
    Charge,
    Subordinate,
    Reserve,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Frontier {
    MissingHistory,
    ActiveUse { active_id: u32, source_pc: u32 },
    AutomaticCharge { active_id: u32, source_pc: u32 },
    PocketEffect { source_pc: u32 },
    GrantEffect { collectible_id: u32 },
    DamageEffect { source_pc: u32 },
    Query { source_pc: u32, field: String },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum EffectFrontier {
    HudFlash573290,
    BatteryEffect48,
    RoomClearTail {
        source_pc: u32,
    },
    /// An unported operator inside an otherwise ported 522250 leaf.
    UseLeaf {
        active_id: u32,
        source_pc: u32,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct History {
    pub unowned: BTreeMap<Word, Frontier>,
    /// Reached HUD, effect and later caller gaps do not erase completed D78
    /// or D8C writes. This records missing operators, not effect seed/value
    /// copies or a second queue of actors. A RoomClear tail can include
    /// healing or familiar effects and is not mere presentation evidence.
    pub unowned_effects: BTreeSet<EffectFrontier>,
}

impl History {
    pub(crate) fn fresh() -> Self {
        Self {
            unowned: BTreeMap::new(),
            unowned_effects: BTreeSet::new(),
        }
    }

    pub(crate) fn saved_words(&self) -> Self {
        let mut saved = self.clone();
        //518350/5116B0 copy D74/D78/D8C, not subordinate D7C. The DTO
        //cannot transfer knowledge of a word whose value it never copied.
        saved
            .unowned
            .entry(Word::Subordinate)
            .or_insert(Frontier::MissingHistory);
        saved
    }
}

pub(crate) fn owns(owner: &PlayerItems, word: Word) -> bool {
    owner
        .active_charge_history
        .as_ref()
        .is_some_and(|history| !history.unowned.contains_key(&word))
}

/// Actual D74 is readable only while its independent source word is owned.
/// Query/inventory completeness and raw zero do not establish this owner.
pub(crate) fn active_id(owner: &PlayerItems) -> Option<u32> {
    owns(owner, Word::ActiveId).then_some(owner.active_id)
}

pub(crate) fn invalidate(state: &mut State, words: &[Word], frontier: Frontier) {
    //Legacy omission remains absent; never reconstruct it from active_id0,
    //a released key, a cached output or inventory membership.
    if let Some(owner) = state.player_items.as_mut() {
        invalidate_owner(owner, words, frontier);
    }
}

pub(crate) fn invalidate_owner(owner: &mut PlayerItems, words: &[Word], frontier: Frontier) {
    if let Some(history) = owner.active_charge_history.as_mut() {
        for word in words {
            history
                .unowned
                .entry(*word)
                .or_insert_with(|| frontier.clone());
        }
    }
}

/// A reached/possible omitted509E10 call never changes D74/D7C. Read only
/// independently owned canonical operands to bound its D78/D8C footprint.
pub(crate) fn note_unowned_setter_delta(state: &mut State, delta: i32, source_pc: u32) {
    let Some(owner) = state.player_items.as_ref() else {
        return;
    };
    let id = active_id(owner);
    if id == Some(0) {
        return;
    }
    let charge = owns(owner, Word::Charge).then_some(owner.active_charge);
    let config = id.and_then(crate::items::definition);
    let charge_may_change = match (charge, config) {
        (Some(old), Some(config)) => old.wrapping_add(delta).min(config.max_charges) != old,
        _ => true,
    };
    let mut reserve_may_change = if id.is_some_and(|id| matches!(id, 164 | 289 | 294 | 338)) {
        false
    } else {
        match (charge, config) {
            (Some(old), Some(config)) => {
                let requested = old.wrapping_add(delta);
                let target = requested.min(config.max_charges);
                requested
                    .wrapping_sub(target)
                    .max(0)
                    .min(config.max_charges)
                    > 0
            }
            _ => true,
        }
    };
    if reserve_may_change {
        reserve_may_change = crate::item_queries::AppearanceQueries::new(
            owner,
            state.weapon_cache.primary,
            state.player.entity_source,
            None,
        )
        .have(63)
            != Some(false);
    }
    let rising_may_happen = match (charge, config) {
        (Some(old), Some(config)) => old.wrapping_add(delta).min(config.max_charges) > old,
        _ => true,
    };
    let mut words = Vec::new();
    if charge_may_change {
        words.push(Word::Charge);
    }
    if reserve_may_change {
        words.push(Word::Reserve);
    }
    invalidate(
        state,
        &words,
        Frontier::Query {
            source_pc,
            field: format!("Possible unimplemented509E10(D78{delta:+})"),
        },
    );
    if rising_may_happen {
        note_effect(state, EffectFrontier::HudFlash573290);
    }
    if reserve_may_change {
        note_effect(state, EffectFrontier::BatteryEffect48);
    }
}

/// Reached515F40 can select TEMP326 cooldown1. Preserve the actual D78
/// branch: nonzero only calls509E10(old-1); zero may invoke5172E8.
/// This records an omitted writer, without running an invented326 effect.
pub(crate) fn note_possible_temporary326(state: &mut State) {
    let Some(owner) = state.player_items.as_ref() else {
        return;
    };
    if owns(owner, Word::Charge) && owner.active_charge != 0 {
        note_unowned_setter_delta(state, -1, 0x5173a3);
        return;
    }
    //Unknown D78 cannot exclude zero; the virtual effect has no closed
    //active-word footprint. A known nonzero branch never loses D74/D7C.
    invalidate(
        state,
        &[
            Word::ActiveId,
            Word::Charge,
            Word::Subordinate,
            Word::Reserve,
        ],
        Frontier::Query {
            source_pc: 0x5172e8,
            field: "Possible unowned TEMP326 virtual damage callback".into(),
        },
    );
}

pub(crate) fn note_effect(state: &mut State, frontier: EffectFrontier) {
    //A reached missing world Init/caller is an actual unowned producer
    //footprint. Do not certify a later ordinary placement from the port's
    //unchanged list, or confuse this frontier with the completed charge words.
    match &frontier {
        EffectFrontier::BatteryEffect48 => crate::ordinary_placement::invalidate(
            state,
            crate::ordinary_placement::Frontier::Factory {
                entity_type: 1000,
                variant: 48,
                subtype: 0,
            },
        ),
        EffectFrontier::RoomClearTail { source_pc } => crate::ordinary_placement::invalidate(
            state,
            crate::ordinary_placement::Frontier::PlayerUpdate {
                field: format!("Unimplemented RoomClear tail at {source_pc:#x}"),
            },
        ),
        EffectFrontier::UseLeaf {
            active_id,
            source_pc,
        } => crate::ordinary_placement::invalidate(
            state,
            crate::ordinary_placement::Frontier::PlayerUpdate {
                field: format!("Unported active{active_id} leaf operator at {source_pc:#x}"),
            },
        ),
        EffectFrontier::HudFlash573290 => {}
    }
    if let Some(history) = state
        .player_items
        .as_mut()
        .and_then(|owner| owner.active_charge_history.as_mut())
    {
        history.unowned_effects.insert(frontier);
    }
}

//Authoring checklist, not a runtime blanket invalidator:
// * actual admitted unsupported522250/5295C0 dispatch: affected canonical words;
// * actual164/289/294/338 automatic D78 update, not merely holding that ID;
// * actual116/max1/zero-charge449 timer boundary, not every released frame;
// * actual unported pocket effect invocation and reached foreign damage use;
// * actual Add effect with an unclosed charge write footprint;
// * source fresh519350 and Save/restore carry this optional validity only.
//Generic ItemQueryHistory uncertainty alone is not any of these events.

/// Reached52A640 charge/input prefix. `full` is the caller's real Phase::Update,
/// not a reconstructed Native age. This does not claim unknown active Use.
pub(crate) fn before_advance(state: &mut State, controls_enabled: bool, full: bool) {
    use crate::active_charges::{Frontier, Word};
    if state.player.dead {
        return;
    }
    let Some(owner) = state.player_items.as_ref() else {
        return;
    };
    //52A665 and52A66B..7D return before both automatic and input work.
    if state
        .player
        .entity_source
        .is_some_and(|source| source.entity_type == 1 && source.variant == 1)
        || (owner.active_state_d84 != 0 && owner.active_state_d80 != 0)
    {
        return;
    }
    if state
        .player
        .entity_source
        .is_none_or(|source| source.entity_type != 1 || source.variant != 0)
    {
        crate::active_charges::invalidate(
            state,
            &[
                Word::ActiveId,
                Word::Charge,
                Word::Subordinate,
                Word::Reserve,
            ],
            Frontier::Query {
                source_pc: 0x52a640,
                field: "actual Player entity".into(),
            },
        );
        return;
    }
    if !crate::active_charges::owns(owner, Word::ActiveId) {
        return;
    }
    let id = owner.active_id;
    if id == 0 {
        return;
    }
    let Some(config) = crate::items::definition(id) else {
        return;
    };
    let maximum = config.max_charges;
    let charge_known = crate::active_charges::owns(owner, Word::Charge);
    let charge = owner.active_charge;
    let subordinate = owner.active_subordinate_charge;
    let subordinate_known = crate::active_charges::owns(owner, Word::Subordinate);
    let d80 = owner.active_state_d80;
    let d84 = owner.active_state_d84;
    let nine_volt = if full && charge_known && maximum == 1 && charge == 0 {
        crate::item_queries::AppearanceQueries::new(
            owner,
            state.weapon_cache.primary,
            state.player.entity_source,
            None,
        )
        .have(116)
    } else {
        Some(false)
    };

    let mut after_automatic = charge;
    if full {
        //52A71A..736 ->52AAC0: the four timed actives add one directly.
        //This does not call509E10 or invent a full-charge sound.
        if matches!(id, 164 | 289 | 294 | 338) && charge_known && charge < maximum {
            after_automatic = charge.wrapping_add(1);
            state
                .player_items
                .as_mut()
                .expect("checked active owner")
                .active_charge = after_automatic;
        }
        //The max1 NineVolt timer also reads actual manager counters when it
        //starts. Its unported conditional D7C/store-to-D78 boundary remains
        //specific; ordinary absent116 or max!=1 clears D7C literally.
        if maximum == 1 && !charge_known {
            //The unknown D78 test can choose either a literal clear or the
            //actual NineVolt timer path. Preserve neither stale D7C value.
            crate::active_charges::invalidate(
                state,
                &[Word::Subordinate],
                Frontier::AutomaticCharge {
                    active_id: id,
                    source_pc: 0x52a77e,
                },
            );
        } else if maximum == 1 && after_automatic == 0 && nine_volt != Some(false) {
            crate::active_charges::invalidate(
                state,
                &[Word::Subordinate],
                Frontier::AutomaticCharge {
                    active_id: id,
                    source_pc: 0x52a78c,
                },
            );
            if !subordinate_known || subordinate.wrapping_add(1) > 449 {
                crate::active_charges::invalidate(
                    state,
                    &[Word::Charge],
                    Frontier::AutomaticCharge {
                        active_id: id,
                        source_pc: 0x52a7ae,
                    },
                );
            }
        } else if maximum != 1 || charge_known {
            state
                .player_items
                .as_mut()
                .expect("checked active owner")
                .active_subordinate_charge = 0;
            //An actual literal source overwrite may refresh this word inside
            //an existing history; legacy omitted history remains absent.
            if let Some(history) = state
                .player_items
                .as_mut()
                .and_then(|owner| owner.active_charge_history.as_mut())
            {
                history.unowned.remove(&Word::Subordinate);
            }
        }
    }
    if !controls_enabled || state.player.input_cooldown > 0 {
        return;
    }
    let requested =
        state.public_actions.active_triggered || (id == 326 && state.public_actions.active_pressed);
    if !requested {
        return;
    }
    let owner = state.player_items.as_ref().expect("checked active owner");
    if !crate::active_charges::owns(owner, Word::Charge) {
        crate::active_charges::invalidate(
            state,
            &[
                Word::ActiveId,
                Word::Charge,
                Word::Subordinate,
                Word::Reserve,
            ],
            Frontier::ActiveUse {
                active_id: id,
                source_pc: 0x52a94d,
            },
        );
        return;
    }
    let current_charge = owner.active_charge;
    if id != 326 && current_charge != maximum && d80 == 0 && d84 <= 0 {
        //The zero-charge path additionally reaches actualHave205 before the
        //possible SharpPlug damage/refill callback. A known positive partial
        //charge skips that path and is a source-owned refusal.
        let sharp_plug_absent = current_charge != 0
            || crate::item_queries::AppearanceQueries::new(
                owner,
                state.weapon_cache.primary,
                state.player.entity_source,
                None,
            )
            .have(205)
                == Some(false);
        if sharp_plug_absent {
            return;
        }
        crate::active_charges::invalidate(
            state,
            &[
                Word::ActiveId,
                Word::Charge,
                Word::Subordinate,
                Word::Reserve,
            ],
            Frontier::ActiveUse {
                active_id: id,
                source_pc: 0x52aad0,
            },
        );
        return;
    }
    //Only the ported leaves (active_items::USES) close their Use/discharge.
    //Other reached per-ID guards/effects are conditional unavailable domains,
    //not a declaration that522250 actually ran. Empty and known refusals above
    //preserve the word owner; generic query uncertainty alone never clears it.
    if crate::active_items::ported(id).is_none() || d80 != 0 || d84 != 0 {
        crate::active_charges::invalidate(
            state,
            &[
                Word::ActiveId,
                Word::Charge,
                Word::Subordinate,
                Word::Reserve,
            ],
            Frontier::ActiveUse {
                active_id: id,
                source_pc: 0x52aa26,
            },
        );
    }
}

/// PlayerManager5BF1B0 -> Player50A2B0, invoked only after the actual
/// descriptor-clear award caller. Source text only; not an event test hook.
pub(crate) fn room_clear(state: &mut State) -> Result<(), String> {
    //The real later tail is reached even when a conditional charge operand
    //is unavailable. Do not let an early partial-prefix return certify its
    //unimplemented healing/familiar producer footprint as absent.
    room_clear_charge(state)?;
    crate::active_charges::note_effect(
        state,
        crate::active_charges::EffectFrontier::RoomClearTail {
            source_pc: 0x50a2f9,
        },
    );
    Ok(())
}

fn room_clear_charge(state: &mut State) -> Result<(), String> {
    use crate::active_charges::{Frontier, Word};
    let Some(owner) = state.player_items.as_ref() else {
        return Ok(());
    };
    if state
        .player
        .entity_source
        .is_none_or(|source| source.entity_type != 1 || source.variant != 0)
    {
        crate::active_charges::invalidate(
            state,
            &[Word::Charge, Word::Reserve],
            Frontier::Query {
                source_pc: 0x5bf1db,
                field: "actual PlayerManager charge caller".into(),
            },
        );
        return Ok(());
    }
    if !crate::active_charges::owns(owner, Word::ActiveId) {
        return Ok(());
    }
    let active_id = owner.active_id;
    //Actual50A2C3/CB skips323/326, not a config-derived approximation.
    //Active0 reaches509E10's no-op without requiring its unused D78 value.
    if active_id != 0 && !matches!(active_id, 323 | 326) {
        let Some(config) = crate::items::definition(active_id) else {
            crate::active_charges::invalidate(
                state,
                &[Word::Charge, Word::Reserve],
                Frontier::Query {
                    source_pc: 0x50a3f0,
                    field: "actual active ItemConfig".into(),
                },
            );
            return Ok(());
        };
        if !crate::active_charges::owns(owner, Word::Charge) {
            return Ok(());
        }
        let old = owner.active_charge;
        let mut target = old.wrapping_add(1);
        //The source comparison is max−1 == old+1, not old==max−1.
        //If false, the pure Have3 result cannot change the charge prefix.
        if config.max_charges.wrapping_sub(1) == target {
            let aaa = crate::item_queries::AppearanceQueries::new(
                owner,
                state.weapon_cache.primary,
                state.player.entity_source,
                None,
            )
            .has_trinket(3);
            match aaa {
                Some(false) => {}
                Some(true) => {
                    target = old.wrapping_add(2);
                    state.audio.admit(
                        &mut state.boot_lifecycle.shared,
                        state.system_clock.system_counter.wrapping_sub(1),
                        crate::audio::SoundRequest {
                            id: 170,
                            volume: 1.0,
                            delay: 2,
                            looping: false,
                            pitch: 1.0,
                        },
                    );
                    crate::familiars::admit_battery(state)?;
                }
                None => {
                    //This actual conditional target is unresolved. Retain
                    //the current canonical value rather than inventing a
                    //specific trinket result, sound or local continuation.
                    crate::active_charges::invalidate(
                        state,
                        &[Word::Charge],
                        Frontier::Query {
                            source_pc: 0x50a401,
                            field: "HaveTrinket3 extra charge".into(),
                        },
                    );
                    return Ok(());
                }
            }
        }
        crate::active_items::set_charge(state, target)?;
    }
    Ok(())
}
