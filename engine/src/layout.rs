//! Recovered rectangular floor growth before special-room assignment.
//!
//! Input is the generator's own RNG35 seed, not a run or decoration seed.
//! Native special placement can resize these rectangles. Outer map retries,
//! curses, secret-room addition and template assignment remain separate gaps.
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub const GRID_WIDTH: i32 = 13;
pub const GRID_CELLS: usize = 169;
pub const GROWTH_CONTRACT: &str =
    "recovered-rectangle-growth-and-dead-end-expansion;before-special-assignment-and-outer-retries";

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GrowthRequest {
    pub generator_seed: u32,
    pub additional_rooms: u8,
    /// Bits 0..3 are authored 1x1, 1x2, 2x1 and 2x2 availability.
    pub shape_mask: u8,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeneratedRoom {
    pub index: usize,
    pub origin: Cell,
    pub width: i32,
    pub height: i32,
    pub connector: Cell,
    /// Incoming cardinal direction 0:left, 1:up, 2:right, 3:down.
    /// The already inserted starting room has no incoming direction.
    pub direction: Option<u8>,
    pub distance: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Growth {
    pub contract: String,
    pub input: GrowthRequest,
    pub rooms: Vec<GeneratedRoom>,
    /// Room indices in the 13x13 grid; -1 denotes an unused cell.
    pub grid: Vec<i32>,
    pub added_rooms: u8,
    pub growth_loops: u32,
    pub base_rng_draws: u32,
    pub expansion_attempts: u8,
    pub dead_end_indices: Vec<usize>,
    pub rng_draws: u32,
    pub final_rng_state: u32,
    pub growth_queue_exhausted: bool,
}

struct Rng35 {
    state: u32,
    draws: u32,
}
impl Rng35 {
    fn random(&mut self, bound: u32) -> u32 {
        self.state ^= self.state >> 5;
        self.state ^= self.state << 9;
        self.state ^= self.state >> 7;
        self.draws += 1;
        self.state % bound
    }
}

struct Builder {
    rng: Rng35,
    shape_mask: u8,
    rooms: Vec<GeneratedRoom>,
    grid: [i32; GRID_CELLS],
    end_flags: Vec<bool>,
}

fn cell_index(cell: Cell) -> Option<usize> {
    if (0..GRID_WIDTH).contains(&cell.x) && (0..GRID_WIDTH).contains(&cell.y) {
        Some((cell.x + cell.y * GRID_WIDTH) as usize)
    } else {
        None
    }
}

impl Builder {
    fn fits(&self, origin: Cell, width: i32, height: i32) -> bool {
        if origin.x < 0
            || origin.y < 0
            || origin.x + width > GRID_WIDTH
            || origin.y + height > GRID_WIDTH
        {
            return false;
        }
        (origin.y..origin.y + height).all(|y| {
            (origin.x..origin.x + width).all(|x| self.grid[(x + y * GRID_WIDTH) as usize] < 0)
        })
    }

    fn insert(&mut self, mut room: GeneratedRoom) -> usize {
        let index = self.rooms.len();
        room.index = index;
        for y in room.origin.y..room.origin.y + room.height {
            for x in room.origin.x..room.origin.x + room.width {
                self.grid[(x + y * GRID_WIDTH) as usize] = index as i32;
            }
        }
        self.rooms.push(room);
        self.end_flags.push(false);
        index
    }

    fn neighbours(&self, cell: Cell) -> usize {
        let own = self.grid[cell_index(cell).expect("source connector in grid")];
        [(-1, 0), (0, -1), (1, 0), (0, 1)]
            .iter()
            .filter(|(dx, dy)| {
                cell_index(Cell {
                    x: cell.x + dx,
                    y: cell.y + dy,
                })
                .is_some_and(|index| self.grid[index] >= 0 && self.grid[index] != own)
            })
            .count()
    }

    fn candidates(&mut self, parent: &GeneratedRoom, all_shapes: bool) -> Vec<GeneratedRoom> {
        let Cell { x, y } = parent.origin;
        let nominal = [
            (x - 1, y),
            (x, y - 1),
            (x + parent.width, y),
            (x, y + parent.height),
            (x - 1, y + 1),
            (x + 1, y - 1),
            (x + parent.width, y + 1),
            (x + 1, y + parent.height),
        ];
        let mut candidates = Vec::new();
        for (slot, (nx, ny)) in nominal.into_iter().enumerate() {
            if (matches!(slot, 4 | 6) && parent.height == 1)
                || (matches!(slot, 5 | 7) && parent.width == 1)
                || !self.fits(Cell { x: nx, y: ny }, 1, 1)
            {
                continue;
            }
            let direction = (slot % 4) as u8;
            let candidate = |ax, ay, width, height| GeneratedRoom {
                index: 0,
                origin: Cell { x: ax, y: ay },
                width,
                height,
                connector: Cell { x: nx, y: ny },
                direction: Some(direction),
                distance: parent.distance + 1,
            };
            if self.shape_mask & 1 != 0 {
                candidates.push(candidate(nx, ny, 1, 1));
            }
            let dx = if direction == 0 { -1 } else { 0 };
            let dy = if direction == 1 { -1 } else { 0 };
            let large = if matches!(direction, 0 | 2) {
                [
                    (nx + dx, ny, 2, 1),
                    (nx, ny, 1, 2),
                    (nx, ny - 1, 1, 2),
                    (nx + dx, ny, 2, 2),
                    (nx + dx, ny - 1, 2, 2),
                ]
            } else {
                [
                    (nx, ny + dy, 1, 2),
                    (nx, ny, 2, 1),
                    (nx - 1, ny, 2, 1),
                    (nx, ny + dy, 2, 2),
                    (nx - 1, ny + dy, 2, 2),
                ]
            };
            for (ax, ay, width, height) in large {
                // 5907c0 consumes all five gates before availability/fit tests.
                let gate = self.rng.random(20);
                if gate != 0 && self.shape_mask & 1 != 0 && !all_shapes {
                    continue;
                }
                let shape = match (width, height) {
                    (1, 2) => 1,
                    (2, 1) => 2,
                    (2, 2) => 3,
                    _ => unreachable!("source large shape catalogue"),
                };
                if self.shape_mask & (1 << shape) != 0
                    && self.fits(Cell { x: ax, y: ay }, width, height)
                {
                    candidates.push(candidate(ax, ay, width, height));
                }
            }
        }
        candidates
    }

    fn dead_ends(&mut self) -> Vec<usize> {
        // 58ee80 retains flags, then clears each child's incoming parent.
        for room in self.rooms.iter().skip(1) {
            if self.neighbours(room.connector) <= 1 {
                self.end_flags[room.index] = true;
            }
        }
        for room in self.rooms.iter().skip(1) {
            let (dx, dy) = [(1, 0), (0, 1), (-1, 0), (0, -1)]
                [usize::from(room.direction.expect("nonstart incoming direction"))];
            let parent_cell = Cell {
                x: room.connector.x + dx,
                y: room.connector.y + dy,
            };
            let parent =
                self.grid[cell_index(parent_cell).expect("source incoming parent in grid")];
            assert!(parent >= 0, "source incoming parent is occupied");
            self.end_flags[parent as usize] = false;
        }
        self.end_flags
            .iter()
            .enumerate()
            .filter_map(|(index, flag)| flag.then_some(index))
            .collect()
    }

    fn expand(&mut self) {
        // 5918c0 scans creation order without shuffle, child-count or queue draws.
        for index in 0..self.rooms.len() {
            let parent = self.rooms[index].clone();
            for candidate in self.candidates(&parent, true) {
                if self.fits(candidate.origin, candidate.width, candidate.height)
                    && self.neighbours(candidate.connector) <= 1
                {
                    self.insert(candidate);
                    return;
                }
            }
        }
    }
}

pub fn generate(input: GrowthRequest) -> Result<Growth, String> {
    if input.generator_seed == 0 {
        return Err("a nonzero independently supplied generator seed is required".into());
    }
    // Level::Init passes at most 20 (58bf57), 45 under Labyrinth (58c079),
    // plus hard's 2 + RandomInt(2) (58c08e).
    if !(1..=48).contains(&input.additional_rooms) {
        return Err("additional room count must be within Level::Init's 1..48 range".into());
    }
    if input.shape_mask == 0 || input.shape_mask & !15 != 0 {
        return Err("shape availability must be a nonempty four-bit mask".into());
    }
    let start = Cell { x: 6, y: 6 };
    let mut builder = Builder {
        rng: Rng35 {
            state: input.generator_seed,
            draws: 0,
        },
        shape_mask: input.shape_mask,
        rooms: Vec::new(),
        grid: [-1; GRID_CELLS],
        end_flags: Vec::new(),
    };
    builder.insert(GeneratedRoom {
        index: 0,
        origin: start,
        width: 1,
        height: 1,
        connector: start,
        direction: None,
        distance: 0,
    });
    let mut queue = VecDeque::from([start]);
    let mut added = 0;
    let mut loops = 0;
    while !queue.is_empty() && added < input.additional_rooms {
        let origin = queue.pop_front().expect("nonempty source growth queue");
        let parent = builder.rooms[builder.grid[cell_index(origin).unwrap()] as usize].clone();
        let mut candidates = builder.candidates(&parent, false);
        for index in (1..candidates.len()).rev() {
            let other = builder.rng.random((index + 1) as u32) as usize;
            candidates.swap(index, other);
        }
        let wanted = if origin == start {
            if input.additional_rooms > 15 {
                candidates.len()
            } else {
                2 + builder.rng.random(2) as usize
            }
        } else {
            let draws = (0..4).map(|_| builder.rng.random(2) as usize).sum();
            candidates.len().min(draws)
        };
        let mut selected = 0;
        loops += 1;
        for candidate in candidates {
            if !builder.fits(candidate.origin, candidate.width, candidate.height)
                || builder.neighbours(candidate.connector) > 1
            {
                continue;
            }
            let child_origin = candidate.origin;
            builder.insert(candidate);
            if builder.rng.random(3) != 0 {
                queue.push_front(child_origin);
            } else {
                queue.push_back(child_origin);
            }
            selected += 1;
            added += 1;
            // Native limit check occurs after insertion, including wanted=0.
            if added >= input.additional_rooms || selected >= wanted {
                break;
            }
        }
    }
    let exhausted = queue.is_empty();
    let base_draws = builder.rng.draws;
    let mut ends = builder.dead_ends();
    let mut attempts = 0;
    while ends.len() < 5 && attempts < 5 {
        builder.expand();
        attempts += 1;
        ends = builder.dead_ends();
    }
    Ok(Growth {
        contract: GROWTH_CONTRACT.into(),
        input,
        rooms: builder.rooms,
        grid: builder.grid.into(),
        added_rooms: added,
        growth_loops: loops,
        base_rng_draws: base_draws,
        expansion_attempts: attempts,
        dead_end_indices: ends,
        rng_draws: builder.rng.draws,
        final_rng_state: builder.rng.state,
        growth_queue_exhausted: exhausted,
    })
}
