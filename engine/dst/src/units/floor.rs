//! Fresh normal Basement generation (`engine/src/floor.rs`, `layout.rs`,
//! `room_templates.rs`) and per-descriptor room admission (`room.rs`).
use crate::harness::{self, Case};
use crate::props;
use rebirth_cleanroom::floor::{self, Floor};
use rebirth_cleanroom::layout::{GRID_CELLS, GRID_WIDTH};
use rebirth_cleanroom::{room_assets, room_spawns, seed, Room};
use serde_json::json;
use std::collections::{BTreeSet, VecDeque};
use std::panic::{self, AssertUnwindSafe};

/// The 13x13 level grid's centre cell holds the starting room.
const START_CELL: usize = 84;

pub fn run(case: &mut Case) {
    let e = &mut *case.entropy;
    props::always!(
        floor::generate_basement(0).is_err(),
        "floor: reserved stage seed zero is rejected"
    );
    for _ in 0..(case.steps / 300).max(4) {
        // Production derives the stage seed from a run seed; raw stage seeds
        // also cover values no run seed reaches.
        let run_seed = e.nonzero_u32();
        let stage_seed = match e.below(8) {
            // Run codes whose original floors took two attempts
            // (docs/basement-floor-evidence.md): a dictionary for retries.
            0 => {
                let code = *e.pick(&["QNXPTVT1", "0YAZCZ7N", "BHX1EF8A"]);
                seed::stage1_seed(seed::decode(code).expect("documented seed code"))
            }
            1..=4 => seed::stage1_seed(run_seed),
            _ => e.nonzero_u32(),
        };
        let generated = floor::generate_basement(stage_seed);
        let again = floor::generate_basement(stage_seed);
        props::always!(
            generated == again,
            "floor: generation is a pure function of the stage seed",
            json!({ "stageSeed": stage_seed })
        );
        match generated {
            Ok(floor) => {
                props::always!(true, "floor: every nonzero stage seed generates a floor");
                structure(stage_seed, &floor);
                admission(stage_seed, &floor);
            }
            Err(error) => {
                props::always!(
                    false,
                    "floor: every nonzero stage seed generates a floor",
                    json!({ "stageSeed": stage_seed, "error": error })
                );
            }
        }
    }
}

fn footprint(width: u8, height: u8) -> Option<(i32, i32)> {
    match (width, height) {
        (13, 7) => Some((1, 1)),
        (13, 14) => Some((1, 2)),
        (26, 7) => Some((2, 1)),
        (26, 14) => Some((2, 2)),
        _ => None,
    }
}

fn structure(stage_seed: u32, floor: &Floor) {
    let details = || json!({ "stageSeed": stage_seed });
    let count = floor.descriptors.len();
    props::always!(
        floor.grid.len() == GRID_CELLS
            && floor.grid[START_CELL] >= 0
            && floor.grid[START_CELL] as usize == floor.starting_descriptor
            && floor.starting_descriptor < count,
        "floor: the starting descriptor occupies the grid centre",
        details()
    );
    props::always!(
        floor
            .grid
            .iter()
            .all(|cell| *cell >= -1 && *cell < count as i32)
            && floor
                .descriptors
                .iter()
                .enumerate()
                .all(|(index, room)| room.index == index),
        "floor: grid cells and descriptors index the descriptor table",
        details()
    );

    // Each descriptor owns exactly its rectangular footprint from its origin.
    let mut cells_of: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); count];
    for (cell, owner) in floor.grid.iter().enumerate() {
        if let Ok(owner) = usize::try_from(*owner) {
            if owner < count {
                cells_of[owner].insert(cell);
            }
        }
    }
    let footprints_exact = floor.descriptors.iter().enumerate().all(|(index, room)| {
        let Some((width, height)) = footprint(room.template.width, room.template.height) else {
            return false;
        };
        let origin_x = (room.grid_index % GRID_CELLS) as i32 % GRID_WIDTH;
        let origin_y = room.grid_index as i32 / GRID_WIDTH;
        let expected: BTreeSet<usize> = (0..height)
            .flat_map(|dy| (0..width).map(move |dx| (origin_x + dx, origin_y + dy)))
            .filter(|(x, y)| (0..GRID_WIDTH).contains(x) && (0..GRID_WIDTH).contains(y))
            .map(|(x, y)| (x + y * GRID_WIDTH) as usize)
            .collect();
        expected.len() == (width * height) as usize && cells_of[index] == expected
    });
    props::always!(
        footprints_exact,
        "floor: every descriptor owns exactly its shape's footprint at its origin",
        details()
    );

    // Neighbor links are geometric adjacency and are reciprocated.
    let adjacent = |a: usize, b: usize| {
        cells_of[a].iter().any(|&cell| {
            let (x, y) = (cell as i32 % GRID_WIDTH, cell as i32 / GRID_WIDTH);
            [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| {
                let (nx, ny) = (x + dx, y + dy);
                (0..GRID_WIDTH).contains(&nx)
                    && (0..GRID_WIDTH).contains(&ny)
                    && cells_of[b].contains(&((nx + ny * GRID_WIDTH) as usize))
            })
        })
    };
    let mut links_valid = true;
    for (index, room) in floor.descriptors.iter().enumerate() {
        for neighbor in room.neighbors.iter().flatten() {
            let reciprocal = floor
                .descriptors
                .get(*neighbor)
                .is_some_and(|other| other.neighbors.contains(&Some(index)));
            links_valid &= *neighbor != index && reciprocal && adjacent(index, *neighbor);
        }
    }
    props::always!(
        links_valid,
        "floor: neighbor links are reciprocal edge adjacencies",
        details()
    );

    let mut reached = BTreeSet::from([floor.starting_descriptor]);
    let mut queue = VecDeque::from([floor.starting_descriptor]);
    while let Some(index) = queue.pop_front() {
        for neighbor in floor.descriptors[index].neighbors.iter().flatten() {
            if *neighbor < count && reached.insert(*neighbor) {
                queue.push_back(*neighbor);
            }
        }
    }
    props::always!(
        reached.len() == count,
        "floor: every descriptor is reachable from the starting room",
        json!({ "stageSeed": stage_seed, "reached": reached.len(), "descriptors": count })
    );

    // 58c260 retries until boss, super secret, shop and treasure all fit.
    let rooms_of = |kind: u32| {
        floor
            .descriptors
            .iter()
            .filter(|room| room.template.room_type == kind)
            .count()
    };
    let mandatory = [
        (5, rooms_of(5)),
        (8, rooms_of(8)),
        (2, rooms_of(2)),
        (4, rooms_of(4)),
    ];
    props::always!(
        mandatory.iter().all(|(_, rooms)| *rooms == 1),
        "floor: exactly one boss, super secret, shop and treasure room",
        json!({ "stageSeed": stage_seed, "counts": mandatory })
    );
    props::sometimes!(rooms_of(7) == 1, "floor: a secret room is placed");
    props::sometimes!(
        floor
            .descriptors
            .iter()
            .any(|room| room.template.width == 26 || room.template.height == 14),
        "floor: a large room shape is placed"
    );

    // Retries continue generator, primary and weight state.
    let reports = &floor.attempt_reports;
    let continuous = reports.windows(2).all(|pair| {
        pair[0].final_generator_state == pair[1].initial_generator_state
            && pair[0].final_primary_state == pair[1].initial_primary_state
            && pair[0].outcome == "retry"
    });
    props::always!(
        (1..=100).contains(&floor.attempts)
            && reports.len() == floor.attempts as usize
            && reports.last().is_some_and(|last| last.outcome == "placed")
            && continuous
            && (8..=9).contains(&floor.additional_rooms),
        "floor: attempts are bounded, contiguous and end with a placement",
        json!({ "stageSeed": stage_seed, "attempts": floor.attempts })
    );
    props::sometimes!(floor.attempts > 1, "floor: a generation retry is exercised");

    let saved = serde_json::to_string(floor).expect("floor serializes");
    props::always!(
        serde_json::from_str::<Floor>(&saved).ok().as_ref() == Some(floor),
        "floor: floors round-trip through JSON",
        details()
    );
}

/// Every generated descriptor must resolve to authored content and admit as
/// a Room: traversal constructs any of them on first entry.
fn admission(stage_seed: u32, floor: &Floor) {
    for (index, descriptor) in floor.descriptors.iter().enumerate() {
        let resolved = room_assets::for_descriptor(descriptor)
            .and_then(|asset| room_spawns::select(&asset, descriptor.spawn_seed));
        props::always!(
            resolved.is_ok(),
            "floor: every descriptor resolves authored layout and spawns",
            json!({ "stageSeed": stage_seed, "descriptor": index, "error": resolved.err() })
        );
        let admitted = panic::catch_unwind(AssertUnwindSafe(|| {
            Room::from_floor_descriptor(floor, index)
        }));
        match admitted {
            Ok(room) => {
                props::always!(
                    true,
                    "floor: every descriptor admits as a room without panicking"
                );
                let doors_match = room.appearance.doors.len()
                    == descriptor.neighbors.iter().flatten().count()
                    && room.appearance.doors.iter().all(|door| {
                        descriptor.neighbors[usize::from(door.slot)]
                            == Some(door.target_room as usize)
                    });
                props::always!(
                    doors_match
                        && room.grid.is_some()
                        && room.left < room.right
                        && room.top < room.bottom,
                    "floor: admitted rooms have one door per neighbor and bounded geometry",
                    json!({ "stageSeed": stage_seed, "descriptor": index })
                );
            }
            Err(_) => {
                let message = harness::take_panic();
                props::always!(
                    false,
                    "floor: every descriptor admits as a room without panicking",
                    json!({
                        "stageSeed": stage_seed,
                        "descriptor": index,
                        "template": descriptor.template,
                        "panic": message,
                    })
                );
            }
        }
    }
}
