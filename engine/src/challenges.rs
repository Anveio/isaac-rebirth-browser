//! Source-authored Challenge definitions and the actual Player starting grant callback.
//! Resource inputs are XML bytes read before boot, never cached stats or actor state.
use crate::{items, player_items, State};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const AUTHORED_XML: &str = include_str!(concat!(env!("OUT_DIR"), "/challenges.xml"));

///The boot input contains authored resource bytes only. Player stats, item
///counts, actor state and captured seeds are not accepted on this surface.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceInputs {
    pub challenges_xml: String,
    ///Complete authored STB bytes, never a decoded actor/state DTO.
    pub special_rooms: Option<Vec<u8>>,
}

///Exact594620 arguments stored at54eb4e/54eb5e. No semantic alias to
///seeded-run or achievement flags is inferred from the Challenge ID.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunParameters {
    pub source_r8: u32,
    pub source_r9: bool,
}

impl RunParameters {
    pub(crate) const CHALLENGE_MENU: Self = Self {
        source_r8: 0,
        source_r9: true,
    };
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CurseSample {
    pub initial_local_state: u32,
    pub final_local_state: u32,
    pub threshold: u32,
    pub threshold_result: Option<u32>,
    pub selection_result: Option<u32>,
    pub selected_mask: u32,
    pub filtered_mask: u32,
    pub room_adjustment_result: u32,
    pub local_draws: u32,
}

///58bffb copies CURRENT primary state into an independent RNG35. The
///conditional sampler and subsequent Bound2 never advance the primary stream.
pub(crate) fn sample_initial_curse(
    primary_state: u32,
    stage: i32,
    definition: &Definition,
    profile: &crate::Profile,
    parameters: RunParameters,
    has_black_candle: bool,
) -> Result<CurseSample, String> {
    if profile.achievements.len() < 179 || profile.counters.len() != 95 {
        return Err("curse sampler requires actual Persistent achievement/counter owners".into());
    }
    let mode_one = parameters.source_r8 == 1;
    let mut threshold = if mode_one { 10 } else { 80 };
    if profile.achievements[4] {
        threshold = if mode_one { 5 } else { 30 };
    }
    if profile.achievements[33] {
        threshold = if mode_one { 3 } else { 10 };
    }
    if profile.counters[10] != 0 {
        threshold = if mode_one { 2 } else { 5 };
    }
    let mut local = items::Rng::new(primary_state, [5, 9, 7]);
    let mut draws = 0;
    let mut threshold_result = None;
    let mut selection_result = None;
    let mut selected_mask = 0;
    if !has_black_candle {
        let result = local.bounded(threshold);
        draws += 1;
        threshold_result = Some(result);
        if result == 0 {
            let choice = local.bounded(6);
            draws += 1;
            selection_result = Some(choice);
            selected_mask = match choice {
                0 if stage % 2 == 1 && stage <= 7 => 2,
                0 => 0,
                1 => 4,
                2 => 1,
                3 => 8,
                4 => 32,
                5 => 64,
                _ => unreachable!("bounded6"),
            };
        }
    }
    let filtered_mask = definition.apply_curse_filter(selected_mask);
    //58c089 consumes this even source_r8=0/filter127;58c097 then restores
    //PRIMARY pointer before58c0ba Next. No additional localNext is invented.
    let room_adjustment_result = local.bounded(2);
    draws += 1;
    Ok(CurseSample {
        initial_local_state: primary_state,
        final_local_state: local.state,
        threshold,
        threshold_result,
        selection_result,
        selected_mask,
        filtered_mask,
        room_adjustment_result,
        local_draws: draws,
    })
}

impl ResourceInputs {
    pub(crate) fn catalog(&self) -> Result<Catalog, String> {
        if self.challenges_xml.len() > 65_536 {
            return Err("Challenge XML input exceeds 64 KiB".into());
        }
        Catalog::from_xml(&self.challenges_xml)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Definition {
    pub id: u32,
    pub name: String,
    pub starting_items: Vec<u32>,
    pub starting_trinkets: Vec<u32>,
    pub starting_pill: i32,
    pub starting_card: i32,
    pub end_stage: u32,
    pub player_type: u32,
    pub achievements: Vec<u32>,
    pub soul_hp: i32,
    pub black_hp: i32,
    pub can_shoot: bool,
    pub alternate_path: bool,
    pub max_damage: bool,
    pub min_fire_rate: bool,
    pub min_shot_speed: bool,
    pub get_curse: u32,
    pub curse_filter: u32,
    pub room_filter: BTreeSet<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Catalog {
    definitions: BTreeMap<u32, Definition>,
}

impl Catalog {
    pub(crate) fn authored() -> Result<Self, String> {
        Self::from_xml(AUTHORED_XML)
    }

    pub fn from_xml(xml: &str) -> Result<Self, String> {
        //5961f0 starts each working row from the source default template and
        //597490 preserves the comma-list item order, including repetitions.
        let doc = roxmltree::Document::parse(xml).map_err(|error| error.to_string())?;
        let root = doc.root_element();
        if root.tag_name().name() != "challenges" {
            return Err("Challenge resource requires a challenges root".into());
        }
        let mut definitions = BTreeMap::new();
        for row in root.children().filter(|node| node.is_element()) {
            if row.tag_name().name() != "challenge" {
                return Err("Challenge resource contains an unsupported element".into());
            }
            let number = |name: &str, default: u32| -> Result<u32, String> {
                row.attribute(name).map_or(Ok(default), |value| {
                    value
                        .parse()
                        .map_err(|_| format!("invalid Challenge {name}"))
                })
            };
            let signed = |name: &str, default: i32| -> Result<i32, String> {
                row.attribute(name).map_or(Ok(default), |value| {
                    value
                        .parse()
                        .map_err(|_| format!("invalid Challenge {name}"))
                })
            };
            let boolean = |name: &str, default: bool| -> Result<bool, String> {
                row.attribute(name)
                    .map_or(Ok(default), |value| match value {
                        "true" => Ok(true),
                        "false" => Ok(false),
                        _ => Err(format!("invalid Challenge {name}")),
                    })
            };
            let list = |name: &str| -> Result<Vec<u32>, String> {
                let Some(value) = row.attribute(name) else {
                    return Ok(Vec::new());
                };
                if value.trim().is_empty() {
                    return Ok(Vec::new());
                }
                value
                    .split(',')
                    .map(|word| {
                        word.trim()
                            .parse()
                            .map_err(|_| format!("invalid Challenge {name} list"))
                    })
                    .collect()
            };
            let id = number("id", 0)?;
            if id == 0 {
                return Err("Challenge id must be nonzero".into());
            }
            let definition = Definition {
                id,
                name: row.attribute("name").unwrap_or("").to_owned(),
                starting_items: list("startingitems")?,
                starting_trinkets: list("startingtrinkets")?,
                starting_pill: signed("startingpill", -1)?,
                starting_card: signed("startingcard", 0)?,
                end_stage: number("endstage", 12)?,
                player_type: number("playertype", 0)?,
                achievements: list("achievements")?,
                soul_hp: signed("soulhp", 0)?,
                black_hp: signed("blackhp", 0)?,
                can_shoot: boolean("canshoot", true)?,
                alternate_path: boolean("altpath", false)?,
                max_damage: boolean("maxdamage", false)?,
                min_fire_rate: boolean("minfirerate", false)?,
                min_shot_speed: boolean("minshotspeed", false)?,
                get_curse: number("getcurse", 0)?,
                curse_filter: number("cursefilter", 0)?,
                room_filter: list("roomfilter")?.into_iter().collect(),
            };
            if definitions.insert(id, definition).is_some() {
                return Err("duplicate Challenge id".into());
            }
        }
        if definitions.keys().copied().ne(1..=20) {
            return Err("Challenge menu requires the authored twenty definition IDs".into());
        }
        Ok(Self { definitions })
    }

    pub fn get(&self, id: u32) -> Result<&Definition, String> {
        self.definitions
            .get(&id)
            .ok_or_else(|| format!("Challenge {id} definition absent"))
    }

    pub(crate) fn adjacent_unlocked(
        &self,
        selected: u32,
        direction: i32,
        achievements: &[bool],
    ) -> Result<u32, String> {
        let mut candidate = selected;
        for _ in 0..20 {
            candidate = (candidate as i32 - 1 + direction).rem_euclid(20) as u32 + 1;
            if self.get(candidate)?.is_unlocked(achievements) {
                return Ok(candidate);
            }
        }
        Err("profile has no unlocked authored Challenge".into())
    }
}

impl Definition {
    pub(crate) fn is_unlocked(&self, achievements: &[bool]) -> bool {
        self.achievements
            .iter()
            .all(|&id| achievements.get(id as usize).copied().unwrap_or(false))
    }
    ///58c600 applies this after the real unconditional threshold draw and its
    ///conditional bounded6 draw. This pure operation never replaces that sampler.
    pub(crate) fn apply_curse_filter(&self, selected: u32) -> u32 {
        (selected | self.get_curse) & !self.curse_filter
    }

    /// Current narrow runtime scope: the declared item-focused Challenge1
    ///view, normal Isaac, neutral filters. Other definitions remain parsed.
    pub(crate) fn validate_initial_scope(&self) -> Result<(), String> {
        if self.id != 1
            || self.player_type != 0
            || !self.achievements.is_empty()
            || !self.starting_trinkets.is_empty()
            || self.starting_pill != -1
            || self.starting_card != 0
            || self.soul_hp != 0
            || self.black_hp != 0
            || !self.can_shoot
            || self.alternate_path
            || self.max_damage
            || self.min_fire_rate
            || self.min_shot_speed
            || !self.room_filter.is_empty()
            || self.get_curse != 0
            || self.curse_filter != 127
        {
            return Err(
                "Challenge initialization outside the implemented item-focused scope".into(),
            );
        }
        if self.starting_items.contains(&330)
            && self
                .starting_items
                .iter()
                .any(|&id| !matches!(id, 1 | 4 | 118 | 330))
        {
            return Err("Challenge Soy Milk composition outside the mapped cache frontier".into());
        }
        //Independent52E280 startup order proof. Each prefix uses its actual
        //common52CC90/cache/pool callback; this grants no firing/history owner.
        let epic_startup = matches!(
            self.starting_items.as_slice(),
            [168, 2] | [2, 168] | [4, 168, 2] | [4, 2, 168]
        );
        for &id in &self.starting_items {
            items::definition(id).ok_or("Challenge starting collectible config absent")?;
            if !(matches!(id, 1 | 4 | 6 | 68 | 114 | 118 | 329 | 304 | 330)
                || (epic_startup && matches!(id, 2 | 168)))
            {
                return Err("Challenge starting grant outside the mapped item frontier".into());
            }
        }
        Ok(())
    }
}

///519bf9 reads actual Config98 during Player construction, before Level.Init.
///Co-op variant1/30e8==26 and other character constructors are unadmitted.
pub(crate) fn configure_player(state: &mut State, definition: &Definition) -> Result<(), String> {
    definition.validate_initial_scope()?;
    if state.weapon_admission.source_player_type != Some(definition.player_type) {
        return Err("Challenge Player constructor requires its actual source type owner".into());
    }
    state.player.can_shoot = definition.can_shoot;
    //The admitted definition explicitly has neutral caps. Its future nonzero
    //caps need actual scalar owners; this consumer does not clone an evaluator.
    Ok(())
}

///54ec4f→Manager5bf160→Player52e1a0 runs AFTER Level.Init/initial Save
///and BEFORE Game54eaa0 fade. Every actual AddCollectible evaluates its cache
///before581750 removes that item from the pool; final511f10 follows the loop.
pub(crate) fn initialize_player(state: &mut State, definition: &Definition) -> Result<(), String> {
    definition.validate_initial_scope()?;
    for &id in &definition.starting_items {
        let config = items::definition(id).ok_or("Challenge collectible config absent")?;
        player_items::add_collectible(state, id, config.max_charges, true)?;
        state
            .item_pools
            .as_mut()
            .ok_or("Challenge ItemPool owner absent")?
            .removed[id as usize] = true;
    }
    //52e340 invokes511f10 unconditionally, even when the requested mask is0.
    crate::player_stats::evaluate_cache(state)?;
    state.player.initialized_3074 = true;
    Ok(())
}
