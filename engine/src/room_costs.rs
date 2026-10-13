//! Actual signed Room+e98 path costs, separate from authored grid selectors.
//! Only source-admitted producers mutate this fixed-capacity owner.
use crate::grid::MAX_GRID_CELLS;
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
