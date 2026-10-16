//! Native-world grid queries, regular blast selection and player penetration.
//!
//! Room descriptors include the boundary ring. These kernels reconstruct the
//! source predicates given resolved grid objects; they do not resolve authored
//! spawn selectors, implement grid drops or infer a floor graph. Source evidence
//! and independently captured fixtures live in docs/combat-evidence.md.
use crate::motion::WorldVector;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The native pathfinder reserves 448 room-cell entries (28 × 16).
pub const MAX_GRID_CELLS: usize = 448;
const MAX_COLLISION_POINTS: u16 = 128;
// A resource bound for diagnostic calls, not a recovered gameplay radius tier.
const MAX_QUERY_RADIUS: f32 = 4096.0;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GridCell {
    pub selector: u32,
    pub entity: Option<GridEntity>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GridEntity {
    pub kind: u32,
    pub variant: u32,
    pub state: u32,
    pub collision_class: u32,
    pub spawn_seed: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_owner: Option<crate::grid_records::GridOwner>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GridDescriptor {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<GridCell>,
}

#[derive(Clone, Copy, Debug)]
pub struct DoorCollision {
    pub slot: u8,
    pub open: bool,
    pub hidden: bool,
    /// Dice-room doorways are closed with the double-lock variant2.
    pub double_lock: bool,
    /// SetRoomTypes565b00 closes a doorway into a bedroom (18/19) with the
    /// cracked variant3, which only bombs damage and break.
    pub cracked: bool,
    /// SetRoomTypes565b00: an uncleared miniboss (type6 or surprise flag)
    /// room bars its ordinary non-secret doorways with variant4.
    pub barred: bool,
}

/// Admit supported authored rocks and spikes with their actual local spawn seeds. The
/// boundary/door entries are collision profiles; their independent actor RNG
/// and callbacks are not represented by those geometry-only spawn seeds.
/// Unsupported selectors stay verbatim and use the source fallback query.
pub fn for_room(
    asset: &crate::room_assets::RoomAsset,
    spawns: &crate::room_spawns::RoomSpawns,
    decoration_seed: u32,
    doors: &[DoorCollision],
) -> Result<GridDescriptor, String> {
    let width = u16::from(asset.template.width) + 2;
    let height = u16::from(asset.template.height) + 2;
    let count = usize::from(width) * usize::from(height);
    if width < 3 || height < 3 || count > MAX_GRID_CELLS {
        return Err("authored room grid exceeds native cell bounds".into());
    }
    let mut cells = vec![GridCell::default(); count];
    for (index, cell) in cells.iter_mut().enumerate() {
        let column = index % usize::from(width);
        let row = index / usize::from(width);
        if column == 0
            || column + 1 == usize::from(width)
            || row == 0
            || row + 1 == usize::from(height)
        {
            cell.entity = Some(GridEntity {
                kind: 15,
                variant: 0,
                state: 0,
                collision_class: 4,
                spawn_seed: 0,
                source_owner: None,
            });
        }
    }
    for door in doors {
        let authored = asset
            .doors
            .iter()
            .find(|candidate| {
                candidate.enabled
                    && crate::room_assets::door_slot(candidate.x, candidate.y) == Ok(door.slot)
            })
            .ok_or("admitted door has no authored boundary position")?;
        let column = usize::try_from(authored.x + 1).map_err(|_| "negative door column")?;
        let row = usize::try_from(authored.y + 1).map_err(|_| "negative door row")?;
        if column >= usize::from(width) || row >= usize::from(height) {
            return Err("authored door outside boundary grid".into());
        }
        cells[column + row * usize::from(width)].entity = Some(GridEntity {
            kind: 16,
            // Fresh stage1 ordinary/curse/treasure doors use variant7 when
            // opened, or barred variant4 from an uncleared miniboss room; the
            // admitted key-closed target2/12 branch uses variant1.
            variant: if door.hidden {
                6
            } else if door.open {
                if door.barred {
                    4
                } else {
                    7
                }
            } else if door.double_lock {
                2
            } else if door.cracked {
                3
            } else {
                1
            },
            state: if door.open { 2 } else { 1 },
            collision_class: if door.open { 5 } else { 4 },
            spawn_seed: 0,
            source_owner: None,
        });
    }
    for spawn in &spawns.selectors {
        if spawn.category != crate::room_spawns::SpawnCategory::Grid {
            continue;
        }
        let index = usize::from(spawn.grid_index);
        let cell = cells
            .get_mut(index)
            .ok_or("authored grid selector outside room")?;
        cell.selector = u32::from(spawn.entity_type);
        if spawn.entity_type == 1930 && spawn.variant == 0 && spawn.subtype == 0 {
            cell.entity = Some(GridEntity {
                kind: 8,
                // Spike graphics configuration 0x56f830 chooses 1..4 once from
                // the actual grid spawn seed; active state0 has no blocker.
                variant: crate::room_spawns::next35(spawn.spawn_seed) % 4 + 1,
                state: 0,
                collision_class: 0,
                spawn_seed: spawn.spawn_seed,
                source_owner: None,
            });
        }
        if spawn.variant == 0 && spawn.subtype == 0 {
            let rock_frame =
                crate::room_spawns::next35(decoration_seed.wrapping_add(index as u32) | 1) % 3;
            let admitted = match spawn.entity_type {
                // Rock-family Init56ed00 (state0 -> intact state1, frame drawn
                // like an ordinary rock): Block1900 is kind3, the room's
                // tinted rock kind4. Tinted drops remain a separate callback.
                1900 => Some((3, rock_frame, 1, 3)),
                // Factory5ce858: raw1002 is the alt rock kind6 directly (no
                // Random1001 classification), with the ordinary rock frame.
                1002 => Some((6, rock_frame, 1, 3)),
                // Raw1940 spider web kind10 (Init571c70): class0 and no path
                // cost. Its frame/flip/rotation draws belong to the Room.Init
                // construction pass; `spider_webs` owns its player slowing.
                1940 => Some((10, 0, 0, 0)),
                1000 if spawn.grid_index == spawns.tinted_rock_index => Some((4, rock_frame, 1, 3)),
                // Pit3000 Init568990 collides as class1; its autotile frame is
                // linked after admission by `link_pits`.
                3000 => Some((7, 0, 0, 1)),
                // Factory5ce6b0: raw1001 is the bomb rock kind5 directly.
                1001 => Some((5, rock_frame, 1, 3)),
                // Raw1300 TNT kind12: Init leaves class0 (the first Update
                // writes class2 while intact).
                1300 => Some((12, 0, 0, 0)),
                // Raw1500 poop: its factory's local RNG35 Int(40) keeps the
                // ordinary variant0 unless it draws 0 (corn, golden and
                // rainbow rolls are not admitted).
                1500 if !crate::room_spawns::next35(spawn.spawn_seed).is_multiple_of(40) => {
                    Some((14, 0, 0, 3))
                }
                _ => None,
            };
            if let Some((kind, variant, state, collision_class)) = admitted {
                cell.entity = Some(GridEntity {
                    kind,
                    variant,
                    state,
                    collision_class,
                    spawn_seed: spawn.spawn_seed,
                    source_owner: None,
                });
            }
        }
        // Tinted Rock admission/drops are still a separate callback. Source
        // factory5ceba5 nevertheless classifies other raw1000 rocks before
        // Init can link ordinary neighbors: a new RNG35 Random1001 chooses
        // explosive kind5 on0..9, alternate kind6 on10..15, otherwise kind2.
        if spawn.entity_type == 1000
            && spawn.variant == 0
            && spawn.subtype == 0
            && spawn.grid_index != spawns.tinted_rock_index
        {
            cell.entity = Some(GridEntity {
                kind: match crate::room_spawns::next35(spawn.spawn_seed) % 1001 {
                    0..=9 => 5,
                    10..=15 => 6,
                    _ => 2,
                },
                // Rock normal has three authored frames. Native SetType(-1)
                // seeds RNG35 with (decoration seed + grid index) | 1.
                variant: crate::room_spawns::next35(decoration_seed.wrapping_add(index as u32) | 1)
                    % 3,
                state: 1,
                collision_class: 3,
                spawn_seed: spawn.spawn_seed,
                source_owner: None,
            });
        }
    }
    // Rock Init runs after admission with the complete grid available. It
    // visits indices in order and skips rocks already linked by an earlier one.
    for index in 0..count {
        let Some(rock) = cells[index].entity.as_ref() else {
            continue;
        };
        if rock.kind != 2 || rock.variant >= 1000 {
            continue;
        }
        let mut random = crate::room_spawns::next35(rock.spawn_seed);
        if random.is_multiple_of(4) {
            continue;
        }
        let candidates = [
            index + 1,
            index + usize::from(width),
            index + usize::from(width) + 1,
        ];
        let mut mask = 0_u32;
        for (bit, neighbor) in candidates.iter().enumerate() {
            if cells
                .get(*neighbor)
                .and_then(|cell| cell.entity.as_ref())
                .is_some_and(|entity| entity.kind == 2 && entity.variant < 1000)
            {
                mask |= 1 << bit;
            }
        }
        if mask == 7 {
            random = crate::room_spawns::next35(random);
            if !random.is_multiple_of(4) {
                random = crate::room_spawns::next35(random);
                mask -= random % 7;
            }
        }
        let links: &[(usize, u32)] = match mask {
            1 | 5 => &[(index, 1000), (candidates[0], 1001)],
            2 | 3 | 6 => &[(index, 1002), (candidates[1], 1003)],
            7 => &[
                (index, 1004),
                (candidates[0], 1005),
                (candidates[1], 1006),
                (candidates[2], 1007),
            ],
            _ => &[],
        };
        for &(at, variant) in links {
            cells[at]
                .entity
                .as_mut()
                .expect("selected ordinary rock exists")
                .variant = variant;
        }
    }
    Ok(GridDescriptor {
        width,
        height,
        cells,
    })
}

/// GridEntity_Pit slot2 5684c0, run by Room.Init 5cf550 over every grid
/// entity in index order: the pit frame from its orthogonal pit neighbours
/// (L1|U2|R4|D8) and, for corners, the diagonals. An isolated pit draws its
/// frame from the shared MT (Random(2) << 4).
pub fn link_pits(grid: &mut GridDescriptor, shared: &mut crate::mt::Mt19937) {
    let width = i64::from(grid.width);
    let height = i64::from(grid.height);
    let pit = |cells: &[GridCell], column: i64, row: i64| {
        column >= 0
            && row >= 0
            && column < width
            && row < height
            && cells[(column + row * width) as usize]
                .entity
                .as_ref()
                .is_some_and(|entity| entity.kind == 7)
    };
    for index in 0..grid.cells.len() {
        if !pit(&grid.cells, index as i64 % width, index as i64 / width) {
            continue;
        }
        let (column, row) = (index as i64 % width, index as i64 / width);
        let at = |dx: i64, dy: i64| pit(&grid.cells, column + dx, row + dy);
        let (up_left, up_right, down_left, down_right) =
            (at(-1, -1), at(1, -1), at(-1, 1), at(1, 1));
        let mask = u32::from(at(-1, 0))
            | u32::from(at(0, -1)) << 1
            | u32::from(at(1, 0)) << 2
            | u32::from(at(0, 1)) << 3;
        let frame = match mask {
            0 => shared.bounded(2) << 4,
            3 => {
                if up_left {
                    3
                } else {
                    17
                }
            }
            6 => {
                if up_right {
                    6
                } else {
                    18
                }
            }
            9 => {
                if down_left {
                    9
                } else {
                    19
                }
            }
            12 => {
                if down_right {
                    12
                } else {
                    20
                }
            }
            11 => {
                if up_left {
                    11
                } else {
                    26
                }
            }
            14 => {
                if up_right {
                    14
                } else {
                    25
                }
            }
            7 => match (up_left, up_right) {
                (true, true) => 7,
                (true, false) => 27,
                (false, true) => 28,
                (false, false) => 29,
            },
            13 => match (down_left, down_right) {
                (true, true) => 13,
                (true, false) => 30,
                (false, true) => 31,
                (false, false) => 32,
            },
            15 => match (up_left, up_right) {
                (true, true) => {
                    if down_right || down_left {
                        15
                    } else {
                        24
                    }
                }
                (false, true) => 21,
                (true, false) => 22,
                (false, false) => 23,
            },
            mask => mask,
        };
        grid.cells[index]
            .entity
            .as_mut()
            .expect("checked pit")
            .variant = frame;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid {
    descriptor: GridDescriptor,
}

impl Grid {
    /// Collision geometry of the ordinary 1x1 room's boundary ring. This
    /// profile has no admitted grid actors or random spawn seeds; floor/room
    /// admission supplies those separately, including live door apertures.
    pub fn closed_1x1_boundary() -> Self {
        let width = 15;
        let height = 9;
        let cells = (0..width * height)
            .map(|index| GridCell {
                selector: 0,
                entity: (index < width
                    || index >= width * (height - 1)
                    || index % width == 0
                    || index % width == width - 1)
                    .then_some(GridEntity {
                        kind: 15,
                        variant: 0,
                        state: 0,
                        collision_class: 4,
                        spawn_seed: 0,
                        source_owner: None,
                    }),
            })
            .collect();
        Self {
            descriptor: GridDescriptor {
                width,
                height,
                cells,
            },
        }
    }

    pub fn new(descriptor: GridDescriptor) -> Result<Self, String> {
        let count = usize::from(descriptor.width) * usize::from(descriptor.height);
        if descriptor.width == 0
            || descriptor.height == 0
            || count > MAX_GRID_CELLS
            || descriptor.cells.len() != count
        {
            return Err("grid dimensions require exactly 1..448 cells".into());
        }
        Ok(Self { descriptor })
    }

    pub fn descriptor(&self) -> &GridDescriptor {
        &self.descriptor
    }

    /// Room::GetGridIndex uses floorf on the original 40-unit lattice.
    pub fn index(&self, point: WorldVector) -> Option<usize> {
        if !point.is_finite() {
            return None;
        }
        let column = ((point.x - 40.0) / 40.0 + 0.5).floor();
        let row = ((point.y - 120.0) / 40.0 + 0.5).floor();
        if column < 0.0
            || row < 0.0
            || column >= f32::from(self.descriptor.width)
            || row >= f32::from(self.descriptor.height)
        {
            return None;
        }
        Some(column as usize + row as usize * usize::from(self.descriptor.width))
    }

    pub fn position(&self, index: usize) -> Option<WorldVector> {
        if index >= self.descriptor.cells.len() {
            return None;
        }
        let width = usize::from(self.descriptor.width);
        Some(WorldVector {
            x: 40.0 + (index % width) as f32 * 40.0,
            y: 120.0 + (index / width) as f32 * 40.0,
        })
    }

    pub fn collision_at(&self, point: WorldVector) -> u32 {
        self.index(point).map_or(4, |index| self.collision(index))
    }

    /// Regular Tear has native grid class4, queried at actual ground after its
    /// full Update (442d69..442dff). That flag is read by the next Tear.Update.
    /// Grid kind7 is excluded; empty/outside cells do not create a contact.
    pub fn contacts_regular_tear(&self, point: WorldVector) -> bool {
        let Some(index) = self.index(point) else {
            return false;
        };
        let cell = &self.descriptor.cells[index];
        matches!(self.collision(index), 2..=5)
            && cell
                .entity
                .as_ref()
                .map_or(cell.selector != 0, |entity| entity.kind != 7)
    }

    /// Type9 uses the same class4 center query, but442da1..daa admits
    /// collision class2 only for type2. Fresh enemy projectiles accept3..5.
    pub fn contacts_enemy_projectile(&self, point: WorldVector) -> bool {
        let Some(index) = self.index(point) else {
            return false;
        };
        let cell = &self.descriptor.cells[index];
        matches!(self.collision(index), 3..=5)
            && cell
                .entity
                .as_ref()
                .map_or(cell.selector != 0, |entity| entity.kind != 7)
    }

    pub fn collision(&self, index: usize) -> u32 {
        let Some(cell) = self.descriptor.cells.get(index) else {
            return 4;
        };
        if let Some(entity) = &cell.entity {
            return entity.collision_class;
        }
        match cell.selector {
            3000 => 1,
            1000.. => 3,
            _ => 0,
        }
    }

    /// RoomLoad5cedf0 derives these bounds from the authored interior size,
    /// using GridPosition(width+1) minus (20,20), then size times40.
    pub fn clamp_room(&self, point: WorldVector, margin: f32) -> WorldVector {
        WorldVector {
            x: point.x.clamp(
                60.0 + margin,
                60.0 + f32::from(self.descriptor.width - 2) * 40.0 - margin,
            ),
            y: point.y.clamp(
                140.0 + margin,
                140.0 + f32::from(self.descriptor.height - 2) * 40.0 - margin,
            ),
        }
    }

    pub fn room_center(&self) -> WorldVector {
        WorldVector {
            x: (60.0 + (60.0 + f32::from(self.descriptor.width - 2) * 40.0)) * 0.5,
            y: (140.0 + (140.0 + f32::from(self.descriptor.height - 2) * 40.0)) * 0.5,
        }
    }

    /// The short CheckLine5c74a0(mode1,threshold0) rays used by Chest scatter.
    /// Advance ten units before every single-point query, including overshoot.
    /// Source also counts positive signed Room+e98 path costs: for these
    /// at-most70-unit rays there are at most seven queries, so its <=8 budget
    /// cannot fail regardless of the individual costs. Longer queries need
    /// the actual cost owner and are outside this bounded helper's scope.
    pub fn scatter_line_clear(&self, from: WorldVector, to: WorldVector) -> bool {
        let delta = WorldVector {
            x: from.x - to.x,
            y: from.y - to.y,
        };
        let distance = (delta.x * delta.x + delta.y * delta.y).sqrt();
        if !distance.is_finite() || distance > 70.0 {
            return false;
        }
        if distance <= 0.0 {
            return true;
        }
        let step = WorldVector {
            x: delta.x / distance * 10.0,
            y: delta.y / distance * 10.0,
        };
        let mut point = from;
        let mut travelled = 0.0;
        while travelled < distance {
            point.x -= step.x;
            point.y -= step.y;
            let Some(index) = self.index(point) else {
                return false;
            };
            if self.collision(index) != 0 {
                return false;
            }
            travelled += 10.0;
        }
        true
    }

    /// CheckLine5c74a0(mode3,threshold0): four corners at +/-5 for each
    /// ten-unit step. Class1 is exempt; other nonzero collision blocks.
    /// Empty-collision samples with actual signed Room costs>0 contribute
    /// to the <=8 budget; authored selectors are not this mutable owner.
    pub fn projectile_line_clear(
        &self,
        from: WorldVector,
        to: WorldVector,
        costs: &crate::room_costs::RoomCosts,
    ) -> bool {
        self.line_clear(from, to, costs, true, 0)
    }

    /// CheckLine5c74a0 mode0 (Charger and Grub charges): every nonzero grid
    /// collision blocks, class1 included.
    pub fn charge_line_clear(
        &self,
        from: WorldVector,
        to: WorldVector,
        costs: &crate::room_costs::RoomCosts,
    ) -> bool {
        self.line_clear(from, to, costs, false, 0)
    }

    /// CheckLine mode0 with a cost threshold, as FindGridPath5b41e0 refreshes
    /// its line of sight (threshold 0; 900 while charmed).
    pub fn path_line_clear(
        &self,
        from: WorldVector,
        to: WorldVector,
        costs: &crate::room_costs::RoomCosts,
        threshold: i32,
    ) -> bool {
        self.line_clear(from, to, costs, false, threshold)
    }

    fn line_clear(
        &self,
        from: WorldVector,
        to: WorldVector,
        costs: &crate::room_costs::RoomCosts,
        exempt_class1: bool,
        threshold: i32,
    ) -> bool {
        let delta = WorldVector {
            x: from.x - to.x,
            y: from.y - to.y,
        };
        let distance = (delta.x * delta.x + delta.y * delta.y).sqrt();
        if distance <= 0.0 {
            return true;
        }
        let step = WorldVector {
            x: delta.x / distance * 10.0,
            y: delta.y / distance * 10.0,
        };
        let mut point = from;
        let mut travelled = 0.0;
        let mut marked_samples = 0;
        while travelled < distance {
            point = WorldVector {
                x: point.x - step.x,
                y: point.y - step.y,
            };
            for offset in [
                WorldVector { x: -5.0, y: -5.0 },
                WorldVector { x: -5.0, y: 5.0 },
                WorldVector { x: 5.0, y: 5.0 },
                WorldVector { x: 5.0, y: -5.0 },
            ] {
                let Some(index) = self.index(point + offset) else {
                    return false;
                };
                match self.collision(index) {
                    1 if exempt_class1 => {}
                    0 => marked_samples += u32::from(costs.entries()[index] > threshold),
                    _ => return false,
                }
            }
            travelled += 10.0;
        }
        marked_samples <= 8
    }

    /// Native grid type. Authored Block selectors 1900 are type3 although the
    /// port admits them without a grid entity record.
    fn kind(&self, index: usize) -> Option<u32> {
        let cell = &self.descriptor.cells[index];
        match &cell.entity {
            Some(entity) => Some(entity.kind),
            None => (cell.selector == 1900).then_some(3),
        }
    }

    /// Source CheckLine(mode=2): ordinary rocks permit the line; kind3 blocks
    /// it, kind16 doors bypass collision-class rejection, and kind15 walls
    /// block it unless ignoreWalls is set. The source advances ten units
    /// before testing, including the final step that may overshoot the
    /// endpoint by less than ten units.
    pub fn blast_line_clear(&self, from: WorldVector, to: WorldVector, ignore_walls: bool) -> bool {
        let delta = WorldVector {
            x: from.x - to.x,
            y: from.y - to.y,
        };
        let distance = (delta.x * delta.x + delta.y * delta.y).sqrt();
        if distance <= 0.0 {
            return true;
        }
        let step = WorldVector {
            x: delta.x / distance * 10.0,
            y: delta.y / distance * 10.0,
        };
        let mut point = from;
        let mut travelled = 0.0;
        while travelled < distance {
            point.x -= step.x;
            point.y -= step.y;
            let Some(index) = self.index(point) else {
                return false;
            };
            let kind = self.kind(index);
            if kind == Some(3) {
                return false;
            }
            let passes_class = kind == Some(16) || (ignore_walls && kind == Some(15));
            if !passes_class && !matches!(self.collision(index), 0..=3) {
                return false;
            }
            travelled += 10.0;
        }
        true
    }

    /// Sorted source grid candidates, prior to each object's virtual Destroy.
    /// This differs from the explosion's strict entity-center radius predicate.
    pub fn blast_candidates(&self, origin: WorldVector, radius: f32) -> Result<Vec<usize>, String> {
        if !origin.is_finite() || !radius.is_finite() || radius <= 0.0 || radius > MAX_QUERY_RADIUS
        {
            return Err("blast query requires a finite origin and radius in (0,4096]".into());
        }
        let rays = (radius * 8.0 / 40.0).ceil() as u32;
        let angle_step = (std::f64::consts::TAU / f64::from(rays)) as f32;
        let mut selected = BTreeSet::new();
        if let Some(index) = self.index(origin) {
            selected.insert(index);
        }
        let mut angle = 0.0_f32;
        let mut direction = WorldVector { x: 1.0, y: 0.0 };
        for _ in 0..rays {
            let mut distance = 0.0_f32;
            while distance < radius {
                distance = (distance + 20.0).min(radius);
                let point = WorldVector {
                    x: origin.x + direction.x * distance,
                    y: origin.y + direction.y * distance,
                };
                let Some(index) = self.index(point) else {
                    continue;
                };
                let Some(kind) = self.kind(index) else {
                    continue;
                };
                selected.insert(index);
                if kind == 3 {
                    break;
                }
            }
            angle += angle_step;
            direction = WorldVector {
                x: crate::fmath::cos(angle),
                y: crate::fmath::sin(angle),
            };
        }
        Ok(selected
            .into_iter()
            .filter(|&index| {
                self.blast_line_clear(
                    origin,
                    self.position(index).expect("selected index is valid"),
                    false,
                )
            })
            .collect())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CollisionGeometry {
    pub radius: f32,
    pub scale: WorldVector,
    pub points: u16,
}

/// Source collision points are cached when geometry changes, not rebuilt each
/// gameplay update. Their order matters to the f32 sum of the contact normal.
#[derive(Clone, Debug, PartialEq)]
pub struct Collider {
    geometry: CollisionGeometry,
    samples: Vec<WorldVector>,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GridContact {
    pub normal: WorldVector,
    pub previous_velocity: WorldVector,
    pub velocity_changed: bool,
}

impl Collider {
    pub fn new(geometry: CollisionGeometry) -> Result<Self, String> {
        if !geometry.radius.is_finite()
            || geometry.radius < 0.0
            || !geometry.scale.is_finite()
            || geometry.scale.x < 0.0
            || geometry.scale.y < 0.0
            || geometry.points > MAX_COLLISION_POINTS
            || geometry.radius * geometry.scale.x > MAX_QUERY_RADIUS
            || geometry.radius * geometry.scale.y > MAX_QUERY_RADIUS
        {
            return Err(
                "collision geometry requires finite nonnegative sizes and at most128 points".into(),
            );
        }
        let mut samples = Vec::with_capacity(usize::from(geometry.points));
        for index in (0..geometry.points)
            .step_by(2)
            .chain((1..geometry.points).step_by(2))
        {
            let doubled = f32::from(index) + f32::from(index);
            let angle =
                (f64::from(doubled) * std::f64::consts::PI / f64::from(geometry.points)) as f32;
            let sample = WorldVector {
                x: crate::fmath::cos(angle) * (geometry.radius * geometry.scale.x),
                y: crate::fmath::sin(angle) * (geometry.radius * geometry.scale.y),
            };
            if !sample.is_finite() {
                return Err("collision point geometry overflowed".into());
            }
            samples.push(sample);
        }
        Ok(Self { geometry, samples })
    }

    pub fn geometry(&self) -> CollisionGeometry {
        self.geometry
    }
    pub fn samples(&self) -> &[WorldVector] {
        &self.samples
    }

    /// Source displacement admission used before mass-weighted player pushing.
    pub fn blocks_player_shift(
        &self,
        grid: &Grid,
        position: WorldVector,
        shift: WorldVector,
    ) -> bool {
        let point = position + shift;
        self.samples
            .iter()
            .any(|&sample| !matches!(grid.collision_at(point + sample), 0 | 5))
    }

    /// Fresh player ground-collision class5. Both native player update phases
    /// call this solver. Render prediction may request position-only correction.
    /// The caller handles source room-bound clamping and special grid damage.
    pub fn resolve_player(
        &self,
        grid: &Grid,
        position: &mut WorldVector,
        velocity: &mut WorldVector,
        preview: bool,
    ) -> Result<Option<GridContact>, String> {
        if !position.is_finite() || !velocity.is_finite() {
            return Err("player collision requires finite world vectors".into());
        }
        let mut normal = WorldVector { x: 0.0, y: 0.0 };
        for &sample in &self.samples {
            if !matches!(grid.collision_at(*position + sample), 0 | 5) {
                normal = normal + sample;
            }
        }
        let length_squared = normal.x * normal.x + normal.y * normal.y;
        if length_squared < 0.001 {
            return Ok(None);
        }
        let inverse_length = 1.0 / length_squared.sqrt();
        normal = normal * inverse_length;
        let previous_velocity = *velocity;
        let inward = -normal.x * velocity.x - normal.y * velocity.y;
        let velocity_changed = inward < 0.0 && !preview;
        let next_velocity = if velocity_changed {
            *velocity + normal * (inward * 0.9 - 0.1)
        } else {
            *velocity
        };
        let speed_squared = next_velocity.x * next_velocity.x + next_velocity.y * next_velocity.y;
        if !next_velocity.is_finite() || !speed_squared.is_finite() {
            return Err("player collision overflowed velocity".into());
        }
        let speed = speed_squared.sqrt().max(2.0);
        let scaled_normal = WorldVector {
            x: normal.x * self.geometry.scale.x,
            y: normal.y * self.geometry.scale.y,
        };
        let radius = (scaled_normal.x * scaled_normal.x + scaled_normal.y * scaled_normal.y).sqrt()
            * self.geometry.radius;
        let mut upper = radius;
        let mut lower = radius - speed;
        for _ in 0..12 {
            let middle = (upper - lower) * 0.5 + lower;
            if matches!(grid.collision_at(*position + normal * middle), 0 | 5) {
                lower = middle;
            } else {
                upper = middle;
            }
        }
        let next_position = *position + normal * (lower - radius);
        if !next_position.is_finite() {
            return Err("player collision overflowed position".into());
        }
        *position = next_position;
        *velocity = next_velocity;
        Ok(Some(GridContact {
            normal,
            previous_velocity,
            velocity_changed,
        }))
    }

    /// Source ordinary nonplayer ground class5. Unlike the player solver,
    /// position correction only runs for inward motion. Full collision first
    /// reflects the normal component, then scales the entire velocity using
    /// the original inward dot and reflected speed. Prediction preserves it.
    pub fn resolve_nonplayer_ground(
        &self,
        grid: &Grid,
        position: &mut WorldVector,
        velocity: &mut WorldVector,
        preview: bool,
    ) -> Result<Option<GridContact>, String> {
        self.resolve_nonplayer_ground_class(grid, position, velocity, preview, false)
    }

    ///442CD4 /443512: class6 ignores grid collision1, and refines six times.
    ///The caller owns source room-bound clamping and full/preview placement.
    pub fn resolve_nonplayer_no_pits(
        &self,
        grid: &Grid,
        position: &mut WorldVector,
        velocity: &mut WorldVector,
        preview: bool,
    ) -> Result<Option<GridContact>, String> {
        self.resolve_nonplayer_ground_class(grid, position, velocity, preview, true)
    }

    fn resolve_nonplayer_ground_class(
        &self,
        grid: &Grid,
        position: &mut WorldVector,
        velocity: &mut WorldVector,
        preview: bool,
        ignores_pits: bool,
    ) -> Result<Option<GridContact>, String> {
        if !position.is_finite() || !velocity.is_finite() {
            return Err("nonplayer collision requires finite world vectors".into());
        }
        let mut normal = WorldVector { x: 0.0, y: 0.0 };
        for &sample in &self.samples {
            if grid.collision_at(*position + sample) > u32::from(ignores_pits) {
                normal = normal + sample;
            }
        }
        if normal.x == 0.0 && normal.y == 0.0 {
            return Ok(None);
        }
        let squared = normal.x * normal.x + normal.y * normal.y;
        if !squared.is_finite() || squared <= 0.0 {
            return Err("nonplayer collision normal overflowed".into());
        }
        normal = normal * (1.0 / squared.sqrt());
        let previous_velocity = *velocity;
        let inward = -normal.x * velocity.x - normal.y * velocity.y;
        if !inward.is_finite() {
            return Err("nonplayer collision speed overflowed".into());
        }
        let changed = inward < 0.0;
        if changed {
            let reflected = if preview {
                *velocity
            } else {
                *velocity + normal * (inward * 1.8 - 0.1)
            };
            let speed = (reflected.x * reflected.x + reflected.y * reflected.y).sqrt();
            if !reflected.is_finite() || !speed.is_finite() || (!preview && speed <= 0.0) {
                return Err("nonplayer collision reflection overflowed".into());
            }
            // Source 443117 -> 44346c skips refinement for zero-speed preview.
            // Tiny finite inward velocity can underflow the f32 squared length.
            if preview && speed <= 0.0 {
                return Ok(None);
            }
            let scaled_normal = WorldVector {
                x: normal.x * self.geometry.scale.x,
                y: normal.y * self.geometry.scale.y,
            };
            let radius = (scaled_normal.x * scaled_normal.x + scaled_normal.y * scaled_normal.y)
                .sqrt()
                * self.geometry.radius;
            let mut upper = radius;
            let mut lower = radius - speed.max(2.0);
            for _ in 0..if ignores_pits { 6 } else { 12 } {
                let middle = (upper - lower) * 0.5 + lower;
                if grid.collision_at(*position + normal * middle) <= u32::from(ignores_pits) {
                    lower = middle;
                } else {
                    upper = middle;
                }
            }
            let next_position = *position + normal * (lower - radius);
            let next_velocity = if preview {
                *velocity
            } else {
                reflected * (1.0 + (inward / speed) * 0.5)
            };
            if !next_position.is_finite() || !next_velocity.is_finite() {
                return Err("nonplayer collision response overflowed".into());
            }
            *position = next_position;
            *velocity = next_velocity;
        }
        Ok(Some(GridContact {
            normal,
            previous_velocity,
            velocity_changed: changed && !preview,
        }))
    }

    /// Source class3 flies over ground obstacles; only wall/door classes4/5
    /// block samples. The six-step query preserves the prior full feedback in
    /// prediction; the caller owns exact-bound contact and normal records.
    pub fn resolve_nonplayer_walls(
        &self,
        grid: &Grid,
        position: &mut WorldVector,
        velocity: &mut WorldVector,
        preview: bool,
    ) -> Result<Option<GridContact>, String> {
        if !position.is_finite() || !velocity.is_finite() {
            return Err("nonplayer collision requires finite world vectors".into());
        }
        *position = grid.clamp_room(*position, 0.0);
        let mut normal = WorldVector { x: 0.0, y: 0.0 };
        for &sample in &self.samples {
            if matches!(grid.collision_at(*position + sample), 4 | 5) {
                normal = normal + sample;
            }
        }
        if normal.x == 0.0 && normal.y == 0.0 {
            return Ok(None);
        }
        let squared = normal.x * normal.x + normal.y * normal.y;
        if !squared.is_finite() || squared <= 0.0 {
            return Err("nonplayer collision normal overflowed".into());
        }
        normal = normal * (1.0 / squared.sqrt());
        let previous_velocity = *velocity;
        let inward = -normal.x * velocity.x - normal.y * velocity.y;
        if !inward.is_finite() {
            return Err("nonplayer collision speed overflowed".into());
        }
        let changed = inward < 0.0;
        if changed {
            let reflected = if preview {
                *velocity
            } else {
                *velocity + normal * (inward * 1.8 - 0.1)
            };
            let speed = (reflected.x * reflected.x + reflected.y * reflected.y).sqrt();
            if !reflected.is_finite() || !speed.is_finite() || speed <= 0.0 {
                return Err("nonplayer collision reflection overflowed".into());
            }
            let scaled_normal = WorldVector {
                x: normal.x * self.geometry.scale.x,
                y: normal.y * self.geometry.scale.y,
            };
            let radius = (scaled_normal.x * scaled_normal.x + scaled_normal.y * scaled_normal.y)
                .sqrt()
                * self.geometry.radius;
            let mut upper = radius;
            let mut lower = radius - speed.max(2.0);
            for _ in 0..6 {
                let middle = (upper - lower) * 0.5 + lower;
                if !matches!(grid.collision_at(*position + normal * middle), 4 | 5) {
                    lower = middle;
                } else {
                    upper = middle;
                }
            }
            let next_position = *position + normal * (lower - radius);
            let next_velocity = if preview {
                *velocity
            } else {
                reflected * (1.0 + (inward / speed) * 0.5)
            };
            if !next_position.is_finite() || !next_velocity.is_finite() {
                return Err("nonplayer collision response overflowed".into());
            }
            *position = next_position;
            *velocity = next_velocity;
        }
        Ok(Some(GridContact {
            normal,
            previous_velocity,
            velocity_changed: changed && !preview,
        }))
    }
}
