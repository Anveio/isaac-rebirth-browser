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
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemPools {
    pub seed: u32,
    pub root_rng: Rng,
    pub secondary_rng: Rng,
    pub pools: Vec<Pool>,
    pub special_factor: f32,
    pub removed: Vec<bool>,
    pub selected: Vec<bool>,
    pub room_routes: [i32; 23],
    pub pill_effects: [i32; 10],
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
            secondary_rng,
            pools,
            special_factor: 1.0,
            removed: vec![false; 347],
            selected: vec![false; 347],
            room_routes: [
                -1, -1, 1, -1, 0, 2, -1, 5, -1, -1, 12, 0, 6, -1, 3, 4, 0, 0, 0, 0, 8, 0, 1,
            ],
            pill_effects,
        })
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
