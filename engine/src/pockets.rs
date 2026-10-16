//! Actual Player3044/3048 and304C/3050 records. Pool selection remains in
//! the canonical pickup Init; capacity, insertion and presentation are callers.
use crate::player_items;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PocketSlot {
    pub id: u32,
    pub card: bool,
}
impl PocketSlot {
    const EMPTY: Self = Self { id: 0, card: true };
}

///These records retain the last actually owned contents. An unimplemented
///writer invalidates current ownership explicitly; it never consumes a pocket
///or manufactures the result of a Card/Pill callback.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum PocketFrontier {
    CardEffect { card_id: u32 },
    PillEffect { pill_color: u32 },
    ModifierConversion { collectible_id: u32 },
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeldPockets {
    pub slots: [PocketSlot; 2],
    pub frontier: Option<PocketFrontier>,
}
impl HeldPockets {
    pub(crate) fn fresh() -> Self {
        //5197C0/CA and5197D1/DB: ID0 and Card tag1 in both records.
        Self {
            slots: [PocketSlot::EMPTY; 2],
            frontier: None,
        }
    }
}

///52DDA4/52DEF1 convert only a nonempty record of the opposite kind. The
///worldspawn/item-RNG conversion callback is tracked until implemented; empty
///or already-matching records remain owned, without inferring new contents.
pub(crate) fn note_modifier_grant(owner: &mut player_items::PlayerItems, id: u32) {
    let Some(held) = owner
        .held_pockets
        .as_mut()
        .filter(|held| held.frontier.is_none())
    else {
        return;
    };
    if held
        .slots
        .iter()
        .any(|slot| slot.id != 0 && ((id == 251 && !slot.card) || (id == 252 && slot.card)))
    {
        held.frontier = Some(PocketFrontier::ModifierConversion { collectible_id: id });
    }
}
