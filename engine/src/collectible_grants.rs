//! Collectible grant rows for AddCollectible 52CC90, one row per collectible
//! like `active_items::USES`. Any collectible can be granted (pedestals,
//! Challenge starts, D4/D100 rerolls).
//!
//! `tools/original/item-xrefs.py` resolves 52CC90's id tests for every
//! collectible and lists the Player fields each reachable path writes
//! (`addCollectible` in references/isaac/foundation/items/item-xrefs.json).
//! Only 240 (Experimental Treatment) and 258 (Missing No.) write the
//! persistent stat input words D28..D3C. No other Add path writes them, C70,
//! D40/D58, 13BC/13C0 or the pill-effect words 1388+, and the callees that
//! only one collectible reaches write none of them (checked one call deep).
//! The active charge words are written only for config kind 3 (52D19B). So
//! every other collectible keeps every word, and a row here names only a
//! collectible whose Add path writes some.
//!
//! The stat outputs themselves come from the cache Evaluate511F10 callbacks
//! over owned counts, so a row only covers the Add event, not the stats.

/// Player+D28/D44/13BC/13C0, the MoveSpeed inputs.
pub(crate) const MOVE_SPEED: u16 = 1;
/// Player+D34, the Range input.
pub(crate) const RANGE: u16 = 2;
/// Player+D2C, the FireDelay input.
pub(crate) const FIRE_DELAY: u16 = 4;
/// Player+D30, the Damage input.
pub(crate) const DAMAGE: u16 = 8;
/// Player+D38, the ShotSpeed input.
pub(crate) const SHOT_SPEED: u16 = 16;
/// The attack aim rotation C70.
pub(crate) const AIM_C70: u16 = 32;
/// The Luck inputs D3C/D40/D58.
pub(crate) const LUCK: u16 = 64;
/// The pill-effect counter words 1388+ and the count/record query stores.
pub(crate) const QUERY_STORES: u16 = 128;
/// The active charge words D74/D7C/D80/D8C. 52CC90 writes them only on its
/// config kind 3 (active) path at 52D19B, so a passive Add keeps them.
pub(crate) const ACTIVE_CHARGES: u16 = 256;

const ALL: u16 = MOVE_SPEED
    | RANGE
    | FIRE_DELAY
    | DAMAGE
    | SHOT_SPEED
    | AIM_C70
    | LUCK
    | QUERY_STORES
    | ACTIVE_CHARGES;

#[derive(Clone, Copy, Debug)]
pub(crate) struct GrantRow {
    pub id: u32,
    /// The input words this Add path writes; every other word is kept.
    pub writes: u16,
}

const fn row(id: u32, writes: u16) -> GrantRow {
    GrantRow { id, writes }
}

pub(crate) const ROWS: &[GrantRow] = &[
    // 52CC90 writes D28..D3C inline; experimental_treatment owns which of
    // its histories survive.
    row(240, ALL), // Experimental Treatment
    // Its Use of the D4 (522250) rewrites D28..D3C and every count.
    row(258, ALL), // Missing No.
];

pub(crate) fn row_for(id: u32) -> Option<&'static GrantRow> {
    ROWS.iter().find(|row| row.id == id)
}

/// Whether the Add path of `id` keeps the given input word.
pub(crate) fn keeps(id: u32, word: u16) -> bool {
    row_for(id).is_none_or(|row| row.writes & word == 0)
}
