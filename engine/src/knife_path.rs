//! Actual ordinary Brim child8/0/1 path helpers, sharing source577020 cubic.
use crate::{
    grid::Grid,
    laser_line::{cubic, CurvePoint, LaserLine},
    motion::WorldVector,
};

///476450 with actual zero offset: clamp origin inside the expanded Room,
///extend1000 units, intersect X then Y, and clamp the final endpoint.
fn endpoint(
    origin: WorldVector,
    direction: WorldVector,
    room: &Grid,
) -> Result<WorldVector, String> {
    let minimum = room
        .position(0)
        .ok_or("Knife line Room minimum is absent")?;
    let maximum = room
        .position(room.descriptor().cells.len() - 1)
        .ok_or("Knife line Room maximum is absent")?;
    let origin = WorldVector {
        x: origin.x.clamp(minimum.x + 1.0, maximum.x - 1.0),
        y: origin.y.clamp(minimum.y + 1.0, maximum.y - 1.0),
    };
    let mut end = origin + direction * 1000.0;
    if end.x < minimum.x || end.x > maximum.x {
        let edge = if end.x < minimum.x {
            minimum.x
        } else {
            maximum.x
        };
        end.y = ((end.y - origin.y) * (edge - origin.x)) / (end.x - origin.x) + origin.y;
        end.x = edge;
    }
    if end.y < minimum.y || end.y > maximum.y {
        let edge = if end.y < minimum.y {
            minimum.y
        } else {
            maximum.y
        };
        end.x = ((end.x - origin.x) * (edge - origin.y)) / (end.y - origin.y) + origin.x;
        end.y = edge;
    }
    end.x = end.x.clamp(minimum.x, maximum.x);
    end.y = end.y.clamp(minimum.y, maximum.y);
    if !end.is_finite() {
        return Err("Knife line endpoint overflowed actual source geometry".into());
    }
    Ok(end)
}

///473BCE mode0 has two control triples. The ordered target query has no
///mode0 mutations/RNG; actual nonzero modes retain their own unadmitted paths.
pub(crate) fn rebuild(
    line: &mut LaserLine,
    player: WorldVector,
    angle: f32,
    room: &Grid,
) -> Result<(), String> {
    let direction = crate::knives::degree_direction(angle);
    line.initialize(30, 0);
    line.width = Some(150.0);
    line.begin_curve();
    line.origin = player;
    line.offset = endpoint(player, direction, room)? + direction * 200.0;
    let middle = (line.origin + line.offset) * 0.5;
    line.controls.push(CurvePoint {
        position: line.origin,
        incoming: line.origin,
        outgoing: middle,
    });
    line.controls.push(CurvePoint {
        position: line.offset,
        incoming: middle,
        outgoing: line.offset,
    });
    Ok(())
}

///577150 chooses literal endpoint branches; endpointb is not recomputed
///as a+(b-a). Only the interior branch performs that interpolation.
fn segment_projection(point: WorldVector, a: WorldVector, b: WorldVector) -> (f32, f32) {
    let distance = |endpoint: WorldVector| {
        crate::knives::squared_length(WorldVector {
            x: point.x - endpoint.x,
            y: point.y - endpoint.y,
        })
    };
    let segment = WorldVector {
        x: b.x - a.x,
        y: b.y - a.y,
    };
    let length = crate::knives::squared_length(segment);
    if length == 0.0 {
        return (distance(a), 0.0);
    }
    let parameter = ((point.x - a.x) * segment.x + (point.y - a.y) * segment.y) / length;
    if parameter < 0.0 {
        return (distance(a), 0.0);
    }
    if parameter > 1.0 {
        return (distance(b), 1.0);
    }
    (distance(a + segment * parameter), parameter)
}

///5773A0 uses five actual chord probes, then measures the corresponding
///cubic point. Its returned parameter is not a generic closest-curve solve.
fn nearest_pair(point: WorldVector, a: CurvePoint, b: CurvePoint) -> (f32, f32) {
    let mut best_distance = 1_000_000.0;
    let mut best_parameter = 0.0;
    let mut previous = cubic(a, b, 0.0);
    let mut previous_parameter = 0.0;
    for index in 1..=5 {
        let parameter = index as f32 / 5.0;
        let next = cubic(a, b, parameter);
        let (distance, local_parameter) = segment_projection(point, previous, next);
        if distance < best_distance {
            best_parameter =
                (parameter - previous_parameter) * local_parameter + previous_parameter;
            let at = cubic(a, b, best_parameter);
            let delta = WorldVector {
                x: point.x - at.x,
                y: point.y - at.y,
            };
            best_distance = crate::knives::squared_length(delta);
        }
        previous = next;
        previous_parameter = parameter;
    }
    (best_distance, best_parameter)
}

///47424E keeps progress within the current segment and chooses the closest
///later candidate. A missing forward candidate returns the literal sentinel.
pub(crate) fn nearest_forward(
    point: WorldVector,
    controls: &[CurvePoint],
    current: u32,
    previous: f32,
) -> (u32, f32) {
    let mut best_distance = 10_000_000.0; //47424e's66cd98, distinct from5773fe.
    let mut selected = 0;
    let mut parameter = -100.0;
    for (index, pair) in controls.windows(2).enumerate().skip(current as usize) {
        let (distance, next) = nearest_pair(point, pair[0], pair[1]);
        if distance < best_distance && (index != current as usize || next > previous) {
            best_distance = distance;
            selected = index as u32;
            parameter = next;
        }
    }
    (selected, parameter)
}
