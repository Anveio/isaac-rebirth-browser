//! Source-derived normal Basement I/II generation for the fresh profile.
//! Independent stage primary, generator and local selector streams are kept
//! separate. Authored catalogues supply every template and spawn selector.
use crate::challenges::CurseInputs;
use crate::difficulty::Difficulty;
use crate::layout::{self, Cell, GeneratedRoom, Growth, GrowthRequest, GRID_CELLS, GRID_WIDTH};
use crate::levels::BossStep;
use crate::room_assets;
use crate::room_templates::{AuthoredTemplate, SelectionRequest, TemplatePicker};
use serde::{Deserialize, Serialize};

pub const CONTRACT: &str = "fresh-profile-normal-isaac-basement-without-labyrinth;source-growth-fitting-secret-primary-routing-retry-continuity";

/// Actual Level inputs carried from the continuing run. The stage controller
/// must supply the retained stage seed and game flags, and health before the
/// new-stage Player callback. Alternate stages, challenges, Labyrinth and
/// equipped modifiers are outside this path.
#[derive(Clone, Copy, Debug)]
pub struct NormalBasementRequest {
    pub stage: u32,
    pub stage_seed: u32,
    pub game_flags: u64,
    pub red_hp: u8,
    pub max_red_hp: u8,
    pub soul_hp: i32,
    pub coins: u32,
    /// The run's Game difficulty; its rules shape Level::Init.
    pub difficulty: Difficulty,
    /// The curse sampler's inputs, drawn from only when a rule needs the
    /// curse stream (hard's extra rooms).
    pub curse: CurseInputs,
}

/// A keys-only view of the descriptor map. With an owned fresh map, cells
/// outside both sets are known absent. Unsupported factories invalidate only
/// their selected cell; they do not create an actual persistence key.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PersistentSpawnCells {
    pub present: Vec<u16>,
    pub unknown: Vec<u16>,
}

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
    /// Descriptor+D0/E8/EC Shop stock; legacy history remains unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shop: Option<crate::shop::Stock>,
    /// Keys of the actual descriptor+88 persistent-spawn map. Level reset
    /// 5882f0..588314 owns an empty map; legacy restores have no owner.
    /// This key view does not certify saved actor payloads or re-Init.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persistent_spawn_cells: Option<PersistentSpawnCells>,
    /// Descriptor+40 selects generated surprise-miniboss substitution. Source
    /// constructors clear it; the generator's tail (589feb..58a9d1) sets it on
    /// the secret room from stage 5 and on the shop from stage 4.
    pub surprise_miniboss: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Floor {
    pub contract: String,
    pub stage: u32,
    pub stage_seed: u32,
    /// The Game difficulty the floor was generated at.
    #[serde(default, skip_serializing_if = "is_normal")]
    pub difficulty: Difficulty,
    ///None keeps the existing ordinary floor's unimplemented curse owner
    ///explicit. Challenge uses the actual local sampler and filtered mask.
    pub challenge_curse: Option<crate::challenges::CurseSample>,
    /// Level+8, the floor's curse mask (58bfe8..58c058): 1 Darkness, 2
    /// Labyrinth, 4 Lost, 8 Unknown, 0x10 Cursed, 0x20 Maze, 0x40 Blind.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub curses: u32,
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
    /// Curse of the Labyrinth: the boss fit's runway check (58fb71).
    labyrinth: bool,
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
            labyrinth: growth.input.labyrinth,
        }
    }
    fn fit_small(&mut self, mask: u64, boss: bool) -> Option<usize> {
        self.fit(mask, (1, 1), boss)
            .expect("a 1x1 fit never reaches candidate5")
    }
    /// 58fad0/58f480: the first dead end (boss: farther than one step) that
    /// one of the candidates around its connector fits. A candidate must have
    /// the template's cell dims (8ed500) and its required door, cover only
    /// free cells or the dead end's own, and touch at most one other room
    /// (per slot, each slot's door in the template mask). The dead end takes
    /// the candidate's rectangle; a failed fit changes nothing.
    fn fit(&mut self, mask: u64, dims: (i32, i32), boss: bool) -> Result<Option<usize>, String> {
        for position in 0..self.ends.len() {
            let index = self.ends[position];
            if boss && self.rooms[index].distance <= 1 {
                continue;
            }
            // 58fb71: under Curse of the Labyrinth the boss dead end also
            // needs a free runway for the second boss room.
            if boss && self.labyrinth && !self.labyrinth_runway(index) {
                continue;
            }
            if self.fit_room(index, mask, dims)? {
                self.ends.remove(position);
                return Ok(Some(index));
            }
        }
        Ok(None)
    }
    /// 58fb89..58fc7a: the three cells past the dead end's connector, in its
    /// growth direction, lie on the grid, hold no other room and touch none.
    fn labyrinth_runway(&self, index: usize) -> bool {
        const STEPS: [(i32, i32); 4] = [(-1, 0), (0, -1), (1, 0), (0, 1)];
        let room = &self.rooms[index];
        let Some(direction) = room.direction else {
            return false;
        };
        let (dx, dy) = STEPS[usize::from(direction % 4)];
        let foreign = |cell: Cell| {
            cell_index(cell).is_some_and(|at| self.grid[at] >= 0 && self.grid[at] != index as i32)
        };
        (1..=3).all(|step| {
            let cell = Cell {
                x: room.connector.x + dx * step,
                y: room.connector.y + dy * step,
            };
            cell_index(cell).is_some()
                && !foreign(cell)
                && STEPS.iter().all(|(nx, ny)| {
                    !foreign(Cell {
                        x: cell.x + nx,
                        y: cell.y + ny,
                    })
                })
        })
    }
    /// 591d71: the second boss room joins the generator like a grown room.
    fn insert(&mut self, room: GeneratedRoom) -> usize {
        let index = self.rooms.len();
        for row in 0..room.height {
            for column in 0..room.width {
                if let Some(cell) = cell_index(Cell {
                    x: room.origin.x + column,
                    y: room.origin.y + row,
                }) {
                    self.grid[cell] = index as i32;
                }
            }
        }
        self.rooms.push(room);
        index
    }
    /// 58f480: one of the candidates around the room's connector fits. A
    /// candidate must have the template's cell dims (8ed500) and its
    /// required door, cover only free cells or the room's own, and touch at
    /// most one other room (per slot, each slot's door in the template
    /// mask). The room takes the candidate's rectangle; a failed fit changes
    /// nothing.
    fn fit_room(&mut self, index: usize, mask: u64, dims: (i32, i32)) -> Result<bool, String> {
        {
            let room = self.rooms[index].clone();
            let Some(direction) = room.direction else {
                return Ok(false);
            };
            let opposite = (direction + 2) % 4;
            let Cell { x, y } = room.connector;
            // Horizontal entries grow left (0) or right (2); vertical ones
            // up (1) or down (3), with the axes swapped.
            let (dx, dy) = match direction {
                0 => (-1, 0),
                1 => (0, -1),
                _ => (0, 0),
            };
            let (long, cross, step) = if matches!(direction, 0 | 2) {
                ((2, 1), (1, 2), (0, -1))
            } else {
                ((1, 2), (2, 1), (-1, 0))
            };
            let candidates = [
                (Cell { x, y }, (1, 1), Some(opposite)),
                (
                    Cell {
                        x: x + dx,
                        y: y + dy,
                    },
                    long,
                    Some(opposite),
                ),
                (Cell { x, y }, cross, Some(opposite + 4)),
                (
                    Cell {
                        x: x + step.0,
                        y: y + step.1,
                    },
                    cross,
                    Some(opposite),
                ),
                (
                    Cell {
                        x: x + dx,
                        y: y + dy,
                    },
                    (2, 2),
                    Some(opposite + 4),
                ),
                // 58f480 never sets candidate5's door slot (stale stack).
                (
                    Cell {
                        x: x + dx + step.0,
                        y: y + dy + step.1,
                    },
                    (2, 2),
                    None,
                ),
            ];
            for (origin, (width, height), slot) in candidates {
                if (width, height) != dims {
                    continue;
                }
                let Some(slot) = slot else {
                    return Err("2x2 boss fit candidate5 reads an unset door slot".into());
                };
                if mask & (1 << slot) == 0 {
                    continue;
                }
                let candidate = GeneratedRoom {
                    origin,
                    width,
                    height,
                    ..room.clone()
                };
                let free = (0..height).all(|row| {
                    (0..width).all(|column| {
                        cell_index(Cell {
                            x: origin.x + column,
                            y: origin.y + row,
                        })
                        .is_some_and(|cell| self.grid[cell] < 0 || self.grid[cell] == index as i32)
                    })
                });
                if !free {
                    continue;
                }
                let mut neighbors = 0;
                let mut valid = true;
                for (slot, cell) in slots(&candidate) {
                    if !valid_slot(&candidate, slot) {
                        continue;
                    }
                    if let Some(cell) = cell_index(cell) {
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
                // Rejection later re-adds this index without undoing the shape.
                for value in &mut self.grid {
                    if *value == index as i32 {
                        *value = -1;
                    }
                }
                for row in 0..height {
                    for column in 0..width {
                        let cell = cell_index(Cell {
                            x: origin.x + column,
                            y: origin.y + row,
                        })
                        .expect("checked in bounds");
                        self.grid[cell] = index as i32;
                    }
                }
                self.rooms[index] = candidate;
                return Ok(true);
            }
        }
        Ok(false)
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
fn is_zero(value: &u32) -> bool {
    *value == 0
}

/// Level+8 curse bits with a consumer in the port.
pub const LABYRINTH: u32 = 2;
pub const LOST: u32 = 4;
pub const UNKNOWN: u32 = 8;

impl Floor {
    /// Level+8190: the level's last boss room, the second one under Curse
    /// of the Labyrinth; the descriptor that takes the trapdoor (5c9788).
    pub fn last_boss_room(&self) -> Option<usize> {
        self.descriptors
            .iter()
            .rposition(|descriptor| descriptor.template.room_type == 5)
    }

    pub fn labyrinth(&self) -> bool {
        self.curses & LABYRINTH != 0
    }
}

fn is_normal(difficulty: &Difficulty) -> bool {
    *difficulty == Difficulty::Normal
}

/// 58c100..58c130: the normal-room difficulty range, the difficulty's row on
/// odd floors and 5..10 on even floors.
pub fn difficulty_band(stage: u32, difficulty: Difficulty) -> (u8, u8) {
    if stage % 2 == 1 {
        crate::difficulty::rules(difficulty).odd_stage_room_difficulty
    } else {
        (5, 10)
    }
}

/// Boss variant base 584240 for a fresh profile (every achievement byte zero,
/// no devil room visited), run from `stage`'s draw chain. The Level's own
/// stage (`level_stage`, Level+0 at 5842c7) decides whether the family draw
/// happens; Curse of the Labyrinth asks for the next stage's boss.
fn boss(
    primary: &mut Primary,
    flags: &mut u64,
    stage: u32,
    level_stage: u32,
) -> Result<u32, String> {
    let steps = crate::levels::bosses(stage)
        .ok_or("boss selection requires the stage's imported boss chain")?;
    let odd = stage % 2 == 1;
    // The chapter family (flag 0x100000) is drawn on odd Levels up to stage
    // 8 (Level+0 at 5842c7) and read elsewhere; the closing family (flag 16)
    // is drawn on odd `stage` (584b2f). The variant follows `stage` either
    // way (tables 675e80/675ec8, 584a05).
    let family = |flags: &mut u64, flag: u64, primary: &mut Primary| {
        let draws = if flag == 0x100000 {
            level_stage % 2 == 1 && level_stage <= 8
        } else {
            odd
        };
        if draws {
            let set = primary.random(2) == 1;
            if set {
                *flags |= flag;
            } else {
                *flags &= !flag;
            }
            set
        } else {
            *flags & flag != 0
        }
    };
    for step in steps {
        match *step {
            BossStep::Next => {
                primary.next();
            }
            BossStep::Draw(bound) => {
                primary.random(bound);
            }
            BossStep::Family {
                chance,
                flag,
                when_set,
                otherwise,
            } => {
                if chance.is_none_or(|chance| primary.random(chance) == 0) {
                    let set = family(flags, flag, primary);
                    return Ok(if odd == set { when_set } else { otherwise });
                }
            }
            BossStep::Single {
                chance,
                odd_only,
                boss,
            } => {
                if primary.random(chance) == 0 && (odd || !odd_only) {
                    return Ok(boss);
                }
            }
            BossStep::Fixed(boss) => return Ok(boss),
        }
    }
    Err("stage chapter's boss chain selected no boss".into())
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
        shop: Some(crate::shop::Stock::fresh()),
        persistent_spawn_cells: Some(PersistentSpawnCells {
            present: Vec::new(),
            unknown: Vec::new(),
        }),
        surprise_miniboss: false,
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

#[derive(Clone, Copy)]
struct AttemptParameters {
    request: NormalBasementRequest,
    count: u8,
    shape_mask: u8,
    /// Level+8 bit 2, Curse of the Labyrinth.
    labyrinth: bool,
}

/// 589854: boss variant bases 3500..3999 span 50 variants, 4000..4999 one,
/// others ten.
fn boss_variants(minimum: u32) -> (u32, u32) {
    let span = match minimum {
        3500..=3999 => 49,
        4000..=4999 => 0,
        _ => 9,
    };
    (minimum, minimum + span)
}

/// 591b80 fits a boss room by its shape's cell dims (8ed500); Double
/// Trouble's rooms are 26x7 or 13x14.
fn boss_dims(template: &AuthoredTemplate) -> Result<(i32, i32), String> {
    match template.shape_index() {
        0 => Ok((1, 1)),
        1 => Ok((1, 2)),
        2 => Ok((2, 1)),
        3 => Ok((2, 2)),
        _ => Err("boss template has no 8ed500 cell dims".into()),
    }
}

fn attempt(
    primary: &mut Primary,
    generator_state: &mut u32,
    special: &mut TemplatePicker,
    basement: &mut TemplatePicker,
    flags: &mut u64,
    parameters: AttemptParameters,
) -> Result<Option<Placed>, String> {
    let AttemptParameters {
        request,
        count,
        shape_mask,
        labyrinth,
    } = parameters;
    let growth = layout::generate(GrowthRequest {
        generator_seed: *generator_state,
        additional_rooms: count,
        shape_mask,
        labyrinth,
    })?;
    *generator_state = growth.final_rng_state;
    // 58c09f..58c0b6: five dead ends, six past stage 1, one more under
    // Curse of the Labyrinth.
    if growth.dead_end_indices.len() < 5 + usize::from(request.stage != 1) + usize::from(labyrinth)
    {
        return Ok(None);
    }
    let mut fitting = Fitting::new(growth);
    let mut descriptors = Vec::new();
    let minimum = boss(primary, flags, request.stage, request.stage)?;
    let boss_seed = primary.next();
    let selected = select(special, boss_seed, 5, boss_variants(minimum), 4, 0, 10)?;
    let dims = boss_dims(&selected.1)?;
    let Some(index) = fitting.fit(selected.1.door_mask, dims, true)? else {
        return Ok(None);
    };
    // 591cc0: under Curse of the Labyrinth the first boss room grows the
    // second boss room's cell from the generator's own RNG.
    let second_boss = if labyrinth {
        layout::labyrinth_boss_neighbor(
            &fitting.rooms,
            &fitting.grid,
            shape_mask,
            generator_state,
            index,
            selected.1.door_mask,
        )
        .map(|room| fitting.insert(room))
    } else {
        None
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
    if labyrinth {
        // 589d5c..589e32: the next stage's boss takes the second boss room
        // (the level's last, Level+8190); the primary stream is then reset
        // to its state before that boss's selection.
        let saved = primary.state;
        let minimum = boss(primary, flags, request.stage + 1, request.stage)?;
        let seed = primary.next();
        let selected = select(special, seed, 5, boss_variants(minimum), 4, 0, 10)?;
        let dims = boss_dims(&selected.1)?;
        let Some(index) = second_boss else {
            return Ok(None);
        };
        if !fitting.fit_room(index, selected.1.door_mask, dims)? {
            return Ok(None);
        }
        record(
            &mut descriptors,
            &fitting,
            index,
            0,
            selected.0,
            selected.1,
            seed,
        );
        primary.state = saved;
    }
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
    let mut shop = None;
    for (kind, range) in [(2, (0, 1)), (4, (0, u32::MAX))] {
        let seed = primary.next();
        let selected = select(special, seed, kind, range, 4, 0, 10)?;
        let Some(index) = fitting.fit_small(selected.1.door_mask, false) else {
            return Ok(None);
        };
        if kind == 2 {
            shop = Some(descriptors.len());
        }
        record(
            &mut descriptors,
            &fitting,
            index,
            0,
            selected.0,
            selected.1,
            seed,
        );
        if kind == 4 && labyrinth {
            // 589f46 -> 58bae3: a second treasure room, its seed drawn from
            // the primary stream, which is then reset.
            let saved = primary.state;
            let seed = primary.next();
            let selected = select(special, seed, 4, (0, u32::MAX), 4, 0, 10)?;
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
            primary.state = saved;
        }
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
        accepted = primary.random(4) == 0
            && i32::from(request.max_red_hp) <= i32::from(request.red_hp) + request.soul_hp;
    }
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
    // 58a8b3: from stage 3, 58ae1a's R10 may offer Ultra Pride 2260 once
    // per run (flag 0x1_0000_0000), discarding one word before its seed.
    let ultra_pride = request.stage > 2 && primary.random(10) == 0 && *flags & 0x1_0000_0000 == 0;
    let (minimum, maximum, rare) = if ultra_pride {
        primary.next();
        (2260, 2269, false)
    } else {
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
            return Err("normal Basement retry exhausted the source miniboss candidate set".into());
        }
        let minimum =
            available[primary.random(available.len() as u32) as usize] + if rare { 100 } else { 0 };
        (minimum, minimum + 9, rare)
    };
    let seed = primary.next();
    let selected = select(special, seed, 6, (minimum, maximum), 4, 0, 10)?;
    let mut accepted = primary.random(4) == 0;
    if !accepted {
        accepted = primary.random(3) == 0 && request.stage == 1;
    }
    let miniboss_placed = optional(
        &mut fitting,
        &mut descriptors,
        selected,
        accepted,
        primary.state,
    )
    .is_some();
    if miniboss_placed {
        *flags |= match minimum - if rare { 100 } else { 0 } {
            2000 => 0x200,
            2010 => 0x400,
            2020 => 0x800,
            2030 => 0x1000,
            2050 => 0x2000,
            2060 => 0x4000,
            2260 => 0x1_0000_0000,
            _ => unreachable!(),
        };
    }
    let seed = primary.next();
    let selected = select(special, seed, 11, (0, u32::MAX), 4, 0, 10)?;
    // 58b28e..58b818: R2 is always drawn; from stage 3 only full health counts.
    let roll = primary.random(2);
    let accepted = (roll == 0 || request.stage > 2)
        && request.stage > 1
        && i32::from(request.max_red_hp) <= i32::from(request.red_hp) + request.soul_hp;
    optional(
        &mut fitting,
        &mut descriptors,
        selected,
        accepted,
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
        // 58b3ac: even floors of the first four chapters with five coins.
        matches!(request.stage, 2 | 4 | 6 | 8) && request.coins > 4,
        primary.state,
    );
    let kind = if primary.random(2) != 0 { 19 } else { 18 };
    let seed = primary.next();
    let selected = select(special, seed, kind, (0, u32::MAX), 4, 0, 10)?;
    let mut accepted = primary.random(50) == 0;
    if !accepted {
        accepted = primary.random(5) == 0
            && ((request.soul_hp <= 0 && request.red_hp <= 1)
                || (request.soul_hp <= 2 && request.max_red_hp == 0));
    }
    optional(
        &mut fitting,
        &mut descriptors,
        selected,
        accepted,
        primary.state,
    );
    let selected = select(special, secret_seed, 7, (0, u32::MAX), 4, 0, 10)?;
    let secret = fitting.secret(generator_state, &descriptors).map(|index| {
        record(
            &mut descriptors,
            &fitting,
            index,
            0,
            selected.0,
            selected.1,
            secret_seed,
        );
        descriptors.len() - 1
    });
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
            let selected = match basement.choose(SelectionRequest {
                seed: primary.next(),
                room_type: 1,
                shape,
                required_doors: fitting.mask(index),
                min_variant: 0,
                max_variant: u32::MAX,
                min_difficulty: difficulty_band(request.stage, request.difficulty).0,
                max_difficulty: difficulty_band(request.stage, request.difficulty).1,
            }) {
                Ok(selected) => (
                    selected.catalog_index,
                    basement.catalog()[selected.catalog_index].clone(),
                ),
                Err(_) => return Ok(None), // Source normal selector NULL aborts placement.
            };
            record(
                &mut descriptors,
                &fitting,
                index,
                crate::levels::chapter_id(request.stage, false)
                    .ok_or("normal rooms require the stage's room set")? as u8,
                selected.0,
                selected.1,
                primary.state,
            );
        }
    }
    surprise_miniboss(
        primary,
        flags,
        &request,
        miniboss_placed,
        secret,
        shop,
        &mut descriptors,
    );
    // Level AddRoom 584bc0 writes the descriptor's index into every grid
    // cell its RoomData covers (width / 13 by height / 7 cells from the
    // generated origin), in descriptor order, so a later room overwrites an
    // earlier footprint. Normal placement selects templates of the generated
    // shape; a template larger than its generated room (an authored Start
    // Room) keeps only the cells no later descriptor takes.
    let footprint = |descriptor: &Descriptor| {
        let mut room = fitting.rooms[descriptor.generated_index].clone();
        room.width = i32::from(descriptor.template.width) / 13;
        room.height = i32::from(descriptor.template.height) / 7;
        room
    };
    let mut grid = vec![-1; GRID_CELLS];
    for descriptor in &descriptors {
        let room = footprint(descriptor);
        for row in 0..room.height {
            for column in 0..room.width {
                if let Some(cell) = cell_index(Cell {
                    x: room.origin.x + column,
                    y: room.origin.y + row,
                }) {
                    grid[cell] = descriptor.index as i32;
                }
            }
        }
    }
    // Level 587210 then walks the descriptors in order. Descriptor+18[slot]
    // is the cell beyond each slot of the RoomData footprint; the door bit
    // the generator gave (Descriptor+10) survives only when that cell holds
    // a room whose own mask still has the door facing back (587190, the
    // facing slot from 587070). A secret room (type 7) first opens a door
    // its RoomData allows on both sides. A Start Room larger than its
    // generated cell therefore keeps only the generated directions, each
    // now leading to the cell beyond its footprint.
    let beyond: Vec<[Option<usize>; 8]> = descriptors
        .iter()
        .map(|descriptor| {
            let mut cells = [None; 8];
            for (slot, cell) in slots(&footprint(descriptor)) {
                cells[slot as usize] = cell_index(cell);
            }
            cells
        })
        .collect();
    for descriptor in &mut descriptors {
        descriptor.door_mask = fitting.mask(descriptor.generated_index);
    }
    for index in 0..descriptors.len() {
        for slot in 0..8u8 {
            let bit = 1u64 << slot;
            let Some((cell, target)) = beyond[index][slot as usize]
                .and_then(|cell| Some((cell, usize::try_from(grid[cell]).ok()?)))
            else {
                descriptors[index].door_mask &= !bit;
                continue;
            };
            let back = crate::maze::entry_slot(&descriptors[target], cell, (slot + 2) & 3);
            if descriptors[index].template.room_type == 7
                && descriptors[index].template.door_mask & bit != 0
                && descriptors[target].template.door_mask & (1 << back) != 0
            {
                descriptors[index].door_mask |= bit;
                descriptors[target].door_mask |= 1 << back;
            }
            if descriptors[target].door_mask & (1 << back) == 0 {
                descriptors[index].door_mask &= !bit;
            }
        }
    }
    for (descriptor, cells) in descriptors.iter_mut().zip(&beyond) {
        for slot in 0..8 {
            descriptor.neighbors[slot] = cells[slot]
                .filter(|_| descriptor.door_mask & (1 << slot) != 0)
                .and_then(|cell| usize::try_from(grid[cell]).ok());
        }
    }
    Ok(Some(Placed { descriptors, grid }))
}

/// 589feb..58a9d1: after the normal rooms every attempt draws two
/// RandomFloats (5c2db0) against the surprise-miniboss chance. The first may
/// flag the secret room from stage 5 and the second the shop from stage 4;
/// both need that no miniboss room was placed this attempt (589ff7 also
/// clears it when flags 0x18000 are both set). Game+0x10eafc only counts
/// payouts the port does not admit (552650), so its penalty stays 0.
/// Flagging the secret room (58a0c0..58a11b) reopens the shop's chance only
/// when a local RandomInt(1000), seeded from the primary state with shift
/// 0xc, returns 0; that local RNG does not advance the primary.
/// PORT GAP: the five off-grid descriptors after it (58a137..58a433) and
/// their primary words are not generated.
fn surprise_miniboss(
    primary: &mut Primary,
    flags: &mut u64,
    request: &NormalBasementRequest,
    miniboss_placed: bool,
    secret: Option<usize>,
    shop: Option<usize>,
    descriptors: &mut [Descriptor],
) {
    let mut open = !miniboss_placed && *flags & 0x18000 != 0x18000;
    let mut base = if request.coins > 19 { 1.05_f32 } else { 1.0 };
    if *flags & 0x4000_0000 != 0 {
        base += 0.1;
        *flags &= !0x4000_0000;
    }
    if *flags & 0x8000_0000 != 0 {
        base += 0.02;
        *flags &= !0x8000_0000;
    }
    let chance = base * 0.333_333_34_f32;
    let random_float = |primary: &mut Primary| primary.next() as f32 * 2.328_306_4e-10_f32;
    if chance > random_float(primary) && open && request.stage > 4 {
        if let Some(secret) = secret {
            descriptors[secret].surprise_miniboss = true;
            open = next12(primary.state) % 1000 == 0;
        }
    }
    if chance > random_float(primary) && open && request.stage > 3 {
        if let Some(shop) = shop {
            descriptors[shop].surprise_miniboss = true;
        }
    }
}

pub fn generate_basement(stage_seed: u32) -> Result<Floor, String> {
    generate_normal_basement(first_floor(
        stage_seed,
        Difficulty::Normal,
        CurseInputs::default(),
    ))
}

/// Basement I's request for a fresh character.
pub fn first_floor(
    stage_seed: u32,
    difficulty: Difficulty,
    curse: CurseInputs,
) -> NormalBasementRequest {
    NormalBasementRequest {
        stage: 1,
        stage_seed,
        game_flags: 0,
        red_hp: 6,
        max_red_hp: 6,
        soul_hp: 0,
        coins: 0,
        difficulty,
        curse,
    }
}

pub fn generate_normal_basement(request: NormalBasementRequest) -> Result<Floor, String> {
    generate_normal_basement_with_special(request, None)
}

pub(crate) fn generate_normal_basement_with_special(
    request: NormalBasementRequest,
    special: Option<&[room_assets::RoomAsset]>,
) -> Result<Floor, String> {
    generate_normal_basement_inner(request, None, special)
}

pub(crate) fn generate_basement_with_special(
    stage_seed: u32,
    difficulty: Difficulty,
    curse: CurseInputs,
    special: Option<&[room_assets::RoomAsset]>,
) -> Result<Floor, String> {
    generate_normal_basement_with_special(first_floor(stage_seed, difficulty, curse), special)
}

pub(crate) fn generate_challenge_basement(
    stage_seed: u32,
    definition: &crate::challenges::Definition,
    profile: &crate::Profile,
    has_black_candle: bool,
    special: Option<&[room_assets::RoomAsset]>,
) -> Result<Floor, String> {
    definition.validate_initial_scope()?;
    generate_normal_basement_inner(
        // 59d218: Challenges run at normal difficulty; their own sampler
        // below owns the curse stream.
        first_floor(stage_seed, Difficulty::Normal, CurseInputs::default()),
        Some((definition, profile, has_black_candle)),
        special,
    )
}

fn generate_normal_basement_inner(
    request: NormalBasementRequest,
    challenge: Option<(&crate::challenges::Definition, &crate::Profile, bool)>,
    special: Option<&[room_assets::RoomAsset]>,
) -> Result<Floor, String> {
    let stage_seed = request.stage_seed;
    let Some(node) = crate::levels::node(request.stage) else {
        return Err("normal generation admits only the level framework's stage nodes".into());
    };
    if stage_seed == 0 {
        return Err("nonzero stage seed required".into());
    }
    let special_assets =
        special.map_or_else(room_assets::special, <[room_assets::RoomAsset]>::to_vec);
    // 5d49b0: room set stage - even + (type != 0); stage type 0 only.
    let basement_assets = crate::levels::stage_chapter(request.stage)
        .and_then(|chapter| chapter.rooms)
        .ok_or("stage chapter has no imported room set")?();
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
    // 58c18d queries actual normal templates in the floor's difficulty band;
    // 58c20f admits a rectangular shape only when its count exceeds four.
    let mut shape_counts = [0u32; 4];
    let (minimum_difficulty, maximum_difficulty) =
        difficulty_band(request.stage, request.difficulty);
    for template in basement.catalog() {
        if template.room_type == 1
            && (minimum_difficulty..=maximum_difficulty).contains(&template.difficulty)
            && template.shape_index() < 4
        {
            shape_counts[usize::from(template.shape_index())] += 1;
        }
    }
    let shape_mask = shape_counts
        .iter()
        .enumerate()
        .fold(0u8, |mask, (shape, count)| {
            mask | if *count > 4 { 1 << shape } else { 0 }
        });
    let mut primary = Primary {
        state: stage_seed,
        draws: 0,
    };
    primary.next(); // 587ec0 reset; fixed global MT draws are a separate stream.
    primary.random(1); // Backdrop palette.
    let count = (request.stage * 10 / 3 + 5 + primary.random(2)).min(20);
    let challenge_curse = challenge
        .map(|(definition, profile, has_black_candle)| {
            crate::challenges::sample_initial_curse(
                primary.state,
                request.stage as i32,
                definition,
                profile,
                crate::challenges::RunParameters::CHALLENGE_MENU,
                has_black_candle,
            )
        })
        .transpose()?;
    if challenge_curse.is_some_and(|sample| sample.filtered_mask != 0) {
        return Err("Challenge floor requires unimplemented nonzero curse consumers".into());
    }
    // 58bfe8..58c058: an ordinary floor rolls its curse on a copy of the
    // primary stream at both difficulties; the odds are the difficulty's
    // `curse_odds` row.
    let sample = challenge_curse.unwrap_or_else(|| {
        crate::challenges::sample_curse(
            primary.state,
            request.stage as i32,
            request.difficulty,
            request.curse,
            |mask| mask,
        )
    });
    let curses = sample.filtered_mask;
    let labyrinth = curses & LABYRINTH != 0;
    // 58c061: Curse of the Labyrinth scales the count by f64 1.8 (676188),
    // at most 45; otherwise Curse of the Lost (58c4c0) adds four rooms.
    let count = if labyrinth {
        ((f64::from(count) * 1.8) as u32).min(45)
    } else if curses & LOST != 0 {
        count + 4
    } else {
        count
    };
    // 58c08e: hard's extra rooms take the curse stream's last RandomInt(2).
    let count = count
        + crate::difficulty::rules(request.difficulty)
            .extra_rooms
            .map_or(0, |rooms| rooms + sample.room_adjustment_result);
    let count = u8::try_from(count).map_err(|_| "floor room count exceeds the generator")?;
    let mut generator_state = primary.next(); // Constructor, once per floor.
    let mut flags = request.game_flags;
    let mut reports = Vec::new();
    for attempt_number in 1..=100 {
        let initial_generator_state = generator_state;
        let initial_primary_state = primary.state;
        let result = attempt(
            &mut primary,
            &mut generator_state,
            &mut special,
            &mut basement,
            &mut flags,
            AttemptParameters {
                request,
                count,
                shape_mask,
                labyrinth,
            },
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
                contract: if challenge.is_some() {
                    "source-authored-challenge1-neutral-filter-fresh-isaac-basement".into()
                } else {
                    node.contract.into()
                },
                stage: request.stage,
                stage_seed,
                difficulty: request.difficulty,
                challenge_curse,
                curses,
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
