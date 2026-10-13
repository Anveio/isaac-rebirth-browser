//! Source-derived fresh-profile normal Basement generation.
//! Independent stage primary, generator and local selector streams are kept
//! separate. Authored catalogues supply every template and spawn selector.
use crate::layout::{self, Cell, GeneratedRoom, Growth, GrowthRequest, GRID_CELLS, GRID_WIDTH};
use crate::room_assets;
use crate::room_templates::{AuthoredTemplate, SelectionRequest, TemplatePicker};
use serde::{Deserialize, Serialize};

pub const CONTRACT: &str = "fresh-profile-normal-isaac-basement-without-labyrinth;source-growth-fitting-secret-primary-routing-retry-continuity";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Descriptor {
    pub index: usize,
    pub generated_index: usize,
    pub grid_index: usize,
    pub asset_stage: u8,
    pub catalog_index: usize,
    pub template: AuthoredTemplate,
    pub door_mask: u64,
    /// Neighbor descriptor indices, rather than a captured/native grid map.
    pub neighbors: [Option<usize>; 8],
    pub room_init_seed: u32,
    pub decoration_seed: u32,
    pub spawn_seed: u32,
    pub following_seed: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Floor {
    pub contract: String,
    pub stage_seed: u32,
    pub additional_rooms: u8,
    pub attempts: u32,
    pub descriptors: Vec<Descriptor>,
    /// Final native-style grid cells contain descriptor indices or -1.
    pub grid: Vec<i32>,
    pub starting_descriptor: usize,
    pub primary_state: u32,
    pub primary_draws: u32,
    pub generator_state: u32,
    pub game_flags: u64,
    /// Native weights reset once before the outer loop and survive retries.
    pub special_weights: Vec<f32>,
    pub basement_weights: Vec<f32>,
    pub attempt_reports: Vec<AttemptReport>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AttemptReport {
    pub attempt: u32,
    pub initial_generator_state: u32,
    pub final_generator_state: u32,
    pub initial_primary_state: u32,
    pub final_primary_state: u32,
    pub outcome: String,
}

struct Primary {
    state: u32,
    draws: u32,
}
fn next35(mut state: u32) -> u32 {
    state ^= state >> 5;
    state ^= state << 9;
    state ^= state >> 7;
    state
}
impl Primary {
    fn next(&mut self) -> u32 {
        self.state = next35(self.state);
        self.draws += 1;
        self.state
    }
    fn random(&mut self, bound: u32) -> u32 {
        self.next() % bound
    }
}
fn next12(mut state: u32) -> u32 {
    state ^= state >> 2;
    state ^= state << 7;
    state ^= state >> 9;
    state
}
fn cell_index(cell: Cell) -> Option<usize> {
    ((0..GRID_WIDTH).contains(&cell.x) && (0..GRID_WIDTH).contains(&cell.y))
        .then_some((cell.x + cell.y * GRID_WIDTH) as usize)
}
fn slots(room: &GeneratedRoom) -> [(u8, Cell); 8] {
    let Cell { x, y } = room.origin;
    [
        (0, Cell { x: x - 1, y }),
        (1, Cell { x, y: y - 1 }),
        (
            2,
            Cell {
                x: x + room.width,
                y,
            },
        ),
        (
            3,
            Cell {
                x,
                y: y + room.height,
            },
        ),
        (4, Cell { x: x - 1, y: y + 1 }),
        (5, Cell { x: x + 1, y: y - 1 }),
        (
            6,
            Cell {
                x: x + room.width,
                y: y + 1,
            },
        ),
        (
            7,
            Cell {
                x: x + 1,
                y: y + room.height,
            },
        ),
    ]
}
fn valid_slot(room: &GeneratedRoom, slot: u8) -> bool {
    !((matches!(slot, 4 | 6) && room.height == 1) || (matches!(slot, 5 | 7) && room.width == 1))
}

struct Fitting {
    rooms: Vec<GeneratedRoom>,
    grid: Vec<i32>,
    ends: Vec<usize>,
    original_non_ends: Vec<usize>,
}
impl Fitting {
    fn new(growth: Growth) -> Self {
        let mut ends = growth.dead_end_indices;
        // 58eb80 uses selection-sort; equal distances need its actual swaps.
        for left in 0..ends.len() {
            let mut best = left;
            for right in left + 1..ends.len() {
                if growth.rooms[ends[right]].distance > growth.rooms[ends[best]].distance {
                    best = right;
                }
            }
            ends.swap(left, best);
        }
        let original_non_ends = (0..growth.rooms.len())
            .filter(|index| !ends.contains(index))
            .collect();
        Self {
            rooms: growth.rooms,
            grid: growth.grid,
            ends,
            original_non_ends,
        }
    }
    fn fit_small(&mut self, mask: u64, boss: bool) -> Option<usize> {
        for position in 0..self.ends.len() {
            let index = self.ends[position];
            let room = &self.rooms[index];
            if boss && room.distance <= 1 {
                continue;
            }
            let opposite = (room.direction? + 2) % 4;
            if mask & (1 << opposite) == 0 {
                continue;
            }
            let Cell { x, y } = room.connector;
            let mut neighbors = 0;
            let mut valid = true;
            for (slot, (dx, dy)) in [(-1, 0), (0, -1), (1, 0), (0, 1)].into_iter().enumerate() {
                if let Some(cell) = cell_index(Cell {
                    x: x + dx,
                    y: y + dy,
                }) {
                    let neighbor = self.grid[cell];
                    if neighbor >= 0 && neighbor != index as i32 {
                        neighbors += 1;
                        valid &= mask & (1 << slot) != 0;
                    }
                }
            }
            if neighbors > 1 || !valid {
                continue;
            }
            // 58f480 candidate0 shrinks at the incoming connector. Rejection
            // later re-adds this index without undoing the shape mutation.
            for value in &mut self.grid {
                if *value == index as i32 {
                    *value = -1;
                }
            }
            self.grid[cell_index(room.connector)?] = index as i32;
            self.rooms[index].origin = room.connector;
            self.rooms[index].width = 1;
            self.rooms[index].height = 1;
            self.ends.remove(position);
            return Some(index);
        }
        None
    }
    fn restore_rejected(&mut self, index: Option<usize>) {
        if let Some(index) = index {
            self.ends.push(index);
        }
    }
    fn mask(&self, index: usize) -> u64 {
        let room = &self.rooms[index];
        slots(room)
            .into_iter()
            .filter(|(slot, _)| valid_slot(room, *slot))
            .fold(0, |mask, (slot, cell)| {
                if cell_index(cell)
                    .is_some_and(|cell| self.grid[cell] >= 0 && self.grid[cell] != index as i32)
                {
                    mask | (1 << slot)
                } else {
                    mask
                }
            })
    }
    fn secret(&mut self, state: &mut u32, assigned: &[Descriptor]) -> Option<usize> {
        let mut forbidden = [false; GRID_CELLS];
        for descriptor in assigned {
            let room = &self.rooms[descriptor.generated_index];
            for (slot, cell) in slots(room) {
                if !valid_slot(room, slot) {
                    continue;
                }
                if matches!(descriptor.template.room_type, 5 | 8)
                    || descriptor.template.door_mask & (1 << slot) == 0
                {
                    if let Some(cell) = cell_index(cell) {
                        forbidden[cell] = true;
                    }
                }
            }
        }
        let mut maximum = 0;
        let mut choices = Vec::new();
        for (cell, forbidden) in forbidden.iter().enumerate() {
            if self.grid[cell] >= 0 || *forbidden {
                continue;
            }
            *state = next35(*state);
            let mut score = 10 + *state % 5;
            let point = Cell {
                x: cell as i32 % GRID_WIDTH,
                y: cell as i32 / GRID_WIDTH,
            };
            let neighbors = [(-1, 0), (0, -1), (1, 0), (0, 1)]
                .into_iter()
                .filter(|(dx, dy)| {
                    cell_index(Cell {
                        x: point.x + dx,
                        y: point.y + dy,
                    })
                    .is_some_and(|cell| self.grid[cell] >= 0)
                })
                .count();
            if neighbors == 0 {
                continue;
            }
            score -= match neighbors {
                1 => 6,
                2 => 3,
                _ => 0,
            };
            if score > maximum {
                maximum = score;
                choices.clear();
                choices.push(cell);
            } else if score == maximum {
                choices.push(cell);
            }
        }
        if choices.is_empty() {
            return None;
        }
        *state = next35(*state);
        let cell = choices[*state as usize % choices.len()];
        let index = self.rooms.len();
        let origin = Cell {
            x: cell as i32 % GRID_WIDTH,
            y: cell as i32 / GRID_WIDTH,
        };
        self.rooms.push(GeneratedRoom {
            index,
            origin,
            width: 1,
            height: 1,
            connector: origin,
            direction: None,
            distance: 0,
        });
        self.grid[cell] = index as i32;
        Some(index)
    }
}

fn select(
    picker: &mut TemplatePicker,
    seed: u32,
    kind: u32,
    range: (u32, u32),
    shape: u8,
    doors: u64,
    maximum_difficulty: u8,
) -> Result<(usize, AuthoredTemplate), String> {
    let selected = picker.choose(SelectionRequest {
        seed,
        room_type: kind,
        shape,
        required_doors: doors,
        min_variant: range.0,
        max_variant: range.1,
        min_difficulty: 1,
        max_difficulty: maximum_difficulty,
    })?;
    Ok((
        selected.catalog_index,
        picker.catalog()[selected.catalog_index].clone(),
    ))
}
fn boss(primary: &mut Primary, flags: &mut u64) -> u32 {
    primary.next(); // Local RNG1 creation; fresh unavailable families never use it.
    if primary.random(4) == 0 {
        if primary.random(2) == 1 {
            *flags |= 0x100000;
            5020
        } else {
            *flags &= !0x100000;
            5140
        }
    } else {
        primary.random(2);
        primary.random(5);
        primary.random(2);
        primary.random(3);
        primary.random(10);
        primary.random(3);
        primary.random(4);
        if primary.random(5) == 0 {
            return 2010;
        }
        primary.random(2);
        if primary.random(4) == 0 {
            return 2050;
        }
        primary.random(2);
        primary.random(2);
        primary.random(2);
        if primary.random(2) == 1 {
            *flags |= 16;
            1010
        } else {
            *flags &= !16;
            1020
        }
    }
}
fn record(
    descriptors: &mut Vec<Descriptor>,
    fitting: &Fitting,
    index: usize,
    stage: u8,
    catalog_index: usize,
    template: AuthoredTemplate,
    seed: u32,
) {
    let decoration_seed = next12(seed);
    let spawn_seed = next12(decoration_seed);
    let following_seed = next12(spawn_seed);
    descriptors.push(Descriptor {
        index: descriptors.len(),
        generated_index: index,
        grid_index: cell_index(fitting.rooms[index].origin).expect("source room in grid"),
        asset_stage: stage,
        catalog_index,
        template,
        door_mask: 0,
        neighbors: [None; 8],
        room_init_seed: seed,
        decoration_seed,
        spawn_seed,
        following_seed,
    });
}
fn optional(
    fitting: &mut Fitting,
    descriptors: &mut Vec<Descriptor>,
    selected: (usize, AuthoredTemplate),
    accepted: bool,
    seed: u32,
) -> Option<usize> {
    let (catalog, template) = selected;
    let index = fitting.fit_small(template.door_mask, false);
    if accepted {
        if let Some(index) = index {
            record(descriptors, fitting, index, 0, catalog, template, seed);
            return Some(index);
        }
    }
    fitting.restore_rejected(index);
    None
}

struct Placed {
    descriptors: Vec<Descriptor>,
    grid: Vec<i32>,
}

fn attempt(
    primary: &mut Primary,
    generator_state: &mut u32,
    count: u8,
    special: &mut TemplatePicker,
    basement: &mut TemplatePicker,
    flags: &mut u64,
) -> Result<Option<Placed>, String> {
    let growth = layout::generate(GrowthRequest {
        generator_seed: *generator_state,
        additional_rooms: count,
        shape_mask: 15,
    })?;
    *generator_state = growth.final_rng_state;
    if growth.dead_end_indices.len() < 5 {
        return Ok(None);
    }
    let mut fitting = Fitting::new(growth);
    let mut descriptors = Vec::new();
    let minimum = boss(primary, flags);
    let boss_seed = primary.next();
    let selected = select(special, boss_seed, 5, (minimum, minimum + 9), 4, 0, 10)?;
    let Some(index) = fitting.fit_small(selected.1.door_mask, true) else {
        return Ok(None);
    };
    record(
        &mut descriptors,
        &fitting,
        index,
        0,
        selected.0,
        selected.1,
        boss_seed,
    );
    let super_seed = primary.next();
    let selected = select(special, super_seed, 8, (0, u32::MAX), 4, 0, 10)?;
    let Some(index) = fitting.fit_small(selected.1.door_mask, false) else {
        return Ok(None);
    };
    record(
        &mut descriptors,
        &fitting,
        index,
        0,
        selected.0,
        selected.1,
        super_seed,
    );
    // Seed is reserved before mandatory shop/treasure, allocation comes after
    // every optional placement and admission draw (589a00 vs 589a2a).
    let secret_seed = primary.next();
    for (kind, range) in [(2, (0, 1)), (4, (0, u32::MAX))] {
        let seed = primary.next();
        let selected = select(special, seed, kind, range, 4, 0, 10)?;
        let Some(index) = fitting.fit_small(selected.1.door_mask, false) else {
            return Ok(None);
        };
        record(
            &mut descriptors,
            &fitting,
            index,
            0,
            selected.0,
            selected.1,
            seed,
        );
    }
    let mut dice = primary.random(50) == 0;
    if !dice {
        primary.random(5);
        dice = false;
    } // Fresh Isaac field+bcc is0.
    let kind = if dice { 21 } else { 13 };
    let seed = primary.next();
    let selected = select(special, seed, kind, (0, u32::MAX), 4, 0, 10)?;
    let mut accepted = primary.random(7) == 0;
    if !accepted {
        accepted = primary.random(4) == 0;
    } //6<=6+0.
    optional(
        &mut fitting,
        &mut descriptors,
        selected,
        accepted,
        primary.state,
    );
    let library_variant = [0, 1, 3, 8][primary.random(4) as usize];
    let library_index = special
        .catalog()
        .iter()
        .position(|room| room.room_type == 12 && room.variant == library_variant)
        .ok_or("authored library variant missing")?;
    let library = (library_index, special.catalog()[library_index].clone());
    let mut accepted = primary.random(20) == 0;
    if !accepted {
        accepted = primary.random(4) == 0 && *flags & 0x100 != 0;
    }
    optional(
        &mut fitting,
        &mut descriptors,
        library,
        accepted,
        primary.state,
    );
    let seed = primary.next();
    let selected = select(special, seed, 10, (0, u32::MAX), 4, 0, 10)?;
    let mut accepted = primary.random(2) == 0;
    if !accepted {
        accepted = primary.random(4) == 0 && *flags & 64 != 0;
    }
    optional(
        &mut fitting,
        &mut descriptors,
        selected,
        accepted,
        primary.state,
    );
    let rare = primary.random(80) == 0;
    let available: Vec<u32> = [
        (2000, 0x200),
        (2010, 0x400),
        (2020, 0x800),
        (2030, 0x1000),
        (2050, 0x2000),
        (2060, 0x4000),
    ]
    .into_iter()
    .filter_map(|(variant, flag)| (*flags & flag == 0).then_some(variant))
    .collect();
    if available.is_empty() {
        return Err("fresh normal retry exhausted the source miniboss candidate set".into());
    }
    let minimum =
        available[primary.random(available.len() as u32) as usize] + if rare { 100 } else { 0 };
    let seed = primary.next();
    let selected = select(special, seed, 6, (minimum, minimum + 9), 4, 0, 10)?;
    let mut accepted = primary.random(4) == 0;
    if !accepted {
        accepted = primary.random(3) == 0;
    }
    if optional(
        &mut fitting,
        &mut descriptors,
        selected,
        accepted,
        primary.state,
    )
    .is_some()
    {
        *flags |= match minimum - if rare { 100 } else { 0 } {
            2000 => 0x200,
            2010 => 0x400,
            2020 => 0x800,
            2030 => 0x1000,
            2050 => 0x2000,
            2060 => 0x4000,
            _ => unreachable!(),
        };
    }
    let seed = primary.next();
    let selected = select(special, seed, 11, (0, u32::MAX), 4, 0, 10)?;
    primary.random(2); // Consumed before unconditional stage1 rejection.
    optional(
        &mut fitting,
        &mut descriptors,
        selected,
        false,
        primary.state,
    );
    let mut alternate_arcade = primary.random(10) == 0;
    if !alternate_arcade {
        primary.random(3);
        alternate_arcade = false;
    }
    let seed = primary.next();
    let selected = select(
        special,
        seed,
        if alternate_arcade { 20 } else { 9 },
        (0, u32::MAX),
        4,
        0,
        10,
    )?;
    optional(
        &mut fitting,
        &mut descriptors,
        selected,
        false,
        primary.state,
    );
    let kind = if primary.random(2) != 0 { 19 } else { 18 };
    let seed = primary.next();
    let selected = select(special, seed, kind, (0, u32::MAX), 4, 0, 10)?;
    let accepted = primary.random(50) == 0;
    if !accepted {
        primary.random(5);
    } // Fresh6HP fails low-health predicate.
    optional(
        &mut fitting,
        &mut descriptors,
        selected,
        accepted,
        primary.state,
    );
    let selected = select(special, secret_seed, 7, (0, u32::MAX), 4, 0, 10)?;
    if let Some(index) = fitting.secret(generator_state, &descriptors) {
        record(
            &mut descriptors,
            &fitting,
            index,
            0,
            selected.0,
            selected.1,
            secret_seed,
        );
    }
    let remaining: Vec<_> = fitting
        .original_non_ends
        .iter()
        .chain(&fitting.ends)
        .copied()
        .collect();
    for index in remaining {
        if index == 0 {
            let catalog = special
                .catalog()
                .iter()
                .position(|room| room.room_type == 1 && room.variant == 2)
                .ok_or("authored Start Room missing")?;
            record(
                &mut descriptors,
                &fitting,
                index,
                0,
                catalog,
                special.catalog()[catalog].clone(),
                primary.state,
            );
        } else {
            let room = &fitting.rooms[index];
            let shape = match (room.width, room.height) {
                (1, 1) => 0,
                (1, 2) => 1,
                (2, 1) => 2,
                (2, 2) => 3,
                _ => return Err("unrecovered generated shape".into()),
            };
            let selected = match select(
                basement,
                primary.next(),
                1,
                (0, u32::MAX),
                shape,
                fitting.mask(index),
                5,
            ) {
                Ok(selected) => selected,
                Err(_) => return Ok(None), // Source normal selector NULL aborts placement.
            };
            record(
                &mut descriptors,
                &fitting,
                index,
                1,
                selected.0,
                selected.1,
                primary.state,
            );
        }
    }
    let mut generated_to_descriptor = vec![None; fitting.rooms.len()];
    for descriptor in &descriptors {
        generated_to_descriptor[descriptor.generated_index] = Some(descriptor.index);
    }
    for descriptor in &mut descriptors {
        descriptor.door_mask = fitting.mask(descriptor.generated_index);
        let room = &fitting.rooms[descriptor.generated_index];
        for (slot, cell) in slots(room) {
            if valid_slot(room, slot) {
                descriptor.neighbors[slot as usize] = cell_index(cell)
                    .and_then(|cell| usize::try_from(fitting.grid[cell]).ok())
                    .and_then(|index| {
                        if index == room.index {
                            None
                        } else {
                            generated_to_descriptor[index]
                        }
                    });
            }
        }
    }
    let grid = fitting
        .grid
        .iter()
        .map(|index| {
            if *index < 0 {
                -1
            } else {
                generated_to_descriptor[*index as usize].expect("all source rooms assigned") as i32
            }
        })
        .collect();
    Ok(Some(Placed { descriptors, grid }))
}

pub fn generate_basement(stage_seed: u32) -> Result<Floor, String> {
    if stage_seed == 0 {
        return Err("nonzero stage seed required".into());
    }
    let special_assets = room_assets::special();
    let basement_assets = room_assets::basement();
    let mut special = TemplatePicker::new(
        special_assets
            .into_iter()
            .map(|asset| asset.template)
            .collect(),
    )?;
    let mut basement = TemplatePicker::new(
        basement_assets
            .into_iter()
            .map(|asset| asset.template)
            .collect(),
    )?;
    let mut primary = Primary {
        state: stage_seed,
        draws: 0,
    };
    primary.next(); // 587ec0 reset; fixed global MT draws are a separate stream.
    primary.random(1); // Backdrop palette.
    let count = 8 + primary.random(2) as u8;
    let mut generator_state = primary.next(); // Constructor, once per floor.
    let mut flags = 0;
    let mut reports = Vec::new();
    for attempt_number in 1..=100 {
        let initial_generator_state = generator_state;
        let initial_primary_state = primary.state;
        let result = attempt(
            &mut primary,
            &mut generator_state,
            count,
            &mut special,
            &mut basement,
            &mut flags,
        )?;
        reports.push(AttemptReport {
            attempt: attempt_number,
            initial_generator_state,
            final_generator_state: generator_state,
            initial_primary_state,
            final_primary_state: primary.state,
            outcome: if result.is_some() { "placed" } else { "retry" }.into(),
        });
        if let Some(Placed { descriptors, grid }) = result {
            primary.next(); // 58c337: post-placement level stream word.
            let starting_descriptor = grid[84] as usize;
            return Ok(Floor {
                contract: CONTRACT.into(),
                stage_seed,
                additional_rooms: count,
                attempts: attempt_number,
                descriptors,
                grid,
                starting_descriptor,
                primary_state: primary.state,
                primary_draws: primary.draws,
                generator_state,
                game_flags: flags,
                special_weights: special.current_weights().to_vec(),
                basement_weights: basement.current_weights().to_vec(),
                attempt_reports: reports,
            });
        }
    }
    Err("source normal Basement generation exceeded 100 attempts".into())
}
