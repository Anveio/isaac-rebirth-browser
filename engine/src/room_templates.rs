//! Native RoomConfig filtering and weighted selection with a supplied seed.
//!
//! Catalogues remain authored data; selected records retain their catalogue
//! indices so callers can resolve complete spawn groups without substituting
//! entity selectors. Level routing and special-room placement are separate.
use serde::{Deserialize, Serialize};

pub const SELECTION_CONTRACT: &str =
    "recovered-authored-filter-rng7-f32-weighted-selection-and-weight-decay;supplied-selector-seed";
const MAX_TEMPLATES: usize = 100_000;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthoredTemplate {
    #[serde(rename = "type")]
    pub room_type: u32,
    pub variant: u32,
    pub difficulty: u8,
    pub name: String,
    pub weight: f32,
    pub width: u8,
    pub height: u8,
    pub door_mask: u64,
}
impl AuthoredTemplate {
    pub fn shape_index(&self) -> u8 {
        match (self.width, self.height) {
            (13, 7) => 0,
            (13, 14) => 1,
            (26, 7) => 2,
            (26, 14) => 3,
            _ => 4, // Native loader retains invalid-size records and marks shape4.
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SelectionRequest {
    pub seed: u32,
    #[serde(rename = "type")]
    pub room_type: u32,
    /// Shape4 means any shape, matching the native query contract.
    pub shape: u8,
    pub required_doors: u64,
    pub min_variant: u32,
    pub max_variant: u32,
    pub min_difficulty: u8,
    pub max_difficulty: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Selection {
    pub contract: String,
    pub catalog_index: usize,
    #[serde(rename = "type")]
    pub room_type: u32,
    pub variant: u32,
    pub name: String,
    pub shape: u8,
    pub door_mask: u64,
    pub compatible_count: usize,
    pub exact_mask_count: usize,
    pub selected_pool_count: usize,
    pub used_exact_mask_pool: bool,
    pub random_draws: u8,
    pub selection_float: f32,
    pub exact_mask_gate: Option<f32>,
    pub selected_pool_weight: f32,
    pub selection_threshold: f32,
    pub remaining_weight: f32,
}

struct Rng7(u32);
impl Rng7 {
    fn random_float(&mut self) -> f32 {
        self.0 ^= self.0 >> 1;
        self.0 ^= self.0 << 21;
        self.0 ^= self.0 >> 20;
        // Native cvtsi2ss converts the unsigned word first, then mulss scales.
        self.0 as f32 * (1.0_f32 / 4_294_967_296.0_f32)
    }
}

pub struct TemplatePicker {
    catalog: Vec<AuthoredTemplate>,
    weights: Vec<f32>,
}
impl TemplatePicker {
    pub fn new(catalog: Vec<AuthoredTemplate>) -> Result<Self, String> {
        if catalog.is_empty() || catalog.len() > MAX_TEMPLATES {
            return Err("authored catalogue must contain 1..100000 records".into());
        }
        if catalog
            .iter()
            .any(|room| !room.weight.is_finite() || room.weight < 0.0 || room.door_mask > 255)
        {
            return Err("catalogue weights must be finite/nonnegative and masks eight-bit".into());
        }
        let weights = catalog.iter().map(|room| room.weight).collect();
        Ok(Self { catalog, weights })
    }

    pub fn catalog(&self) -> &[AuthoredTemplate] {
        &self.catalog
    }

    pub fn current_weights(&self) -> &[f32] {
        &self.weights
    }

    pub fn choose(&mut self, request: SelectionRequest) -> Result<Selection, String> {
        if request.seed == 0
            || request.shape > 4
            || request.required_doors > 255
            || request.min_variant > request.max_variant
            || request.min_difficulty > request.max_difficulty
        {
            return Err("invalid supplied template-selection contract".into());
        }
        let compatible: Vec<usize> = self
            .catalog
            .iter()
            .enumerate()
            .filter_map(|(index, room)| {
                (room.room_type == request.room_type
                    && (request.shape == 4 || room.shape_index() == request.shape)
                    && (request.min_variant..=request.max_variant).contains(&room.variant)
                    && (request.min_difficulty..=request.max_difficulty).contains(&room.difficulty)
                    && room.door_mask & request.required_doors == request.required_doors)
                    .then_some(index)
            })
            .collect();
        let exact: Vec<usize> = compatible
            .iter()
            .copied()
            .filter(|index| self.catalog[*index].door_mask == request.required_doors)
            .collect();
        let total = |indices: &[usize]| {
            indices
                .iter()
                .fold(0.0_f32, |sum, index| sum + self.weights[*index])
        };
        let compatible_weight = total(&compatible);
        let exact_weight = total(&exact);
        if !compatible_weight.is_finite() || compatible_weight <= 0.0 {
            return Err("no finite positive compatible template weight".into());
        }
        let mut rng = Rng7(request.seed);
        let selection = rng.random_float();
        let gate = (!exact.is_empty()).then(|| rng.random_float());
        // Native constant0x67afe0 is the f32 value9.989999771118164.
        let use_exact =
            gate.is_some_and(|value| value < (exact_weight / compatible_weight) * 9.99_f32);
        let (pool, pool_weight) = if use_exact {
            (&exact, exact_weight)
        } else {
            (&compatible, compatible_weight)
        };
        let threshold = selection * pool_weight;
        let mut cumulative = 0.0_f32;
        for index in pool {
            cumulative += self.weights[*index];
            if cumulative > threshold {
                // 5d5878 mutates current weight; the authored record is retained.
                self.weights[*index] *= 0.1_f32;
                let room = &self.catalog[*index];
                return Ok(Selection {
                    contract: SELECTION_CONTRACT.into(),
                    catalog_index: *index,
                    room_type: room.room_type,
                    variant: room.variant,
                    name: room.name.clone(),
                    shape: room.shape_index(),
                    door_mask: room.door_mask,
                    compatible_count: compatible.len(),
                    exact_mask_count: exact.len(),
                    selected_pool_count: pool.len(),
                    used_exact_mask_pool: use_exact,
                    random_draws: if gate.is_some() { 2 } else { 1 },
                    selection_float: selection,
                    exact_mask_gate: gate,
                    selected_pool_weight: pool_weight,
                    selection_threshold: threshold,
                    remaining_weight: self.weights[*index],
                });
            }
        }
        // Native Float can round to1; strict cumulative> then returns null.
        Err("native weighted selector found no candidate above its threshold".into())
    }
}
