//! NPCAI_Pathfinder (Entity_NPC+7d8) for families that wander the room grid.
//! MoveRandomly5b4d40 picks a cardinal target up to eleven cells away from
//! the signed Room+e98 path costs, then steers toward it each update.
use crate::{
    entities::NpcWorld, motion::WorldVector, mt::Mt19937, npcs::Npc, room_costs::RoomCosts,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Pathfinder {
    /// +8: chosen target; zero means none.
    pub target: WorldVector,
    /// +10: cached random wander direction, independent of last_target +734.
    #[serde(default = "zero")]
    pub wander_direction_10: WorldVector,
    /// +18: last chosen direction index, or -1.
    pub direction: i32,
    /// +1c: stuck counter.
    pub stuck: i32,
    /// +20: Game+ce9f4 at the previous call.
    pub last_update: u32,
    /// +24: FindGridPath's lagged grid cell of the owner; -1 after Reset.
    #[serde(default = "no_cell")]
    pub cell: i32,
    /// +28: cell-adoption delay.
    #[serde(default)]
    pub cell_delay: i32,
    /// +2c: re-plan timer.
    #[serde(default)]
    pub replan: i32,
    /// +30: flood map, a Room+e98 copy whose visited cells hold -distance.
    #[serde(default = "RoomCosts::zeroed")]
    pub flood: RoomCosts,
    /// +730: line of sight to the target cell.
    #[serde(default)]
    pub direct: bool,
    /// +731: the flood reached `cell`.
    #[serde(default)]
    pub found: bool,
    /// +734: last target whose cell was open (cost <= 999).
    #[serde(default = "zero")]
    pub last_target: WorldVector,
    /// +73c: EvadeTarget panic counter, 3 when the threat comes in range.
    #[serde(default)]
    pub panic: i32,
    /// +740: EvadeTarget's distance-test slack.
    #[serde(default)]
    pub slack: f32,
    /// +744: EvadeTarget's pressure, clamped to 20.
    #[serde(default)]
    pub pressure: f32,
    /// +748: grid indices EvadeTarget already tried (cleared every 30 frames).
    #[serde(default)]
    pub tried: std::collections::BTreeSet<i32>,
    /// Port frontier, not a source field: a MoveRandomly probe or FindGridPath
    /// read off the lattice of a room larger than 200 cells took Room+e94 as
    /// -1 (see `cost`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub unmodelled_crawlspace_read: bool,
}

fn no_cell() -> i32 {
    -1
}

fn zero() -> WorldVector {
    ZERO
}

impl Pathfinder {
    /// Entity_NPC construction leaves the pathfinder empty.
    pub(crate) fn new() -> Self {
        Self {
            target: ZERO,
            wander_direction_10: ZERO,
            direction: -1,
            stuck: 0,
            last_update: 0,
            cell: -1,
            cell_delay: 0,
            replan: 0,
            flood: RoomCosts::zeroed(),
            direct: false,
            found: false,
            last_target: ZERO,
            panic: 0,
            slack: 0.0,
            pressure: 0.0,
            tried: std::collections::BTreeSet::new(),
            unmodelled_crawlspace_read: false,
        }
    }

    /// Reset5b4010: everything but the flood map (+30) and the tried set (+748).
    pub(crate) fn reset(&mut self) {
        *self = Self {
            last_update: u32::MAX,
            flood: std::mem::replace(&mut self.flood, RoomCosts::zeroed()),
            tried: std::mem::take(&mut self.tried),
            unmodelled_crawlspace_read: self.unmodelled_crawlspace_read,
            ..Self::new()
        };
    }

    /// ClearVectors 5B40A0 used by AddFear: preserve all other path state.
    pub(crate) fn clear_vectors(&mut self) {
        self.target = ZERO;
        self.wander_direction_10 = ZERO;
    }
}

/// The two embedded Pathfinder vectors for a family without Port ownership
/// of the remaining pathfinder. Native 49E9F0 constructs these even for Horf;
/// Npc.pathfinder == None does not establish their absence in Native.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PathVectors {
    /// Embedded pathfinder +8, not Entity's ordinary target vector.
    pub target: WorldVector,
    /// Embedded pathfinder +10, not last_target +734.
    pub wander_direction_10: WorldVector,
}

impl PathVectors {
    /// Constructor 5B4140 and its Reset 5B4010 leave these vectors zero.
    /// This does not claim ownership of the other constructor/reset fields.
    pub(crate) fn new() -> Self {
        Self {
            target: ZERO,
            wander_direction_10: ZERO,
        }
    }

    pub(crate) fn clear_vectors(&mut self) {
        self.target = ZERO;
        self.wander_direction_10 = ZERO;
    }
}

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };
/// Left, up, right, down.
const DIRECTIONS: [WorldVector; 4] = [
    WorldVector { x: -1.0, y: 0.0 },
    WorldVector { x: 0.0, y: -1.0 },
    WorldVector { x: 1.0, y: 0.0 },
    WorldVector { x: 0.0, y: 1.0 },
];

/// 612820 multiplies by the reciprocal length; zero vectors stay zero.
fn normalized(vector: WorldVector) -> WorldVector {
    let length = (vector.x * vector.x + vector.y * vector.y).sqrt();
    if length > 0.0 {
        vector * (1.0 / length)
    } else {
        vector
    }
}

fn sub(a: WorldVector, b: WorldVector) -> WorldVector {
    WorldVector {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}

fn length(vector: WorldVector) -> f32 {
    (vector.x * vector.x + vector.y * vector.y).sqrt()
}

fn cost(world: &NpcWorld, path: &mut Pathfinder, point: WorldVector) -> Result<i32, String> {
    // Outside the lattice GetGridIndex5c6fa0 returns -1 and the source reads
    // the word before Room+e98: Room+e94, which Room.Init (5cf3a0/5cfe40)
    // leaves at -1 or a grid index below width*height. Every caller tests
    // the cost against 199, 300 or 700, so in a room of at most 200 cells
    // any such value reads as an open cell. (MoveRandomly's dirs[-1] side
    // probe reaches speed*40 below the end point, which leaves the lattice
    // for Larry Jr.'s speed 2.) Room+e94 is the floor's hidden crawlspace
    // rock: Room.Init seeds an RNG from Game+81a8 (Level::Init's draw after
    // generation, 58c33c), picks one non-boss room, and only there picks a
    // grid index (5cfe34; Rock.Destroy 56cfbf compares it). Every other room
    // keeps -1. Larger rooms read -1 too, exact outside the crawlspace room,
    // and record that the field is not modelled.
    let Some(index) = world.grid.index(point) else {
        let descriptor = world.grid.descriptor();
        if u32::from(descriptor.width) * u32::from(descriptor.height) > 200 {
            path.unmodelled_crawlspace_read = true;
        }
        return Ok(-1);
    };
    world
        .path_costs
        .as_deref()
        .ok_or("pathfinder requires source Room.Init path costs")?
        .get(index)
}

/// MoveRandomly5b4d40(ignore_status = false, speed). `friction` is the
/// owner's current +470 as its AI left it this update. Fear and confusion
/// statuses are not admitted, so their branches never run.
pub(crate) fn move_randomly(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &NpcWorld,
    speed: f32,
    friction: f32,
) -> Result<(), String> {
    let mut path = npc
        .pathfinder
        .take()
        .ok_or("pathfinding NPC requires its NPCAI_Pathfinder")?;
    let result = move_with(npc, &mut path, random, world, speed, friction);
    npc.pathfinder = Some(path);
    result
}

fn move_with(
    npc: &mut Npc,
    path: &mut Pathfinder,
    random: &mut Mt19937,
    world: &NpcWorld,
    speed: f32,
    friction: f32,
) -> Result<(), String> {
    let now = world.game_counter;
    let last = path.last_update;
    path.last_update = now;
    // Room+1c..+28 interior bounds.
    let descriptor = world.grid.descriptor();
    let minimum = WorldVector { x: 60.0, y: 140.0 };
    let maximum = WorldVector {
        x: 60.0 + f32::from(descriptor.width - 2) * 40.0,
        y: 140.0 + f32::from(descriptor.height - 2) * 40.0,
    };
    // 5b5859 indexes dirs[-1] for the first candidate's side probe, reading
    // the adjacent stack words: the previous counter's bits, then speed.
    let direction = |index: i32| -> WorldVector {
        if index < 0 {
            WorldVector {
                x: f32::from_bits(last),
                y: speed,
            }
        } else {
            DIRECTIONS[index as usize]
        }
    };
    let mut tries = 10;
    loop {
        if path.target == ZERO {
            if tries == 0 {
                return Ok(());
            }
            tries -= 1;
            choose(
                npc, path, random, world, minimum, maximum, tries, &direction,
            )?;
            continue;
        }
        if now.wrapping_sub(last) as i32 > 1 {
            path.target = ZERO;
            return Ok(());
        }
        // 5b4e65 selects rather than clamps: max first, then the upper bound.
        let x = if path.target.x > minimum.x {
            path.target.x
        } else {
            minimum.x
        };
        path.target.x = if maximum.x > x { x } else { maximum.x };
        path.target.y = path.target.y.max(minimum.y).min(maximum.y);
        let delta = sub(npc.position + npc.velocity, path.target);
        if length(delta) < 5.0 {
            path.target = ZERO;
        }
        if length(npc.velocity) < 0.6 {
            path.stuck += 1;
            if path.stuck > 10 {
                path.target = ZERO;
            }
        }
        let acceleration = (normalized(delta) * -1.0) * speed;
        if friction != 0.0 {
            npc.velocity = npc.velocity
                + WorldVector {
                    x: acceleration.x / friction,
                    y: acceleration.y / friction,
                };
        }
        let probe = npc.position + normalized(npc.velocity) * 40.0;
        if cost(world, path, probe)? > 700 && length(npc.velocity) > 3.0 {
            path.stuck += 1;
            if path.stuck > 2 {
                path.target = ZERO;
            }
        }
        return Ok(());
    }
}

#[allow(clippy::too_many_arguments)]
fn choose(
    npc: &mut Npc,
    path: &mut Pathfinder,
    random: &mut Mt19937,
    world: &NpcWorld,
    minimum: WorldVector,
    maximum: WorldVector,
    tries: i32,
    direction: &dyn Fn(i32) -> WorldVector,
) -> Result<(), String> {
    let mut steps = [0i32; 4];
    for (index, step) in steps.iter_mut().enumerate() {
        let count = random.bounded(12) as i32;
        let mut distance = count * 40;
        let heading = DIRECTIONS[index];
        let candidate = npc.position + heading * distance as f32;
        if candidate.x > maximum.x
            || minimum.x > candidate.x
            || candidate.y > maximum.y
            || minimum.y > candidate.y
        {
            continue;
        }
        let mut reached = count;
        if count > 1 {
            let mut offset = 40;
            for probe in 1..count {
                let cell = cost(world, path, npc.position + heading * offset as f32)?;
                if cell > 300 && !(cell <= 999 && tries <= 0) {
                    reached = probe - 1;
                    distance = reached * 40;
                    break;
                }
                offset += 40;
            }
        }
        let end = npc.position + heading * distance as f32;
        let turn = index as i32 + 1;
        let left = end + direction((turn - 2) % 4) * 40.0;
        let right = end + DIRECTIONS[(turn & 3) as usize] * 40.0;
        let left_cost = cost(world, path, left)?;
        let right_cost = cost(world, path, right)?;
        if right_cost > 199 && left_cost > 199 && random.bounded(10) != 0 {
            continue;
        }
        *step = reached;
    }
    let start = random.bounded(4) as i32;
    let mut best = -1;
    let mut best_steps = 0;
    for turn in start..=start + 3 {
        let candidate = turn % 4;
        if best_steps < steps[candidate as usize] {
            best = candidate;
            best_steps = steps[candidate as usize];
        }
    }
    if best_steps <= 1 && random.bounded(10) != 0 {
        best = -1;
    }
    if (path.direction - best).abs() == 2 && random.bounded(5) != 0 {
        path.stuck = 0;
        return Ok(());
    }
    if best < 0 {
        path.stuck = 0;
        return Ok(());
    }
    path.direction = best;
    let heading = DIRECTIONS[best as usize];
    let reach = npc.position + heading * (best_steps * 40) as f32;
    let index = world
        .grid
        .index(reach)
        .ok_or("pathfinder target left the room grid")?;
    let center = world
        .grid
        .position(index)
        .ok_or("pathfinder target cell outside the room grid")?;
    let collision = world.grid.collision_at(center);
    if collision != 0 && !(npc.grid_collision_class == 3 && collision != 4) {
        path.stuck = 0;
        return Ok(());
    }
    path.target = center;
    npc.velocity = if heading.x.abs() > heading.y.abs() {
        WorldVector {
            x: npc.velocity.x,
            y: 0.0,
        }
    } else {
        WorldVector {
            x: 0.0,
            y: npc.velocity.y,
        }
    };
    path.stuck = 0;
    Ok(())
}

/// GetGridIndex with the native -1 sentinel.
fn cell_of(world: &NpcWorld, point: WorldVector) -> i32 {
    world.grid.index(point).map_or(-1, |index| index as i32)
}

/// A Room+e98 word by native index; -1 reads Room+e94 as `cost` does.
fn cost_at(world: &NpcWorld, path: &mut Pathfinder, index: i32) -> Result<i32, String> {
    if index < 0 {
        let descriptor = world.grid.descriptor();
        if u32::from(descriptor.width) * u32::from(descriptor.height) > 200 {
            path.unmodelled_crawlspace_read = true;
        }
        return Ok(-1);
    }
    world
        .path_costs
        .as_deref()
        .ok_or("FindGridPath requires source Room.Init path costs")?
        .get(index as usize)
}

/// GetGridPosition5c78b0 for a valid cell.
fn centre(world: &NpcWorld, index: i32) -> Result<WorldVector, String> {
    if index < 0 {
        return Err("FindGridPath centre left the room grid".into());
    }
    world
        .grid
        .position(index as usize)
        .ok_or_else(|| "FindGridPath centre left the room grid".into())
}

/// IndexXY5c7870.
fn index_xy(width: i32, height: i32, x: i32, y: i32) -> i32 {
    if x < 0 || y < 0 || x >= width || y >= height {
        -1
    } else {
        y * width + x
    }
}

fn neighbours(width: i32, height: i32, cell: i32) -> [i32; 4] {
    let (x, y) = (cell % width, cell / width);
    [
        index_xy(width, height, x - 1, y),
        index_xy(width, height, x, y - 1),
        index_xy(width, height, x + 1, y),
        index_xy(width, height, x, y + 1),
    ]
}

/// libstdc++ __push_heap with comp(a, b) = a.dist > b.dist.
fn sift_up(heap: &mut [(i32, i32)], mut hole: usize, value: (i32, i32)) {
    while hole > 0 {
        let parent = (hole - 1) / 2;
        if heap[parent].1 <= value.1 {
            break;
        }
        heap[hole] = heap[parent];
        hole = parent;
    }
    heap[hole] = value;
}

fn push_heap(heap: &mut Vec<(i32, i32)>, value: (i32, i32)) {
    heap.push(value);
    let hole = heap.len() - 1;
    sift_up(heap, hole, value);
}

/// libstdc++ pop_heap: the last element sifts down from the root, ties
/// taking the right child, then sifts up.
fn pop_heap(heap: &mut Vec<(i32, i32)>) {
    let last = heap.len() - 1;
    if last == 0 {
        heap.pop();
        return;
    }
    let value = heap[last];
    heap[last] = heap[0];
    let n = last as isize;
    let mut hole = 0_isize;
    let mut child = 0_isize;
    while child < (n - 1) / 2 {
        child = 2 * (child + 1);
        if heap[child as usize].1 > heap[(child - 1) as usize].1 {
            child -= 1;
        }
        heap[hole as usize] = heap[child as usize];
        hole = child;
    }
    if n & 1 == 0 && child == (n - 2) / 2 {
        child = 2 * (child + 1);
        heap[hole as usize] = heap[(child - 1) as usize];
        hole = child - 1;
    }
    sift_up(&mut heap[..last], hole as usize, value);
    heap.pop();
}

fn flood(path: &mut Pathfinder, world: &NpcWorld, start: i32) -> Result<(), String> {
    let descriptor = world.grid.descriptor();
    let costs = world
        .path_costs
        .as_deref()
        .ok_or("FindGridPath requires source Room.Init path costs")?;
    (path.flood, path.found) =
        flood_costs(costs, descriptor.width, descriptor.height, start, path.cell)?;
    Ok(())
}

/// 5b498f: Dijkstra from the `start` (target) cell over a copy of a
/// `width` x `height` room's costs, for the `owner` cell's walk. A visited
/// open cell (cost 0..=949) holds -distance, entering an empty cell costing
/// 1 and any other open cell 7. The flood stops once the nearest unexpanded
/// cell is half the room's cell count away, or after the expansion that
/// visits `owner`; it returns the map and whether `owner` was visited.
pub fn flood_costs(
    costs: &RoomCosts,
    width: u16,
    height: u16,
    start: i32,
    owner: i32,
) -> Result<(RoomCosts, bool), String> {
    let cells = usize::from(width) * usize::from(height);
    if cells == 0 || cells > crate::grid::MAX_GRID_CELLS {
        return Err(format!(
            "FindGridPath flood requires an admitted room grid, not {width}x{height}"
        ));
    }
    let (width, height) = (i32::from(width), i32::from(height));
    let mut flood = costs.clone();
    let mut found = false;
    let half = (width * height) / 2;
    let mut heap = Vec::new();
    push_heap(&mut heap, (start, 0));
    while let Some(&(current, distance)) = heap.first() {
        if found || distance >= half {
            break;
        }
        pop_heap(&mut heap);
        for next in neighbours(width, height, current) {
            if next < 0 {
                continue;
            }
            let cost = flood.get(next as usize)?;
            if !(0..=949).contains(&cost) {
                continue;
            }
            let reached = if cost == 0 {
                distance + 1
            } else {
                distance + 7
            };
            flood.store(next as usize, -reached)?;
            if owner == next {
                found = true;
            }
            push_heap(&mut heap, (next, reached));
        }
    }
    Ok((flood, found))
}

/// 5b47f8: the visited neighbour nearest the target (first of L, U, R, D).
fn step(path: &Pathfinder, world: &NpcWorld) -> Result<WorldVector, String> {
    let descriptor = world.grid.descriptor();
    let (width, height) = (i32::from(descriptor.width), i32::from(descriptor.height));
    let mut best = path.cell;
    let mut best_value = -1000;
    for (side, next) in neighbours(width, height, path.cell).into_iter().enumerate() {
        if world.room_type == 16 && side == 1 && next >= 0 {
            return Err("FindGridPath crawlspace gravity cells are not admitted".into());
        }
        if next >= 0 {
            let value = path.flood.get(next as usize)?;
            if value < 0 && value > best_value {
                best_value = value;
                best = next;
            }
        }
    }
    centre(world, best)
}

/// NPCAI_Pathfinder::FindGridPath5b41e0: re-plan a flood path toward
/// `target` every 10..19 calls (the only shared draw), walk cell centres or
/// straight at a target in sight, and stamp `marker` into the owner's cell
/// cost. Charm, confusion and fear are not admitted.
pub(crate) fn find_grid_path(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    target: WorldVector,
    speed: f32,
    marker: i32,
    friction: &mut f32,
) -> Result<(), String> {
    let mut path = npc
        .pathfinder
        .take()
        .ok_or("pathfinding NPC requires its NPCAI_Pathfinder")?;
    let result = find_with(
        npc, &mut path, random, world, target, speed, marker, friction,
    );
    npc.pathfinder = Some(path);
    result
}

#[expect(
    clippy::too_many_arguments,
    reason = "FindGridPath5b41e0's owner, pathfinder, target, speed, marker and friction"
)]
fn find_with(
    npc: &mut Npc,
    path: &mut Pathfinder,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    target: WorldVector,
    speed: f32,
    marker: i32,
    friction: &mut f32,
) -> Result<(), String> {
    let mut goal = cell_of(world, target);
    if goal >= 0 && cost_at(world, path, goal)? <= 999 {
        path.last_target = target;
    } else if path.last_target != ZERO {
        goal = cell_of(world, path.last_target);
    }
    let cell = cell_of(world, npc.position);
    let origin = centre(world, cell)?;
    if cell != path.cell {
        path.cell_delay -= 10;
        if path.cell_delay < 0 {
            path.cell = cell;
            path.cell_delay = if speed > 0.2 {
                (20.0_f32 / speed) as i32
            } else {
                100
            };
        }
    }
    let mut slowdown = 1.0;
    if world.game_counter.wrapping_add(1).is_multiple_of(3) {
        let probe = npc.position + normalized(npc.velocity) * 40.0;
        let ahead = cost_at(world, path, cell_of(world, probe))?;
        if (1..=949).contains(&ahead) {
            slowdown = 0.7;
        }
    }
    if path.cell < 0 {
        return Err("FindGridPath owner left the room grid".into());
    }
    let costs = world
        .path_costs
        .as_deref_mut()
        .ok_or("FindGridPath requires source Room.Init path costs")?;
    if costs.get(path.cell as usize)? <= 949 {
        costs.store(path.cell as usize, marker)?;
    }
    if crate::plain_npc::on_frame(npc, world, 10)? {
        let sighted = centre(world, cell_of(world, target))?;
        let costs = world
            .path_costs
            .as_deref()
            .ok_or("FindGridPath requires source Room.Init path costs")?;
        path.direct =
            world.grid.path_line_clear(origin, sighted, costs, 0) || npc.grid_collision_class == 3;
    }
    let distance = length(sub(target, npc.position));
    let waypoint = if (path.direct && 750.0 > distance) || path.cell == goal {
        path.replan -= 10;
        path.cell_delay -= 50;
        target
    } else {
        let timer = path.replan;
        let cell = path.cell;
        if cost_at(world, path, cell)? != 1000 && timer <= 0 {
            path.replan = random.bounded(10) as i32 + 10;
            flood(path, world, goal)?;
        } else {
            path.replan = timer - 1;
        }
        if path.found {
            step(path, world)?
        } else {
            ZERO
        }
    };
    if waypoint != ZERO {
        let delta = sub(waypoint, npc.position);
        let length = length(delta);
        if length > 1.0 {
            let scale = (length * 0.1).min(2.0);
            let push = (slowdown * speed) * scale;
            *friction *= 0.7;
            if *friction != 0.0 {
                let accel = WorldVector {
                    x: (delta.x / length) * push,
                    y: (delta.y / length) * push,
                };
                npc.velocity = npc.velocity
                    + WorldVector {
                        x: accel.x / *friction,
                        y: accel.y / *friction,
                    };
            }
        } else {
            *friction *= 0.5;
        }
    } else {
        *friction *= 0.85;
    }
    Ok(())
}

/// Wander 5b5bd0 (calm branch): pick a free point 40 away when there is no
/// target, then steer toward it at 0.5 / friction until within 3, on a grid
/// hit, or (when nearly stopped) on a bounded(10) zero.
pub(crate) fn wander(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &NpcWorld,
    friction: f32,
) -> Result<(), String> {
    let position = npc.position;
    let path = npc
        .pathfinder
        .as_mut()
        .ok_or("Wander requires Entity_NPC+7d8")?;
    if path.target == ZERO {
        let candidate = position + crate::npcs::direction(random) * 40.0;
        if world.grid.collision_at(candidate) == 0 {
            path.target = candidate;
        }
    }
    if path.target == ZERO {
        return Ok(());
    }
    let towards = WorldVector {
        x: path.target.x - position.x,
        y: path.target.y - position.y,
    };
    let length = (towards.x * towards.x + towards.y * towards.y).sqrt();
    if 3.0 > length || npc.grid_collided {
        path.target = ZERO;
        return Ok(());
    }
    let speed = npc.velocity.x * npc.velocity.x + npc.velocity.y * npc.velocity.y;
    if 0.1 > speed && random.bounded(10) == 0 {
        path.target = ZERO;
        return Ok(());
    }
    // length >= 3 here, so the source's `len > 0` test always passes.
    let step = WorldVector {
        x: towards.x / length * 0.5,
        y: towards.y / length * 0.5,
    };
    if friction != 0.0 {
        npc.velocity = npc.velocity
            + WorldVector {
                x: step.x / friction,
                y: step.y / friction,
            };
    }
    Ok(())
}

/// EvadeTarget 5b5ea0 (uncharmed): every 14th frame the threat in range
/// (250 with a clear mode-1 line, else 170) sets the panic counter to 3,
/// otherwise it decays. Calm, the owner wanders; panicking, it pushes away
/// and searches for a cell far from the threat that it can reach in a line,
/// then paths there with FindGridPath (speed 1.079, marker 0).
pub(crate) fn evade_target(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    threat: WorldVector,
    friction: &mut f32,
) -> Result<(), String> {
    let position = npc.position;
    let delta = sub(threat, position);
    let distance = length(delta);
    let fourteenth = crate::plain_npc::on_frame(npc, world, 14)?;
    let thirtieth = crate::plain_npc::on_frame(npc, world, 30)?;
    let tenth = crate::plain_npc::on_frame(npc, world, 10)?;
    let reach = if fourteenth {
        Some(mode1(world, position, threat)?)
    } else {
        None
    };
    let path = npc
        .pathfinder
        .as_mut()
        .ok_or("EvadeTarget requires Entity_NPC+7d8")?;
    // cmpnltss keeps the pressure when it is not above 20.
    if path.pressure > 20.0 {
        path.pressure = 20.0;
    }
    if let Some(clear) = reach {
        let limit = if clear { 250.0 } else { 170.0 };
        if limit > distance {
            if path.panic <= 0 {
                path.target = ZERO;
            }
            path.panic = 3;
        } else {
            path.panic -= 1;
        }
    }
    if thirtieth {
        path.tried.clear();
    }
    if path.panic <= 0 {
        path.slack = 0.0;
        path.pressure *= 0.9;
        wander(npc, random, world, *friction)?;
        *friction *= 0.9;
        return Ok(());
    }
    if path.target == ZERO {
        pick(npc, random, world, threat, delta, distance, *friction)?;
    }
    // STEER 5b6276.
    let path = npc
        .pathfinder
        .as_mut()
        .ok_or("EvadeTarget requires Entity_NPC+7d8")?;
    if path.target == ZERO {
        return Ok(());
    }
    if tenth {
        let waypoint = path.target;
        let seen = mode1(world, threat, waypoint)?;
        let path = npc
            .pathfinder
            .as_mut()
            .ok_or("EvadeTarget requires Entity_NPC+7d8")?;
        if seen && 20.0 > path.pressure {
            path.target = ZERO;
        } else {
            let away = sub(threat, waypoint);
            if away.x * away.x + away.y * away.y < 40000.0
                || away.x.abs() < 20.0
                || away.y.abs() < 20.0
            {
                path.target = ZERO;
            }
        }
    }
    let waypoint = npc.pathfinder.as_ref().map_or(ZERO, |path| path.target);
    if waypoint != ZERO {
        find_grid_path(npc, random, world, waypoint, 1.079, 0, friction)?;
    }
    Ok(())
}

fn mode1(world: &NpcWorld, from: WorldVector, to: WorldVector) -> Result<bool, String> {
    let costs = world
        .path_costs
        .as_deref()
        .ok_or("EvadeTarget requires source Room.Init path costs")?;
    world.grid.mode1_line_clear(from, to, costs, 0)
}

/// PICK 5b5ffc: the flee push, then up to ceil(7 - 0.3 pressure) new
/// candidate cells (set hits cost 0.3) 30..480 beyond the owner.
fn pick(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &NpcWorld,
    threat: WorldVector,
    delta: WorldVector,
    distance: f32,
    friction: f32,
) -> Result<(), String> {
    let mut away = ZERO;
    if distance > 0.0 {
        away = WorldVector {
            x: delta.x / -distance,
            y: delta.y / -distance,
        };
    }
    if (away.x.abs() - away.y.abs()).abs() < 0.25 {
        if random.bounded(2) == 0 {
            away.x *= 0.5;
        } else {
            away.y *= 0.5;
        }
    }
    let push = away * 1.5;
    if friction != 0.0 {
        npc.velocity = npc.velocity
            + WorldVector {
                x: push.x / friction,
                y: push.y / friction,
            };
    }
    let owner = npc.position;
    let base = owner + away * 30.0;
    let mut spent = 0.0_f32;
    // The budget comparisons are false for NaN, like the source's comiss.
    let within = |pressure: f32, spent: f32| {
        matches!(
            (7.0 - 0.3 * pressure).partial_cmp(&spent),
            Some(std::cmp::Ordering::Greater)
        )
    };
    let pressure = npc
        .pathfinder
        .as_ref()
        .ok_or("EvadeTarget requires Entity_NPC+7d8")?
        .pressure;
    if !within(pressure, 0.0) {
        return Ok(());
    }
    loop {
        let radius = random.uniform() * 450.0 + 30.0;
        let mut angle = crate::npcs::direction(random);
        angle.y *= 0.5;
        let candidate = base + angle * radius;
        let index = world.grid.index(candidate);
        let key = index.map_or(-1, |index| index as i32);
        let path = npc
            .pathfinder
            .as_mut()
            .ok_or("EvadeTarget requires Entity_NPC+7d8")?;
        if path.tried.contains(&key) {
            spent = (spent - 0.7) + 1.0;
        } else {
            path.tried.insert(key);
            // GetGridPosition(-1) is (0,120), outside the room: class 4.
            let cell = index.and_then(|index| world.grid.position(index));
            let accepted = match cell {
                Some(cell) if world.grid.collision_at(cell) == 0 => {
                    if !mode1(world, owner, cell)? {
                        false
                    } else if mode1(world, threat, owner)? && path.pressure < 20.0 {
                        path.pressure += 0.35;
                        false
                    } else {
                        let far = length(sub(threat, cell));
                        let need =
                            (((distance + 100.0) + radius / 3.0) + path.pressure) - path.slack;
                        if need > far {
                            path.slack += 10.0;
                            false
                        } else {
                            path.target = cell;
                            if path.pressure < 20.0 {
                                path.pressure *= 0.5;
                            }
                            true
                        }
                    }
                }
                _ => false,
            };
            if accepted {
                return Ok(());
            }
            spent += 1.0;
        }
        if !within(path.pressure, spent) {
            return Ok(());
        }
    }
}
