//! Collectible grant rows for AddCollectible 52CC90, one row per collectible
//! like `active_items::USES`. A row names the persistent Player input words
//! that the collectible's Add path is proven to leave alone. Any collectible
//! can still be granted (pedestals, Challenge starts, D4/D100 rerolls); a
//! collectible without a row reaches an unported Add effect, so every
//! persistent input word below loses its owner and becomes unknown.
//!
//! The stat outputs themselves come from the cache Evaluate511F10 callbacks
//! over owned counts, so a row only covers the Add event, not the stats.

/// Player+D28/D44/13BC/13C0, the MoveSpeed inputs.
pub(crate) const MOVE_SPEED: u8 = 1;
/// Player+D34, the Range input.
pub(crate) const RANGE: u8 = 2;
/// Player+D2C, the FireDelay input.
pub(crate) const FIRE_DELAY: u8 = 4;
/// Player+D30, the Damage input.
pub(crate) const DAMAGE: u8 = 8;
/// Player+D38, the ShotSpeed input.
pub(crate) const SHOT_SPEED: u8 = 16;
/// The attack aim rotation C70.
pub(crate) const AIM_C70: u8 = 32;
/// The Luck inputs D3C/D40/D58.
pub(crate) const LUCK: u8 = 64;

const STATS: u8 = MOVE_SPEED | RANGE | FIRE_DELAY | DAMAGE | SHOT_SPEED;

#[derive(Clone, Copy, Debug)]
pub(crate) struct GrantRow {
    pub id: u32,
    /// The input words this Add path keeps.
    pub keeps: u8,
}

const fn row(id: u32, keeps: u8) -> GrantRow {
    GrantRow { id, keeps }
}

pub(crate) const ROWS: &[GrantRow] = &[
    row(1, STATS | LUCK),           // The Sad Onion
    row(2, STATS | AIM_C70),        // The Inner Eye
    row(3, STATS),                  // Spoon Bender
    row(4, STATS | AIM_C70 | LUCK), // Cricket's Head
    row(6, STATS | LUCK),           // Number One
    row(12, DAMAGE),                // Magic Mushroom
    row(14, MOVE_SPEED | RANGE),    // Roid Rage
    row(27, MOVE_SPEED),            // Wooden Spoon
    row(28, MOVE_SPEED),            // The Belt
    row(52, STATS),                 // Dr. Fetus
    row(68, STATS),                 // Technology
    row(114, STATS | LUCK),         // Mom's Knife
    row(118, STATS | LUCK),         // Brimstone
    row(143, STATS),                // Speed Ball
    row(152, STATS | AIM_C70),      // Technology 2
    row(168, STATS),                // Epic Fetus
    row(169, STATS | AIM_C70),      // Polyphemus
    row(222, FIRE_DELAY),           // Anti-Gravity
    row(229, STATS),                // Monstro's Lung
    row(304, STATS | LUCK),         // Libra
    row(329, STATS | LUCK),         // The Ludovico Technique
    row(330, STATS),                // Soy Milk
];

pub(crate) fn row_for(id: u32) -> Option<&'static GrantRow> {
    ROWS.iter().find(|row| row.id == id)
}

/// Whether the Add path of `id` keeps the given input word.
pub(crate) fn keeps(id: u32, word: u8) -> bool {
    row_for(id).is_some_and(|row| row.keeps & word != 0)
}
