//! Level::load_room 586c90 for a room whose Descriptor+40 surprise-miniboss
//! flag the generator set (589feb..58a9d1): the room is built from a type-6
//! miniboss template instead of its own, while the descriptor keeps its type
//! (2 shop, 7 secret, 14 devil), seeds and backdrop.
use crate::{
    floor::Floor,
    items::Rng,
    room_assets::{self, RoomAsset},
    room_templates::{SelectionRequest, TemplatePicker},
};

/// Game flag 0x8000: a Greed surprise room was loaded this run.
const GREED_SEEN: u64 = 0x8000;
/// Game flag 0x10000: a Super Greed surprise room was loaded.
const SUPER_GREED_SEEN: u64 = 0x1_0000;

/// The miniboss layout for descriptor `index`, or None when it is not
/// flagged. The pick seeds SetSeed(Descriptor+4c, preset 1) and takes one
/// word for 5d5600, which decays the picked template's weight in the run's
/// special catalogue. Greed 2040..2049 while flag 0x8000 is clear (then set),
/// otherwise Super Greed 2140..2149 (setting 0x10000); a devil room takes
/// Krampus 2300..2309.
/// PORT GAP: load_room repeats the pick on every entry; the port builds the
/// room once and restores it on later entries, so a revisit neither decays
/// another weight nor sets 0x10000.
pub(crate) fn layout(
    floor: &mut Floor,
    index: usize,
    special: Option<&[RoomAsset]>,
) -> Result<Option<RoomAsset>, String> {
    let descriptor = floor
        .descriptors
        .get(index)
        .ok_or("surprise room descriptor index outside Floor")?;
    if !descriptor.surprise_miniboss {
        return Ok(None);
    }
    let seed = Rng::new(descriptor.spawn_seed, [1, 5, 16]).raw();
    let range = if descriptor.template.room_type == 14 {
        (2300, 2309)
    } else if floor.game_flags & GREED_SEEN == 0 {
        floor.game_flags |= GREED_SEEN;
        (2040, 2049)
    } else {
        floor.game_flags |= SUPER_GREED_SEEN;
        (2140, 2149)
    };
    let assets = special.map_or_else(room_assets::special, <[RoomAsset]>::to_vec);
    let mut picker =
        TemplatePicker::new(assets.iter().map(|asset| asset.template.clone()).collect())?;
    picker.restore_weights(&floor.special_weights)?;
    let selected = picker.choose(SelectionRequest {
        seed,
        room_type: 6,
        shape: 4,
        required_doors: 0,
        min_variant: range.0,
        max_variant: range.1,
        min_difficulty: 1,
        max_difficulty: 10,
    })?;
    floor.special_weights = picker.current_weights().to_vec();
    Ok(Some(assets[selected.catalog_index].clone()))
}
