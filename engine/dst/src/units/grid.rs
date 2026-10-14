//! Grid queries and the ground-collision solvers (`engine/src/grid.rs`) on
//! generated rooms, independent of how any tape reaches a contact.
use crate::entropy::Entropy;
use crate::harness::Case;
use crate::props;
use rebirth_cleanroom::grid::{Collider, CollisionGeometry, Grid};
use rebirth_cleanroom::motion::WorldVector;
use rebirth_cleanroom::{floor, seed, Room};
use serde_json::json;

/// Motion bound for "ordinary" inputs: far above any gameplay speed.
const MAX_SPEED: f32 = 1_000.0;

pub fn run(case: &mut Case) {
    let e = &mut *case.entropy;
    for _ in 0..(case.steps / 100).max(8) {
        let Ok(floor) = floor::generate_basement(seed::stage1_seed(e.nonzero_u32())) else {
            continue;
        };
        let index = e.index(floor.descriptors.len());
        let Some(descriptor) = Room::from_floor_descriptor(&floor, index).grid else {
            continue;
        };
        let Ok(grid) = Grid::new(descriptor) else {
            props::always!(false, "grid: admitted room grids are valid grids");
            continue;
        };
        props::always!(true, "grid: admitted room grids are valid grids");
        let cells = grid.descriptor().cells.len();
        let round_trips = (0..cells).all(|cell| {
            grid.position(cell)
                .is_some_and(|position| grid.index(position) == Some(cell))
        });
        props::always!(
            round_trips && grid.position(cells).is_none(),
            "grid: every cell centre indexes back to its cell"
        );
        blast_queries(e, &grid);
        for _ in 0..48 {
            let collider = collider(e);
            let position = near_cell(e, &grid);
            let velocity = motion(e);
            let preview = e.chance(1, 2);
            nonplayer(&grid, &collider, position, velocity, preview);
            player(&grid, &collider, position, velocity, preview);
        }
    }
}

/// The engine's player and regular-bomb colliders, or random geometry.
fn collider(e: &mut Entropy) -> Collider {
    let geometry = match e.below(4) {
        0 => CollisionGeometry {
            radius: 10.0,
            scale: WorldVector { x: 1.0, y: 1.0 },
            points: 40,
        },
        1 => CollisionGeometry {
            radius: 16.0,
            scale: WorldVector { x: 1.0, y: 1.0 },
            points: 12,
        },
        _ => CollisionGeometry {
            radius: e.range(2, 30) as f32,
            scale: WorldVector {
                x: e.range(50, 150) as f32 / 100.0,
                y: e.range(50, 150) as f32 / 100.0,
            },
            points: *e.pick(&[4, 8, 12, 16, 24, 32, 40]),
        },
    };
    Collider::new(geometry).expect("bounded geometry is valid")
}

/// A point within a cell or two of a random cell centre, so walls, rocks and
/// open floor are all touched.
fn near_cell(e: &mut Entropy, grid: &Grid) -> WorldVector {
    let centre = grid
        .position(e.index(grid.descriptor().cells.len()))
        .expect("in-range cell");
    WorldVector {
        x: centre.x + e.range(0, 120) as f32 - 60.0,
        y: centre.y + e.range(0, 120) as f32 - 60.0,
    }
}

/// Zero, residual (whose f32 squares underflow), ordinary and fast speeds in
/// a random direction.
fn motion(e: &mut Entropy) -> WorldVector {
    let magnitude = match e.below(4) {
        0 => 0.0,
        1 => 10f32.powi(-(e.range(20, 38) as i32)),
        2 => e.range(1, 1000) as f32 / 1000.0,
        _ => e.range(1, 60) as f32,
    };
    let direction = WorldVector {
        x: e.range(0, 200) as f32 / 100.0 - 1.0,
        y: e.range(0, 200) as f32 / 100.0 - 1.0,
    };
    let length = speed(direction);
    if length == 0.0 {
        return WorldVector {
            x: magnitude,
            y: 0.0,
        };
    }
    WorldVector {
        x: direction.x / length * magnitude,
        y: direction.y / length * magnitude,
    }
}

fn speed(vector: WorldVector) -> f32 {
    (vector.x * vector.x + vector.y * vector.y).sqrt()
}

/// Correction moves only along the contact normal, away from the wall.
fn along_normal(before: WorldVector, after: WorldVector, normal: WorldVector) -> bool {
    let shift = WorldVector {
        x: after.x - before.x,
        y: after.y - before.y,
    };
    let across = shift.x * normal.y - shift.y * normal.x;
    let into = shift.x * normal.x + shift.y * normal.y;
    across.abs() <= 1e-3 * (speed(shift) + 1.0) && into <= 1e-3
}

fn nonplayer(
    grid: &Grid,
    collider: &Collider,
    start: WorldVector,
    motion: WorldVector,
    preview: bool,
) {
    let (mut position, mut velocity) = (start, motion);
    let result = collider.resolve_nonplayer_ground(grid, &mut position, &mut velocity, preview);
    let details = || json!({ "position": start, "velocity": motion, "preview": preview, "geometry": collider.geometry() });
    props::always!(
        result.is_ok() || speed(motion) > MAX_SPEED,
        "grid: nonplayer ground resolution never fails for bounded motion",
        json!({ "error": result.as_ref().err(), "inputs": details() })
    );
    let Ok(contact) = result else {
        return;
    };
    // Source 443117 -> 44346c: an inward preview whose squared speed underflows
    // skips refinement (no contact, nothing changed) where the full update
    // still resolves the contact.
    if preview && contact.is_none() && motion.x * motion.x + motion.y * motion.y == 0.0 {
        let (mut full_position, mut full_velocity) = (start, motion);
        let full =
            collider.resolve_nonplayer_ground(grid, &mut full_position, &mut full_velocity, false);
        props::sometimes!(
            matches!(full, Ok(Some(contact)) if contact.velocity_changed),
            "grid: an underflowing inward preview skips refinement"
        );
    }
    props::always!(
        position.is_finite() && velocity.is_finite(),
        "grid: nonplayer resolution keeps finite vectors",
        details()
    );
    props::always!(
        !preview || velocity == motion,
        "grid: a nonplayer preview never changes velocity",
        details()
    );
    match contact {
        None => props::always!(
            position == start && velocity == motion,
            "grid: nonplayer resolution without contact changes nothing",
            details()
        ),
        Some(contact) => {
            props::always!(
                along_normal(start, position, contact.normal),
                "grid: nonplayer correction moves only out along the contact normal",
                json!({ "after": position, "normal": contact.normal, "inputs": details() })
            );
            if contact.velocity_changed {
                let away = velocity.x * contact.normal.x + velocity.y * contact.normal.y;
                props::always!(
                    away < 0.0,
                    "grid: a full nonplayer reflection leaves the wall",
                    json!({ "after": velocity, "normal": contact.normal, "inputs": details() })
                );
            }
            props::sometimes!(
                preview && speed(motion) > 0.0 && speed(motion) < 1e-22,
                "grid: a residual-speed preview contact is resolved"
            );
        }
    }
}

fn player(
    grid: &Grid,
    collider: &Collider,
    start: WorldVector,
    motion: WorldVector,
    preview: bool,
) {
    let (mut position, mut velocity) = (start, motion);
    let result = collider.resolve_player(grid, &mut position, &mut velocity, preview);
    let details = || json!({ "position": start, "velocity": motion, "preview": preview, "geometry": collider.geometry() });
    props::always!(
        result.is_ok() || speed(motion) > MAX_SPEED,
        "grid: player ground resolution never fails for bounded motion",
        json!({ "error": result.as_ref().err(), "inputs": details() })
    );
    let Ok(contact) = result else {
        return;
    };
    props::always!(
        position.is_finite() && velocity.is_finite() && (!preview || velocity == motion),
        "grid: player resolution keeps finite vectors and previews keep velocity",
        details()
    );
    match contact {
        None => props::always!(
            position == start && velocity == motion,
            "grid: player resolution without contact changes nothing",
            details()
        ),
        Some(contact) => {
            props::always!(
                along_normal(start, position, contact.normal),
                "grid: player correction moves only out along the contact normal",
                json!({ "after": position, "normal": contact.normal, "inputs": details() })
            );
            props::sometimes!(
                contact.velocity_changed,
                "grid: a player wall contact changes velocity"
            );
        }
    }
}

fn blast_queries(e: &mut Entropy, grid: &Grid) {
    let origin = near_cell(e, grid);
    let radius = *e.pick(&[1.0, 20.0, 75.0, 75.5, 200.0, 4096.0]);
    match grid.blast_candidates(origin, radius) {
        Ok(selected) => props::always!(
            selected.windows(2).all(|pair| pair[0] < pair[1])
                && selected
                    .iter()
                    .all(|cell| *cell < grid.descriptor().cells.len()),
            "grid: blast candidates are ascending, unique cells of the room",
            json!({ "origin": origin, "radius": radius })
        ),
        Err(error) => props::always!(
            false,
            "grid: blast candidates are ascending, unique cells of the room",
            json!({ "origin": origin, "radius": radius, "error": error })
        ),
    }
    let invalid = *e.pick(&[0.0, -1.0, 4096.5, f32::INFINITY, f32::NAN]);
    props::always!(
        grid.blast_candidates(origin, invalid).is_err(),
        "grid: blast queries reject non-positive, oversized and non-finite radii"
    );
}
