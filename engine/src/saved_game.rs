//! The supported run-admission save path: Clear, Room.Save, Level copy and
//! PlayerManager copy. Missing grid-object constructor state remains explicit.
use crate::{floor, items, mt, Inventory, Player, Room, State};
use serde::{Deserialize, Serialize};

/// PlayerManager's second RNG is constructed at +40 by 5bf020/5c2cb0.
/// It is distinct from the +0 manager RNG initialized from the startup seed.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayerManagerSecondaryRng {
    pub seed: u32,
}

impl Default for PlayerManagerSecondaryRng {
    fn default() -> Self {
        Self { seed: 0xaa17_414f }
    }
}

/// Actual supported Level, current-room and Player values remain internal.
/// Render snapshots serialize only the small summary below.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedRun {
    pub run_seed: String,
    pub seeded_run: bool,
    pub stage: u32,
    pub game_update_count: u32,
    pub current_room: usize,
    pub floor: floor::Floor,
    pub room: Room,
    pub player: Player,
    pub inventory: Inventory,
    pub item_pools: items::ItemPools,
}

impl SavedRun {
    pub fn copy_from(state: &State, item_pools: items::ItemPools) -> Self {
        Self {
            run_seed: state.run_seed.clone(),
            seeded_run: state.seeded_run,
            stage: state.stage,
            game_update_count: state.game_update_count,
            current_room: state.current_room,
            floor: state.floor.clone(),
            room: state.room.clone(),
            player: state.player.clone(),
            inventory: state.inventory.clone(),
            item_pools,
        }
    }

    pub fn summary(
        &self,
        empty_cells: usize,
        occupied_pending: Vec<usize>,
        copied_rows: usize,
    ) -> SavedRunSummary {
        SavedRunSummary {
            run_seed: self.run_seed.clone(),
            seeded_run: self.seeded_run,
            stage: self.stage,
            game_update_count: self.game_update_count,
            descriptor_count: self.floor.descriptors.len(),
            copied_descriptor_count: copied_rows,
            red_hp: self.player.hp,
            max_red_hp: self.player.max_hp,
            inventory: self.inventory.clone(),
            empty_grid_cells_saved: empty_cells,
            occupied_grid_seeds_pending: occupied_pending,
            item_pools: SavedItemPoolsSummary::from_pools(&self.item_pools),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedRunSummary {
    pub run_seed: String,
    pub seeded_run: bool,
    pub stage: u32,
    pub game_update_count: u32,
    pub descriptor_count: usize,
    pub copied_descriptor_count: usize,
    pub red_hp: u8,
    pub max_red_hp: u8,
    pub inventory: Inventory,
    pub empty_grid_cells_saved: usize,
    /// Occupied cells also copy object+20, whose real 560ff0 constructor
    /// admission is not yet routed. No substitute secondary seed is created.
    pub occupied_grid_seeds_pending: Vec<usize>,
    pub item_pools: SavedItemPoolsSummary,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedWeightSample {
    pub pool_id: u32,
    pub item_id: u32,
    pub weight: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedRemovedSample {
    pub item_id: usize,
    pub removed: bool,
}

/// The actual pool copy remains internal. Only fields whose native save stores
/// are recovered at 581ed0 appear in this bounded diagnostic.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedItemPoolsSummary {
    pub pool_seed: u32,
    pub root_rng_seed: u32,
    pub secondary_rng_seed: u32,
    pub special_factor: f32,
    pub special_counters: Vec<i32>,
    pub removed_samples: Vec<SavedRemovedSample>,
    pub pill_effects: [i32; 10],
    pub weight_samples: Vec<SavedWeightSample>,
}

impl SavedItemPoolsSummary {
    fn from_pools(pools: &items::ItemPools) -> Self {
        let mut weight_samples = Vec::new();
        for id in [0, 15] {
            let pool = &pools.pools[id];
            for (position, index) in [0, pool.entries.len().saturating_sub(1)]
                .into_iter()
                .enumerate()
            {
                if position == 1 && index == 0 {
                    continue;
                }
                if let Some(entry) = pool.entries.get(index) {
                    weight_samples.push(SavedWeightSample {
                        pool_id: pool.id,
                        item_id: entry.id,
                        weight: entry.current_weight,
                    });
                }
            }
        }
        Self {
            pool_seed: pools.seed,
            root_rng_seed: pools.root_rng.words()[0],
            secondary_rng_seed: pools.secondary_rng.words()[0],
            special_factor: pools.special_factor,
            special_counters: pools
                .pools
                .iter()
                .map(|pool| pool.special_counter)
                .collect(),
            removed_samples: [0, 1, 346]
                .map(|item_id| SavedRemovedSample {
                    item_id,
                    removed: pools.removed[item_id],
                })
                .into(),
            pill_effects: pools.pill_effects,
            weight_samples,
        }
    }
}

/// Room.Save 5ce2d0 replaces empty cells in the active descriptor with two
/// real raw MT seeds. Occupied grid records require their actual object state.
pub fn save_empty_grid_cells(
    table: &mut mt::DescriptorSeeds,
    shared: &mut mt::Mt19937,
    room: &Room,
    descriptor: usize,
) -> Result<(usize, Vec<usize>), String> {
    let grid = room
        .grid
        .as_ref()
        .ok_or("saved room requires its live grid")?;
    let count = usize::from(grid.width) * usize::from(grid.height);
    if count != grid.cells.len() || count > mt::PADDED_GRID_CELLS {
        return Err("saved room grid dimensions exceed source capacity".into());
    }
    let mut empty = 0;
    let mut occupied = Vec::new();
    for (index, cell) in grid.cells.iter().enumerate() {
        if cell.entity.is_none() {
            table.replace(descriptor, index, [shared.next_u32(), shared.next_u32()])?;
            empty += 1;
        } else {
            occupied.push(index);
        }
    }
    Ok((empty, occupied))
}
