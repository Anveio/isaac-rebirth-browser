//! Regular planted bombs (`engine/src/bombs.rs`): a bomb's whole life from
//! DropBomb's birth through random update, intermediate and skipped phases
//! to removal, the placement cooldown, and Isaac's blast envelope and
//! knockback.
//!
//! A bomb copies the player's position and a tenth of its velocity, detonates
//! exactly once on its 60th full update, shows its corpse for ten more and is
//! then removed. Updates move it by its velocity and slow it by 0.95 (by half
//! again below unit speed), so it only ever slows; intermediate phases present
//! it half a step ahead and skipped phases change nothing. Clocks that
//! disagree are refused without change. Placement re-arms after exactly 30
//! full updates. A blast reaches the player strictly inside the blast radius
//! plus Isaac's 10 and pushes it straight away from the bomb by the remaining
//! depth, at most 15, on half its velocity; an exact centre hit keeps it.
use crate::entropy::Entropy;
use crate::harness::Case;
use crate::props;
use rebirth_cleanroom::bombs::{self, BombLifetime, BombMotion, BombPlacementCooldown};
use rebirth_cleanroom::motion::WorldVector;
use rebirth_cleanroom::phase::Phase;
use serde_json::json;

/// A regular bomb's fuse (it detonates on the full update after 59), the
/// full updates its corpse shows, and the placement cooldown, from the
/// recorded bomb evidence (`docs/bomb-evidence.md`).
const FUSE: u8 = 59;
const CORPSE: u8 = 10;
const COOLDOWN: u32 = 30;
const ISAAC_RADIUS: f64 = 10.0;

pub fn run(case: &mut Case) {
    let e = &mut *case.entropy;
    for _ in 0..(case.steps / 8).max(64) {
        life(e);
        cooldown(e);
    }
    for _ in 0..case.steps.max(256) {
        blast(e);
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

fn wild(e: &mut Entropy, gameplay: f32) -> WorldVector {
    let component = |e: &mut Entropy| match e.below(32) {
        0 => f32::NAN,
        1 => f32::INFINITY,
        2 => signed(e, 3e38),
        _ => signed(e, gameplay),
    };
    WorldVector {
        x: component(e),
        y: component(e),
    }
}

fn phase(e: &mut Entropy) -> Phase {
    match e.below(10) {
        0 => Phase::Skipped,
        1..=3 => Phase::Interpolate,
        _ => Phase::Update,
    }
}

fn norm(v: WorldVector) -> f64 {
    let (x, y) = (f64::from(v.x), f64::from(v.y));
    (x * x + y * y).sqrt()
}

fn same(a: &BombMotion, b: &BombMotion) -> bool {
    let words = |m: &BombMotion| {
        [
            m.position.x,
            m.position.y,
            m.presented_position.x,
            m.presented_position.y,
            m.velocity.x,
            m.velocity.y,
        ]
        .map(f32::to_bits)
    };
    words(a) == words(b)
        && (
            a.fuse_updates_remaining,
            a.initial_fuse_updates,
            a.full_updates,
            a.animation_frame,
            a.lifetime,
        ) == (
            b.fuse_updates_remaining,
            b.initial_fuse_updates,
            b.full_updates,
            b.animation_frame,
            b.lifetime,
        )
}

fn rank(lifetime: BombLifetime) -> u8 {
    match lifetime {
        BombLifetime::Alive => 0,
        BombLifetime::Exploding => 1,
        BombLifetime::Removed => 2,
    }
}

/// The clock contract the source keeps: finite vectors, a regular (59) or
/// Dr. Fetus (30) fuse, and fuse, update count and frame in step with the
/// lifetime.
fn consistent(bomb: &BombMotion) -> bool {
    let finite = bomb.position.is_finite()
        && bomb.presented_position.is_finite()
        && bomb.velocity.is_finite();
    let initial = bomb.initial_fuse_updates;
    let spent =
        bomb.fuse_updates_remaining == 0 && u16::from(bomb.full_updates) == u16::from(initial) + 1;
    finite
        && matches!(initial, 59 | 30)
        && match bomb.lifetime {
            BombLifetime::Alive => {
                bomb.fuse_updates_remaining <= initial
                    && bomb.full_updates == initial - bomb.fuse_updates_remaining
                    && bomb.animation_frame == bomb.full_updates.min(58)
            }
            BombLifetime::Exploding => spent && bomb.animation_frame < CORPSE,
            BombLifetime::Removed => spent && bomb.animation_frame == CORPSE,
        }
}

/// A bomb with one clock or field changed, or `None` when the change leaves
/// it as it was.
fn corrupt(e: &mut Entropy, bomb: &BombMotion) -> Option<BombMotion> {
    let mut broken = *bomb;
    match e.below(6) {
        0 => broken.fuse_updates_remaining = broken.fuse_updates_remaining.wrapping_add(1),
        1 => broken.full_updates = broken.full_updates.wrapping_add(1),
        2 => broken.animation_frame = broken.animation_frame.wrapping_add(e.range(1, 3) as u8),
        3 => broken.initial_fuse_updates = *e.pick(&[0, 58, 60, 255]),
        4 => broken.position.x = f32::NAN,
        _ => {
            broken.lifetime = match bomb.lifetime {
                BombLifetime::Alive => BombLifetime::Removed,
                _ => BombLifetime::Alive,
            }
        }
    }
    (!same(&broken, bomb)).then_some(broken)
}

fn life(e: &mut Entropy) {
    let player = if e.chance(1, 16) {
        wild(e, 1000.0)
    } else {
        vector(e, 1000.0)
    };
    let velocity = if e.chance(1, 16) {
        wild(e, 20.0)
    } else {
        vector(e, 20.0)
    };
    let details = || json!({ "player": player, "velocity": velocity });
    let born = BombMotion::spawn(player, velocity);
    let admitted = player.is_finite() && velocity.is_finite();
    props::always!(
        born.is_ok() || !admitted,
        "bombs: a bomb is born from finite vectors",
        details()
    );
    props::sometimes!(!admitted, "bombs: births from non-finite vectors are tried");
    let (Ok(mut bomb), true) = (born, admitted) else {
        return;
    };
    props::always!(
        bomb.position == player
            && bomb.presented_position == player
            && bomb.velocity == velocity * 0.1
            && bomb.fuse_updates_remaining == FUSE
            && bomb.initial_fuse_updates == FUSE
            && bomb.full_updates == 0
            && bomb.animation_frame == 0
            && bomb.lifetime == BombLifetime::Alive,
        "bombs: a bomb is born at the player with a tenth of its velocity and a full fuse",
        details()
    );
    let birth = phase(e);
    let mut presented = bomb;
    let shown = presented.present_birth(birth);
    props::always!(
        (shown.is_ok() || birth == Phase::Skipped)
            && match shown {
                Ok(()) => {
                    presented.presented_position
                        == if birth == Phase::Interpolate {
                            bomb.position + (bomb.velocity * 1.0) * 0.5
                        } else {
                            bomb.position
                        }
                }
                Err(_) => same(&presented, &bomb),
            },
        "bombs: a newly placed bomb is presented in any admitted phase, half a step ahead in an intermediate one",
        json!({ "bomb": bomb, "phase": format!("{birth:?}") })
    );
    if shown.is_ok() {
        bomb = presented;
    }
    let (mut updates, mut detonations) = (0_u32, 0_u32);
    let mut detonated_at = None;
    let mut removed_at = None;
    // Seventy full updates remove a regular bomb; leave room for the
    // phases between them.
    for _ in 0..400 {
        if bomb.lifetime == BombLifetime::Removed && e.chance(1, 4) {
            break;
        }
        if e.chance(1, 64) {
            if let Some(broken) = corrupt(e, &bomb) {
                let mut refused = broken;
                let phase = phase(e);
                let result = refused.advance(phase);
                // A consistent bomb must run. An inconsistent one may be
                // refused or run (a bomb kind with another fuse may be
                // ported); a refusal leaves it unchanged.
                props::always!(
                    (result.is_ok() || !consistent(&broken))
                        && (result.is_ok() || same(&refused, &broken)),
                    "bombs: a bomb with consistent clocks runs, and a refused one is left unchanged",
                    json!({ "bomb": broken, "phase": format!("{phase:?}") })
                );
                props::sometimes!(
                    !consistent(&broken),
                    "bombs: bombs whose clocks disagree are tried"
                );
            }
        }
        let phase = phase(e);
        let before = bomb;
        let result = bomb.advance(phase);
        let step = || json!({ "start": details(), "before": before, "phase": format!("{phase:?}"), "after": bomb });
        // Only a position about to pass the top of f32 can overflow.
        let bounded =
            norm(before.position) + 2.0 * norm(before.velocity) < f64::from(f32::MAX) / 2.0;
        props::always!(
            result.is_ok() || (!bounded && same(&before, &bomb)),
            "bombs: a bomb's own life fails only when its position overflows, and is left unchanged",
            step()
        );
        props::sometimes!(!bounded, "bombs: bombs near the top of f32 are tried");
        let Ok(detonated) = result else {
            return;
        };
        props::always!(
            rank(before.lifetime) <= rank(bomb.lifetime),
            "bombs: a bomb's lifetime only moves forward",
            step()
        );
        if phase == Phase::Skipped || before.lifetime == BombLifetime::Removed {
            props::always!(
                !detonated && same(&before, &bomb),
                "bombs: a skipped phase or a removed bomb changes nothing",
                step()
            );
            continue;
        }
        if phase == Phase::Interpolate {
            let mut moved = bomb;
            moved.presented_position = before.presented_position;
            props::always!(
                !detonated
                    && same(&moved, &before)
                    && bomb.presented_position == before.position + (before.velocity * 1.0) * 0.5,
                "bombs: an intermediate phase presents a bomb half a step ahead and moves nothing",
                step()
            );
            continue;
        }
        updates += 1;
        let slowed = before.velocity * 0.95;
        let slow = slowed.x * slowed.x + slowed.y * slowed.y < 1.0;
        props::always!(
            bomb.position == before.position + before.velocity
                && bomb.presented_position == bomb.position
                && bomb.velocity == if slow { slowed * 0.5 } else { slowed },
            "bombs: an update moves a bomb by its velocity and slows it by 0.95, halving it below unit speed",
            step()
        );
        props::always!(
            norm(bomb.velocity) <= 0.95 * norm(before.velocity) * (1.0 + 1e-6),
            "bombs: a bomb only ever slows",
            step()
        );
        props::sometimes!(
            slow && norm(before.velocity) > 0.5,
            "bombs: a slow bomb is halved"
        );
        if detonated {
            detonations += 1;
            detonated_at.get_or_insert(updates);
        }
        if bomb.lifetime == BombLifetime::Removed {
            removed_at.get_or_insert(updates);
        }
    }
    let fuse = u32::from(FUSE);
    props::always!(
        detonations == u32::from(updates > fuse)
            && detonated_at == (updates > fuse).then_some(fuse + 1)
            && removed_at
                == (updates > fuse + u32::from(CORPSE)).then_some(fuse + 1 + u32::from(CORPSE)),
        "bombs: a bomb detonates once on its 60th full update and is removed ten updates later",
        json!({ "start": details(), "updates": updates, "detonatedAt": detonated_at, "removedAt": removed_at })
    );
    props::sometimes!(
        removed_at.is_some(),
        "bombs: a bomb's life runs to its removal"
    );
}

fn cooldown(e: &mut Entropy) {
    let mut cooldown = BombPlacementCooldown::default();
    let mut placements = 0;
    let mut since = 0_u32;
    for _ in 0..e.range(16, 300) {
        let phase = phase(e);
        let before = cooldown;
        let admitted = cooldown.advance(phase);
        props::always!(
            admitted == (phase != Phase::Skipped && before.remaining_updates == 0)
                && cooldown.remaining_updates
                    == if phase == Phase::Update {
                        before.remaining_updates.saturating_sub(1)
                    } else {
                        before.remaining_updates
                    },
            "bombs: placement is admitted only once the cooldown is spent, which only full updates spend",
            json!({ "before": before, "phase": format!("{phase:?}"), "after": cooldown })
        );
        if phase == Phase::Update && !admitted {
            since += 1;
        }
        if admitted {
            if placements > 0 {
                props::always!(
                    since == COOLDOWN,
                    "bombs: placement re-arms after exactly 30 full updates",
                    json!({ "updates": since })
                );
            }
            if e.chance(1, 2) {
                cooldown.record_placement();
                placements += 1;
                since = 0;
            }
        }
    }
    props::sometimes!(placements > 1, "bombs: a player places bombs back to back");
}

/// A point `distance` from `centre` in a random direction, taken from a
/// normalized random vector: host trig would tie the case to the
/// platform's libm.
fn away(e: &mut Entropy, centre: WorldVector, distance: f64) -> WorldVector {
    let (x, y) = (f64::from(signed(e, 1.0)), f64::from(signed(e, 1.0)));
    let length = (x * x + y * y).sqrt().max(1e-9);
    WorldVector {
        x: (f64::from(centre.x) + distance * x / length) as f32,
        y: (f64::from(centre.y) + distance * y / length) as f32,
    }
}

fn blast(e: &mut Entropy) {
    let bomb = if e.chance(1, 32) {
        wild(e, 1000.0)
    } else {
        vector(e, 1000.0)
    };
    let radius = match e.below(8) {
        0 => 0.0,
        1 => 75.0,
        _ => unit(e) * 200.0,
    };
    let reach = f64::from(radius) + ISAAC_RADIUS;
    // Mostly near the envelope, sometimes on the bomb itself, and sometimes
    // on the envelope, which a uniform distance almost never meets.
    let player = match e.below(16) {
        0 => bomb,
        1 => wild(e, 1000.0),
        2 => away(e, bomb, reach),
        _ => {
            let distance = reach * f64::from(unit(e)) * 1.5;
            away(e, bomb, distance)
        }
    };
    let velocity = if e.chance(1, 32) {
        wild(e, 20.0)
    } else {
        vector(e, 20.0)
    };
    let details =
        || json!({ "player": player, "velocity": velocity, "bomb": bomb, "radius": radius });
    let result = bombs::isaac_blast_velocity(player, velocity, bomb, radius);
    let finite = player.is_finite() && velocity.is_finite() && bomb.is_finite();
    let (dx, dy) = (
        f64::from(player.x) - f64::from(bomb.x),
        f64::from(player.y) - f64::from(bomb.y),
    );
    let distance = (dx * dx + dy * dy).sqrt();
    let bounded = distance < 1e18;
    props::always!(
        match &result {
            Ok(blasted) => blasted.is_none_or(|velocity| velocity.is_finite()),
            Err(_) => !finite || !bounded,
        },
        "bombs: a blast is resolved for finite vectors, never into a non-finite velocity",
        details()
    );
    let (Ok(blasted), true) = (result, finite) else {
        return;
    };
    // The source compares f32 squares; only well clear of the envelope does
    // an f64 distance decide the side.
    let margin = 1e-5 * reach + 1e-4 * distance.min(1e6) + 1e-3;
    if distance + margin < reach {
        props::always!(
            blasted.is_some(),
            "bombs: a blast reaches a player strictly inside its radius plus Isaac's",
            details()
        );
    } else if distance > reach + margin {
        props::always!(
            blasted.is_none(),
            "bombs: a blast misses a player beyond its radius plus Isaac's",
            details()
        );
    } else {
        props::sometimes!(true, "bombs: a player stands on a blast's edge");
    }
    let Some(pushed) = blasted else {
        return;
    };
    if player == bomb {
        props::always!(
            pushed == velocity,
            "bombs: a blast centred on the player keeps its velocity",
            details()
        );
        return;
    }
    // The source's f32 square can underflow to the centre case.
    let (sx, sy) = (player.x - bomb.x, player.y - bomb.y);
    if sx * sx + sy * sy == 0.0 {
        return;
    }
    let impulse = (reach - distance).min(15.0);
    let expected = (
        0.5 * f64::from(velocity.x) + dx / distance * impulse,
        0.5 * f64::from(velocity.y) + dy / distance * impulse,
    );
    let slack = 1e-5 * (norm(velocity) + 15.0 + reach);
    props::always!(
        (f64::from(pushed.x) - expected.0).abs() <= slack
            && (f64::from(pushed.y) - expected.1).abs() <= slack,
        "bombs: a blast halves the player's velocity and pushes it straight away from the bomb by its depth, at most 15",
        details()
    );
    props::sometimes!(
        reach - distance > 15.0,
        "bombs: a deep blast's push is capped at 15"
    );
}
