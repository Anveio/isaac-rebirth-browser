//! EvaluateItems' Familiar block 513E80: one CheckFamiliar 508D10 call per
//! Familiar variant, in the binary's order, with the count law each call
//! passes. Level-ups (Cube of Meat, Ball of Bandages) and the Full Key are
//! count laws here, so the reconcile itself stays generic.
use crate::player_items::PlayerItems;

/// How 513E80 computes one CheckFamiliar count.
#[derive(Clone, Copy, Debug)]
pub(crate) enum CountSource {
    /// Owned plus temporary count (5E3ED0) of the collectible.
    Owned(u32),
    /// The owned count clamped above at the bound, without the temporary
    /// count when `temporary` is false (Isaac's Heart 51461E).
    OwnedMin {
        item: u32,
        bound: i32,
        temporary: bool,
    },
    /// Dead Cat 5146EB: (owned + 8) / 9.
    DeadCat(u32),
    /// Presence of a temporary record (5E3D70).
    TempPresent(u32),
    /// The temporary count alone (5E3ED0).
    TempCount(u32),
    /// Cube of Meat / Ball of Bandages 514760: n = owned + temp. Levels
    /// 1..3 take count 1 when n % 4 equals the level; level 4 takes n / 4.
    Level { item: u32, level: u32 },
    /// Key Piece 1 26, Key Piece 2 27 and the Full Key 28 (514941): with
    /// 238 and 239 both held (515468) the calls are 26(0), 27(0), 28(1);
    /// otherwise 28(0), 26, 27. Expanded by `calls`.
    KeyPieces,
    /// Presence of a held trinket (HasTrinket 508FE0).
    Trinket(u32),
    /// A temporary trinket record (5E3DC0; Eve's Bird Foot, trinket 60).
    /// Temporary trinket records are not modeled, so the count is zero.
    TempTrinket,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RosterRow {
    pub variant: u32,
    pub count: CountSource,
    /// The RNG argument's owner: a collectible id, or a trinket id when
    /// `trinket_rng`.
    pub rng: u32,
    pub trinket_rng: bool,
}

const fn owned(variant: u32, item: u32) -> RosterRow {
    RosterRow {
        variant,
        count: CountSource::Owned(item),
        rng: item,
        trinket_rng: false,
    }
}

const fn row(variant: u32, count: CountSource, rng: u32) -> RosterRow {
    RosterRow {
        variant,
        count,
        rng,
        trinket_rng: false,
    }
}

const fn trinket(variant: u32, count: CountSource, rng: u32) -> RosterRow {
    RosterRow {
        variant,
        count,
        rng,
        trinket_rng: true,
    }
}

/// 513E80..514A2B in call order. The Cube of Meat and Ball of Bandages
/// branches call their levels in 1, 2, 3, 4 order on every path
/// (514B30/514B60/5154D8/5154F8 only change the counts).
pub(crate) const ROSTER: &[RosterRow] = &[
    owned(1, 8),
    owned(2, 113),
    owned(3, 88),
    owned(4, 99),
    owned(5, 100),
    owned(6, 95),
    owned(7, 67),
    owned(8, 188),
    owned(9, 163),
    owned(10, 167),
    owned(11, 174),
    owned(20, 131),
    owned(21, 94),
    owned(22, 96),
    owned(23, 98),
    owned(24, 144),
    owned(30, 128),
    owned(31, 57),
    owned(32, 112),
    owned(17, 155),
    owned(16, 170),
    owned(25, 178),
    owned(41, 11),
    owned(42, 187),
    owned(35, 172),
    owned(50, 264),
    owned(53, 267),
    owned(68, 206),
    owned(51, 265),
    owned(52, 266),
    owned(54, 268),
    owned(75, 321),
    owned(57, 271),
    owned(64, 278),
    owned(56, 270),
    owned(76, 319),
    owned(77, 320),
    owned(65, 279),
    owned(55, 269),
    owned(58, 272),
    owned(59, 273),
    owned(66, 280),
    owned(67, 281),
    owned(74, 322),
    row(
        62,
        CountSource::OwnedMin {
            item: 276,
            bound: 1,
            temporary: false,
        },
        276,
    ),
    owned(79, 318),
    owned(63, 277),
    owned(61, 275),
    row(
        33,
        CountSource::OwnedMin {
            item: 10,
            bound: 10,
            temporary: false,
        },
        10,
    ),
    row(40, CountSource::DeadCat(81), 81),
    row(14, CountSource::TempPresent(117), 117),
    row(60, CountSource::TempPresent(274), 274),
    row(44, CountSource::Level { item: 73, level: 1 }, 73),
    row(45, CountSource::Level { item: 73, level: 2 }, 73),
    row(46, CountSource::Level { item: 73, level: 3 }, 73),
    row(47, CountSource::Level { item: 73, level: 4 }, 73),
    row(
        69,
        CountSource::Level {
            item: 207,
            level: 1,
        },
        207,
    ),
    row(
        70,
        CountSource::Level {
            item: 207,
            level: 2,
        },
        207,
    ),
    row(
        71,
        CountSource::Level {
            item: 207,
            level: 3,
        },
        207,
    ),
    row(
        72,
        CountSource::Level {
            item: 207,
            level: 4,
        },
        207,
    ),
    row(48, CountSource::TempCount(107), 107),
    row(78, CountSource::TempCount(325), 325),
    row(28, CountSource::KeyPieces, 238),
    trinket(12, CountSource::Trinket(54), 54),
    trinket(13, CountSource::Trinket(57), 57),
    trinket(15, CountSource::TempTrinket, 60),
];

fn total(owner: &PlayerItems, item: u32) -> i32 {
    owner
        .owned_counts
        .get(item as usize)
        .copied()
        .unwrap_or(0)
        .wrapping_add(owner.temporary_count(item))
}

impl CountSource {
    /// The count argument 513E80 passes; 508D10 treats negatives as zero.
    pub(crate) fn count(self, owner: &PlayerItems) -> i32 {
        let owned = |item: u32| owner.owned_counts.get(item as usize).copied().unwrap_or(0);
        match self {
            Self::Owned(item) => total(owner, item),
            Self::OwnedMin {
                item,
                bound,
                temporary,
            } => {
                let count = if temporary {
                    total(owner, item)
                } else {
                    owned(item)
                };
                count.min(bound)
            }
            Self::DeadCat(item) => owned(item).wrapping_add(8) / 9,
            Self::TempPresent(item) => i32::from(owner.temporary_count(item) != 0),
            Self::TempCount(item) => owner.temporary_count(item),
            Self::Level { item, level } => {
                let n = total(owner, item);
                // 51477A..51479A: signed n / 4 and n % 4 (truncating).
                if level == 4 {
                    n / 4
                } else {
                    i32::from(n % 4 == level as i32)
                }
            }
            // Expanded into three calls by `calls`.
            Self::KeyPieces => 0,
            Self::Trinket(id) => i32::from(owner.trinket_slots.contains(&id)),
            Self::TempTrinket => 0,
        }
    }
}

/// One 508D10 call: variant, count and the RNG owner.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Call {
    pub variant: u32,
    pub count: i32,
    pub rng: u32,
    pub trinket_rng: bool,
}

/// Every 508D10 call 513E80 makes for this inventory, in order.
pub(crate) fn calls(owner: &PlayerItems) -> Vec<Call> {
    let mut calls = Vec::with_capacity(ROSTER.len() + 2);
    for row in ROSTER {
        if let CountSource::KeyPieces = row.count {
            let held = |item: usize| owner.owned_counts.get(item).copied().unwrap_or(0) > 0;
            let key = |variant, count, rng| Call {
                variant,
                count,
                rng,
                trinket_rng: false,
            };
            if held(238) && held(239) {
                calls.push(key(26, 0, 238));
                calls.push(key(27, 0, 239));
                calls.push(key(28, 1, 238));
            } else {
                calls.push(key(28, 0, 238));
                calls.push(key(26, total(owner, 238), 238));
                calls.push(key(27, total(owner, 239), 239));
            }
            continue;
        }
        calls.push(Call {
            variant: row.variant,
            count: row.count.count(owner),
            rng: row.rng,
            trinket_rng: row.trinket_rng,
        });
    }
    calls
}
