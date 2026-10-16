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
