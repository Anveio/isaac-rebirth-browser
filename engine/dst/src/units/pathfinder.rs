//! FindGridPath's flood (`pathfinder::flood_costs`, 5b498f) over room-shaped
//! and arbitrary cost grids, against an independent node-weighted Dijkstra.
//!
//! The flood labels an open cell with -distance when it is first relaxed,
//! and its heap pops nearest first, so every label is the shortest distance
//! whatever order ties take. Ties only decide which equal-distance cells
//! expand before the expansion that visits the owner cell, so the visited
//! set is checked exactly when the owner is not visited and between bounds
//! when it is.
use crate::entropy::Entropy;
use crate::harness::Case;
use crate::props;
use rebirth_cleanroom::grid::MAX_GRID_CELLS;
use rebirth_cleanroom::pathfinder::flood_costs;
use rebirth_cleanroom::room_costs::RoomCosts;
use serde::Serialize;
use serde_json::json;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// Room grid sizes: 1x1, 2x1, 1x2 and 2x2 rooms and the two closets.
const ROOMS: [(u16, u16); 6] = [(15, 9), (28, 9), (15, 16), (28, 16), (15, 5), (5, 9)];

#[derive(Clone, Copy, Serialize)]
struct Request {
    width: u16,
    height: u16,
    start: i32,
    owner: i32,
}

pub fn run(case: &mut Case) {
    let e = &mut *case.entropy;
    for _ in 0..(case.steps / 4).max(64) {
        let (width, height) = size(e);
        let costs = costs(e, width, height);
        let cells = i32::from(width) * i32::from(height);
        let start = cell(e, cells);
        let owner = match e.below(8) {
            0 => start,
            _ => cell(e, cells),
        };
        let request = Request {
            width,
            height,
            start,
            owner,
        };
        let details = || {
            let shown = usize::try_from(cells).map_or(0, |cells| cells.min(MAX_GRID_CELLS));
            json!({ "request": request, "costs": &costs.entries()[..shown] })
        };
        let admitted =
            width > 0 && height > 0 && usize::from(width) * usize::from(height) <= MAX_GRID_CELLS;
        let result = flood_costs(&costs, width, height, start, owner);
        props::always!(
            result.is_ok() == admitted,
            "pathfinder: the flood runs on every admitted grid and rejects any other",
            details()
        );
        let Ok((flood, found)) = result else {
            continue;
        };
        props::always!(
            flood_costs(&costs, width, height, start, owner).ok() == Some((flood.clone(), found)),
            "pathfinder: the flood is a pure function of its grid, start and owner",
            details()
        );
        check(request, &costs, &flood, found, &details);
    }
}

/// Mostly the engine's room grids, then any admitted size, then sizes the
/// grid admission refuses.
fn size(e: &mut Entropy) -> (u16, u16) {
    match e.below(16) {
        0..=9 => *e.pick(&ROOMS),
        10..=14 => {
            let width = e.range(1, 40) as u16;
            let tallest = (MAX_GRID_CELLS / usize::from(width)).min(40) as u32;
            (width, e.range(1, tallest) as u16)
        }
        _ => match e.below(3) {
            0 => (0, e.range(0, 20) as u16),
            1 => (e.range(1, 20) as u16, 0),
            _ => (e.range(23, 60) as u16, e.range(20, 60) as u16),
        },
    }
}

/// Room.Init's costs (walls and doors 1000, rocks 1000 or 0, pits 3000 or 0,
/// spikes 999) with FindGridPath's 900 owner stamps and other values
/// besides, or arbitrary words. Cells beyond the room are noise the flood
/// must never touch.
fn costs(e: &mut Entropy, width: u16, height: u16) -> RoomCosts {
    let mut costs = RoomCosts::zeroed();
    let walled = e.chance(3, 4);
    let blocked = e.below(60);
    let arbitrary = e.chance(1, 16);
    let (width, height) = (usize::from(width), usize::from(height));
    for (index, word) in costs.entries_mut().iter_mut().enumerate() {
        let inside = width > 0 && index < width * height;
        let border = inside && {
            let (x, y) = (index % width, index / width);
            x == 0 || y == 0 || x + 1 == width || y + 1 == height
        };
        *word = if !inside || arbitrary {
            // Every magnitude, so small open words turn up too.
            (e.u32() as i32) >> e.below(32)
        } else if walled && border {
            1000
        } else if e.below(100) >= blocked {
            0
        } else {
            match e.below(16) {
                0..=5 => 1000,
                6 | 7 => 3000,
                8 => 999,
                9..=11 => 900,
                12 | 13 => e.range(1, 949) as i32,
                14 => e.range(950, 5000) as i32,
                _ => -(e.range(1, 2000) as i32),
            }
        };
    }
    costs
}

/// A cell of the grid, or occasionally one beside it.
fn cell(e: &mut Entropy, cells: i32) -> i32 {
    match e.below(32) {
        0 => -1,
        1 => cells.max(0) + e.below(8) as i32,
        _ if cells > 0 => e.below(cells as u64) as i32,
        _ => 0,
    }
}

fn open(cost: i32) -> bool {
    (0..=949).contains(&cost)
}

fn weight(cost: i32) -> i32 {
    if cost == 0 {
        1
    } else {
        7
    }
}

fn neighbours(width: usize, height: usize, cell: usize) -> impl Iterator<Item = usize> {
    let (x, y) = (cell % width, cell / width);
    [
        (x > 0).then(|| cell - 1),
        (y > 0).then(|| cell - width),
        (x + 1 < width).then(|| cell + 1),
        (y + 1 < height).then(|| cell + width),
    ]
    .into_iter()
    .flatten()
}

/// Shortest distances from `start`, entering an open cell for its weight.
fn distances(costs: &[i32], width: usize, height: usize, start: usize) -> Vec<Option<i32>> {
    let mut best = vec![None; width * height];
    let mut queue = BinaryHeap::from([Reverse((0, start))]);
    best[start] = Some(0);
    while let Some(Reverse((distance, cell))) = queue.pop() {
        if best[cell] != Some(distance) {
            continue;
        }
        for next in neighbours(width, height, cell) {
            if next == start || !open(costs[next]) {
                continue;
            }
            let reached = distance + weight(costs[next]);
            if best[next].is_none_or(|known| reached < known) {
                best[next] = Some(reached);
                queue.push(Reverse((reached, next)));
            }
        }
    }
    best
}

fn check(
    request: Request,
    costs: &RoomCosts,
    flood: &RoomCosts,
    found: bool,
    details: &dyn Fn() -> serde_json::Value,
) {
    let Request {
        width,
        height,
        start,
        owner,
    } = request;
    let (width, height) = (usize::from(width), usize::from(height));
    let cells = width * height;
    let before = costs.entries();
    let after = flood.entries();
    let changed = |index: usize| before[index] != after[index];
    props::always!(
        (0..MAX_GRID_CELLS)
            .filter(|&index| changed(index))
            .all(|index| index < cells && open(before[index]) && after[index] < 0),
        "pathfinder: the flood only marks open cells of its grid, with negative distances",
        details()
    );
    let Ok(start) = usize::try_from(start) else {
        return;
    };
    if start >= cells {
        return;
    }
    let half = i32::try_from(cells / 2).expect("admitted grids are small");
    let distance = distances(before, width, height, start);
    // The nearest expanded neighbour decides a cell's label: Dijkstra's
    // distance, or for the start cell its relabelling by a neighbour.
    let nearest = |cell: usize| {
        neighbours(width, height, cell)
            .filter_map(|next| distance[next])
            .min()
    };
    let label = |cell: usize| nearest(cell).map(|near| near + weight(before[cell]));
    let visited: Vec<usize> = (0..cells).filter(|&cell| changed(cell)).collect();
    props::always!(
        visited.iter().all(|&cell| label(cell) == Some(-after[cell])
            && (cell == start || distance[cell] == Some(-after[cell]))),
        "pathfinder: every flood label is the cell's shortest distance from the start",
        details()
    );
    let reachable = |cell: usize, bound: i32, inclusive: bool| {
        open(before[cell])
            && nearest(cell).is_some_and(|near| near < bound || (inclusive && near == bound))
    };
    let expected_found =
        usize::try_from(owner).is_ok_and(|owner| owner < cells && reachable(owner, half, false));
    props::always!(
        found == expected_found
            && (!found
                || usize::try_from(owner).is_ok_and(|owner| owner < cells && changed(owner))),
        "pathfinder: the flood visits the owner exactly when it lies within half the room's cells",
        details()
    );
    if found != expected_found {
        return;
    }
    // A visited owner is then a grid cell with an expanded neighbour.
    let stop = usize::try_from(owner)
        .ok()
        .filter(|_| found)
        .and_then(|owner| nearest(owner).map(|at| (owner, at)));
    let (lowest, highest): (Vec<usize>, Vec<usize>) = match stop {
        // Every cell nearer than the owner's expanding neighbour expands
        // first; cells at that distance may or may not.
        Some((owner, at)) => (
            (0..cells)
                .filter(|&cell| cell == owner || reachable(cell, at, false))
                .collect(),
            (0..cells)
                .filter(|&cell| reachable(cell, at, true))
                .collect(),
        ),
        None => {
            let all: Vec<usize> = (0..cells)
                .filter(|&cell| reachable(cell, half, false))
                .collect();
            (all.clone(), all)
        }
    };
    // The expansion that visits the owner runs to its end: some nearest
    // neighbour of the owner has every open neighbour visited.
    let completed = stop.is_none_or(|(owner, at)| {
        neighbours(width, height, owner)
            .filter(|&next| distance[next] == Some(at))
            .any(|next| {
                neighbours(width, height, next).all(|cell| !open(before[cell]) || changed(cell))
            })
    });
    let within = completed
        && lowest.iter().all(|cell| visited.contains(cell))
        && visited.iter().all(|cell| highest.contains(cell));
    props::always!(
        within,
        "pathfinder: the flood visits every open cell nearer than where it stops, and no farther one",
        details()
    );
    let open_reachable = (0..cells)
        .filter(|&cell| cell != start && distance[cell].is_some())
        .count();
    props::sometimes!(found, "pathfinder: a flood visits its owner cell");
    props::sometimes!(
        !found && visited.len() < open_reachable,
        "pathfinder: a flood stops at half the room's cells before every reachable cell"
    );
    props::sometimes!(
        found && lowest.len() < highest.len(),
        "pathfinder: equal-distance cells tie at the expansion that visits the owner"
    );
    props::sometimes!(
        changed(start),
        "pathfinder: a flood relabels its own start cell"
    );
    props::sometimes!(
        visited.iter().any(|&cell| before[cell] != 0),
        "pathfinder: a flood crosses a stamped or costly open cell"
    );
}
