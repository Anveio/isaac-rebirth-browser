//! The mass contact responses (`engine/src/combat.rs`, source 445710) over
//! arbitrary bodies, not only the radii, masses and speeds gameplay reaches.
//!
//! Oracles are physical and computed in f64 from the inputs: the response
//! runs exactly for strictly overlapping circles, moves positions and
//! velocities only along the centre line, keeps the pair's centre of mass in
//! place, conserves momentum between non-player bodies (the player takes
//! half its share), leaves an ordinary pair exactly a radius sum apart, and
//! hands the player's collider exactly the displacement it applies. Rejected
//! and non-overlapping pairs are left untouched, and an NPC pair resolves
//! the same whichever body comes first.
use crate::entropy::Entropy;
use crate::harness::Case;
use crate::props;
use rebirth_cleanroom::combat::{self, ContactBody, ContactResolution};
use rebirth_cleanroom::implemented_npc_kinds;
use rebirth_cleanroom::motion::WorldVector;
use serde::Serialize;
use serde_json::json;
use std::cell::Cell;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
enum Pair {
    PlayerNpc,
    PlayerPickup,
    PlayerProjectile,
    TearNpc,
    NpcNpc,
    BombNpc,
}

const PAIRS: [Pair; 6] = [
    Pair::PlayerNpc,
    Pair::PlayerPickup,
    Pair::PlayerProjectile,
    Pair::TearNpc,
    Pair::NpcNpc,
    Pair::BombNpc,
];

impl Pair {
    fn player(self) -> bool {
        matches!(
            self,
            Pair::PlayerNpc | Pair::PlayerPickup | Pair::PlayerProjectile
        )
    }
}

pub fn run(case: &mut Case) {
    let e = &mut *case.entropy;
    let kinds: Vec<u32> = implemented_npc_kinds().collect();
    for _ in 0..case.steps.max(256) {
        let pair = *e.pick(&PAIRS);
        let (first_type, second_type) = types(e, pair, &kinds);
        let first = body(e, first_type, None);
        let second = body(e, second_type, Some(&first));
        let block = pair.player() && e.chance(1, 4);
        check(pair, first, second, block, &kinds);
    }
}

/// The source types each response admits, occasionally replaced by one it
/// must reject.
fn types(e: &mut Entropy, pair: Pair, kinds: &[u32]) -> (u32, u32) {
    let npc = *e.pick(kinds);
    let mut types = match pair {
        Pair::PlayerNpc => (
            1,
            if e.chance(1, 2) {
                npc
            } else {
                e.range(10, 999)
            },
        ),
        Pair::PlayerPickup => (1, 5),
        Pair::PlayerProjectile => (1, 9),
        Pair::TearNpc => (2, npc),
        Pair::NpcNpc => (*e.pick(kinds), npc),
        Pair::BombNpc => (4, npc),
    };
    if e.chance(1, 16) {
        let wrong = *e.pick(&[0, 1, 2, 3, 4, 5, 9, 1000, u32::MAX]);
        if e.chance(1, 2) {
            types.0 = wrong;
        } else {
            types.1 = wrong;
        }
    }
    types
}

fn unit(e: &mut Entropy) -> f32 {
    e.u32() as f32 / u32::MAX as f32
}

fn signed(e: &mut Entropy, magnitude: f32) -> f32 {
    (unit(e) * 2.0 - 1.0) * magnitude
}

/// A body near gameplay scales, sometimes extreme or invalid. The second
/// body lies at a chosen distance from the first, mostly overlapping it.
fn body(e: &mut Entropy, entity_type: u32, near: Option<&ContactBody>) -> ContactBody {
    let radius = match e.below(32) {
        0 => 0.0,
        1 => -1.0 - unit(e),
        2 => f32::NAN,
        3 => 1e30,
        _ => 1.0 + unit(e) * 39.0,
    };
    let mass = match e.below(32) {
        0 => 0.0,
        1 => -unit(e),
        2 => f32::INFINITY,
        3 => 1e38,
        4 => 1e-6,
        _ => 0.1 + unit(e) * 99.9,
    };
    let speed = match e.below(16) {
        0 => 1e3,
        1 => 1e30,
        _ => 10.0,
    };
    let mut velocity = WorldVector {
        x: signed(e, speed),
        y: signed(e, speed),
    };
    if e.chance(1, 64) {
        velocity.y = f32::NAN;
    }
    let position = match near {
        None => WorldVector {
            x: 60.0 + unit(e) * 520.0,
            y: 140.0 + unit(e) * 280.0,
        },
        Some(first) => {
            let reach = first.radius.max(0.0) + radius.max(0.0);
            let reach = if reach.is_finite() { reach } else { 40.0 };
            let distance = match e.below(16) {
                0 => 0.0,
                1 => 1e-5,
                2 => reach,
                _ => unit(e) * 1.5 * reach,
            };
            let (x, y) = (signed(e, 1.0), signed(e, 1.0));
            let length = (x * x + y * y).sqrt();
            let (cos, sin) = if length > 0.0 {
                (x / length, y / length)
            } else {
                (1.0, 0.0)
            };
            WorldVector {
                x: first.position.x + cos * distance,
                y: first.position.y + sin * distance,
            }
        }
    };
    let position = if e.chance(1, 64) {
        WorldVector {
            x: f32::INFINITY,
            y: position.y,
        }
    } else {
        position
    };
    ContactBody {
        id: e.u32(),
        entity_type,
        position,
        velocity,
        radius,
        mass,
    }
}

fn admitted(pair: Pair, first: u32, second: u32, kinds: &[u32]) -> bool {
    let npc = |kind: u32| kinds.contains(&kind);
    match pair {
        Pair::PlayerNpc => first == 1 && (10..1000).contains(&second),
        Pair::PlayerPickup => first == 1 && second == 5,
        Pair::PlayerProjectile => first == 1 && second == 9,
        Pair::TearNpc => first == 2 && npc(second),
        Pair::NpcNpc | Pair::BombNpc => (first == 4 || npc(first)) && npc(second),
    }
}

fn valid(body: &ContactBody) -> bool {
    body.position.is_finite()
        && body.velocity.is_finite()
        && body.radius.is_finite()
        && body.radius >= 0.0
        && body.mass.is_finite()
        && body.mass > 0.0
}

/// Ordinary scales, where the response must resolve.
fn ordinary(body: &ContactBody) -> bool {
    valid(body)
        && body.radius <= 1e3
        && (1e-3..=1e4).contains(&body.mass)
        && body.velocity.x.abs().max(body.velocity.y.abs()) <= 1e3
}

fn resolve(
    pair: Pair,
    first: &mut ContactBody,
    second: &mut ContactBody,
    block: bool,
    proposed: &Cell<Option<WorldVector>>,
) -> Result<ContactResolution, String> {
    let collider = |shift: WorldVector| {
        proposed.set(Some(shift));
        block
    };
    match pair {
        Pair::PlayerNpc => combat::resolve_player_npc_contact(first, second, collider),
        Pair::PlayerPickup => combat::resolve_player_pickup_contact(first, second, collider),
        Pair::PlayerProjectile => {
            combat::resolve_player_projectile_contact(first, second, collider)
        }
        Pair::TearNpc => combat::resolve_tear_npc_contact(first, second),
        Pair::NpcNpc | Pair::BombNpc => combat::resolve_npc_or_bomb_pair(first, second),
    }
}

/// The admitted callbacks run before the mass response: a regular bomb
/// damps both velocities (456c20); a regular tear speeds an Attack Fly by
/// half its own velocity (5462a9) and advances by 0.8 of the NPC's (54638a).
fn callback(pair: Pair, first: &mut ContactBody, second: &mut ContactBody) {
    match pair {
        Pair::BombNpc | Pair::NpcNpc if first.entity_type == 4 => {
            second.velocity = second.velocity * 0.2;
            first.velocity = first.velocity * 0.5;
        }
        Pair::TearNpc => {
            if second.entity_type == 18 {
                second.velocity = second.velocity + first.velocity * 0.5;
            }
            first.position = first.position + second.velocity * 0.8;
        }
        _ => {}
    }
}

/// Bitwise equality, so an unchanged NaN still compares equal.
fn same(a: &ContactBody, b: &ContactBody) -> bool {
    let bits = |body: &ContactBody| {
        [
            body.position.x,
            body.position.y,
            body.velocity.x,
            body.velocity.y,
            body.radius,
            body.mass,
        ]
        .map(f32::to_bits)
    };
    a.id == b.id && a.entity_type == b.entity_type && bits(a) == bits(b)
}

#[derive(Clone, Copy)]
struct V(f64, f64);

impl V {
    fn of(vector: WorldVector) -> Self {
        V(f64::from(vector.x), f64::from(vector.y))
    }
    fn sub(self, other: V) -> V {
        V(self.0 - other.0, self.1 - other.1)
    }
    fn add(self, other: V) -> V {
        V(self.0 + other.0, self.1 + other.1)
    }
    fn scale(self, factor: f64) -> V {
        V(self.0 * factor, self.1 * factor)
    }
    fn length(self) -> f64 {
        (self.0 * self.0 + self.1 * self.1).sqrt()
    }
    fn cross(self, other: V) -> f64 {
        self.0 * other.1 - self.1 * other.0
    }
}

/// f32 rounding allowance for a quantity of `size` built from values up to
/// `magnitude`.
fn close(error: f64, size: f64, magnitude: f64) -> bool {
    error <= 1e-4 * size + 1e-5 * (1.0 + magnitude)
}

fn check(pair: Pair, first: ContactBody, second: ContactBody, block: bool, kinds: &[u32]) {
    let details = || {
        json!({
            "pair": pair,
            "block": block,
            "first": first,
            "second": second,
        })
    };
    let (mut a, mut b) = (first, second);
    let proposed = Cell::new(None);
    let result = resolve(pair, &mut a, &mut b, block, &proposed);
    let accepted = admitted(pair, first.entity_type, second.entity_type, kinds)
        && valid(&first)
        && valid(&second);
    props::always!(
        accepted || result.is_err(),
        "contact: pairs outside the source types or with invalid geometry are rejected",
        details()
    );
    props::always!(
        !(accepted && ordinary(&first) && ordinary(&second)) || result.is_ok(),
        "contact: every admitted pair at ordinary scales resolves",
        details()
    );
    let Ok(resolution) = result else {
        props::always!(
            same(&a, &first) && same(&b, &second),
            "contact: a rejected pair leaves both bodies unchanged",
            details()
        );
        return;
    };
    let delta = V::of(second.position).sub(V::of(first.position));
    let distance = delta.length();
    let reach = f64::from(first.radius) + f64::from(second.radius);
    let magnitude = [first.position, second.position]
        .iter()
        .map(|p| f64::from(p.x.abs().max(p.y.abs())))
        .fold(reach, f64::max);
    if (distance - reach).abs() > 1e-5 * (1.0 + magnitude) {
        props::always!(
            resolution.colliding == (distance < reach),
            "contact: the response runs exactly for strictly overlapping circles",
            details()
        );
    }
    if !resolution.colliding {
        props::always!(
            same(&a, &first) && same(&b, &second) && proposed.get().is_none(),
            "contact: a pair that does not overlap is left unchanged",
            details()
        );
        return;
    }
    let (mut ac, mut bc) = (first, second);
    callback(pair, &mut ac, &mut bc);
    // The engine's own coincidence threshold, on its f32 distance.
    let engine_distance = {
        let dx = second.position.x - first.position.x;
        let dy = second.position.y - first.position.y;
        (dx * dx + dy * dy).sqrt()
    };
    if engine_distance < 0.0001 {
        props::always!(
            same(&a, &ac) && same(&b, &bc) && proposed.get().is_none(),
            "contact: coincident centres only run the callbacks",
            details()
        );
        props::sometimes!(true, "contact: coincident centres collide");
        return;
    }
    let moved_a = V::of(a.position).sub(V::of(ac.position));
    let moved_b = V::of(b.position).sub(V::of(bc.position));
    let kicked_a = V::of(a.velocity).sub(V::of(ac.velocity));
    let kicked_b = V::of(b.velocity).sub(V::of(bc.velocity));
    let speed = [ac.velocity, bc.velocity]
        .iter()
        .map(|v| f64::from(v.x.abs().max(v.y.abs())))
        .fold(0.0, f64::max);
    // 445710 scales a fast separating pair's overlap vector by 1 + 2*dot
    // before the split (the engine's f32 test, so its boundary cannot
    // disagree); otherwise the pair ends exactly a radius sum apart. Near
    // dot = -1/2 that scaling cancels in f32, which the source shares, and
    // its rounding turns the vector off the line by up to an ulp over it.
    let (separating, cancel) = {
        let (rx, ry) = (ac.velocity.x - bc.velocity.x, ac.velocity.y - bc.velocity.y);
        let (dx, dy) = (
            second.position.x - first.position.x,
            second.position.y - first.position.y,
        );
        let relative_speed = (rx * rx + ry * ry).sqrt();
        let separating =
            relative_speed > (first.radius + second.radius) * 0.5 && rx * dx + ry * dy < 0.0;
        let dot = (rx / relative_speed) * (dx / engine_distance)
            + (ry / relative_speed) * (dy / engine_distance);
        (
            separating,
            if separating {
                f64::from((1.0 + 2.0 * dot).abs())
            } else {
                1.0
            },
        )
    };
    let along = |change: V, scale: f64| {
        let off = change.cross(delta).abs() / distance;
        close(off * cancel.min(1.0), change.length(), scale)
    };
    props::always!(
        along(moved_a, magnitude)
            && along(moved_b, magnitude)
            && along(kicked_a, speed)
            && along(kicked_b, speed),
        "contact: positions and velocities change only along the centre line",
        details()
    );
    let (mass_a, mass_b) = (f64::from(first.mass), f64::from(second.mass));
    let blocked = resolution.player_displacement_blocked;
    props::always!(
        blocked == (pair.player() && block),
        "contact: only the player's collider blocks its displacement",
        details()
    );
    if blocked {
        props::always!(
            a.position == ac.position && a.velocity == ac.velocity,
            "contact: a blocked player keeps its position and velocity",
            details()
        );
        props::sometimes!(true, "contact: a player's collider blocks its displacement");
    } else {
        let centre = moved_a.scale(mass_a).add(moved_b.scale(mass_b));
        props::always!(
            close(
                centre.length(),
                moved_a.length() * mass_a + moved_b.length() * mass_b,
                magnitude * (mass_a + mass_b),
            ),
            "contact: separation keeps the pair's centre of mass in place",
            details()
        );
        // The player takes half the impulse share that conserves momentum.
        let player_weight = if pair.player() { 2.0 } else { 1.0 };
        let momentum = kicked_a
            .scale(mass_a * player_weight)
            .add(kicked_b.scale(mass_b));
        props::always!(
            close(
                momentum.length(),
                kicked_a.length() * mass_a * player_weight + kicked_b.length() * mass_b,
                speed * (mass_a + mass_b),
            ),
            "contact: the impulse conserves momentum, the player taking half its share",
            details()
        );
        if let Some(shift) = proposed.get() {
            props::always!(
                close(
                    V::of(shift).sub(moved_a).length(),
                    moved_a.length(),
                    magnitude
                ),
                "contact: the player's collider is offered exactly the displacement applied",
                details()
            );
        }
    }
    // The impulse carries the pair's relative speed (after the callbacks),
    // split by mass; a blocked player's share all goes to the other body.
    let relative_speed = V::of(ac.velocity).sub(V::of(bc.velocity)).length();
    let share = if blocked {
        1.0
    } else {
        mass_a / (mass_a + mass_b)
    };
    props::always!(
        close(
            (kicked_b.length() - relative_speed * share).abs(),
            relative_speed * share,
            speed,
        ),
        "contact: the impulse carries the pair's relative speed, split by mass",
        details()
    );
    props::always!(
        proposed.get().is_some() == pair.player(),
        "contact: the player's collider is consulted for a colliding player contact",
        details()
    );
    props::sometimes!(
        separating,
        "contact: a fast separating pair takes the reversed overlap"
    );
    if !separating && pair != Pair::TearNpc {
        let apart = V::of(b.position).sub(V::of(a.position)).length();
        props::always!(
            close((apart - reach).abs(), reach, magnitude),
            "contact: an ordinary pair ends exactly a radius sum apart",
            details()
        );
    }
    if pair == Pair::NpcNpc && kinds.contains(&first.entity_type) {
        let (mut sa, mut sb) = (second, first);
        let swapped = resolve(pair, &mut sa, &mut sb, false, &Cell::new(None));
        let same = if first.entity_type == second.entity_type {
            swapped.is_ok()
                && close(
                    V::of(sb.position).sub(V::of(a.position)).length(),
                    0.0,
                    magnitude,
                )
                && close(
                    V::of(sa.position).sub(V::of(b.position)).length(),
                    0.0,
                    magnitude,
                )
                && close(
                    V::of(sb.velocity).sub(V::of(a.velocity)).length(),
                    0.0,
                    speed,
                )
                && close(
                    V::of(sa.velocity).sub(V::of(b.velocity)).length(),
                    0.0,
                    speed,
                )
        } else {
            swapped.ok() == Some(resolution) && same(&sb, &a) && same(&sa, &b)
        };
        props::always!(
            same,
            "contact: an NPC pair resolves the same whichever body comes first",
            details()
        );
    }
}
