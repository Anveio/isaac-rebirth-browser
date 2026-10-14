//! The shared MT19937 stream, descriptor seed tables and boot lifecycle
//! (`engine/src/mt.rs`).
use crate::harness::Case;
use crate::props;
use rebirth_cleanroom::mt::{
    BootLifecycle, DescriptorSeeds, Mt19937, PADDED_GRID_CELLS, ROOM_DESCRIPTOR_CAPACITY,
    STATE_WORDS,
};
use rebirth_cleanroom::seed;
use serde_json::json;

const TABLE_PAIRS: usize = ROOM_DESCRIPTOR_CAPACITY * PADDED_GRID_CELLS;

pub fn run(case: &mut Case) {
    reference_outputs();
    let e = &mut *case.entropy;

    // Serialized state is a complete continuation of the stream.
    let mut stream = Mt19937::new(e.u32());
    for _ in 0..e.below(2_000) {
        stream.next_u32();
    }
    let saved = serde_json::to_string(&stream).expect("MT serializes");
    let restored: Result<Mt19937, _> = serde_json::from_str(&saved);
    let ahead = e.below(1_500) as usize;
    let original: Vec<u32> = (0..ahead).map(|_| stream.next_u32()).collect();
    let continued: Option<Vec<u32>> = restored
        .ok()
        .map(|mut restored| (0..ahead).map(|_| restored.next_u32()).collect());
    props::always!(
        continued.as_ref() == Some(&original),
        "mt: serialized state continues the identical stream",
        json!({ "ahead": ahead })
    );

    // Deserialization rejects states the generator cannot represent.
    let mut words: Vec<u32> = (0..STATE_WORDS).map(|_| e.u32()).collect();
    let bad_index = STATE_WORDS as u32 + 1 + e.range(0, 100);
    let invalid_index = json!({ "words": words, "index": bad_index });
    words.truncate(e.index(STATE_WORDS));
    let invalid_length = json!({ "words": words, "index": 0 });
    props::always!(
        serde_json::from_value::<Mt19937>(invalid_index).is_err()
            && serde_json::from_value::<Mt19937>(invalid_length).is_err(),
        "mt: deserialization rejects invalid word counts and cursors"
    );

    for _ in 0..(case.steps / 4).max(64) {
        let before = stream.clone();
        props::always!(
            stream.bounded(0) == 0 && stream == before,
            "mt: Random(0) returns zero without consuming the stream"
        );
        let bound = e.nonzero_u32();
        let sample = stream.bounded(bound);
        props::always!(
            sample < bound,
            "mt: bounded draws lie below their bound",
            json!({ "bound": bound, "sample": sample })
        );
        let uniform = stream.uniform();
        props::always!(
            (0.0..=1.0).contains(&uniform),
            "mt: uniform draws lie in the closed unit interval",
            json!({ "uniform": uniform })
        );
    }

    descriptor_seeds_match_eager_model(e);
    boot_lifecycle(e);
}

/// C++ [rand.predef] fixes default-seeded mt19937's first and 10000th outputs;
/// these are independent of this port.
fn reference_outputs() {
    let mut stream = Mt19937::new(5489);
    let first = stream.next_u32();
    let mut tenth_thousand = first;
    for _ in 1..10_000 {
        tenth_thousand = stream.next_u32();
    }
    props::always!(
        first == 3_499_211_612 && tenth_thousand == 4_123_659_995,
        "mt: default seed reproduces the C++ standard mt19937 outputs",
        json!({ "first": first, "tenThousandth": tenth_thousand })
    );
}

/// The compact lazy table must equal an eager table of every source draw.
fn descriptor_seeds_match_eager_model(e: &mut crate::entropy::Entropy) {
    let mut shared = Mt19937::new(e.u32());
    for _ in 0..e.below(64) {
        shared.next_u32();
    }
    let mut eager_stream = shared.clone();
    let mut model: Vec<[u32; 2]> = (0..TABLE_PAIRS)
        .map(|_| [eager_stream.next_u32(), eager_stream.next_u32()])
        .collect();
    let mut table = DescriptorSeeds::initialize(&mut shared);
    props::always!(
        shared == eager_stream,
        "mt: descriptor initialization consumes exactly two draws per padded cell"
    );
    let other_origin = Mt19937::new(e.u32());
    let other = DescriptorSeeds::initialize(&mut other_origin.clone());
    let mut other_model: Option<Vec<[u32; 2]>> = None;
    for _ in 0..24 {
        let room = e.index(ROOM_DESCRIPTOR_CAPACITY);
        let cell = e.index(PADDED_GRID_CELLS);
        let key = room * PADDED_GRID_CELLS + cell;
        match e.below(4) {
            0 => {
                let pair = [e.u32(), e.u32()];
                let replaced = table.replace(room, cell, pair);
                props::always!(
                    replaced.is_ok(),
                    "mt: in-range descriptor replacement succeeds"
                );
                model[key] = pair;
            }
            1 => {
                // Copy a random room subset from an independent table.
                let model_other = other_model.get_or_insert_with(|| {
                    let mut stream = other_origin.clone();
                    (0..TABLE_PAIRS)
                        .map(|_| [stream.next_u32(), stream.next_u32()])
                        .collect()
                });
                let rooms: Vec<usize> = if e.chance(1, 4) {
                    (0..ROOM_DESCRIPTOR_CAPACITY).collect()
                } else {
                    (0..e.range(1, 4))
                        .map(|_| e.index(ROOM_DESCRIPTOR_CAPACITY))
                        .collect()
                };
                let copied = table.copy_rooms_from(&other, &rooms);
                props::always!(
                    copied.is_ok(),
                    "mt: in-range descriptor room copies succeed"
                );
                for &copied_room in &rooms {
                    let start = copied_room * PADDED_GRID_CELLS;
                    model[start..start + PADDED_GRID_CELLS]
                        .copy_from_slice(&model_other[start..start + PADDED_GRID_CELLS]);
                }
                props::sometimes!(
                    rooms.len() == ROOM_DESCRIPTOR_CAPACITY,
                    "mt: a complete descriptor table copy is exercised"
                );
            }
            _ => {}
        }
        let observed = table.seed_pair(room, cell);
        props::always!(
            observed == Ok(model[key]),
            "mt: lazy descriptor seed pairs equal the eager source table",
            json!({ "room": room, "cell": cell, "observed": observed, "model": model[key] })
        );
    }
    let outside_room = ROOM_DESCRIPTOR_CAPACITY + e.index(8);
    let outside_cell = PADDED_GRID_CELLS + e.index(8);
    props::always!(
        table.seed_pair(outside_room, 0).is_err()
            && table.seed_pair(0, outside_cell).is_err()
            && table.replace(outside_room, 0, [1, 2]).is_err()
            && table.copy_rooms_from(&other, &[outside_room]).is_err(),
        "mt: descriptor queries outside the source grid are rejected"
    );
    let saved = serde_json::to_value(&table).expect("descriptor table serializes");
    let restored: Result<DescriptorSeeds, _> = serde_json::from_value(saved.clone());
    props::always!(
        restored.as_ref().ok() == Some(&table),
        "mt: descriptor tables round-trip through JSON"
    );
    let mut forged = saved;
    forged["mutations"][TABLE_PAIRS.to_string()] = json!([1, 2]);
    props::always!(
        serde_json::from_value::<DescriptorSeeds>(forged).is_err(),
        "mt: descriptor tables reject mutations outside the source grid"
    );
}

fn boot_lifecycle(e: &mut crate::entropy::Entropy) {
    let epoch = if e.chance(1, 4) { 0 } else { e.u32() };
    let first = BootLifecycle::new(epoch);
    props::always!(
        first == BootLifecycle::new(epoch),
        "mt: boot lifecycle is a pure function of the epoch seed",
        json!({ "epoch": epoch })
    );
    let poses_valid = first.secret_poses.len() == 179
        && first.secret_poses.iter().all(|pose| (1..=3).contains(pose));
    props::always!(
        poses_valid && first.menu_run_seed != 0,
        "mt: boot owns 179 authored secret poses and a nonzero menu seed",
        json!({ "epoch": epoch, "menuRunSeed": first.menu_run_seed })
    );
    let (mut state, _, _) = first.into_parts();
    props::always!(
        seed::decode(&state.menu_seed_text) == Ok(state.menu_run_seed),
        "mt: the menu seed text decodes to the menu seed",
        json!({ "epoch": epoch, "text": state.menu_seed_text })
    );
    for _ in 0..e.range(1, 4) {
        let before = state.shared.clone();
        state.reset_character_seed();
        let mut expected = before;
        let draw = expected.next_u32();
        props::always!(
            state.menu_run_seed == draw.max(1)
                && state.shared == expected
                && seed::decode(&state.menu_seed_text) == Ok(state.menu_run_seed),
            "mt: Character reset consumes one shared draw for a nonzero menu seed",
            json!({ "epoch": epoch, "draw": draw })
        );
    }
    let committed = e.nonzero_u32();
    let shared = state.shared.clone();
    let result = state.commit_character_seed(committed);
    props::always!(
        result.is_ok()
            && state.menu_run_seed == committed
            && state.shared == shared
            && seed::decode(&state.menu_seed_text) == Ok(committed),
        "mt: committing a dialog seed consumes no shared draw",
        json!({ "seed": committed })
    );
    let before = state.clone();
    props::always!(
        state.commit_character_seed(0).is_err() && state == before,
        "mt: committing reserved zero is rejected without mutation"
    );
}
