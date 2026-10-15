//! Literal42EDD0 angle wrapping and42EEB0 shortest-angle interpolation.

pub(crate) fn wrap_180(mut angle: f32) -> f32 {
    if angle.abs() > 360.0 {
        angle %= 360.0;
    }
    if angle > 180.0 {
        angle -= 360.0;
    } else if angle < -180.0 {
        angle += 360.0;
    }
    angle
}

pub(crate) fn approach(from: f32, to: f32, factor: f32) -> f32 {
    let mut from = wrap_180(from);
    let to = wrap_180(to);
    let mut difference = to - from;
    if difference.abs() > 180.0 {
        difference = if to > from {
            //42EF30 adjusts saved XMM3;42EF21 uses that value again.
            from += 360.0;
            to - from
        } else {
            (to + 360.0) - from
        };
    }
    wrap_180(difference * factor + from)
}
