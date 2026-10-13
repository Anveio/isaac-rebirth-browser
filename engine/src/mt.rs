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

    /// Level.Save 58cd90 copies every row except its special -3 sentinel.
    /// Ordinary unused -1 rows are copied too. Full table copies preserve the
    /// real origin/mutations compactly after the destination's actual Clear.
    pub fn copy_rooms_from(&mut self, source: &Self, rooms: &[usize]) -> Result<(), String> {
        let mut copied = [false; ROOM_DESCRIPTOR_CAPACITY];
        for &room in rooms {
            if room >= ROOM_DESCRIPTOR_CAPACITY {
                return Err("saved descriptor copy is outside the source table".into());
            }
            copied[room] = true;
        }
        if copied.into_iter().all(|copy| copy) {
            *self = source.clone();
            return Ok(());
        }
        let mut derived = source.origin.clone();
        for (room, copy) in copied.into_iter().enumerate() {
            for cell in 0..PADDED_GRID_CELLS {
                let initial = [derived.next_u32(), derived.next_u32()];
                if copy {
                    let key = Self::key(room, cell)?;
                    self.mutations
                        .insert(key, source.mutations.get(&key).copied().unwrap_or(initial));
                }
            }
        }
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

/// Source startup state at the first rendered Title. Every draw belongs to a
/// generated object or menu field; run/room/ambient routing is still separate.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BootLifecycle {
    pub shared: Mt19937,
    pub saved_game: SavedGameSeeds,
    pub level_grid: DescriptorSeeds,
    pub backdrop_seed: u32,
    pub menu_run_seed: u32,
    pub secret_poses: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GridSeedSample {
    pub room_record: usize,
    pub grid_index: usize,
    pub seeds: [u32; 2],
}

/// Ordinary snapshots carry the mutable shared stream and bounded diagnostics.
/// Historical descriptor origins/mutations remain in Game's internal state.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BootState {
    pub shared: Mt19937,
    pub saved_player_seed: u32,
    pub saved_grid_seed_samples: Vec<GridSeedSample>,
    pub level_grid_seed_samples: Vec<GridSeedSample>,
    pub backdrop_seed: u32,
    pub menu_run_seed: u32,
    pub menu_seed_text: String,
    pub secret_poses: Vec<u8>,
}

impl BootState {
    /// Level.Init 58be30 calls 587ec0 again at actual level admission. Replace
    /// its real descriptor seed table; snapshots carry only bounded samples.
    pub fn initialize_level_grid(&mut self) -> DescriptorSeeds {
        let table = DescriptorSeeds::initialize(&mut self.shared);
        self.level_grid_seed_samples = grid_seed_samples(&table);
        table
    }

    /// LoadProfile first clears the saved state even for an empty profile. Keep
    /// the generated descriptor seed state, rather than advancing a cursor.
    pub fn clear_saved_game(&mut self) -> SavedGameSeeds {
        let saved = SavedGameSeeds::clear(&mut self.shared);
        self.update_saved_game_samples(&saved);
        saved
    }

    pub fn update_saved_game_samples(&mut self, saved: &SavedGameSeeds) {
        self.saved_player_seed = saved.player_rng_seed;
        self.saved_grid_seed_samples = grid_seed_samples(&saved.descriptor_seeds);
    }

    pub fn update_level_grid_samples(&mut self, table: &DescriptorSeeds) {
        self.level_grid_seed_samples = grid_seed_samples(table);
    }

    /// CharacterReset clears its seed string before decoding it; the empty
    /// string selects one fresh shared raw seed, replacing reserved zero by 1.
    pub fn reset_character_seed(&mut self) {
        self.menu_run_seed = self.shared.next_u32().max(1);
        self.menu_seed_text = crate::seed::encode(self.menu_run_seed)
            .expect("Character initializer produces a nonzero seed");
    }

    /// Valid dialog confirmation copies the public code and decodes it into
    /// the same menu seed field. It consumes no shared sample.
    pub fn commit_character_seed(&mut self, seed: u32) -> Result<(), String> {
        self.menu_seed_text = crate::seed::encode(seed)?;
        self.menu_run_seed = seed;
        Ok(())
    }
}

fn grid_seed_samples(table: &DescriptorSeeds) -> Vec<GridSeedSample> {
    [0, 1, ROOM_DESCRIPTOR_CAPACITY - 1]
        .into_iter()
        .flat_map(|room_record| {
            [0, 1, PADDED_GRID_CELLS - 1]
                .into_iter()
                .map(move |grid_index| GridSeedSample {
                    room_record,
                    grid_index,
                    seeds: table
                        .seed_pair(room_record, grid_index)
                        .expect("source bootstrap descriptor dimensions"),
                })
        })
        .collect()
}

impl BootLifecycle {
    pub fn new(boot_epoch_seed: u32) -> Self {
        let mut shared = Mt19937::new(boot_epoch_seed);
        let _startup_saved_game = SavedGameSeeds::construct(&mut shared);
        let level_grid = DescriptorSeeds::initialize(&mut shared);
        let backdrop_seed = shared.next_u32();
        // SaveMenu reset loads each of the three profiles to compose previews,
        // then restores the previously active profile. Each load clears the
        // saved GameState and assigns a fresh player-manager seed.
        for _profile in 1..=3 {
            let _preview_saved_game = SavedGameSeeds::clear(&mut shared);
        }
        let saved_game = SavedGameSeeds::clear(&mut shared);
        // CharacterMenu reset initializes an ordinary nonzero run seed; the
        // public supplied run seed remains independent of this shared stream.
        let menu_run_seed = shared.next_u32().max(1);
        // StatsMenu reset owns one authored pose for each of its 179 secrets.
        let secret_poses = (0..179).map(|_| (shared.bounded(3) + 1) as u8).collect();
        Self {
            shared,
            saved_game,
            level_grid,
            backdrop_seed,
            menu_run_seed,
            secret_poses,
        }
    }

    pub fn into_parts(self) -> (BootState, SavedGameSeeds, DescriptorSeeds) {
        let saved_grid_seed_samples = grid_seed_samples(&self.saved_game.descriptor_seeds);
        let level_grid_seed_samples = grid_seed_samples(&self.level_grid);
        let state = BootState {
            shared: self.shared,
            saved_player_seed: self.saved_game.player_rng_seed,
            saved_grid_seed_samples,
            level_grid_seed_samples,
            backdrop_seed: self.backdrop_seed,
            menu_run_seed: self.menu_run_seed,
            menu_seed_text: crate::seed::encode(self.menu_run_seed)
                .expect("boot Character initializer produces a nonzero seed"),
            secret_poses: self.secret_poses,
        };
        (state, self.saved_game, self.level_grid)
    }
}
