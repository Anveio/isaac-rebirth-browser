//! Source445c60 samples every category before the first real effect birth.
use crate::mt::Mt19937;

#[derive(Clone, Copy)]
pub enum AmountScale {
    Ordinary,
    Challenge23,
}

/// Slots are the source stack+20..38: blood, bone, gut, eye, unused, poop,
/// special63. Large16 changes birth behavior; it never samples a count.
pub fn sample(
    amount: u32,
    flags: u32,
    scale: AmountScale,
    shared: &mut Mt19937,
) -> Result<[u32; 7], String> {
    let mut counts = [0; 7];
    if amount == 0 {
        return Ok(counts);
    }
    let divisor = (flags & 0x27).count_ones();
    if flags & !0x7f != 0 || divisor == 0 {
        return Err("gib counts require mapped flags and a real source category divisor".into());
    }
    let amount = amount
        .checked_mul(match scale {
            AmountScale::Ordinary => 1,
            AmountScale::Challenge23 => 8,
        })
        .filter(|amount| *amount <= i32::MAX as u32 - 1)
        .ok_or("gib amount exceeds the source signed domain")?;
    //445cde..445d36: B2 is consumed even when a category later samples zero.
    let budget = (amount + shared.bounded(2)) / divisor + 1;
    if flags & 1 != 0 {
        counts[0] = shared.bounded(budget) + shared.bounded(budget);
    }
    if flags & 2 != 0 {
        counts[1] = budget / 2 + shared.bounded(budget / 2);
    }
    if flags & 4 != 0 {
        counts[2] = shared.bounded(budget);
    }
    if flags & 8 != 0 {
        counts[3] = shared.bounded(budget.min(3));
    }
    if flags & 32 != 0 {
        counts[5] = shared.bounded(budget);
    }
    if flags & 64 != 0 {
        counts[6] = shared.bounded(budget);
    }
    Ok(counts)
}
