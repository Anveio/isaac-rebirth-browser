//! Source452BD0/453140 bucket insertion and first-encounter pair order.
//! Callers supply actual source AABBs in current manager order. This primitive
//! does not replace current-circle overlap, collision classes or callbacks.
use crate::motion::WorldVector;
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
pub(crate) struct Bounds {
    pub(crate) id: u32,
    pub(crate) position: WorldVector,
    ///Actual radius times collision scale, rotated by the concrete owner if
    ///needed. Sprite scale and visual bounds are not substitutes.
    pub(crate) half_extents: WorldVector,
}

///454110's six partitions share1200x720 and cell80;454070 rounds the
///dimensions with CVTTSS2SI(extent/cell+.5), yielding15x9 source buckets.
///452BD0 truncates each signed coordinate quotient before flattening. A row
///or column outside the nominal rectangle is not clamped independently.
fn insert(buckets: &mut [Vec<u32>], body: Bounds) -> Result<(), String> {
    if !body.position.is_finite()
        || !body.half_extents.is_finite()
        || body.half_extents.x < 0.0
        || body.half_extents.y < 0.0
    {
        return Err("contact partition requires finite actual source bounds".into());
    }
    let low_x = body.position.x - body.half_extents.x;
    let high_x = body.position.x + body.half_extents.x;
    let low_y = body.position.y - body.half_extents.y;
    let high_y = body.position.y + body.half_extents.y;
    //452D54/452D63 skip a rounded zero-size AABB before signed CVTT.
    if low_x >= high_x || low_y >= high_y {
        return Ok(());
    }
    let min_x = (low_x / 80.0_f32) as i32;
    let max_x = (high_x / 80.0_f32) as i32;
    let min_y = (low_y / 80.0_f32) as i32;
    let max_y = (high_y / 80.0_f32) as i32;
    if max_x.checked_sub(min_x).is_none_or(|span| span > 135)
        || max_y.checked_sub(min_y).is_none_or(|span| span > 135)
    {
        return Err("contact partition bounds exceed the admitted source extent".into());
    }
    for row in min_y..=max_y {
        for column in min_x..=max_x {
            let index = row
                .checked_mul(15)
                .and_then(|base| base.checked_add(column));
            if let Some(bucket) = index
                .and_then(|i| usize::try_from(i).ok())
                .and_then(|i| buckets.get_mut(i))
            {
                //452E36..452E4B caps each independently populated bucket40.
                if bucket.len() < 40 {
                    bucket.push(body.id);
                }
            }
        }
    }
    Ok(())
}

///Each side retains manager insertion order. Ascending buckets, then left
///entries, then right entries, followed by453257's actual pair deduplication.
///A concrete callback must recheck its current geometry after earlier pairs.
pub(crate) fn pairs(left: &[Bounds], right: &[Bounds]) -> Result<Vec<(u32, u32)>, String> {
    let mut left_buckets = vec![Vec::new(); 135];
    let mut right_buckets = vec![Vec::new(); 135];
    for &body in left {
        insert(&mut left_buckets, body)?;
    }
    for &body in right {
        insert(&mut right_buckets, body)?;
    }
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for (left_bucket, right_bucket) in left_buckets.iter().zip(&right_buckets) {
        for &a in left_bucket {
            for &b in right_bucket {
                if seen.insert((a, b)) {
                    result.push((a, b));
                }
            }
        }
    }
    Ok(result)
}

///453340 pairs the members of ONE partition (the NPC self-pairs of4545D0):
///ascending buckets, then each bucket's entries i<j in insertion order. An
///entry never pairs with itself, and a pair already met in an earlier
///bucket is skipped (its Entity+C bitset). The callback receives the pair
///in bucket order and must recheck current geometry after earlier pairs.
pub(crate) fn self_pairs(members: &[Bounds]) -> Result<Vec<(u32, u32)>, String> {
    let mut buckets = vec![Vec::new(); 135];
    for &body in members {
        insert(&mut buckets, body)?;
    }
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for bucket in &buckets {
        for (at, &a) in bucket.iter().enumerate() {
            for &b in &bucket[at + 1..] {
                if a != b && seen.insert((a.min(b), a.max(b))) {
                    result.push((a, b));
                }
            }
        }
    }
    Ok(result)
}

///452EE0 queries one partition with a single actor (the NPC/Bomb partition
///against each Player): the actor's current AABB, rows then columns, each
///bucket's entries in insertion order, every member once and never the actor
///itself. A flattened index outside the buckets is skipped, not clamped.
pub(crate) fn query(members: &[Bounds], probe: Bounds) -> Result<Vec<u32>, String> {
    let mut buckets = vec![Vec::new(); 135];
    for &body in members {
        insert(&mut buckets, body)?;
    }
    if !probe.position.is_finite() || !probe.half_extents.is_finite() {
        return Err("contact query requires finite actual source bounds".into());
    }
    let cell = |value: f32| (value / 80.0_f32) as i32;
    let (min_y, max_y) = (
        cell(probe.position.y - probe.half_extents.y),
        cell(probe.position.y + probe.half_extents.y),
    );
    let (min_x, max_x) = (
        cell(probe.position.x - probe.half_extents.x),
        cell(probe.position.x + probe.half_extents.x),
    );
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for row in min_y..=max_y {
        for column in min_x..=max_x {
            let Some(bucket) = row
                .checked_mul(15)
                .and_then(|base| base.checked_add(column))
                .and_then(|index| usize::try_from(index).ok())
                .and_then(|index| buckets.get(index))
            else {
                continue;
            };
            for &id in bucket {
                if id != probe.id && seen.insert(id) {
                    result.push(id);
                }
            }
        }
    }
    Ok(result)
}

/// Membership of the last actual4545D0 mask8 partition rebuild. The local
/// indices are a bijection of source Entity+C dedup keys, never actor roles
/// or cross-game identities. Query reads current geometry separately.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct QuerySnapshot {
    pub(crate) game_counter: u32,
    actors: Vec<crate::actors::ActorRef>,
    buckets: Vec<Vec<u32>>,
}

impl QuerySnapshot {
    ///454110 constructor owns an empty manager118 before any rebuild.
    pub(crate) fn empty() -> Self {
        Self {
            game_counter: 0,
            actors: Vec::new(),
            buckets: vec![Vec::new(); 135],
        }
    }

    pub(crate) fn rebuild(
        game_counter: u32,
        members: &[(crate::actors::ActorRef, Bounds)],
    ) -> Result<Self, String> {
        let mut result = Self::empty();
        result.game_counter = game_counter;
        for &(actor, mut bounds) in members {
            //ActorRef lacks an ordering contract; the supplied current manager
            //position supplies the local dense key and equality owns uniqueness.
            if result.actors.contains(&actor) {
                return Err("query rebuild repeated an actual manager actor".into());
            }
            //The supported snapshot must retain source signed CVTT/flatten
            //semantics; extreme legacy coordinates are unknown, not a Rust
            //saturating-cast or checked-arithmetic empty-bucket substitute.
            for (position, half) in [
                (bounds.position.x, bounds.half_extents.x),
                (bounds.position.y, bounds.half_extents.y),
            ] {
                for value in [position - half, position + half] {
                    let q = value / 80.0;
                    if !q.is_finite() || !(-2147483648.0..2147483648.0).contains(&q) {
                        return Err(
                            "query rebuild signed cell owner is outside supported bounds".into(),
                        );
                    }
                }
            }
            let min_x = ((bounds.position.x - bounds.half_extents.x) / 80.0) as i32;
            let max_x = ((bounds.position.x + bounds.half_extents.x) / 80.0) as i32;
            for row in [
                ((bounds.position.y - bounds.half_extents.y) / 80.0) as i32,
                ((bounds.position.y + bounds.half_extents.y) / 80.0) as i32,
            ] {
                if row
                    .checked_mul(15)
                    .and_then(|base| base.checked_add(min_x))
                    .is_none()
                    || row
                        .checked_mul(15)
                        .and_then(|base| base.checked_add(max_x))
                        .is_none()
                {
                    return Err("query rebuild flattened index would overflow".into());
                }
            }
            bounds.id = u32::try_from(result.actors.len())
                .map_err(|_| "query rebuild dense index overflow")?;
            result.actors.push(actor);
            insert(&mut result.buckets, bounds)?;
        }
        Ok(result)
    }

    pub(crate) fn contains(&self, actor: crate::actors::ActorRef) -> bool {
        self.actors.contains(&actor)
    }

    ///4534E0 scans signed flattened cells row first, then each bucket's
    ///admission order; Entity+C suppresses repeats only after first encounter.
    pub(crate) fn query(
        &self,
        position: WorldVector,
        radius: f32,
    ) -> Result<Vec<crate::actors::ActorRef>, String> {
        if !position.is_finite() || !radius.is_finite() || radius < 0.0 {
            return Err("query requires finite actual position and radius".into());
        }
        let convert = |value: f32| -> Result<i32, String> {
            let q = value / 80.0;
            if !q.is_finite() || !(-2147483648.0..2147483648.0).contains(&q) {
                return Err("query signed source cell index is outside owned bounds".into());
            }
            Ok(q as i32)
        };
        let min_x = convert(position.x - radius)?;
        let max_x = convert(position.x + radius)?;
        let min_y = convert(position.y - radius)?;
        let max_y = convert(position.y + radius)?;
        if max_x.checked_sub(min_x).is_none_or(|n| n > 135)
            || max_y.checked_sub(min_y).is_none_or(|n| n > 135)
        {
            return Err("query source extent exceeds the admitted partition".into());
        }
        if self.buckets.len() != 135
            || self.actors.len() > u32::MAX as usize
            || self
                .actors
                .iter()
                .enumerate()
                .any(|(i, a)| self.actors[..i].contains(a))
            || self
                .buckets
                .iter()
                .any(|b| b.len() > 40 || b.iter().any(|&key| key as usize >= self.actors.len()))
        {
            return Err("query snapshot restored membership is invalid".into());
        }
        for row in [min_y, max_y] {
            if row
                .checked_mul(15)
                .and_then(|base| base.checked_add(min_x))
                .is_none()
                || row
                    .checked_mul(15)
                    .and_then(|base| base.checked_add(max_x))
                    .is_none()
            {
                return Err("query flattened signed index would overflow".into());
            }
        }
        let mut seen = vec![false; self.actors.len()];
        let mut result = Vec::new();
        for row in min_y..=max_y {
            for column in min_x..=max_x {
                let index = row
                    .checked_mul(15)
                    .and_then(|base| base.checked_add(column));
                if let Some(bucket) = index
                    .and_then(|i| usize::try_from(i).ok())
                    .and_then(|i| self.buckets.get(i))
                {
                    for &key in bucket {
                        let key = key as usize;
                        if !seen[key] {
                            seen[key] = true;
                            result.push(self.actors[key]);
                        }
                    }
                }
            }
        }
        Ok(result)
    }
}
