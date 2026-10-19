//! Base Rebirth collectible configuration and the run-owned ItemPool manager.
//! Authored weights remain immutable intervals; rejection/depletion is applied
//! within those intervals by source 581380/581590, not by renormalizing them.
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct CollectibleDefinition {
    pub id: u32,
    pub kind: u8,
    pub name: &'static str,
    pub description: &'static str,
    pub graphic: Option<&'static str>,
    pub achievement: i32,
    pub cache_flags: u32,
    /// Native order: max hearts, red hearts, soul hearts, black hearts,
    /// bombs, keys, coins (ItemConfig +28..40).
    pub inventory: [i32; 7],
    pub max_charges: i32,
    pub cooldown: i32,
    pub special: bool,
}
///Actual base ItemConfig Kind2 namespace, separate from collectibles.
///Metadata alone grants no held-effect, costume or pickup350 capability.
#[derive(Debug)]
pub struct TrinketDefinition {
    pub id: u32,
    pub name: &'static str,
    pub description: &'static str,
    pub graphic: Option<&'static str>,
    pub achievement: i32,
    pub cache_flags: u32,
    ///Parser57E021's config+60 string; empty on all pinned base trinkets.
    pub costume_anm2: &'static str,
}
#[derive(Debug)]
pub struct PoolEntryDefinition {
    pub id: u32,
    pub weight: f32,
    pub decrease_by: f32,
    pub remove_on: f32,
}
#[derive(Debug)]
pub struct PoolDefinition {
    pub id: u32,
    pub name: &'static str,
    pub entries: &'static [PoolEntryDefinition],
}
include!(concat!(env!("OUT_DIR"), "/item_catalog.rs"));

pub fn definition(id: u32) -> Option<&'static CollectibleDefinition> {
    COLLECTIBLES
        .binary_search_by_key(&id, |entry| entry.id)
        .ok()
        .map(|index| &COLLECTIBLES[index])
}

///Look up an already typed Kind2 ID. This never infers kind from the ID.
pub fn trinket_definition(id: u32) -> Option<&'static TrinketDefinition> {
    TRINKETS
        .binary_search_by_key(&id, |entry| entry.id)
        .ok()
        .map(|index| &TRINKETS[index])
}

///Conservative Add registration readset. False closes an empty ANM/skip
///branch; true still needs its actual config+58 binding and caller proof.
///This metadata helper never grants registration or selected-pose capability.
pub(crate) fn collectible_add_may_register_costume(id: u32) -> bool {
    let Some(config) = definition(id) else {
        return false;
    };
    if COSTUME_COLLECTIBLE_IDS.binary_search(&id).is_err() {
        return false;
    }
    match config.kind {
        1 => ![122, 311, 313].contains(&id),
        3 => [40, 86, 130, 181].contains(&id),
        _ => true,
    }
}

/// Source 5c2d40/5c2d70, represented as its actual four native words.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Rng {
    pub state: u32,
    pub right_a: u32,
    pub left: u32,
    pub right_b: u32,
}
impl Rng {
    pub fn new(state: u32, shifts: [u32; 3]) -> Self {
        Self {
            state,
            right_a: shifts[0],
            left: shifts[1],
            right_b: shifts[2],
        }
    }
    pub fn raw(&mut self) -> u32 {
        self.state ^= self.state >> self.right_a;
        self.state ^= self.state << self.left;
        self.state ^= self.state >> self.right_b;
        self.state
    }
    pub fn bounded(&mut self, bound: u32) -> u32 {
        assert!(bound != 0, "native RNG bounded(0)");
        self.raw() % bound
    }
    pub(crate) fn uniform(&mut self) -> f32 {
        // Native unsigned conversion rounds to f32 BEFORE multiplication.
        // A rounded result of 1.0 is permitted by the original implementation.
        self.raw() as f32 * 2.328_306_4e-10_f32
    }
    pub fn words(self) -> [u32; 4] {
        [self.state, self.right_a, self.left, self.right_b]
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PoolEntry {
    pub id: u32,
    pub base_weight: f32,
    pub current_weight: f32,
    pub decrease_by: f32,
    pub remove_on: f32,
    pub eligible: bool,
    pub special: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Pool {
    pub id: u32,
    pub name: String,
    pub base_weight_total: f32,
    pub rng: [Rng; 2],
    pub special_counter: i32,
    pub entries: Vec<PoolEntry>,
}
///ItemPool+7C8/+7C9 per-ID bytes and the independent signed+9B4 count.
///Save581ED0 copies availability/count only; eligibility belongs to the
///current profile's Reset5820A0 and is absent from the saved representation.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrinketPool {
    pub available: Vec<bool>,
    pub count: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eligible: Option<Vec<bool>>,
}
impl TrinketPool {
    fn reset(achievements: &[bool]) -> Self {
        let eligible: Vec<bool> = TRINKET_ACHIEVEMENTS
            .iter()
            .map(|achievement| achievement.is_none_or(|id| id < 0 || achievements[id as usize]))
            .collect();
        let mut available = vec![true; 62];
        available[0] = false;
        available[47] = false;
        let count = available
            .iter()
            .zip(&eligible)
            .filter(|(available, eligible)| **available && **eligible)
            .count() as i32;
        Self {
            available,
            count,
            eligible: Some(eligible),
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.available.len() != 62
            || self.available[0]
            || self.available[47]
            || self
                .eligible
                .as_ref()
                .is_some_and(|bytes| bytes.len() != 62)
        {
            return Err(
                "trinket pool requires its actual 62 availability/eligibility bytes".into(),
            );
        }
        //Load581CF0 preserves the saved count while current-profile
        //eligibility may differ. Remove581770 decrements without clamping.
        //Each removal lowers both availability-count and this signed count.
        let available = self.available.iter().filter(|value| **value).count() as i32;
        if self.count < available - 60 || self.count > available {
            return Err("trinket pool has an inconsistent saved signed count".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemPools {
    pub seed: u32,
    pub root_rng: Rng,
    ///Actual ItemPool+49C words; omitted legacy/unrepresented state is unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary_rng: Option<Rng>,
    pub pools: Vec<Pool>,
    pub special_factor: f32,
    pub removed: Vec<bool>,
    pub selected: Vec<bool>,
    pub room_routes: [i32; 23],
    pub pill_effects: [i32; 10],
    ///Missing legacy data or an unrepresented GetTrinket callback is unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trinkets: Option<TrinketPool>,
}

///Close the selection readset for a complete Init350 batch before any Get.
///Explicit nonzero selectors bypass ItemPool, including unknown legacy pools.
///This proves neither the process type5 cell lifetime nor the body callbacks.
pub fn preflight_trinket_init_batch(
    pools: Option<&ItemPools>,
    subtypes: &[u32],
    current_achievements: Option<&[bool]>,
) -> Result<(), String> {
    let mut selections = 0usize;
    for &subtype in subtypes {
        if subtype == 0 {
            selections += 1;
        } else {
            trinket_definition(subtype).ok_or("Init350 requires a typed Base Kind2 config")?;
        }
    }
    if selections == 0 {
        return Ok(());
    }
    let pools = pools.ok_or("Init350 Get requires its actual ItemPool manager")?;
    let pool = pools
        .trinkets
        .as_ref()
        .ok_or("Init350 Get requires known availability")?;
    pool.validate()?;
    let manager = pools
        .secondary_rng
        .ok_or("Init350 Get requires known ItemPool+49C RNG words")?;
    if [manager.right_a, manager.left, manager.right_b] != [1, 5, 19] {
        return Err("Init350 Get requires the actual secondary shift2 RNG".into());
    }
    //Count-only admission considers every possible selected ID. It never
    //previews RNG, filters a result, or changes the real availability bytes.
    let mut count = pool.count;
    let mut remaining = None;
    for _ in 0..selections {
        let reset;
        let pool = if count <= 0 {
            let achievements = current_achievements
                .filter(|bytes| bytes.len() == 179)
                .ok_or("Init350 reset requires the actual current 179 profile bytes")?;
            reset = TrinketPool::reset(achievements);
            count = reset.count;
            remaining = None;
            &reset
        } else {
            pool
        };
        if remaining.is_none() {
            let eligible = pool
                .eligible
                .as_ref()
                .ok_or("Init350 Get requires known current-profile eligibility")?;
            let mut available = 0usize;
            for (id, (&available_byte, &eligible_byte)) in
                pool.available.iter().zip(eligible).enumerate()
            {
                if available_byte && eligible_byte {
                    trinket_definition(id as u32)
                        .ok_or("Init350 pool includes an unowned Kind2 continuation")?;
                    available += 1;
                }
            }
            remaining = Some(available);
        }
        let available = remaining.as_mut().expect("selection readset initialized");
        if *available == 0 {
            return Err("Init350 batch reaches the separate Get0 card-conversion owner".into());
        }
        *available -= 1;
        count -= 1;
    }
    Ok(())
}

impl ItemPools {
    pub fn new(seed: u32, achievements: &[bool]) -> Result<Self, String> {
        if seed == 0 || achievements.len() != 179 {
            return Err(
                "item pools require the actual nonzero initializer seed and 179 achievement bytes"
                    .into(),
            );
        }
        let mut root_rng = Rng::new(seed, [1, 9, 29]);
        let secondary_rng = Rng::new(root_rng.raw(), [1, 5, 19]);
        let pools = POOL_DEFINITIONS
            .iter()
            .map(|pool| {
                let rng = [
                    Rng::new(root_rng.raw(), [5, 9, 7]),
                    Rng::new(root_rng.raw(), [5, 9, 7]),
                ];
                let mut base_weight_total = 0.0_f32;
                let entries = pool
                    .entries
                    .iter()
                    .map(|entry| {
                        let config =
                            definition(entry.id).expect("validated authored pool collectible");
                        base_weight_total += entry.weight;
                        PoolEntry {
                            id: entry.id,
                            base_weight: entry.weight,
                            current_weight: entry.weight,
                            decrease_by: entry.decrease_by,
                            remove_on: entry.remove_on,
                            eligible: config.achievement < 0
                                || achievements[config.achievement as usize],
                            special: config.special,
                        }
                    })
                    .collect();
                Pool {
                    id: pool.id,
                    name: pool.name.into(),
                    base_weight_total,
                    rng,
                    special_counter: 0,
                    entries,
                }
            })
            .collect();
        //583159 resets trinket bytes after loading collectible pools and
        //before pill shuffles. Reset has no RNG consumer.
        let trinkets = Some(TrinketPool::reset(achievements));
        // 583040 shuffles the nine pill colors and 28 effects, then samples
        // each color without replacement. Keep these actual consumers so the
        // manager's complete RNG state remains its source state.
        let mut colors: Vec<usize> = (1..10).collect();
        let mut effects: Vec<i32> = (0..28).collect();
        for index in (1..colors.len()).rev() {
            colors.swap(index, root_rng.bounded(index as u32 + 1) as usize);
        }
        for index in (1..effects.len()).rev() {
            effects.swap(index, root_rng.bounded(index as u32 + 1) as usize);
        }
        let mut pill_effects = [-1; 10];
        for color in colors {
            let index = root_rng.bounded(effects.len() as u32) as usize;
            pill_effects[color] = effects.remove(index);
        }
        Ok(Self {
            seed,
            root_rng,
            secondary_rng: Some(secondary_rng),
            pools,
            special_factor: 1.0,
            removed: vec![false; 347],
            selected: vec![false; 347],
            room_routes: [
                -1, -1, 1, -1, 0, 2, -1, 5, -1, -1, 12, 0, 6, -1, 3, 4, 0, 0, 0, 0, 8, 0, 1,
            ],
            pill_effects,
            trinkets,
        })
    }

    ///581770 tests availability and current eligibility, then clears only
    ///availability and decrements count. It neither refills nor draws RNG.
    pub(crate) fn remove_trinket(&mut self, id: u32) -> Result<bool, String> {
        let pool = self
            .trinkets
            .as_mut()
            .ok_or("trinket pool availability is unknown")?;
        pool.validate()?;
        let index = usize::try_from(id).map_err(|_| "trinket ID outside base pool")?;
        if index >= 62 {
            return Err("trinket ID outside base pool".into());
        }
        let eligible = pool
            .eligible
            .as_ref()
            .ok_or("current-profile trinket eligibility is unknown")?;
        if !pool.available[index] || !eligible[index] {
            return Ok(false);
        }
        pool.available[index] = false;
        pool.count -= 1;
        Ok(true)
    }

    ///582180 uses the actual manager shift2 once, then a local shift35.
    ///Only the depleted-count branch reads the current profile; no MT is used.
    pub fn get_trinket(&mut self, current_achievements: Option<&[bool]>) -> Result<u32, String> {
        let pool = self
            .trinkets
            .as_ref()
            .ok_or("GetTrinket requires known pool availability")?;
        pool.validate()?;
        let manager = self
            .secondary_rng
            .ok_or("GetTrinket requires known ItemPool+49C RNG words")?;
        if [manager.right_a, manager.left, manager.right_b] != [1, 5, 19] {
            return Err("GetTrinket requires the actual secondary RNG shift2".into());
        }
        //Validate every reached readset before mutation. The actual reset
        //still precedes the first RNG consumer at582191/5822AE.
        let reset = if pool.count <= 0 {
            Some(TrinketPool::reset(
                current_achievements
                    .filter(|bytes| bytes.len() == 179)
                    .ok_or("GetTrinket reset requires the actual current 179 profile bytes")?,
            ))
        } else {
            if pool.eligible.is_none() {
                return Err("GetTrinket requires known current-profile eligibility".into());
            }
            None
        };
        let pool = self.trinkets.as_mut().expect("pool preflighted");
        if let Some(reset) = reset {
            *pool = reset;
        }
        let eligible = pool.eligible.as_ref().expect("eligibility preflighted");
        let manager = self.secondary_rng.as_mut().expect("manager preflighted");
        //5821C1 seeds the local RNG from the updated RAW manager word,
        //not its bounded index. Zero is a real state and is never clamped.
        let mut index = manager.bounded(62) as usize;
        let mut retry = Rng::new(manager.state, [5, 9, 7]);
        for _ in 0..31 {
            if pool.available[index] && eligible[index] {
                pool.available[index] = false;
                pool.count -= 1;
                return Ok(index as u32);
            }
            //5821E9 draws BEFORE decrementing attempts. The31st failed
            //test consumes one more local result that is never tested.
            index = retry.bounded(62) as usize;
        }
        //582209 draws again;582229 increments BEFORE testing each of62
        //sequential cells. Logging on this branch has no RNG consumer.
        index = retry.bounded(62) as usize;
        for _ in 0..62 {
            index = (index + 1) % 62;
            if pool.available[index] && eligible[index] {
                pool.available[index] = false;
                pool.count -= 1;
                return Ok(index as u32);
            }
        }
        //5822B8 can really return0 despite a retained positive count.
        //The caller owns50005D's separate card-conversion continuation.
        Ok(0)
    }

    ///An unrepresented subtype0 Init can reset/select/remove through582180.
    ///Neither its old availability nor the actual manager RNG stays known.
    pub(crate) fn note_unrepresented_trinket_selection(&mut self) {
        self.trinkets = None;
        self.secondary_rng = None;
    }

    pub(crate) fn copy_for_save(&self) -> Result<Self, String> {
        let mut saved = self.clone();
        if let Some(pool) = &mut saved.trinkets {
            pool.validate()?;
            if pool.eligible.is_none() {
                return Err("run save requires current-profile trinket eligibility".into());
            }
            pool.eligible = None;
        }
        Ok(saved)
    }

    pub(crate) fn validate_saved_trinkets(&self) -> Result<(), String> {
        if let Some(pool) = &self.trinkets {
            pool.validate()?;
            if pool.eligible.is_some() {
                return Err(
                    "saved trinket pool must not replace current-profile eligibility".into(),
                );
            }
        }
        Ok(())
    }

    ///54EE90 Init precedes54EEA2 Load. Restore the saved availability/count
    ///without serializing or overwriting the just-initialized profile bytes.
    pub(crate) fn restore_after_init(&self, initialized: &Self) -> Result<Self, String> {
        self.validate_saved_trinkets()?;
        let mut restored = self.clone();
        if let Some(pool) = &mut restored.trinkets {
            pool.eligible = Some(
                initialized
                    .trinkets
                    .as_ref()
                    .and_then(|fresh| fresh.eligible.clone())
                    .ok_or("Continue requires current-profile trinket eligibility")?,
            );
        }
        //A missing legacy pool remains unknown despite the fresh initializer.
        Ok(restored)
    }

    pub fn room_pool(&self, room_type: u32, shared: &mut crate::mt::Mt19937) -> i32 {
        if room_type == 3 {
            shared.bounded(16) as i32
        } else {
            self.room_routes
                .get(room_type as usize)
                .copied()
                .filter(|id| *id >= 0)
                .unwrap_or(0)
        }
    }

    fn pick(&mut self, pool_id: usize, mut sample: f32, decrease: bool) -> Option<usize> {
        let mut remaps = 10;
        loop {
            let pool = &self.pools[pool_id];
            let mut cumulative = 0.0_f32;
            let index = pool.entries.iter().position(|entry| {
                cumulative += entry.base_weight;
                cumulative > sample
            })?;
            let entry = &pool.entries[index];
            let upper_distance = cumulative - sample;
            let residual = entry.base_weight - upper_distance;
            let factor = if entry.special {
                self.special_factor
            } else {
                1.0
            };
            let id = entry.id as usize;
            let selected = if entry.current_weight * factor > residual
                && !self.removed[id]
                && !self.selected[id]
            {
                index
            } else if remaps > 0 {
                sample = pool.base_weight_total * upper_distance;
                remaps -= 1;
                continue;
            } else {
                // Source scan starts AFTER the rejected entry, wraps once,
                // excludes that entry and ignores the special probability.
                (1..pool.entries.len())
                    .map(|offset| (index + offset) % pool.entries.len())
                    .find(|candidate| {
                        let entry = &pool.entries[*candidate];
                        entry.current_weight > 0.0
                            && !self.removed[entry.id as usize]
                            && !self.selected[entry.id as usize]
                    })?
            };
            let entry = &mut self.pools[pool_id].entries[selected];
            if decrease && entry.current_weight > 0.0 {
                entry.current_weight -= entry.decrease_by;
                if entry.current_weight < entry.remove_on {
                    entry.current_weight = 0.0;
                    self.removed[entry.id as usize] = true;
                }
            }
            if entry.special {
                self.special_factor = if self.special_factor > 0.5 {
                    self.special_factor * 0.5
                } else {
                    (self.special_factor * 0.2).max(0.01)
                };
            }
            return Some(selected);
        }
    }

    /// Local shift4 owns selection, including eligibility redraws and the
    /// Bible check. Neither the shared MT nor the pool's RNG35 is advanced.
    pub fn collectible(&mut self, pool_id: i32, decrease: bool, seed: u32) -> Result<u32, String> {
        if seed == 0 {
            return Err("collectible selection seed must be nonzero".into());
        }
        if pool_id == -1 {
            return Ok(0);
        }
        let pool_id = usize::try_from(pool_id).map_err(|_| "invalid item pool")?;
        if pool_id >= self.pools.len() {
            return Err("invalid item pool".into());
        }
        let mut rng = Rng::new(seed, [1, 11, 6]);
        let mut candidate = self.pick(
            pool_id,
            rng.uniform() * self.pools[pool_id].base_weight_total,
            decrease,
        );
        if candidate.is_some_and(|index| !self.pools[pool_id].entries[index].eligible) {
            let mut remaining = 19_i32;
            loop {
                candidate = self.pick(
                    pool_id,
                    rng.uniform() * self.pools[pool_id].base_weight_total,
                    decrease,
                );
                if candidate.is_some_and(|index| !self.pools[pool_id].entries[index].eligible) {
                    remaining -= 1;
                    // Preserve the original branch: at -1 even the final
                    // ineligible result proceeds to the selected-ID mark.
                    if remaining == -1 {
                        break;
                    }
                } else {
                    if remaining == 0 {
                        candidate = None;
                    }
                    break;
                }
            }
        }
        let Some(index) = candidate else {
            return if pool_id == 0 {
                Ok(25)
            } else {
                self.collectible(0, decrease, seed)
            };
        };
        let id = self.pools[pool_id].entries[index].id;
        self.selected[id as usize] = true;
        if !self.removed[33]
            && !self.selected[33]
            && (rng.bounded(self.pools[pool_id].base_weight_total as u32) as i32)
                < self.pools[pool_id].special_counter
        {
            self.selected[33] = true;
            Ok(33)
        } else {
            Ok(id)
        }
    }
}
