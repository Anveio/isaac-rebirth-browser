//! Normal Basement I and II generation (`engine/src/floor.rs`, `layout.rs`,
//! `room_templates.rs`) for fresh and continuing runs, and per-descriptor room
//! admission (`room.rs`). Stage-2 contracts follow
//! `references/isaac/foundation/basement-ii/source-contract.json`.
use crate::entropy::Entropy;
use crate::harness::{self, Case};
use crate::props;
use rebirth_cleanroom::challenges::CurseInputs;
use rebirth_cleanroom::difficulty::Difficulty;
use rebirth_cleanroom::floor::{self, Floor, NormalBasementRequest};
use rebirth_cleanroom::layout::{GRID_CELLS, GRID_WIDTH};
use rebirth_cleanroom::room_assets::RoomAsset;
use rebirth_cleanroom::{room_assets, room_spawns, seed, Room};
use serde_json::json;
use std::collections::{BTreeSet, VecDeque};
use std::panic::{self, AssertUnwindSafe};

/// The 13x13 level grid's centre cell holds the starting room.
const START_CELL: usize = 84;

/// Game flags the generator reads: library (0x100), curse room (64), the
/// saved Monstro/Duke family (16) and the saved horseman family (0x100000).
const READ_FLAGS: [u64; 4] = [0x100, 64, 16, 0x100000];

pub fn run(case: &mut Case) {
    let e = &mut *case.entropy;
    let catalogues = [room_assets::special(), room_assets::basement()];
    let fresh =
        |stage_seed| floor::first_floor(stage_seed, Difficulty::Normal, CurseInputs::default());
    // Only requests no stage generator can serve are pinned: a later stage
    // may be admitted once the port generates it.
    let stage = *e.pick(&[0, u32::MAX]);
    props::always!(
        floor::generate_basement(0).is_err()
            && floor::generate_normal_basement(NormalBasementRequest {
                stage_seed: 0,
                stage: 2,
                ..fresh(1)
            })
            .is_err()
            && floor::generate_normal_basement(NormalBasementRequest {
                stage,
                ..fresh(e.nonzero_u32())
            })
            .is_err(),
        "floor: requests for stage 0, a stage past any floor or stage seed zero are rejected",
        json!({ "stage": stage })
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
        props::always!(
            floor::generate_basement(stage_seed)
                == floor::generate_normal_basement(fresh(stage_seed)),
            "floor: the first-floor entry is the normal generator with fresh run inputs",
            json!({ "stageSeed": stage_seed })
        );
        let difficulty = *e.pick(&[Difficulty::Normal, Difficulty::Hard]);
        let curse = curse_inputs(e);
        let Some(first) = generate(floor::first_floor(stage_seed, difficulty, curse)) else {
            continue;
        };
        swaps(e, &first, &catalogues);
        if e.chance(1, 2) {
            if let Some(second) = generate(continuing(e, &first)) {
                swaps(e, &second, &catalogues);
            }
        }
    }
}

/// A Basement II request as the continuing run supplies it: the retained
/// stage-2 seed, the first floor's resulting flags with the bits the
/// generator reads varied as other events may set them, and in-game health
/// and coins.
fn continuing(e: &mut Entropy, first: &Floor) -> NormalBasementRequest {
    let mut game_flags = first.game_flags;
    for flag in READ_FLAGS {
        if e.chance(1, 3) {
            game_flags ^= flag;
        }
    }
    let max_red_hp = 2 * e.range(0, 12) as u8;
    NormalBasementRequest {
        stage: 2,
        stage_seed: e.nonzero_u32(),
        game_flags,
        red_hp: e.range(0, u32::from(max_red_hp)) as u8,
        max_red_hp,
        soul_hp: e.range(0, 24) as i32,
        coins: e.range(0, 99),
        difficulty: first.difficulty,
        curse: curse_inputs(e),
    }
}

/// Level::Init 58c114/58c124: normal rooms of difficulty 1..5 on a normal
/// run's odd stages and 5..10 on every other stage. Modelled here rather
/// than read from `floor::difficulty_band`, so a changed rules row fails.
fn difficulty_band(stage: u32, difficulty: Difficulty) -> std::ops::RangeInclusive<u8> {
    if stage % 2 == 1 && difficulty == Difficulty::Normal {
        1..=5
    } else {
        5..=10
    }
}

/// Persistent achievements 4/33, counter 10 and Black Candle, each of which
/// changes the curse odds that hard's extra rooms draw from.
fn curse_inputs(e: &mut Entropy) -> CurseInputs {
    CurseInputs {
        achievement_4: e.chance(1, 4),
        achievement_33: e.chance(1, 4),
        counter_10: e.chance(1, 4),
        black_candle: e.chance(1, 8),
    }
}

fn describe(request: &NormalBasementRequest) -> serde_json::Value {
    json!({
        "stage": request.stage,
        "stageSeed": request.stage_seed,
        "gameFlags": request.game_flags,
        "redHp": request.red_hp,
        "maxRedHp": request.max_red_hp,
        "soulHp": request.soul_hp,
        "coins": request.coins,
        "difficulty": request.difficulty,
    })
}

fn generate(request: NormalBasementRequest) -> Option<Floor> {
    let generated = floor::generate_normal_basement(request);
    props::always!(
        generated == floor::generate_normal_basement(request),
        "floor: generation is a pure function of its request",
        describe(&request)
    );
    match generated {
        Ok(floor) => {
            props::always!(
                true,
                "floor: every reachable normal Basement request generates a floor"
            );
            props::sometimes!(
                request.stage == 2,
                "floor: a Basement II floor is generated"
            );
            structure(&request, &floor);
            admission(request.stage_seed, &floor);
            Some(floor)
        }
        Err(error) => {
            props::always!(
                false,
                "floor: every reachable normal Basement request generates a floor",
                json!({ "request": describe(&request), "error": error })
            );
            None
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

fn structure(request: &NormalBasementRequest, floor: &Floor) {
    let stage_seed = request.stage_seed;
    let details = || describe(request);
    let count = floor.descriptors.len();
    props::always!(
        floor.stage == request.stage && floor.stage_seed == stage_seed,
        "floor: a floor records its request's stage and stage seed",
        details()
    );
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
        json!({ "request": details(), "reached": reached.len(), "descriptors": count })
    );

    // 58c260 retries until boss, super secret, shop and treasure all fit;
    // Curse of the Labyrinth adds a second boss (589d5c) and treasure room
    // (58bae3).
    let doubled = if floor.labyrinth() { 2 } else { 1 };
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
        mandatory.iter().all(|(kind, rooms)| *rooms
            == if matches!(kind, 5 | 4) { doubled } else { 1 }),
        "floor: one boss, super secret, shop and treasure room, two bosses and treasure rooms under Curse of the Labyrinth",
        json!({ "request": details(), "counts": mandatory })
    );

    // 58c100..58c130: the starting room is chosen separately.
    let band = difficulty_band(request.stage, request.difficulty);
    props::always!(
        floor
            .descriptors
            .iter()
            .all(|room| room.index == floor.starting_descriptor
                || room.template.room_type != 1
                || band.contains(&room.template.difficulty)),
        "floor: normal rooms stay within the stage's difficulty band",
        details()
    );

    // 584240 on an even floor: the boss comes from the family the first
    // floor saved (horseman 0x100000: 5020/5140; 16: Monstro/Duke), and
    // neither family flag is rewritten.
    if request.stage == 2 {
        let horseman = if request.game_flags & 0x100000 == 0 {
            5020
        } else {
            5140
        };
        let family = if request.game_flags & 16 == 0 {
            1010
        } else {
            1020
        };
        let bosses: Vec<u32> = floor
            .descriptors
            .iter()
            .filter(|room| room.template.room_type == 5)
            .map(|room| room.template.variant)
            .collect();
        let retained = 16 | 0x100000;
        props::always!(
            bosses.iter().all(|variant| [horseman, family]
                .iter()
                .any(|base| (*base..*base + 10).contains(variant)))
                && floor.game_flags & retained == request.game_flags & retained,
            "floor: an even floor's boss comes from the retained family without rewriting it",
            json!({ "request": details(), "bosses": bosses, "flags": floor.game_flags })
        );
    }
    props::sometimes!(rooms_of(7) == 1, "floor: a secret room is placed");
    props::sometimes!(
        floor
            .descriptors
            .iter()
            .any(|room| room.template.width == 26 || room.template.height == 14),
        "floor: a large room shape is placed"
    );

    // Retries continue generator, primary and weight state.
    // Level::Init 58c08e: hard adds 2 rooms plus the curse stream's
    // RandomInt(2) (58c089).
    let (extra_low, extra_high) = match request.difficulty {
        Difficulty::Normal => (0, 0),
        Difficulty::Hard => (2, 3),
    };
    // 58c061: Labyrinth scales the base by 1.8 (at most 45); otherwise Lost
    // (58c4c0) adds four rooms. Hard's extra rooms come after.
    let cursed = |base: u32| {
        if floor.labyrinth() {
            ((f64::from(base) * 1.8) as u32).min(45)
        } else if floor.curses & floor::LOST != 0 {
            base + 4
        } else {
            base
        }
    };
    let base = request.stage * 10 / 3 + 5;
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
            // min(20, stage*10/3 + 5 + bounded35(2)): stage 1 8..9, stage 2
            // 11..12, then the curse adjustment and hard's extra rooms.
            && (cursed(base) + extra_low..=cursed(base + 1) + extra_high)
                .contains(&u32::from(floor.additional_rooms)),
        "floor: attempts are bounded, contiguous and end with a placement",
        json!({ "request": details(), "attempts": floor.attempts, "additionalRooms": floor.additional_rooms, "curses": floor.curses })
    );
    props::sometimes!(floor.attempts > 1, "floor: a generation retry is exercised");

    let saved = serde_json::to_string(floor).expect("floor serializes");
    props::always!(
        serde_json::from_str::<Floor>(&saved).ok().as_ref() == Some(floor),
        "floor: floors round-trip through JSON",
        details()
    );
}

/// Generation weights its authored rooms, so a run admits only a few of the
/// catalogue's 1,064. Each descriptor in turn takes a uniformly chosen
/// authored room the generator could have placed there (same catalogue, type
/// and shape, its doors available, a normal room's difficulty in its stage's
/// range) and must admit like any generated room.
fn swaps(e: &mut Entropy, floor: &Floor, catalogues: &[Vec<RoomAsset>; 2]) {
    for index in 0..floor.descriptors.len() {
        let descriptor = &floor.descriptors[index];
        let Some(catalogue) = catalogues.get(usize::from(descriptor.asset_stage)) else {
            continue;
        };
        let template = &descriptor.template;
        let difficulty = difficulty_band(floor.stage, floor.difficulty);
        let compatible: Vec<usize> = catalogue
            .iter()
            .enumerate()
            .filter(|(_, asset)| {
                let candidate = &asset.template;
                candidate.room_type == template.room_type
                    && (candidate.width, candidate.height) == (template.width, template.height)
                    && descriptor.door_mask & !candidate.door_mask == 0
                    && (template.room_type != 1 || difficulty.contains(&candidate.difficulty))
            })
            .map(|(catalog_index, _)| catalog_index)
            .collect();
        if compatible.is_empty() {
            continue;
        }
        let catalog_index = *e.pick(&compatible);
        let mut swapped = floor.clone();
        let target = &mut swapped.descriptors[index];
        target.catalog_index = catalog_index;
        target.template = catalogue[catalog_index].template.clone();
        admit(floor.stage_seed, &swapped, index);
        props::sometimes!(
            catalog_index != descriptor.catalog_index,
            "floor: another authored room admits in a generated descriptor's place"
        );
    }
}

/// Every generated descriptor must resolve to authored content and admit as
/// a Room: traversal constructs any of them on first entry.
fn admission(stage_seed: u32, floor: &Floor) {
    for index in 0..floor.descriptors.len() {
        admit(stage_seed, floor, index);
    }
}

fn admit(stage_seed: u32, floor: &Floor, index: usize) {
    {
        let descriptor = &floor.descriptors[index];
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
