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
    pub player_items: crate::player_save::SavedPlayerItems,
    pub inventory: Inventory,
    pub item_pools: items::ItemPools,
    pub grid_records: Vec<Option<crate::grid_records::GridRecord>>,
}

impl SavedRun {
    pub fn copy_from(
        state: &State,
        item_pools: items::ItemPools,
        grid_records: Vec<Option<crate::grid_records::GridRecord>>,
        player_items: crate::player_save::SavedPlayerItems,
    ) -> Self {
        Self {
            run_seed: state.run_seed.clone(),
            seeded_run: state.seeded_run,
            stage: state.stage,
            game_update_count: state.game_update_count,
            current_room: state.current_room,
            floor: state.floor.clone(),
            room: state.room.clone(),
            player: state.player.clone(),
            player_items,
            inventory: state.inventory.clone(),
            item_pools,
            grid_records,
        }
    }

    pub fn summary(
        &self,
        empty_cells: usize,
        occupied_pending: Vec<usize>,
        copied_rows: usize,
        level_seeds: &mt::DescriptorSeeds,
        saved_seeds: &mt::DescriptorSeeds,
    ) -> Result<SavedRunSummary, String> {
        let grid = self
            .room
            .grid
            .as_ref()
            .ok_or("saved room requires its live grid")?;
        let mut current_room_seed_samples = Vec::new();
        for grid_index in [0, 1, 16, 17, 56, 85, 103, 134, 447] {
            current_room_seed_samples.push(CurrentRoomSeedSample {
                grid_index,
                level_seeds: level_seeds.seed_pair(self.current_room, grid_index)?,
                saved_seeds: saved_seeds.seed_pair(self.current_room, grid_index)?,
                occupancy: match grid.cells.get(grid_index) {
                    None => GridOccupancy::Padding,
                    Some(cell) if cell.entity.is_some() => GridOccupancy::Occupied,
                    Some(_) => GridOccupancy::Empty,
                },
                record: self.grid_records.get(grid_index).cloned().flatten(),
            });
        }
        Ok(SavedRunSummary {
            run_seed: self.run_seed.clone(),
            seeded_run: self.seeded_run,
            stage: self.stage,
            game_update_count: self.game_update_count,
            descriptor_count: self.floor.descriptors.len(),
            copied_descriptor_count: copied_rows,
            red_hp: self.player.hp,
            max_red_hp: self.player.max_hp,
            player_items: self.player_items.clone(),
            inventory: self.inventory.clone(),
            empty_grid_cells_saved: empty_cells,
            occupied_grid_seeds_pending: occupied_pending,
            item_pools: SavedItemPoolsSummary::from_pools(&self.item_pools),
            current_room_record: self.current_room,
            current_room_seed_samples,
            door_cell_records: [7, 60, 74, 127]
                .into_iter()
                .map(|grid_index| SampledGridRecord {
                    grid_index,
                    record: self.grid_records.get(grid_index).cloned().flatten(),
                })
                .collect(),
        })
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
    pub player_items: crate::player_save::SavedPlayerItems,
    pub inventory: Inventory,
    pub empty_grid_cells_saved: usize,
    /// Only actually constructed occupied owners supply secondary seeds.
    /// Unsupported objects remain pending without a substitute descriptor seed.
    pub occupied_grid_seeds_pending: Vec<usize>,
    pub item_pools: SavedItemPoolsSummary,
    pub current_room_record: usize,
    pub current_room_seed_samples: Vec<CurrentRoomSeedSample>,
    /// Four native starting-room boundary slots, including enabled Door16.
    pub door_cell_records: Vec<SampledGridRecord>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SampledGridRecord {
    pub grid_index: usize,
    pub record: Option<crate::grid_records::GridRecord>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GridOccupancy {
    Empty,
    Occupied,
    Padding,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CurrentRoomSeedSample {
    pub grid_index: usize,
    pub level_seeds: [u32; 2],
    pub saved_seeds: [u32; 2],
    pub occupancy: GridOccupancy,
    pub record: Option<crate::grid_records::GridRecord>,
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

pub struct SavedGridCells {
    pub empty: usize,
    pub occupied_pending: Vec<usize>,
    pub records: Vec<Option<crate::grid_records::GridRecord>>,
}

/// Room.Save 5ce2d0 stores two fresh raw words for null cells and copies actual
/// supported occupied records. Unadmitted object owners remain explicit gaps.
pub fn save_grid_cells(
    table: &mut mt::DescriptorSeeds,
    shared: &mut mt::Mt19937,
    room: &Room,
    descriptor: usize,
) -> Result<SavedGridCells, String> {
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
    let mut records = Vec::with_capacity(count);
    for (index, cell) in grid.cells.iter().enumerate() {
        let record = match &cell.entity {
            // An authored GridBase object without a runtime owner still
            // occupies its cell; Room.Save copies its record without draws.
            None if cell.selector != 0
                && crate::grid_records::constructs_grid_object(cell.selector as u16) =>
            {
                occupied.push(index);
                None
            }
            None => {
                empty += 1;
                Some(crate::grid_records::GridRecord::empty(shared))
            }
            Some(entity) => entity.record().or_else(|| {
                occupied.push(index);
                None
            }),
        };
        if let Some(record) = &record {
            table.replace(descriptor, index, record.seed_pair())?;
        }
        records.push(record);
    }
    Ok(SavedGridCells {
        empty,
        occupied_pending: occupied,
        records,
    })
}
