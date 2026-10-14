//! Room5c7f30 placement with the actual signed448-word cost owner.
//! Callers supply only the actual manager members eligible for their source mode.
use crate::{motion::WorldVector, mt::Mt19937};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };

pub(crate) fn direction(random: &mut Mt19937) -> WorldVector {
    let angle = (f64::from(random.uniform()) * std::f64::consts::PI * 2.0) as f32;
    WorldVector {
        x: angle.cos(),
        y: angle.sin(),
    }
}

// Source always starts at ZERO/invalid-index, so even an empty room consumes
// its actual retry direction. radius500 reset, budget100, clamp/snap order,
// Grid17/18 exclusion and fallback remain shared with the existing Worm caller.
pub(crate) fn pickup_position(
    anchor: WorldVector,
    starting_radius: f32,
    grid: &crate::grid::Grid,
    path_costs: &crate::room_costs::RoomCosts,
    occupied: &[WorldVector],
    random: &mut Mt19937,
) -> Result<WorldVector, String> {
    let mut candidate = ZERO;
    let mut index: Option<usize> = None;
    let mut budget = 100_i32;
    let mut radius = starting_radius;
    loop {
        let admissible = grid.collision_at(candidate) == 0
            && grid.index(candidate).is_none_or(|at| {
                !grid.descriptor().cells[at]
                    .entity
                    .as_ref()
                    .is_some_and(|entity| matches!(entity.kind, 17 | 18))
            })
            && index.is_some_and(|at| path_costs.entries()[at] <= 599);
        if admissible {
            if budget > 0 {
                return Ok(candidate);
            }
            break;
        }
        budget -= 1;
        if budget == -1 {
            break;
        }
        radius = (if radius <= 500.0 { radius } else { 0.0 }) + 5.0;
        candidate = anchor + direction(random) * radius;
        if grid.clamp_room(candidate, 0.0) != candidate {
            continue;
        }
        index = grid.index(candidate);
        let Some(at) = index else {
            continue;
        };
        if path_costs.get(at)? > 599 {
            continue;
        }
        candidate = grid.position(at).expect("valid grid center");
        for actor in occupied {
            let delta = WorldVector {
                x: actor.x - candidate.x,
                y: actor.y - candidate.y,
            };
            if (delta.x * delta.x + delta.y * delta.y).sqrt() < 20.0 {
                index = None;
            }
        }
    }
    let original = grid.clamp_room(anchor, 20.0);
    Ok(grid
        .index(original)
        .and_then(|at| grid.position(at))
        .expect("clamped room anchor has a grid center"))
}
