//! The shared native MT19937 stream, independent of the run's xorshift seed.
//!
//! Application initialization supplies the startup epoch explicitly. Real
//! lifecycle, audio and entity consumers must all use this same state in order.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const STATE_WORDS: usize = 624;
pub const ROOM_DESCRIPTOR_CAPACITY: usize = 135;
pub const PADDED_GRID_CELLS: usize = 448;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields, try_from = "State")]
pub struct Mt19937 {
    words: Vec<u32>,
    index: u16,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct State {
    words: Vec<u32>,
    index: u16,
}

impl TryFrom<State> for Mt19937 {
    type Error = String;
    fn try_from(state: State) -> Result<Self, Self::Error> {
        if state.words.len() != STATE_WORDS || usize::from(state.index) > STATE_WORDS {
            return Err("MT state requires 624 words and index 0..624".into());
        }
        Ok(Self {
            words: state.words,
            index: state.index,
        })
    }
}

impl Mt19937 {
    pub fn new(boot_epoch_seed: u32) -> Self {
        let mut words = vec![0; STATE_WORDS];
        words[0] = boot_epoch_seed;
        for i in 1..STATE_WORDS {
            words[i] = 1_812_433_253_u32
                .wrapping_mul(words[i - 1] ^ (words[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        Self {
            words,
            index: STATE_WORDS as u16,
        }
    }

    pub fn words(&self) -> &[u32] {
        &self.words
    }
    pub fn index(&self) -> usize {
        usize::from(self.index)
    }

    pub fn next_u32(&mut self) -> u32 {
        if self.index() == STATE_WORDS {
            for i in 0..STATE_WORDS {
                let y = (self.words[i] & 0x8000_0000)
                    | (self.words[(i + 1) % STATE_WORDS] & 0x7fff_ffff);
                self.words[i] = self.words[(i + 397) % STATE_WORDS]
                    ^ (y >> 1)
                    ^ ((y & 1).wrapping_mul(0x9908_b0df));
            }
            self.index = 0;
        }
        let mut y = self.words[self.index()];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^= y >> 18;
        y
    }

    /// The source rounds to f32 before scaling; the largest uints can yield1.
    pub fn uniform(&mut self) -> f32 {
        uint_to_uniform(self.next_u32())
    }

    /// Source Random(0) returns zero without consuming the shared stream.
    pub fn bounded(&mut self, bound: u32) -> u32 {
        if bound == 0 {
            0
        } else {
            self.next_u32() % bound
        }
    }
}

pub fn uint_to_uniform(value: u32) -> f32 {
    (value as f32) * (1.0 / 4_294_967_296.0)
}

/// Untouched descriptor grid seeds are generated from their original shared
/// MT state. Only changed cells need storage, rather than copying 120960 words
/// into every render snapshot. The initialization still executes source order.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(
    rename_all = "camelCase",
    deny_unknown_fields,
    try_from = "DescriptorState"
)]
pub struct DescriptorSeeds {
    origin: Mt19937,
    mutations: BTreeMap<u16, [u32; 2]>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DescriptorState {
    origin: Mt19937,
    mutations: BTreeMap<u16, [u32; 2]>,
}

impl TryFrom<DescriptorState> for DescriptorSeeds {
    type Error = String;
    fn try_from(state: DescriptorState) -> Result<Self, Self::Error> {
        if state
            .mutations
            .keys()
            .any(|key| usize::from(*key) >= ROOM_DESCRIPTOR_CAPACITY * PADDED_GRID_CELLS)
        {
            return Err("descriptor seed mutation is outside the source grid".into());
        }
        Ok(Self {
            origin: state.origin,
            mutations: state.mutations,
        })
    }
}

impl DescriptorSeeds {
    /// Source Level/GameState initialization: room-major, grid-major, two
    /// independently assigned raw seeds for each padded grid description.
    pub fn initialize(shared: &mut Mt19937) -> Self {
        let table = Self {
            origin: shared.clone(),
            mutations: BTreeMap::new(),
        };
        for _room in 0..ROOM_DESCRIPTOR_CAPACITY {
            for _cell in 0..PADDED_GRID_CELLS {
                let _seed_pair = [shared.next_u32(), shared.next_u32()];
            }
        }
        table
    }

    fn key(room: usize, cell: usize) -> Result<u16, String> {
        if room >= ROOM_DESCRIPTOR_CAPACITY || cell >= PADDED_GRID_CELLS {
            return Err("descriptor seed query is outside the source grid".into());
        }
        Ok((room * PADDED_GRID_CELLS + cell) as u16)
    }

    pub fn seed_pair(&self, room: usize, cell: usize) -> Result<[u32; 2], String> {
        let key = Self::key(room, cell)?;
        if let Some(pair) = self.mutations.get(&key) {
            return Ok(*pair);
        }
        let mut derived = self.origin.clone();
        for _ in 0..usize::from(key) {
            let _preceding_cell_seeds = [derived.next_u32(), derived.next_u32()];
        }
        Ok([derived.next_u32(), derived.next_u32()])
    }

    /// Used by actual cell lifecycle mutations; callers supply freshly routed
    /// seeds rather than changing the initialization origin or global cursor.
    pub fn replace(&mut self, room: usize, cell: usize, pair: [u32; 2]) -> Result<(), String> {
        let key = Self::key(room, cell)?;
        self.mutations.insert(key, pair);
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedGameSeeds {
    pub descriptor_seeds: DescriptorSeeds,
    pub player_rng_seed: u32,
}

impl SavedGameSeeds {
    pub fn clear(shared: &mut Mt19937) -> Self {
        let descriptor_seeds = DescriptorSeeds::initialize(shared);
        let player_rng_seed = shared.next_u32();
        Self {
            descriptor_seeds,
            player_rng_seed,
        }
    }

    /// The source constructor assigns the grid seeds once, then calls Clear
    /// and replaces them. Both sets of draws belong to the shared stream.
    pub fn construct(shared: &mut Mt19937) -> Self {
        let _initial_descriptor_seeds = DescriptorSeeds::initialize(shared);
        Self::clear(shared)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_standard_vector_and_zero_bound() {
        let mut rng = Mt19937::new(5489);
        let before = rng.clone();
        assert_eq!(rng.bounded(0), 0);
        assert_eq!(rng, before);
        assert_eq!(
            (0..5).map(|_| rng.next_u32()).collect::<Vec<_>>(),
            vec![3499211612, 581869302, 3890346734, 3586334585, 545404204]
        );
        assert_eq!(uint_to_uniform(u32::MAX), 1.0);
    }

    #[test]
    fn imported_state_is_validated_and_continues_through_twists() {
        let mut original = Mt19937::new(1700000000);
        for _ in 0..623 {
            original.next_u32();
        }
        let mut restored: Mt19937 =
            serde_json::from_str(&serde_json::to_string(&original).unwrap()).unwrap();
        for _ in 0..1000 {
            assert_eq!(restored.next_u32(), original.next_u32());
        }
        assert_eq!(restored, original);
        let mut malformed = serde_json::to_value(&original).unwrap();
        malformed["index"] = 625.into();
        assert!(serde_json::from_value::<Mt19937>(malformed.clone()).is_err());
        malformed["index"] = 0.into();
        malformed["words"] = serde_json::json!([1, 2]);
        assert!(serde_json::from_value::<Mt19937>(malformed).is_err());
    }

    #[test]
    fn native_lifecycle_seed_pairs_and_complete_mt_states_match() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../references/isaac/foundation/physics/native-mt.json"
        ))
        .unwrap();
        let mut shared =
            Mt19937::new(fixture["original"]["bootEpochSeed"].as_u64().unwrap() as u32);
        let mut saved = SavedGameSeeds::construct(&mut shared);
        let _level_grid = DescriptorSeeds::initialize(&mut shared);
        let _backdrop_seed = shared.next_u32();
        for (i, checkpoint) in fixture["checkpoints"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            if i == 1 {
                // Save-menu previews load slots1,2,3 and restore the prior
                // slot. Character reset follows, then179 authored stat poses.
                for _profile in 0..4 {
                    saved = SavedGameSeeds::clear(&mut shared);
                }
                let _initial_run_seed = shared.next_u32();
                for _stat_pose in 0..179 {
                    shared.bounded(3);
                }
            }
            let observed: Mt19937 = serde_json::from_value(checkpoint["mt"].clone()).unwrap();
            assert_eq!(
                shared, observed,
                "full native MT state at {}",
                checkpoint["label"]
            );
            assert_eq!(
                saved.player_rng_seed,
                checkpoint["playerRngSeed"].as_u64().unwrap() as u32
            );
            for cell in checkpoint["gridSeeds"].as_array().unwrap() {
                assert_eq!(
                    saved
                        .descriptor_seeds
                        .seed_pair(
                            cell["roomRecord"].as_u64().unwrap() as usize,
                            cell["gridIndex"].as_u64().unwrap() as usize
                        )
                        .unwrap(),
                    [
                        cell["seed10"].as_u64().unwrap() as u32,
                        cell["seed18"].as_u64().unwrap() as u32
                    ]
                );
            }
        }
    }
}
