//! Authored spawn-group selection (`room_spawns::select`, Room Init
//! 0x5cf190..0x5cf2ee) over every authored Basement and special room and over
//! synthetic rooms the authored data never holds: zero and fractional
//! weights, empty and misplaced groups, grids at the padded capacity.
//!
//! The stream checks replay the room's RNG11 and RNG35 through the engine's
//! other xorshift (`items::Rng`); the choice checks are semantic: every group
//! with entries yields one selector holding one of its entries, a weighted
//! entry wins unless the source's first-entry fallback applies, and a lone
//! weighted entry always wins.
use crate::entropy::Entropy;
use crate::harness::Case;
use crate::props;
use rebirth_cleanroom::items::Rng;
use rebirth_cleanroom::mt::PADDED_GRID_CELLS;
use rebirth_cleanroom::room_assets::{self, RoomAsset, SpawnEntry, SpawnGroup};
use rebirth_cleanroom::room_spawns::{self, RoomSpawns, SpawnCategory};
use rebirth_cleanroom::room_templates::AuthoredTemplate;
use serde_json::json;

pub fn run(case: &mut Case) {
    let e = &mut *case.entropy;
    let authored: Vec<RoomAsset> = room_assets::basement()
        .into_iter()
        .chain(room_assets::special())
        .collect();
    for _ in 0..(case.steps / 2).max(256) {
        let asset = if e.chance(1, 2) && !authored.is_empty() {
            e.pick(&authored).clone()
        } else {
            synthetic(e)
        };
        let seed = if e.chance(1, 64) { 0 } else { e.nonzero_u32() };
        check(&asset, seed);
    }
}

fn weight(e: &mut Entropy) -> f32 {
    match e.below(32) {
        0..=5 => 0.0,
        6 => f32::NAN,
        7 => -1.0,
        8 => f32::INFINITY,
        9 => 3e38,
        10 | 11 => 1e-30,
        _ => e.u32() as f32 / u32::MAX as f32 * 2.0,
    }
}

/// A room the authored data never holds, mostly valid.
fn synthetic(e: &mut Entropy) -> RoomAsset {
    let width = match e.below(16) {
        0 => 0,
        1 => e.range(27, 40) as u8,
        _ => e.range(1, 26) as u8,
    };
    let height = match e.below(16) {
        0 => 0,
        1 => e.range(15, 30) as u8,
        _ => e.range(1, 14) as u8,
    };
    let groups = (0..e.below(24))
        .map(|_| {
            let misplaced = e.chance(1, 24);
            let (x, y) = if misplaced {
                (e.range(0, 60) as i16 - 20, e.range(0, 60) as i16 - 20)
            } else {
                (
                    e.range(0, u32::from(width) + 1) as i16 - 1,
                    e.range(0, u32::from(height) + 1) as i16 - 1,
                )
            };
            let entries = (0..e.below(6))
                .map(|_| SpawnEntry {
                    entity_type: *e.pick(&[
                        0, 1, 4, 5, 10, 14, 16, 24, 26, 30, 88, 999, 1000, 1500, 3000,
                    ]),
                    variant: e.below(4) as u16,
                    subtype: e.below(3) as u16,
                    weight: weight(e),
                })
                .collect();
            SpawnGroup { x, y, entries }
        })
        .collect();
    RoomAsset {
        template: AuthoredTemplate {
            room_type: 1,
            variant: 0,
            difficulty: 1,
            name: String::new(),
            weight: 1.0,
            width,
            height,
            door_mask: 0,
        },
        doors: Vec::new(),
        spawns: groups,
    }
}

/// The source's admission: the room's own bounds, then each group with
/// entries in order (empty groups are never inspected).
fn admitted(asset: &RoomAsset, seed: u32) -> bool {
    let (width, height) = (
        i16::from(asset.template.width),
        i16::from(asset.template.height),
    );
    seed != 0
        && width > 0
        && height > 0
        && asset.spawns.len() <= usize::from(u16::MAX)
        && (width as usize + 2) * (height as usize + 2) <= PADDED_GRID_CELLS
        && asset
            .spawns
            .iter()
            .filter(|group| !group.entries.is_empty())
            .all(|group| {
                (-1..=width).contains(&group.x)
                    && (-1..=height).contains(&group.y)
                    && group
                        .entries
                        .iter()
                        .all(|entry| entry.weight.is_finite() && entry.weight >= 0.0)
                    && group
                        .entries
                        .iter()
                        .map(|entry| entry.weight)
                        .sum::<f32>()
                        .is_finite()
            })
}

fn next(state: u32, shifts: [u32; 3]) -> u32 {
    let mut rng = Rng::new(state, shifts);
    rng.raw()
}

const RNG11: [u32; 3] = [2, 7, 7];
const RNG35: [u32; 3] = [5, 9, 7];

fn check(asset: &RoomAsset, seed: u32) {
    let details = || json!({ "seed": seed, "asset": asset });
    let result = room_spawns::select(asset, seed);
    props::always!(
        result.is_ok() == admitted(asset, seed),
        "room_spawns: selection admits exactly the rooms and groups the source admits",
        details()
    );
    let Ok(spawns) = result else {
        props::sometimes!(true, "room_spawns: an invalid room or group is rejected");
        return;
    };
    props::always!(
        room_spawns::select(asset, seed).ok().as_ref() == Some(&spawns),
        "room_spawns: selection is a pure function of the room and its seed",
        details()
    );
    let saved = serde_json::to_value(&spawns).expect("spawns serialize");
    props::always!(
        serde_json::from_value::<RoomSpawns>(saved).ok().as_ref() == Some(&spawns),
        "room_spawns: selection round-trips through JSON",
        details()
    );
    let width = u32::from(asset.template.width);
    let height = u32::from(asset.template.height);
    let grid_width = width + 2;
    // Two draws place the tinted-rock candidate, then one per group.
    let first = next(seed, RNG11);
    let second = next(first, RNG11);
    let (x, y) = (first % width + 1, second % height + 1);
    props::always!(
        u32::from(spawns.tinted_rock_index) == x + y * grid_width
            && (1..=width).contains(&x)
            && (1..=height).contains(&y),
        "room_spawns: the tinted-rock candidate is an interior cell drawn from the room stream",
        details()
    );
    let mut state = second;
    let mut group_seeds = Vec::with_capacity(asset.spawns.len());
    for _ in &asset.spawns {
        state = next(state, RNG11);
        group_seeds.push(state);
    }
    props::always!(
        spawns.final_spawn_state == state,
        "room_spawns: the room stream advances once per group, empty ones included",
        details()
    );
    let filled: Vec<usize> = (0..asset.spawns.len())
        .filter(|&index| !asset.spawns[index].entries.is_empty())
        .collect();
    let indices: Vec<usize> = spawns
        .selectors
        .iter()
        .map(|selector| usize::from(selector.group_index))
        .collect();
    props::always!(
        indices == filled,
        "room_spawns: every group with entries yields exactly one selector, in authored order",
        details()
    );
    props::sometimes!(
        filled.len() < asset.spawns.len(),
        "room_spawns: an empty group is skipped"
    );
    for selector in &spawns.selectors {
        let index = usize::from(selector.group_index);
        let Some(group) = asset.spawns.get(index) else {
            continue;
        };
        let group_seed = group_seeds[index];
        let entity = (1..1000).contains(&selector.entity_type);
        // spawnConfig may change an entity's variant for types 14..=88.
        let alternates = entity && (14..=88).contains(&selector.entity_type);
        let chosen: Vec<usize> = (0..group.entries.len())
            .filter(|&at| {
                let entry = &group.entries[at];
                entry.entity_type == selector.entity_type
                    && entry.subtype == selector.subtype
                    && (alternates || entry.variant == selector.variant)
            })
            .collect();
        props::always!(
            !chosen.is_empty(),
            "room_spawns: a selector holds one of its group's entries",
            details()
        );
        let weighted: Vec<usize> = (0..group.entries.len())
            .filter(|&at| group.entries[at].weight > 0.0)
            .collect();
        props::always!(
            chosen.contains(&0) || chosen.iter().any(|at| weighted.contains(at)),
            "room_spawns: a weighted entry wins unless the first entry is the fallback",
            details()
        );
        // 0x5d49d0: the first entry whose running f32 weight sum strictly
        // exceeds the group draw scaled by the total, else the first entry.
        let total: f32 = group.entries.iter().map(|entry| entry.weight).sum();
        let target = rebirth_cleanroom::mt::uint_to_uniform(group_seed) * total;
        let mut cumulative = 0.0_f32;
        let pick = group
            .entries
            .iter()
            .position(|entry| {
                cumulative += entry.weight;
                cumulative > target
            })
            .unwrap_or(0);
        props::always!(
            chosen.contains(&pick),
            "room_spawns: the pick is 0x5d49d0's strict running-sum comparison against the group draw",
            details()
        );
        if let [only] = weighted[..] {
            props::always!(
                chosen.contains(&only) || chosen.contains(&0),
                "room_spawns: a lone weighted entry always wins",
                details()
            );
        }
        props::sometimes!(
            weighted.is_empty(),
            "room_spawns: a group without weight falls back to its first entry"
        );
        props::sometimes!(
            alternates
                && !chosen
                    .iter()
                    .any(|&at| group.entries[at].variant == selector.variant),
            "room_spawns: spawnConfig chooses an alternate variant"
        );
        let expected_category = if entity {
            SpawnCategory::Entity
        } else {
            SpawnCategory::Grid
        };
        let (gx, gy) = (group.x, group.y);
        props::always!(
            selector.category == expected_category
                && selector.group_seed == group_seed
                && selector.spawn_seed
                    == if entity {
                        next(group_seed, RNG35)
                    } else {
                        group_seed
                    }
                && u32::from(selector.grid_index) == (gx + 1) as u32 + (gy + 1) as u32 * grid_width
                && selector.position.x == 80.0 + f32::from(gx) * 40.0
                && selector.position.y == 160.0 + f32::from(gy) * 40.0,
            "room_spawns: a selector carries its group's cell, position and stream seeds",
            details()
        );
    }
}
