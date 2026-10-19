//! The run-owned ItemPool manager (`engine/src/items.rs`, `docs/item-pools.md`).
//!
//! Game runs draw only the few pedestals they reach and never exhaust a pool,
//! so this unit drives the manager directly: pedestal-style draws, error-room
//! routing, Rosary's Bible modifier, and drains through every retry, treasure
//! fallback and Breakfast boundary. Oracles are the documented contracts:
//! selection never repeats a collectible, consumes no manager or shared
//! stream, depletes monotonically and continues identically after a save.
use crate::entropy::Entropy;
use crate::harness::Case;
use crate::props;
use rebirth_cleanroom::items::{self, ItemPools, POOL_DEFINITIONS};
use rebirth_cleanroom::mt::Mt19937;
use serde_json::json;
use std::collections::BTreeSet;

const ACHIEVEMENTS: usize = 179;
const BREAKFAST: u32 = 25;
const BIBLE: u32 = 33;
const TREASURE: usize = 0;
const ROOM_TYPES: u32 = 23;
/// Room types the floor generator produces that own an authored pool.
const NAMESAKE_ROUTES: &[(u32, &str)] = &[
    (2, "shop"),
    (4, "treasure"),
    (5, "boss"),
    (7, "secret"),
    (10, "curse"),
    (12, "library"),
];
/// More draws than the catalog has collectibles: past this every pool must
/// have reached Breakfast.
const DRAIN_LIMIT: usize = 400;

pub fn run(case: &mut Case) {
    let e = &mut *case.entropy;
    let achievements = achievements(e);
    let seed = e.nonzero_u32();
    rejected_construction(e, seed);
    let Ok(mut pools) = ItemPools::new(seed, &achievements) else {
        props::unreachable!(
            "item_pools: valid construction fails",
            json!({ "seed": seed })
        );
        return;
    };
    constructed(&pools, &achievements, seed);
    trinket_selection(&pools, &achievements);

    let mut shared = Mt19937::new(e.u32());
    let mut drawn = BTreeSet::new();
    for _ in 0..(case.steps / 8).max(64) {
        match e.below(16) {
            0 => {
                // Rosary (72) is the engine's only special-counter writer.
                for pool in &mut pools.pools {
                    pool.special_counter = pool.special_counter.wrapping_add(5);
                }
            }
            1 => continuation(e, &pools),
            2 => drain(e, &mut pools, &mut drawn),
            3 | 4 => {
                let room_type = e.below(u64::from(ROOM_TYPES) + 2) as u32;
                let pool = routed(&pools, room_type, &mut shared);
                let seed = selection_seed(e);
                draw(&mut pools, &mut drawn, pool, true, seed);
            }
            5 => {
                let pool = invalid_pool(e);
                let seed = selection_seed(e);
                draw(&mut pools, &mut drawn, pool, e.chance(1, 2), seed);
            }
            _ => {
                let pool = e.below(pools.pools.len() as u64) as i32;
                let seed = selection_seed(e);
                draw(&mut pools, &mut drawn, pool, e.chance(3, 4), seed);
            }
        }
    }
    let saved = serde_json::to_value(&pools).expect("item pools serialize");
    props::always!(
        serde_json::from_value::<ItemPools>(saved).ok().as_ref() == Some(&pools),
        "item_pools: managers round-trip through JSON"
    );
}

///Exercise the manager contract beyond the few selections reached by a run.
///These are engine invariants, not Original evidence or Init350 body proof.
fn trinket_selection(initial: &ItemPools, achievements: &[bool]) {
    let mut pools = initial.clone();
    let mut selected = BTreeSet::new();
    let count = pools.trinkets.as_ref().expect("fresh trinket pool").count;
    let batch = vec![0; count as usize];
    let before = pools.clone();
    props::always!(
        items::preflight_trinket_init_batch(Some(&pools), &batch, None).is_ok() && pools == before,
        "item_pools: a complete available trinket batch admits without previewing RNG"
    );
    let across_reset = vec![0; count as usize * 2 + 1];
    props::always!(
        items::preflight_trinket_init_batch(Some(&pools), &across_reset, Some(achievements))
            .is_ok()
            && items::preflight_trinket_init_batch(Some(&pools), &across_reset, None).is_err()
            && pools == before,
        "item_pools: whole-batch admission includes every reached profile reset before mutation"
    );
    for _ in 0..count {
        let before = pools.clone();
        let mut manager = before.secondary_rng.expect("fresh manager RNG");
        manager.raw();
        let id = pools.get_trinket(None);
        let expected = id.as_ref().ok().copied().filter(|id| {
            items::trinket_definition(*id).is_some() && *id != 47 && selected.insert(*id)
        });
        props::always!(
            expected.is_some(),
            "item_pools: trinket selection returns each available typed config at most once",
            json!({ "id": id, "count": count, "selected": selected })
        );
        let pool = pools.trinkets.as_ref().expect("selected pool");
        let previous = before.trinkets.as_ref().expect("previous pool");
        props::always!(
            pools.secondary_rng == Some(manager)
                && pool.count == previous.count - 1
                && pool.eligible == previous.eligible
                && pool
                    .available
                    .iter()
                    .zip(&previous.available)
                    .filter(|(a, b)| a != b)
                    .count()
                    == 1
                && expected.is_some_and(|id| !pool.available[id as usize])
                && pools.root_rng == before.root_rng
                && pools.pools == before.pools
                && pools.pill_effects == before.pill_effects,
            "item_pools: trinket Get consumes one manager draw and removes one availability byte"
        );
    }
    let before = pools.clone();
    props::always!(
        pools.get_trinket(None).is_err() && pools == before,
        "item_pools: depleted trinket reset requires the actual profile without partial mutation"
    );
    props::always!(
        pools
            .get_trinket(Some(achievements))
            .is_ok_and(|id| selected.contains(&id))
            && pools
                .trinkets
                .as_ref()
                .is_some_and(|pool| pool.count == count - 1),
        "item_pools: depleted trinket pool refills from the reached current profile"
    );
    let before = pools.clone();
    pools.trinkets = None;
    pools.secondary_rng = None;
    let unknown = pools.clone();
    props::always!(
        pools.get_trinket(Some(achievements)).is_err()
            && pools == unknown
            && items::preflight_trinket_init_batch(None, &[41], None).is_ok()
            && items::preflight_trinket_init_batch(Some(&pools), &[0, 41], Some(achievements))
                .is_err(),
        "item_pools: unknown selection history stays unknown while explicit Init bypasses Get"
    );
    //A retained positive count can disagree with current eligibility. True0
    //is a source result, while the body preflight refuses its card tail.
    pools = before;
    let pool = pools.trinkets.as_mut().expect("retained pool");
    pool.count = 1;
    pool.eligible = Some(vec![false; 62]);
    let previous = pools.clone();
    props::always!(
        items::preflight_trinket_init_batch(Some(&pools), &[0], None).is_err()
            && pools == previous
            && pools.get_trinket(None) == Ok(0)
            && pools.trinkets == previous.trinkets,
        "item_pools: Get0 preserves the count and availability instead of inventing a child"
    );
    pools = initial.clone();
    pools.secondary_rng.as_mut().expect("manager RNG").state = 0;
    let first = pools.trinkets.as_ref().and_then(|pool| {
        pool.available
            .iter()
            .zip(pool.eligible.as_ref()?)
            .position(|(available, eligible)| *available && *eligible)
    });
    props::always!(
        pools.get_trinket(None).ok().map(|id| id as usize) == first
            && pools.secondary_rng.is_some_and(|rng| rng.state == 0),
        "item_pools: a real zero manager word follows the fallback without seed clamping"
    );
    pools = initial.clone();
    pools.trinkets.as_mut().expect("pool").count = 1;
    let before = pools.clone();
    props::always!(
        items::preflight_trinket_init_batch(Some(&pools), &[0, 0], None).is_err()
            && items::preflight_trinket_init_batch(Some(&pools), &[0, 0], Some(achievements))
                .is_ok()
            && pools == before,
        "item_pools: a saved count below eligibility reaches reset even with available bytes left"
    );
    let pool = pools.trinkets.as_mut().expect("pool");
    pool.count = 2;
    pool.eligible = Some((0..62).map(|id| id == 41).collect());
    props::always!(
        items::preflight_trinket_init_batch(Some(&pools), &[0, 0], Some(achievements)).is_err()
            && pools.get_trinket(None) == Ok(41)
            && pools.get_trinket(None) == Ok(0)
            && pools.trinkets.as_ref().is_some_and(|pool| pool.count == 1),
        "item_pools: a saved count above eligibility reaches true0 before the next reset"
    );
}

/// Fresh profiles hold no achievements; the rest cover partial and complete
/// unlock tables so eligibility redraws see every density.
fn achievements(e: &mut Entropy) -> Vec<bool> {
    match e.below(4) {
        0 => vec![false; ACHIEVEMENTS],
        1 => vec![true; ACHIEVEMENTS],
        _ => {
            let density = e.range(1, 15);
            (0..ACHIEVEMENTS)
                .map(|_| e.below(16) < u64::from(density))
                .collect()
        }
    }
}

fn rejected_construction(e: &mut Entropy, seed: u32) {
    let length = loop {
        let length = e.index(2 * ACHIEVEMENTS);
        if length != ACHIEVEMENTS {
            break length;
        }
    };
    props::always!(
        ItemPools::new(0, &[false; ACHIEVEMENTS]).is_err()
            && ItemPools::new(seed, &vec![false; length]).is_err(),
        "item_pools: construction rejects a zero seed and malformed achievement tables",
        json!({ "seed": seed, "length": length })
    );
}

fn constructed(pools: &ItemPools, achievements: &[bool], seed: u32) {
    props::always!(
        ItemPools::new(seed, achievements).ok().as_ref() == Some(pools),
        "item_pools: construction is a pure function of its seed and achievements",
        json!({ "seed": seed })
    );
    let authored = pools.pools.len() == POOL_DEFINITIONS.len()
        && pools
            .pools
            .iter()
            .zip(POOL_DEFINITIONS)
            .all(|(pool, definition)| {
                pool.id == definition.id
                    && pool.special_counter == 0
                    && pool.entries.len() == definition.entries.len()
                    && pool
                        .entries
                        .iter()
                        .zip(definition.entries)
                        .all(|(entry, authored)| {
                            let config = items::definition(entry.id);
                            entry.id == authored.id
                                && entry.base_weight == authored.weight
                                && entry.current_weight == authored.weight
                                && config.is_some_and(|config| {
                                    entry.special == config.special
                                        && entry.eligible
                                            == (config.achievement < 0
                                                || achievements[config.achievement as usize])
                                })
                        })
            })
        && pools.special_factor == 1.0
        && !pools.removed.iter().any(|removed| *removed)
        && !pools.selected.iter().any(|selected| *selected);
    props::always!(
        authored,
        "item_pools: construction mirrors the authored pools and the profile's unlocks",
        json!({ "seed": seed })
    );
    // 583040 samples each of the nine pill colors without replacement.
    let effects: BTreeSet<i32> = pools.pill_effects[1..].iter().copied().collect();
    props::always!(
        pools.pill_effects[0] == -1
            && effects.len() == 9
            && effects.iter().all(|effect| (0..28).contains(effect)),
        "item_pools: pill colors take distinct effects",
        json!({ "pillEffects": pools.pill_effects })
    );
}

/// Error rooms (type 3) pick a pool with one shared draw; every other room
/// type routes statically and consumes none.
fn routed(pools: &ItemPools, room_type: u32, shared: &mut Mt19937) -> i32 {
    let mut expected = shared.clone();
    let pool = pools.room_pool(room_type, shared);
    if room_type == 3 {
        expected.next_u32();
    }
    props::always!(
        *shared == expected,
        "item_pools: only error rooms consume a shared draw to route",
        json!({ "roomType": room_type })
    );
    props::always!(
        usize::try_from(pool).is_ok_and(|pool| pool < pools.pools.len()),
        "item_pools: room routing names an authored pool",
        json!({ "roomType": room_type, "pool": pool })
    );
    if let Some((_, name)) = NAMESAKE_ROUTES.iter().find(|(kind, _)| *kind == room_type) {
        props::always!(
            usize::try_from(pool)
                .ok()
                .and_then(|pool| pools.pools.get(pool))
                .is_some_and(|pool| pool.name == *name),
            "item_pools: special rooms draw from their namesake pools",
            json!({ "roomType": room_type, "pool": pool })
        );
    }
    pool
}

fn invalid_pool(e: &mut Entropy) -> i32 {
    match e.below(4) {
        0 => -1,
        1 => -2 - e.below(1 << 20) as i32,
        2 => i32::MAX - e.below(4) as i32,
        _ => POOL_DEFINITIONS.len() as i32 + e.below(64) as i32,
    }
}

fn selection_seed(e: &mut Entropy) -> u32 {
    if e.chance(1, 32) {
        0
    } else {
        e.nonzero_u32()
    }
}

/// One selection with every per-draw property; returns the collectible.
fn draw(
    pools: &mut ItemPools,
    drawn: &mut BTreeSet<u32>,
    pool: i32,
    decrease: bool,
    seed: u32,
) -> Option<u32> {
    let before = pools.clone();
    let result = pools.collectible(pool, decrease, seed);
    let mut replayed = before.clone();
    props::always!(
        replayed.collectible(pool, decrease, seed) == result && replayed == *pools,
        "item_pools: selection is a pure function of the manager and its inputs",
        json!({ "pool": pool, "seed": seed })
    );
    let valid_pool = usize::try_from(pool)
        .ok()
        .filter(|pool| *pool < before.pools.len());
    let details = json!({ "pool": pool, "seed": seed, "decrease": decrease, "result": result });
    if seed == 0 || (pool != -1 && valid_pool.is_none()) {
        props::always!(
            result.is_err() && *pools == before,
            "item_pools: a zero seed or unknown pool is rejected without mutation",
            details.clone()
        );
        return None;
    }
    let Some(pool) = valid_pool else {
        props::always!(
            result == Ok(0) && *pools == before,
            "item_pools: pool -1 yields no collectible without mutation",
            details.clone()
        );
        return None;
    };
    let Ok(id) = result else {
        props::unreachable!("item_pools: a valid selection fails", details);
        return None;
    };
    let in_pool = |pool: usize| {
        before.pools[pool]
            .entries
            .iter()
            .any(|entry| entry.id == id)
    };
    props::always!(
        in_pool(pool) || in_pool(TREASURE) || id == BREAKFAST || id == BIBLE,
        "item_pools: a draw comes from its pool, the treasure fallback, Breakfast or the Bible",
        details.clone()
    );
    props::always!(
        id == BREAKFAST || (!drawn.contains(&id) && !before.selected[id as usize]),
        "item_pools: no collectible is offered twice except Breakfast",
        details.clone()
    );
    bible_override(&before, pools, pool, decrease, seed, id);
    depletion(&before, pools, decrease, &details);
    if id != BREAKFAST {
        drawn.insert(id);
    }
    let entry = before.pools[pool]
        .entries
        .iter()
        .find(|entry| entry.id == id);
    props::sometimes!(
        pool != TREASURE && entry.is_none() && id != BIBLE && in_pool(TREASURE),
        "item_pools: a draw falls back to the treasure pool"
    );
    props::sometimes!(
        entry.is_some_and(|entry| !entry.eligible),
        "item_pools: a locked collectible survives every eligibility redraw"
    );
    props::sometimes!(
        entry.is_some_and(|entry| entry.special),
        "item_pools: a special collectible is offered"
    );
    Some(id)
}

/// Rosary's special counter is consulted only after selection, so the same
/// draw with every counter at zero yields the candidate the Bible may replace.
/// The candidate is marked before the Bible check: an override marks both.
/// Breakfast enters the boss, beggar and dungeon pools as an ordinary entry.
fn bible_override(
    before: &ItemPools,
    after: &ItemPools,
    pool: usize,
    decrease: bool,
    seed: u32,
    id: u32,
) {
    let mut expected = before.clone();
    for counterfactual in &mut expected.pools {
        counterfactual.special_counter = 0;
    }
    let candidate = expected.collectible(pool as i32, decrease, seed);
    let replaced = candidate != Ok(id);
    for (expected, before) in expected.pools.iter_mut().zip(&before.pools) {
        expected.special_counter = before.special_counter;
    }
    if replaced {
        expected.selected[BIBLE as usize] = true;
    }
    props::always!(
        (!replaced || id == BIBLE) && *after == expected,
        "item_pools: the special counter can only replace a draw with the Bible",
        json!({ "pool": pool, "seed": seed, "candidate": candidate, "result": id })
    );
    props::always!(
        !replaced
            || before.pools[pool].special_counter > 0
            || before.pools[TREASURE].special_counter > 0,
        "item_pools: only Rosary's special counter lets the Bible replace a draw",
        json!({ "pool": pool, "seed": seed })
    );
    let marked: Vec<u32> = (0..after.selected.len() as u32)
        .filter(|id| after.selected[*id as usize] && !before.selected[*id as usize])
        .collect();
    let offered = match id {
        _ if replaced => candidate.is_ok_and(|candidate| {
            let mut both = vec![candidate, BIBLE];
            both.sort_unstable();
            marked == both
        }),
        // Fallback Breakfast is unmarked; only an authored entry marks it.
        BREAKFAST => {
            marked.is_empty()
                || (marked == [BREAKFAST]
                    && before.pools[pool]
                        .entries
                        .iter()
                        .any(|entry| entry.id == BREAKFAST))
        }
        _ => marked == [id],
    };
    props::always!(
        offered,
        "item_pools: a draw marks the offered collectible and any the Bible replaced",
        json!({ "pool": pool, "seed": seed, "result": id, "marked": marked })
    );
    props::sometimes!(replaced, "item_pools: the Bible replaces a draw");
}

/// Weights only fall, maps only fill, and draws without depletion leave
/// every weight as it was.
fn depletion(before: &ItemPools, after: &ItemPools, decrease: bool, details: &serde_json::Value) {
    let entries = || {
        after
            .pools
            .iter()
            .zip(&before.pools)
            .flat_map(|(after, before)| after.entries.iter().zip(&before.entries))
    };
    let maps_fill = after
        .removed
        .iter()
        .zip(&before.removed)
        .chain(after.selected.iter().zip(&before.selected))
        .all(|(after, before)| *after || !*before);
    props::always!(
        maps_fill
            && after.removed.len() == before.removed.len()
            && after.selected.len() == before.selected.len()
            && entries().all(|(after, before)| {
                after.current_weight <= before.current_weight
                    && after.current_weight >= 0.0
                    && after.base_weight == before.base_weight
            }),
        "item_pools: weights only fall and the removed and selected maps only fill",
        details.clone()
    );
    if !decrease {
        props::always!(
            after.removed == before.removed
                && entries().all(|(after, before)| after.current_weight == before.current_weight),
            "item_pools: selection without depletion leaves every weight unchanged",
            details.clone()
        );
    }
    props::always!(
        (0.01..=1.0).contains(&after.special_factor)
            && after.special_factor <= before.special_factor,
        "item_pools: the special-item factor decays toward its floor",
        json!({ "before": before.special_factor, "after": after.special_factor })
    );
    props::sometimes!(
        after.special_factor < before.special_factor,
        "item_pools: offering a special collectible lowers the special factor"
    );
}

/// Drains one pool: every draw is fresh until the pool, then the treasure
/// fallback, are exhausted and only Breakfast remains.
fn drain(e: &mut Entropy, pools: &mut ItemPools, drawn: &mut BTreeSet<u32>) {
    let pool = e.below(pools.pools.len() as u64) as i32;
    let decrease = e.chance(1, 2);
    let mut last = None;
    let mut draws = 0;
    while draws < DRAIN_LIMIT {
        draws += 1;
        let seed = e.nonzero_u32();
        last = draw(pools, drawn, pool, decrease, seed);
        if last == Some(BREAKFAST) {
            break;
        }
    }
    props::always!(
        last == Some(BREAKFAST),
        "item_pools: a drained pool reaches Breakfast within the catalog",
        json!({ "pool": pool, "draws": draws, "last": last })
    );
    props::sometimes!(
        last == Some(BREAKFAST),
        "item_pools: a drained pool yields Breakfast"
    );
}

/// A serialized manager is a complete continuation of the selections.
fn continuation(e: &mut Entropy, pools: &ItemPools) {
    let restored = serde_json::to_string(pools)
        .ok()
        .and_then(|saved| serde_json::from_str::<ItemPools>(&saved).ok());
    let Some(mut restored) = restored else {
        props::unreachable!("item_pools: a manager fails to serialize or restore");
        return;
    };
    let mut original = pools.clone();
    let mut identical = true;
    for _ in 0..e.range(1, 32) {
        let pool = e.below(original.pools.len() as u64) as i32;
        let decrease = e.chance(3, 4);
        let seed = e.nonzero_u32();
        identical &= original.collectible(pool, decrease, seed)
            == restored.collectible(pool, decrease, seed);
    }
    props::always!(
        identical && original == restored,
        "item_pools: a serialized manager continues identically"
    );
}
