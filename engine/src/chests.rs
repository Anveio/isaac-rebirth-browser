//! Concrete ordinary Chest actors. Authored selectors stay unchanged; fresh
//! Init's temporary RNG35 is independent of the entity's persistent RNG30.
use crate::{items::Rng, pickups::Pickup, room_spawns::SpawnSelector};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Chest {
    pub rng: Rng,
    pub grid_collision_class: u32,
    pub entity_collision_class: u32,
    pub price: i32,
    pub cooldown: u32,
}

pub(crate) fn from_selector(
    id: u32,
    selector: &SpawnSelector,
    appear: bool,
) -> Result<Option<Pickup>, String> {
    if selector.entity_type != 5 || selector.variant != 50 {
        return Ok(None);
    }
    if selector.spawn_seed == 0 {
        return Err("chest spawn seed must be nonzero".into());
    }
    // Pickup::Init4ffedd/50026a: Random20 is always first; the red Chest
    // replacement bypasses the second Random50. Rare replacement callbacks
    // remain unsupported actors with their original typed selector retained.
    let mut temporary = Rng::new(selector.spawn_seed, [5, 9, 7]);
    if temporary.bounded(20) == 0 || temporary.bounded(50) == 0 {
        return Ok(None);
    }
    // Init4fff38 forces closed subtype1 even when the authored subtype is0.
    let mut pickup = Pickup::spawn(id, 50, 1, selector.position, appear)?;
    pickup.spawn_seed = selector.spawn_seed;
    pickup.chest = Some(Chest {
        rng: Rng::new(selector.spawn_seed, [4, 3, 17]),
        grid_collision_class: 5,
        entity_collision_class: if appear { 0 } else { 4 },
        price: 0,
        cooldown: 0,
    });
    Ok(Some(pickup))
}
