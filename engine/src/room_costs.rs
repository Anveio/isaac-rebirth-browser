//! Actual signed Room+e98 path costs, separate from authored grid selectors.
//! Only source-admitted producers mutate this fixed-capacity owner.
use crate::grid::{GridDescriptor, MAX_GRID_CELLS};
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomCosts {
    words: [i32; MAX_GRID_CELLS],
}

impl RoomCosts {
    /// Room.Init5cef43 zeroes all 0x700 bytes before grid admission.
    pub fn zeroed() -> Self {
        Self {
            words: [0; MAX_GRID_CELLS],
        }
    }

    /// Geometry admission owns these recovered Init stores independently of
    /// the still-unadmitted derived grid constructors and their RNG consumers.
    pub(crate) fn for_supported_grid(grid: &GridDescriptor) -> Result<Self, String> {
        let mut costs = Self::zeroed();
        for (index, cell) in grid.cells.iter().enumerate() {
            let Some(entity) = &cell.entity else {
                continue;
            };
            let cost = match entity.kind {
                // Rock.Init56ed00 writes1000 for state1 and zero otherwise.
                2 | 5 | 6 => i32::from(entity.state == 1) * 1000,
                // Ordinary Spike.Init56f830 always stores999.
                8 => 999,
                // Block and tinted rock share Rock.Init56ed00.
                3 | 4 => i32::from(entity.state == 1) * 1000,
                // Pit.Init568990: 3000, or 0 once bridged (state1).
                7 => i32::from(entity.state == 0) * 3000,
                // Wall5717e0 and Door5628e0 Init each store1000.
                15 | 16 => 1000,
                _ => continue,
            };
            costs.store(index, cost)?;
        }
        Ok(costs)
    }

    pub fn get(&self, index: usize) -> Result<i32, String> {
        self.words
            .get(index)
            .copied()
            .ok_or_else(|| format!("path-cost cell {index} exceeds source capacity"))
    }

    pub fn store(&mut self, index: usize, value: i32) -> Result<(), String> {
        *self
            .words
            .get_mut(index)
            .ok_or_else(|| format!("path-cost cell {index} exceeds source capacity"))? = value;
        Ok(())
    }

    pub fn entries(&self) -> &[i32; MAX_GRID_CELLS] {
        &self.words
    }

    pub fn entries_mut(&mut self) -> &mut [i32; MAX_GRID_CELLS] {
        &mut self.words
    }
}

impl Serialize for RoomCosts {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.words.as_slice().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RoomCosts {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let words = Vec::<i32>::deserialize(deserializer)?;
        let words = words.try_into().map_err(|words: Vec<i32>| {
            D::Error::custom(format!(
                "Room costs require exactly {MAX_GRID_CELLS} signed words; got {}",
                words.len()
            ))
        })?;
        Ok(Self { words })
    }
}
