//! NPCAI_Pathfinder (Entity_NPC+7d8) for families that wander the room grid.
//! MoveRandomly5b4d40 picks a cardinal target up to eleven cells away from
//! the signed Room+e98 path costs, then steers toward it each update.
use crate::{entities::NpcWorld, motion::WorldVector, mt::Mt19937, npcs::Npc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Pathfinder {
    /// +8: chosen target; zero means none.
    pub target: WorldVector,
    /// +18: last chosen direction index, or -1.
    pub direction: i32,
    /// +1c: stuck counter.
    pub stuck: i32,
    /// +20: Game+ce9f4 at the previous call.
    pub last_update: u32,
}

impl Pathfinder {
    /// Entity_NPC construction leaves the pathfinder empty.
    pub(crate) fn new() -> Self {
        Self {
            target: ZERO,
            direction: -1,
            stuck: 0,
            last_update: 0,
        }
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

fn cost(world: &NpcWorld, point: WorldVector) -> Result<i32, String> {
    // Outside the lattice GetGridIndex5c6fa0 returns -1 and the source reads
    // the word before Room+e98: Room+e94, which Room.Init (5cf3a0/5cfe40)
    // leaves at -1 or a grid index below width*height. Every caller tests
    // the cost against 199, 300 or 700, so in a room of at most 200 cells
    // any such value reads as an open cell. Larger rooms need the field.
    // (MoveRandomly's dirs[-1] side probe reaches speed*40 below the end
    // point, which leaves the lattice for Larry Jr.'s speed 2.)
    let Some(index) = world.grid.index(point) else {
        let descriptor = world.grid.descriptor();
        if u32::from(descriptor.width) * u32::from(descriptor.height) <= 200 {
            return Ok(-1);
        }
        return Err("pathfinder probe left the room grid; Room+e94 is not modelled".into());
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
        if cost(world, probe)? > 700 && length(npc.velocity) > 3.0 {
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
                let cell = cost(world, npc.position + heading * offset as f32)?;
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
        let left_cost = cost(world, left)?;
        let right_cost = cost(world, right)?;
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
