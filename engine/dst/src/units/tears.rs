//! Regular tear kinematics (`engine/src/projectiles.rs`) over arbitrary
//! shots: the velocity a shot stores from its aim and the player's motion
//! (51bce6..51bd71), its eye origin, the damage-derived scale (50d090,
//! 541ff0), collision radius and animation tier (5418e0), and each tear's
//! flight from FireTear's birth through update, intermediate and skipped
//! phases to its removal.
//!
//! Oracles are computed in f64 from the inputs where rounding matters. Along
//! a unit aim a stored velocity carries the shot speed times ten plus 1.2 of
//! the player's forward motion, never motion against the aim; across it, 1.2
//! of the player's sideways motion. The eye offset is perpendicular to the
//! velocity, 0.3 to 0.5 of its length, on the side the eye names. Scale, tier
//! and render offset rise with damage, scale and height. A tear's lifetime
//! only moves forward; a skipped phase changes nothing; an intermediate phase
//! moves only its presentation; and a tear whose falling speed settles
//! positive (acceleration above -0.1) dies within a bounded flight.
use crate::entropy::Entropy;
use crate::harness::Case;
use crate::props;
use rebirth_cleanroom::motion::WorldVector;
use rebirth_cleanroom::phase::Phase;
use rebirth_cleanroom::projectiles::{self, Eye, Lifetime, Parameters, TearMotion};
use serde_json::json;

/// 5418e0's regular-tear thresholds; the scale is compared against each plus
/// 0.05 in binary32.
const TIER_THRESHOLDS: [f32; 12] = [
    0.5, 0.625, 0.75, 0.875, 1.0, 1.125, 1.375, 1.625, 1.875, 2.125, 2.5, 2.75,
];
/// Full updates a settling tear may fly before it must have died.
const FLIGHT: usize = 20_000;

pub fn run(case: &mut Case) {
    let e = &mut *case.entropy;
    for _ in 0..case.steps.max(256) {
        shot(e);
        sizes(e);
    }
    for _ in 0..(case.steps / 16).max(32) {
        flight(e);
    }
}

fn unit(e: &mut Entropy) -> f32 {
    e.u32() as f32 / u32::MAX as f32
}

fn signed(e: &mut Entropy, magnitude: f32) -> f32 {
    (unit(e) * 2.0 - 1.0) * magnitude
}

fn vector(e: &mut Entropy, magnitude: f32) -> WorldVector {
    WorldVector {
        x: signed(e, magnitude),
        y: signed(e, magnitude),
    }
}

/// A component at gameplay scale, sometimes beyond it or invalid.
fn component(e: &mut Entropy, gameplay: f32) -> f32 {
    match e.below(32) {
        0 => 0.0,
        1 => f32::NAN,
        2 => f32::INFINITY,
        3 => signed(e, 1e6),
        4 => signed(e, 3e38),
        _ => signed(e, gameplay),
    }
}

fn wild(e: &mut Entropy, gameplay: f32) -> WorldVector {
    WorldVector {
        x: component(e, gameplay),
        y: component(e, gameplay),
    }
}

/// A unit aim (the four cardinal shots, or any direction), or occasionally
/// an arbitrary vector.
fn aim(e: &mut Entropy) -> (WorldVector, bool) {
    match e.below(8) {
        0..=3 => {
            let (x, y) = *e.pick(&[(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)]);
            (WorldVector { x, y }, true)
        }
        4..=6 => {
            let raw = vector(e, 1.0);
            let length = (raw.x * raw.x + raw.y * raw.y).sqrt();
            if length < 1e-3 {
                return (WorldVector { x: 1.0, y: 0.0 }, true);
            }
            (
                WorldVector {
                    x: raw.x / length,
                    y: raw.y / length,
                },
                true,
            )
        }
        _ => (wild(e, 2.0), false),
    }
}

fn shot_speed(e: &mut Entropy) -> f32 {
    match e.below(16) {
        0 => *e.pick(&[0.6, 1.0, f32::from_bits(0.6_f32.to_bits() - 1), 0.0]),
        1 => *e.pick(&[f32::NAN, f32::INFINITY, -1.0, 1e30]),
        _ => 0.6 + unit(e) * 2.4,
    }
}

fn uniform(e: &mut Entropy) -> f32 {
    match e.below(24) {
        0 => *e.pick(&[0.0, 1.0]),
        1 => *e.pick(&[-0.1, 1.1, f32::NAN, f32::from_bits(1.0_f32.to_bits() + 1)]),
        _ => unit(e),
    }
}

fn norm(v: WorldVector) -> f64 {
    let (x, y) = (f64::from(v.x), f64::from(v.y));
    (x * x + y * y).sqrt()
}

fn dot(a: WorldVector, b: WorldVector) -> f64 {
    f64::from(a.x) * f64::from(b.x) + f64::from(a.y) * f64::from(b.y)
}

fn bits<E>(result: &Result<WorldVector, E>) -> Option<[u32; 2]> {
    result.as_ref().ok().map(|v| [v.x.to_bits(), v.y.to_bits()])
}

/// The stored velocity and eye origin of one shot.
fn shot(e: &mut Entropy) {
    let (aim, unit_aim) = aim(e);
    let inherited = if e.chance(1, 4) {
        wild(e, 10.0)
    } else {
        vector(e, 10.0)
    };
    let shot_speed = shot_speed(e);
    let details = || json!({ "aim": aim, "inherited": inherited, "shotSpeed": shot_speed });
    let stored = projectiles::stored_velocity(aim, inherited, shot_speed);
    props::always!(
        bits(&projectiles::stored_velocity(aim, inherited, shot_speed)) == bits(&stored),
        "tears: the stored velocity is a pure function of its shot",
        details()
    );
    let admitted =
        aim.is_finite() && inherited.is_finite() && shot_speed.is_finite() && shot_speed >= 0.6;
    // Below this bound no intermediate can overflow.
    let bounded = f64::from(shot_speed)
        * (12.0 * norm(aim) + 3.0 * norm(inherited) * (1.0 + norm(aim) * norm(aim)))
        < f64::from(f32::MAX) / 4.0;
    props::always!(
        match &stored {
            Ok(velocity) => admitted && velocity.is_finite(),
            Err(_) => !admitted || !bounded,
        },
        "tears: a velocity is stored exactly for finite vectors and a shot speed of at least 0.6, and is finite",
        details()
    );
    props::sometimes!(!admitted, "tears: an invalid shot is refused");
    let Ok(velocity) = stored else {
        return;
    };
    if unit_aim {
        let speed = f64::from(shot_speed);
        let forward = dot(inherited, aim);
        let slack = 1e-5 * (speed * (10.0 + 2.0 * norm(inherited)) + 1.0);
        props::always!(
            (dot(velocity, aim) - speed * (10.0 + 1.2 * forward.max(0.0))).abs() <= slack,
            "tears: along its aim a shot keeps ten plus 1.2 of the player's forward motion, never motion against it",
            details()
        );
        let along = dot(velocity, aim);
        let across = (
            f64::from(velocity.x) - along * f64::from(aim.x),
            f64::from(velocity.y) - along * f64::from(aim.y),
        );
        let sideways = (
            f64::from(inherited.x) - forward * f64::from(aim.x),
            f64::from(inherited.y) - forward * f64::from(aim.y),
        );
        props::always!(
            (across.0 - 1.2 * speed * sideways.0).abs() <= slack
                && (across.1 - 1.2 * speed * sideways.1).abs() <= slack,
            "tears: across its aim a shot keeps 1.2 of the player's sideways motion",
            details()
        );
        props::sometimes!(
            forward < -1.0,
            "tears: a shot drops the player's motion against its aim"
        );
        props::sometimes!(
            forward > 1.0,
            "tears: a shot inherits the player's motion along its aim"
        );
    }
    eye(e, velocity);
}

fn eye(e: &mut Entropy, velocity: WorldVector) {
    let player = if e.chance(1, 16) {
        wild(e, 1000.0)
    } else {
        vector(e, 1000.0)
    };
    let uniform = uniform(e);
    let details = || json!({ "player": player, "velocity": velocity, "uniform": uniform });
    let positive = projectiles::eye_origin(player, velocity, Eye::Positive, uniform);
    let negative = projectiles::eye_origin(player, velocity, Eye::Negative, uniform);
    props::always!(
        bits(&projectiles::eye_origin(
            player,
            velocity,
            Eye::Positive,
            uniform
        )) == bits(&positive),
        "tears: the eye origin is a pure function of its shot",
        details()
    );
    let admitted = player.is_finite() && (0.0..=1.0).contains(&uniform);
    let bounded = norm(player) + norm(velocity) < f64::from(f32::MAX) / 2.0;
    props::always!(
        [&positive, &negative].iter().all(|origin| match origin {
            Ok(origin) => admitted && origin.is_finite(),
            Err(_) => !admitted || !bounded,
        }),
        "tears: an eye origin is placed exactly for finite vectors and a uniform in 0..=1, and is finite",
        details()
    );
    props::sometimes!(!admitted, "tears: an invalid eye draw is refused");
    let (Ok(positive), Ok(negative)) = (positive, negative) else {
        return;
    };
    let coefficient = f64::from(uniform) * 0.2 + 0.3;
    let speed = norm(velocity);
    let slack = 4e-7 * (norm(player) + speed) + 1e-30;
    let offset = |origin: WorldVector| {
        (
            f64::from(origin.x) - f64::from(player.x),
            f64::from(origin.y) - f64::from(player.y),
        )
    };
    let unit = (
        f64::from(velocity.x) / speed.max(f64::MIN_POSITIVE),
        f64::from(velocity.y) / speed.max(f64::MIN_POSITIVE),
    );
    for (origin, sign) in [(positive, 1.0), (negative, -1.0)] {
        let (dx, dy) = offset(origin);
        props::always!(
            (dx * unit.0 + dy * unit.1).abs() <= slack
                && ((dx * dx + dy * dy).sqrt() - coefficient * speed).abs() <= slack,
            "tears: an eye sits across the shot, 0.3 to 0.5 of its velocity from the player",
            details()
        );
        // The positive eye turns the velocity a quarter turn toward +y.
        let cross = unit.0 * dy - unit.1 * dx;
        props::always!(
            coefficient * speed <= 2.0 * slack || cross * sign > 0.0,
            "tears: each eye sits on its own side of the shot",
            details()
        );
    }
}

/// Scale, collision radius, animation tier and render offset, each against a
/// second input so their monotonicity shows.
fn sizes(e: &mut Entropy) {
    let damage = |e: &mut Entropy| match e.below(16) {
        0 => *e.pick(&[0.0, -0.0, 3.5, -1.0, f32::NAN, f32::INFINITY, 3e38]),
        1..=3 => unit(e) * 1000.0,
        _ => unit(e) * 40.0,
    };
    let (first, second) = (damage(e), damage(e));
    let scale = projectiles::scale(first);
    props::always!(
        projectiles::scale(first).ok().map(f32::to_bits)
            == scale.as_ref().ok().map(|s| s.to_bits()),
        "tears: scale is a pure function of damage",
        json!({ "damage": first })
    );
    props::always!(
        scale.is_ok() == (first.is_finite() && first >= 0.0),
        "tears: scale is computed exactly for finite nonnegative damage",
        json!({ "damage": first })
    );
    if let Ok(scale) = scale {
        let damage = f64::from(first);
        let expected = (0.04 * damage + 0.6 + 0.15 * damage.sqrt()).max(0.01);
        props::always!(
            (f64::from(scale) - expected).abs() <= 1e-6 * expected,
            "tears: scale is 0.6 + 0.04·damage + 0.15·√damage",
            json!({ "damage": first, "scale": scale })
        );
        if let Ok(other) = projectiles::scale(second) {
            props::always!(
                (first <= second) <= (scale <= other),
                "tears: scale never falls as damage rises",
                json!({ "damage": [first, second], "scale": [scale, other] })
            );
        }
    }
    let size = |e: &mut Entropy| match e.below(16) {
        0 => *e.pick(&[
            0.01,
            0.3,
            0.55,
            2.8,
            f32::from_bits(0.01_f32.to_bits() - 1),
            f32::NAN,
            0.0,
        ]),
        1 => *e.pick(&TIER_THRESHOLDS) + 0.05,
        _ => unit(e) * 4.0,
    };
    let (small, large) = (size(e), size(e));
    let tier = projectiles::regular_animation_tier(small);
    props::always!(
        tier.is_ok() == (small.is_finite() && small >= 0.01),
        "tears: an animation tier is chosen exactly for finite scales of at least 0.01",
        json!({ "scale": small })
    );
    if let Ok(tier) = tier {
        let exceeded = TIER_THRESHOLDS
            .iter()
            .filter(|&&threshold| small > threshold + 0.05_f32)
            .count() as u8;
        props::always!(
            tier == if small <= 0.3 {
                1
            } else {
                (2 + exceeded).min(13)
            },
            "tears: the animation tier counts the 5418e0 thresholds a scale exceeds, up to the thirteenth clip",
            json!({ "scale": small, "tier": tier })
        );
        if let Ok(other) = projectiles::regular_animation_tier(large) {
            props::always!(
                (small <= large) <= (tier <= other),
                "tears: the animation tier never falls as scale rises",
                json!({ "scale": [small, large], "tier": [tier, other] })
            );
        }
        props::sometimes!(tier == 13, "tears: a scale reaches the largest tier");
    }
    let authored = match e.below(16) {
        0 => *e.pick(&[0.0, -1.0, f32::NAN, f32::INFINITY, 3e38]),
        _ => unit(e) * 20.0,
    };
    let radius = projectiles::collision_radius(small, authored);
    let admitted = small.is_finite() && small >= 0.01 && authored.is_finite() && authored >= 0.0;
    props::always!(
        match radius {
            Ok(radius) => admitted && radius == authored * small,
            Err(_) => !admitted || !(authored * small).is_finite(),
        },
        "tears: the collision radius is the authored radius times the scale, for admitted parameters",
        json!({ "scale": small, "authored": authored })
    );
    let height = |e: &mut Entropy| match e.below(8) {
        0 => -8.5,
        _ => signed(e, 60.0) - 20.0,
    };
    let acceleration = match e.below(4) {
        0 => *e.pick(&[0.001, f32::from_bits(0.001_f32.to_bits() + 1)]),
        1 => unit(e) * 0.5,
        _ => 0.0,
    };
    let (low, high) = (height(e), height(e));
    let offset = projectiles::height_offset(low, acceleration);
    let model = if acceleration > 0.001 {
        f64::from(low)
    } else {
        let bend = (f64::from(low) + 8.5).max(0.0);
        f64::from(low) * 0.5 - 15.0 + bend * bend
    };
    props::always!(
        (f64::from(offset) - model).abs() <= 1e-5 * (model.abs() + 1.0),
        "tears: the render offset bends below 8.5 height, unless the tear accelerates",
        json!({ "height": low, "acceleration": acceleration, "offset": offset })
    );
    props::always!(
        (low <= high) <= (offset <= projectiles::height_offset(high, acceleration)),
        "tears: the render offset never falls as height rises",
        json!({ "height": [low, high], "acceleration": acceleration })
    );
}

fn rank(lifetime: Lifetime) -> u8 {
    match lifetime {
        Lifetime::Alive => 0,
        Lifetime::Dying => 1,
        Lifetime::Removed => 2,
    }
}

fn same_motion(a: &TearMotion, b: &TearMotion) -> bool {
    let words = |m: &TearMotion| {
        [
            m.position.x,
            m.position.y,
            m.presented_position.x,
            m.presented_position.y,
            m.velocity.x,
            m.velocity.y,
            m.height,
            m.falling_speed,
            m.falling_acceleration,
            m.render_offset_y,
        ]
        .map(f32::to_bits)
    };
    words(a) == words(b) && a.lifetime == b.lifetime
}

/// One tear from birth to removal under random phases and collisions.
fn flight(e: &mut Entropy) {
    let origin = if e.chance(1, 16) {
        wild(e, 1000.0)
    } else {
        vector(e, 1000.0)
    };
    let velocity = match e.below(20) {
        0 => vector(e, 1e38),
        1..=10 => {
            let (x, y) = *e.pick(&[(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)]);
            projectiles::stored_velocity(WorldVector { x, y }, vector(e, 8.0), 0.6 + unit(e) * 1.4)
                .unwrap_or(WorldVector { x: 10.0, y: 0.0 })
        }
        _ => vector(e, 30.0),
    };
    let parameters = match e.below(8) {
        0..=3 => Parameters::FRESH_ISAAC,
        4 => Parameters {
            height: *e.pick(&[f32::NAN, -2.0, -5.0, 0.0]),
            falling_speed: 0.0,
            falling_acceleration: 0.0,
        },
        _ => Parameters {
            height: -unit(e) * 1000.0,
            falling_speed: signed(e, 20.0),
            falling_acceleration: unit(e) * 0.55 - 0.05,
        },
    };
    let final_uniform = uniform(e);
    let details = || json!({ "origin": origin, "velocity": velocity, "parameters": parameters, "uniform": final_uniform });
    let born = TearMotion::spawn(origin, velocity, parameters, final_uniform);
    let admitted = origin.is_finite()
        && velocity.is_finite()
        && parameters.height.is_finite()
        && parameters.falling_speed.is_finite()
        && parameters.falling_acceleration.is_finite()
        && (0.0..=1.0).contains(&final_uniform);
    let bounded = norm(origin) + 2.0 * norm(velocity) < f64::from(f32::MAX) / 2.0;
    props::always!(
        match &born {
            Ok(_) => admitted,
            Err(_) => !admitted || !bounded,
        },
        "tears: a tear is born exactly from finite parameters and a uniform in 0..=1",
        details()
    );
    let Ok(mut tear) = born else {
        return;
    };
    let falling = f64::from(final_uniform) * 0.2 - f64::from(parameters.falling_speed);
    let falling = falling * 0.9 + 0.1 + f64::from(parameters.falling_acceleration);
    let slack = |value: f64| 1e-5 * (value.abs() + 1.0);
    props::always!(
        tear.lifetime == Lifetime::Alive
            && (f64::from(tear.position.x) - (f64::from(origin.x) + f64::from(velocity.x))).abs()
                <= slack(norm(origin) + norm(velocity))
            && (f64::from(tear.position.y) - (f64::from(origin.y) + f64::from(velocity.y))).abs()
                <= slack(norm(origin) + norm(velocity))
            && tear.presented_position == tear.position
            && tear.velocity == velocity
            && tear.falling_acceleration.to_bits() == parameters.falling_acceleration.to_bits()
            && (f64::from(tear.falling_speed) - falling).abs() <= slack(falling)
            && (f64::from(tear.height) - (f64::from(parameters.height) + falling)).abs()
                <= slack(f64::from(parameters.height) + falling)
            && tear.render_offset_y
                == projectiles::height_offset(tear.height, tear.falling_acceleration),
        "tears: a tear is born one step out, falling once, with its velocity and falling acceleration",
        json!({ "start": details(), "tear": tear })
    );
    // Falling speed settles at 1 + 10·acceleration, so a tear whose
    // acceleration exceeds -0.1 eventually falls below -5 height and dies.
    let settles = parameters.falling_acceleration > -0.09;
    let mut updates = 0;
    while tear.lifetime != Lifetime::Removed && updates < FLIGHT {
        let phase = match e.below(10) {
            0 => Phase::Skipped,
            1..=3 => Phase::Interpolate,
            _ => Phase::Update,
        };
        let collides = phase == Phase::Update && e.chance(1, 400);
        let before = tear;
        let result = tear.advance(phase, collides);
        let step = || json!({ "start": details(), "before": before, "phase": format!("{phase:?}"), "collides": collides, "after": tear });
        // Only a position about to pass the top of f32 can overflow.
        let bounded =
            norm(before.position) + 2.0 * norm(before.velocity) < f64::from(f32::MAX) / 2.0;
        props::always!(
            result.is_ok() || !bounded,
            "tears: a tear's flight fails only when its position overflows",
            step()
        );
        props::always!(
            result.is_ok() || same_motion(&before, &tear),
            "tears: a refused step leaves the tear unchanged",
            step()
        );
        props::sometimes!(
            result.is_err(),
            "tears: a step whose position overflows is refused"
        );
        let Ok(lifetime) = result else {
            return;
        };
        props::always!(
            lifetime == tear.lifetime && rank(before.lifetime) <= rank(lifetime),
            "tears: a tear's lifetime only moves forward",
            step()
        );
        // Each step's checks derive the next state from the previous one,
        // so a step that also rewrote the stored velocity (a per-update
        // drag, say) would pass them; the kernel never changes either.
        props::always!(
            tear.velocity.x.to_bits() == before.velocity.x.to_bits()
                && tear.velocity.y.to_bits() == before.velocity.y.to_bits()
                && tear.falling_acceleration.to_bits() == before.falling_acceleration.to_bits(),
            "tears: a tear keeps its velocity and falling acceleration through its flight",
            step()
        );
        if phase == Phase::Skipped || before.lifetime == Lifetime::Removed {
            props::always!(
                same_motion(&before, &tear),
                "tears: a skipped phase or a removed tear changes nothing",
                step()
            );
            continue;
        }
        if phase == Phase::Interpolate {
            let presented = before.position + before.velocity * 0.5;
            props::always!(
                tear.position == before.position
                    && tear.velocity == before.velocity
                    && tear.height == before.height
                    && tear.falling_speed == before.falling_speed
                    && tear.lifetime == before.lifetime
                    && tear.presented_position == presented
                    && tear.render_offset_y
                        == if before.lifetime == Lifetime::Alive {
                            projectiles::height_offset(
                                before.height + before.falling_speed * 0.5,
                                before.falling_acceleration,
                            )
                        } else {
                            before.render_offset_y
                        },
                "tears: an intermediate phase presents half a step ahead and moves nothing",
                step()
            );
            continue;
        }
        updates += 1;
        if before.lifetime == Lifetime::Dying {
            props::always!(
                tear.lifetime == Lifetime::Removed
                    && tear.position == before.position
                    && tear.presented_position == before.position,
                "tears: a dying tear is removed at the next update, where it died",
                step()
            );
            continue;
        }
        let dies = before.height > -5.0 || collides;
        let falling =
            f64::from(before.falling_speed) * 0.9 + 0.1 + f64::from(before.falling_acceleration);
        props::always!(
            tear.position == before.position + before.velocity
                && tear.presented_position == tear.position
                && tear.lifetime
                    == if dies {
                        Lifetime::Dying
                    } else {
                        Lifetime::Alive
                    }
                && if dies {
                    tear.height == before.height && tear.falling_speed == before.falling_speed
                } else {
                    (f64::from(tear.falling_speed) - falling).abs() <= slack(falling)
                        && (f64::from(tear.height) - (f64::from(before.height) + falling)).abs()
                            <= slack(f64::from(before.height) + falling)
                        && tear.render_offset_y
                            == projectiles::height_offset(tear.height, tear.falling_acceleration)
                },
            "tears: an update moves a tear one step and lets it fall, until it lands or collides",
            step()
        );
        props::sometimes!(dies && !collides, "tears: a tear lands");
        props::sometimes!(
            dies && collides && before.height <= -5.0,
            "tears: a tear dies on a collision in flight"
        );
    }
    props::always!(
        !settles || tear.lifetime == Lifetime::Removed,
        "tears: a tear whose falling speed settles positive is removed within its flight",
        json!({ "start": details(), "tear": tear })
    );
}
