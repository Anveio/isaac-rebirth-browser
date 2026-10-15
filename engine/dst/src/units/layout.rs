//! Rectangular floor growth (`engine/src/layout.rs`) over every input it
//! accepts, not only the masks and room counts real floors request.
//!
//! Oracles are structural and independent of the growth order: the grid and
//! the room rectangles agree, every room has an available shape and grows
//! from an earlier room one step farther from the start, the requested rooms
//! are added unless the queue runs dry, dead ends are leaves, and the
//! reported draw count replays the final RNG35 state through the engine's
//! other RNG35 (`items::Rng`).
use crate::entropy::Entropy;
use crate::harness::Case;
use crate::props;
use rebirth_cleanroom::items::Rng;
use rebirth_cleanroom::layout::{self, Cell, Growth, GrowthRequest, GRID_CELLS, GRID_WIDTH};
use serde_json::json;
use std::collections::BTreeSet;

const START: Cell = Cell { x: 6, y: 6 };

pub fn run(case: &mut Case) {
    let e = &mut *case.entropy;
    for _ in 0..(case.steps / 4).max(64) {
        let request = request(e);
        let valid = request.generator_seed != 0
            && (1..=20).contains(&request.additional_rooms)
            && request.shape_mask != 0
            && request.shape_mask & !15 == 0;
        let generated = layout::generate(request);
        let details = json!({ "request": request });
        if !valid {
            props::always!(
                generated.is_err(),
                "layout: seeds, room counts and shape masks outside the source ranges are rejected",
                details
            );
            continue;
        }
        let Ok(growth) = generated else {
            props::unreachable!("layout: a valid growth request fails", details);
            continue;
        };
        props::always!(
            layout::generate(request).ok().as_ref() == Some(&growth),
            "layout: growth is a pure function of its request",
            details.clone()
        );
        let saved = serde_json::to_value(&growth).expect("growth serializes");
        props::always!(
            serde_json::from_value::<Growth>(saved).ok().as_ref() == Some(&growth),
            "layout: growth round-trips through JSON",
            details.clone()
        );
        structure(&growth, &details);
        props::sometimes!(
            growth.expansion_attempts > 0,
            "layout: dead-end expansion runs"
        );
        props::sometimes!(
            request.shape_mask & 1 == 0,
            "layout: a growth without 1x1 rooms completes"
        );
    }
}

fn request(e: &mut Entropy) -> GrowthRequest {
    let generator_seed = if e.chance(1, 32) { 0 } else { e.nonzero_u32() };
    let additional_rooms = match e.below(16) {
        0 => 0,
        1 => 21 + e.below(235) as u8,
        _ => e.range(1, 20) as u8,
    };
    let shape_mask = match e.below(16) {
        0 => 0,
        1 => 16 + e.below(240) as u8,
        _ => e.range(1, 15) as u8,
    };
    GrowthRequest {
        generator_seed,
        additional_rooms,
        shape_mask,
    }
}

fn cells(origin: Cell, width: i32, height: i32) -> impl Iterator<Item = Cell> {
    (origin.y..origin.y + height)
        .flat_map(move |y| (origin.x..origin.x + width).map(move |x| Cell { x, y }))
}

fn index_of(cell: Cell) -> Option<usize> {
    ((0..GRID_WIDTH).contains(&cell.x) && (0..GRID_WIDTH).contains(&cell.y))
        .then(|| (cell.x + cell.y * GRID_WIDTH) as usize)
}

fn structure(growth: &Growth, details: &serde_json::Value) {
    let request = growth.input;
    let rooms = &growth.rooms;
    // Every room's rectangle lies on the grid and owns exactly its cells.
    let mut owned = vec![-1; GRID_CELLS];
    let mut on_grid = growth.grid.len() == GRID_CELLS;
    for (index, room) in rooms.iter().enumerate() {
        on_grid &= room.index == index;
        for cell in cells(room.origin, room.width, room.height) {
            match index_of(cell) {
                Some(at) if owned[at] == -1 => owned[at] = index as i32,
                _ => on_grid = false,
            }
        }
    }
    props::always!(
        on_grid && owned == growth.grid,
        "layout: the grid marks exactly each room's rectangle",
        details.clone()
    );
    let start = rooms.first();
    props::always!(
        start.is_some_and(|room| {
            room.origin == START
                && (room.width, room.height) == (1, 1)
                && room.direction.is_none()
                && room.distance == 0
        }),
        "layout: growth starts from the 1x1 room at the grid centre",
        details.clone()
    );
    let available = |width: i32, height: i32| {
        let bit = match (width, height) {
            (1, 1) => 0,
            (1, 2) => 1,
            (2, 1) => 2,
            (2, 2) => 3,
            _ => return false,
        };
        request.shape_mask & (1 << bit) != 0
    };
    props::always!(
        rooms
            .iter()
            .skip(1)
            .all(|room| available(room.width, room.height)),
        "layout: every grown room has a shape the mask makes available",
        details.clone()
    );
    // A room's connector lies in its rectangle; the cell beyond it, against
    // its incoming direction, belongs to an earlier room one step nearer.
    let mut parents = BTreeSet::new();
    let grown = rooms.iter().skip(1).all(|room| {
        let inside = cells(room.origin, room.width, room.height).any(|cell| cell == room.connector);
        let Some(direction) = room.direction else {
            return false;
        };
        let (dx, dy) = [(1, 0), (0, 1), (-1, 0), (0, -1)][usize::from(direction % 4)];
        let beyond = Cell {
            x: room.connector.x + dx,
            y: room.connector.y + dy,
        };
        let parent = index_of(beyond).map(|at| growth.grid[at]);
        let Some(parent) = parent.and_then(|parent| usize::try_from(parent).ok()) else {
            return false;
        };
        parents.insert(parent);
        inside
            && direction < 4
            && parent < room.index
            && rooms[parent].distance + 1 == room.distance
    });
    props::always!(
        grown,
        "layout: every room grows from an earlier room one step farther from the start",
        details.clone()
    );
    props::always!(
        growth.added_rooms <= request.additional_rooms
            && (growth.added_rooms == request.additional_rooms || growth.growth_queue_exhausted)
            && rooms.len() > usize::from(growth.added_rooms),
        "layout: growth adds the requested rooms unless its queue runs dry",
        details.clone()
    );
    let mut replay = Rng::new(request.generator_seed, [5, 9, 7]);
    for _ in 0..growth.rng_draws {
        replay.raw();
    }
    props::always!(
        replay.state == growth.final_rng_state && growth.base_rng_draws <= growth.rng_draws,
        "layout: the reported draws reproduce the final RNG35 state",
        details.clone()
    );
    props::always!(
        growth
            .dead_end_indices
            .iter()
            .all(|end| *end != 0 && *end < rooms.len() && !parents.contains(end)),
        "layout: dead ends are grown rooms no other room grows from",
        details.clone()
    );
    props::always!(
        growth.expansion_attempts <= 5
            && (growth.expansion_attempts == 5 || growth.dead_end_indices.len() >= 5),
        "layout: expansion stops at five dead ends or five attempts",
        details.clone()
    );
}
