//! Regular pickup subtypes (`pickups::regular_subtype`: Pickup.Init's local
//! RNG35 rolls, 500150, 4ffcae and 4ffd4e) and the pickups they spawn as.
//!
//! A nonzero subtype is kept. A zero one is rolled from a nonzero spawn seed
//! for the four regular families only, into the set each family documents
//! (hearts 1, 2, 3, 4 and 6; coins 1 to 3; keys 1 and 2; bombs 1, 2, 3 and
//! 5), and every member is rolled. Every roll spawns as a pickup with its
//! Appear and Idle clips that keeps its rolled subtype, the transient troll
//! bombs (40/3, 40/5) drawing the regular bomb's visual until converted.
use crate::entropy::Entropy;
use crate::harness::Case;
use crate::props;
use rebirth_cleanroom::motion::WorldVector;
use rebirth_cleanroom::pickups::{self, Pickup};
use serde_json::json;

/// The regular families and the subtypes each rolls.
const FAMILIES: [(u32, &[u32]); 4] = [
    (10, &[1, 2, 3, 4, 6]),
    (20, &[1, 2, 3]),
    (30, &[1, 2]),
    (40, &[1, 2, 3, 5]),
];

pub fn run(case: &mut Case) {
    let e = &mut *case.entropy;
    for _ in 0..(case.steps * 4).max(1024) {
        roll(e);
    }
}

fn roll(e: &mut Entropy) {
    let variant = match e.below(8) {
        0 => *e.pick(&[0, 5, 50, 69, 70, 90, 100, 300, 340, 350, u32::MAX]),
        _ => e.pick(&FAMILIES).0,
    };
    let subtype = match e.below(8) {
        0 => e.range(1, 10),
        1 => u32::MAX,
        _ => 0,
    };
    let seed = if e.chance(1, 64) { 0 } else { e.nonzero_u32() };
    let details = || json!({ "variant": variant, "subtype": subtype, "seed": seed });
    let rolled = pickups::regular_subtype(variant, subtype, seed);
    props::always!(
        pickups::regular_subtype(variant, subtype, seed).ok() == rolled.as_ref().ok().copied(),
        "pickups: a subtype roll is a pure function of its selector and seed",
        details()
    );
    let family = FAMILIES.iter().find(|(family, _)| *family == variant);
    if subtype != 0 {
        props::always!(
            rolled.as_ref().ok() == Some(&subtype),
            "pickups: a nonzero subtype is kept",
            details()
        );
        return;
    }
    props::always!(
        rolled.is_ok() == (family.is_some() && seed != 0),
        "pickups: a zero subtype is rolled exactly for the regular families from a nonzero seed",
        details()
    );
    let (Ok(rolled), Some((_, members))) = (rolled, family) else {
        return;
    };
    props::always!(
        members.contains(&rolled),
        "pickups: a rolled subtype is one its family documents",
        json!({ "variant": variant, "seed": seed, "rolled": rolled })
    );
    match (variant, rolled) {
        (10, 3) => props::sometimes!(true, "pickups: a soul heart is rolled"),
        (10, 4) => props::sometimes!(true, "pickups: an eternal heart is rolled"),
        (10, 6) => props::sometimes!(true, "pickups: a black heart is rolled"),
        (20, 2) => props::sometimes!(true, "pickups: a nickel is rolled"),
        (20, 3) => props::sometimes!(true, "pickups: a dime is rolled"),
        (30, 2) => props::sometimes!(true, "pickups: a golden key is rolled"),
        (40, 2) => props::sometimes!(true, "pickups: a double bomb is rolled"),
        (40, 3) => props::sometimes!(true, "pickups: a troll bomb is rolled"),
        (40, 5) => props::sometimes!(true, "pickups: a megatroll bomb is rolled"),
        _ => {}
    }
    let appear = e.chance(1, 2);
    let position = WorldVector {
        x: e.range(0, 600) as f32,
        y: e.range(0, 400) as f32,
    };
    let spawned = Pickup::spawn(e.u32(), variant, rolled, position, appear);
    let troll = variant == 40 && matches!(rolled, 3 | 5);
    props::always!(
        spawned.as_ref().is_ok_and(|pickup| {
            pickup.variant == variant
                && pickup.subtype == rolled
                && pickup.spritesheet_item_id == if troll { 1 } else { rolled }
                && pickup.animation == if appear { "Appear" } else { "Idle" }
                && pickup.position == position
                && pickup.collision_radius > 0.0
                && pickup.configured_friction.is_finite()
        }),
        "pickups: every rolled subtype spawns as a pickup that keeps it, troll bombs drawing the bomb's visual",
        json!({ "variant": variant, "rolled": rolled, "appear": appear, "spawned": spawned.as_ref().ok() })
    );
}
