//! Authored room-template selection (`room_templates::TemplatePicker`, the
//! 5d5878 weighted picker floors draw their rooms from) over the authored
//! Basement and special catalogues and synthetic ones, through long request
//! sequences whose picks decay the chosen template's weight.
//!
//! A model of the current weights and an independent compatibility filter
//! check every answer: the pick satisfies its request, the exact door-mask
//! pool is used only for exact masks, only the picked weight changes (to a
//! tenth), the reported counts and draws match, and the request is refused
//! exactly when it is malformed or nothing compatible carries weight. The
//! picker's RNG7 draws are replayed through `items::Rng`.
use crate::entropy::Entropy;
use crate::harness::Case;
use crate::props;
use rebirth_cleanroom::items::Rng;
use rebirth_cleanroom::room_assets;
use rebirth_cleanroom::room_templates::{AuthoredTemplate, SelectionRequest, TemplatePicker};
use serde_json::json;

const SHAPES: [(u8, u8); 5] = [(13, 7), (13, 14), (26, 7), (26, 14), (9, 5)];

pub fn run(case: &mut Case) {
    let e = &mut *case.entropy;
    let basement: Vec<AuthoredTemplate> = room_assets::basement()
        .into_iter()
        .map(|asset| asset.template)
        .collect();
    let special: Vec<AuthoredTemplate> = room_assets::special()
        .into_iter()
        .map(|asset| asset.template)
        .collect();
    for _ in 0..(case.steps / 300).max(4) {
        let catalog = match e.below(4) {
            0 => basement.clone(),
            1 => special.clone(),
            _ => synthetic(e),
        };
        let valid_catalog = !catalog.is_empty()
            && catalog.len() <= 100_000
            && catalog
                .iter()
                .all(|room| room.weight.is_finite() && room.weight >= 0.0 && room.door_mask <= 255);
        let built = TemplatePicker::new(catalog.clone());
        props::always!(
            built.is_ok() == valid_catalog,
            "room_templates: a picker is built exactly from valid catalogues",
            json!({ "size": catalog.len() })
        );
        let (Ok(mut picker), Ok(mut twin)) = (built, TemplatePicker::new(catalog.clone())) else {
            props::sometimes!(true, "room_templates: an invalid catalogue is rejected");
            continue;
        };
        let mut weights: Vec<f32> = catalog.iter().map(|room| room.weight).collect();
        for _ in 0..e.range(16, 200) {
            let request = request(e, &catalog);
            let result = picker.choose(request);
            props::always!(
                twin.choose(request) == result,
                "room_templates: two pickers fed the same requests answer alike",
                json!({ "request": request })
            );
            check(&catalog, &mut weights, request, &result);
            props::always!(
                picker.current_weights() == weights.as_slice()
                    && picker.catalog() == catalog.as_slice(),
                "room_templates: only the picked template's weight changes, to a tenth",
                json!({ "request": request })
            );
        }
    }
}

fn synthetic(e: &mut Entropy) -> Vec<AuthoredTemplate> {
    (0..e.range(0, 60))
        .map(|variant| {
            let (width, height) = *e.pick(&SHAPES);
            AuthoredTemplate {
                room_type: e.range(1, 4),
                variant,
                difficulty: *e.pick(&[0, 1, 2, 5, 10, 15]),
                name: String::new(),
                weight: match e.below(24) {
                    0 => f32::NAN,
                    1 => -1.0,
                    2..=5 => 0.0,
                    6 => 1e-30,
                    7 => 3e38,
                    _ => e.u32() as f32 / u32::MAX as f32 * 4.0,
                },
                width,
                height,
                door_mask: if e.chance(1, 64) {
                    256
                } else {
                    u64::from(e.u32() & 0xff)
                },
            }
        })
        .collect()
}

/// Mostly a request some template satisfies, anchored on a random entry,
/// sometimes a malformed one.
fn request(e: &mut Entropy, catalog: &[AuthoredTemplate]) -> SelectionRequest {
    let anchor = e.pick(catalog);
    let mut request = SelectionRequest {
        seed: e.nonzero_u32(),
        room_type: anchor.room_type,
        shape: if e.chance(1, 3) {
            4
        } else {
            anchor.shape_index()
        },
        required_doors: anchor.door_mask & u64::from(e.u32()),
        min_variant: anchor.variant.saturating_sub(e.range(0, 3)),
        max_variant: anchor.variant + e.range(0, 50),
        min_difficulty: anchor.difficulty.saturating_sub(e.range(0, 2) as u8),
        max_difficulty: anchor.difficulty.saturating_add(e.range(0, 5) as u8),
    };
    if e.chance(1, 4) {
        request.required_doors = anchor.door_mask;
    }
    match e.below(24) {
        0 => request.seed = 0,
        1 => request.shape = e.range(5, 9) as u8,
        2 => request.required_doors = 256 + e.below(4),
        3 => (request.min_variant, request.max_variant) = (5, 4),
        4 => (request.min_difficulty, request.max_difficulty) = (5, 4),
        5 => request.room_type = e.range(0, 30),
        _ => {}
    }
    request
}

/// RNG7 (`items::Rng` shifts 1/21/20), each output scaled by 2^-32 in f32.
fn draws(seed: u32) -> (f32, f32) {
    let mut rng = Rng::new(seed, [1, 21, 20]);
    let scale = |value: u32| value as f32 * (1.0_f32 / 4_294_967_296.0_f32);
    let first = scale(rng.raw());
    (first, scale(rng.raw()))
}

fn check(
    catalog: &[AuthoredTemplate],
    weights: &mut [f32],
    request: SelectionRequest,
    result: &Result<rebirth_cleanroom::room_templates::Selection, String>,
) {
    let details = || json!({ "request": request, "result": result });
    let malformed = request.seed == 0
        || request.shape > 4
        || request.required_doors > 255
        || request.min_variant > request.max_variant
        || request.min_difficulty > request.max_difficulty;
    let fits = |room: &AuthoredTemplate| {
        room.room_type == request.room_type
            && (request.shape == 4 || room.shape_index() == request.shape)
            && (request.min_variant..=request.max_variant).contains(&room.variant)
            && (request.min_difficulty..=request.max_difficulty).contains(&room.difficulty)
            && room.door_mask & request.required_doors == request.required_doors
    };
    let compatible: Vec<usize> = (0..catalog.len())
        .filter(|&at| fits(&catalog[at]))
        .collect();
    let exact: Vec<usize> = compatible
        .iter()
        .copied()
        .filter(|&at| catalog[at].door_mask == request.required_doors)
        .collect();
    let weight = |pool: &[usize]| pool.iter().fold(0.0_f32, |sum, &at| sum + weights[at]);
    let compatible_weight = weight(&compatible);
    let weighted = compatible_weight.is_finite() && compatible_weight > 0.0;
    let (selection, gate) = draws(request.seed);
    // 0x67afe0 holds the f32 9.99 the exact-mask gate scales by.
    let use_exact =
        weighted && !exact.is_empty() && gate < (weight(&exact) / compatible_weight) * 9.99_f32;
    let pool = if use_exact { &exact } else { &compatible };
    let threshold = selection * weight(pool);
    let mut cumulative = 0.0_f32;
    let pick = pool.iter().copied().find(|&at| {
        cumulative += weights[at];
        cumulative > threshold
    });
    let Ok(chosen) = result else {
        // Beyond malformed requests and weightless pools, a threshold that
        // rounds up to the pool's whole weight (a draw of one, or a pool of
        // subnormal decayed weights) finds no strictly larger running sum.
        props::always!(
            malformed || !weighted || pick.is_none(),
            "room_templates: a request is refused exactly when malformed, weightless or past its pool",
            details()
        );
        props::sometimes!(
            !malformed && !weighted,
            "room_templates: a request no compatible template weighs is refused"
        );
        return;
    };
    props::always!(
        !malformed && weighted && pick.is_some(),
        "room_templates: a request is refused exactly when malformed, weightless or past its pool",
        details()
    );
    let index = chosen.catalog_index;
    let room = &catalog[index];
    props::always!(
        compatible.contains(&index)
            && chosen.room_type == room.room_type
            && chosen.variant == room.variant
            && chosen.door_mask == room.door_mask
            && chosen.shape == room.shape_index(),
        "room_templates: the pick satisfies its request",
        details()
    );
    props::always!(
        chosen.compatible_count == compatible.len()
            && chosen.exact_mask_count == exact.len()
            && chosen.random_draws == if exact.is_empty() { 1 } else { 2 }
            && chosen.selection_float == selection
            && chosen.exact_mask_gate == (!exact.is_empty()).then_some(gate),
        "room_templates: the reported pools and RNG7 draws match an independent filter",
        details()
    );
    props::always!(
        chosen.used_exact_mask_pool == use_exact
            && (!use_exact || room.door_mask == request.required_doors)
            && pick == Some(index),
        "room_templates: the pick is 5d5878's running-sum choice from the gated pool",
        details()
    );
    props::sometimes!(
        use_exact,
        "room_templates: the exact door-mask pool is used"
    );
    props::sometimes!(
        !use_exact && !exact.is_empty(),
        "room_templates: the compatible pool is used though exact masks exist"
    );
    props::sometimes!(
        weights[index] < room.weight,
        "room_templates: a template already picked is picked again"
    );
    weights[index] *= 0.1_f32;
    props::always!(
        chosen.remaining_weight == weights[index],
        "room_templates: the picked template keeps a tenth of its weight",
        details()
    );
}
