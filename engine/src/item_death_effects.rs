//! NPC Kill47FC80 item effects, before the idempotent death transition.
//! Each effect consumes only its actual callback inputs; inventory cache
//! completion and source-unknown familiar effects are independent owners.
use crate::{
    actors::{ActorKind, ActorRef, Scheduler},
    items::Rng,
    motion::WorldVector,
    pickups::Pickup,
    player_items::PlayerItems,
};

///47FCE9 Have225 tests positive permanent count once. Empty actual trinket
/// slots and no queued config close HasTrinket22's false branch; its positive
/// branch remains outside this consumer. No unknown RNG is synthesized.
pub(crate) fn gimpy_half_heart(
    items: &PlayerItems,
    max_hp: f32,
    entity_rng: &mut Option<Rng>,
) -> Result<bool, String> {
    let count = *items
        .owned_counts
        .get(225)
        .ok_or("Gimpy callback requires actual Player225 count")?;
    if count <= 0 {
        return Ok(false);
    }
    if items.trinket_slots != [0, 0] || !items.queue_is_empty() {
        return Err("Gimpy callback requires the actual trinket22 owner".into());
    }
    if !max_hp.is_finite() || max_hp <= 0.0 {
        return Err("Gimpy callback requires actual positive finite maximum HP".into());
    }
    let bound = if max_hp < 10.0 {
        100
    } else if max_hp < 30.0 {
        20
    } else {
        10
    };
    let rng = entity_rng
        .as_mut()
        .ok_or("Gimpy callback Entity7C0 RNG owner absent")?;
    Ok(rng.bounded(bound) == 0)
}

/// Immediate47FE78 Spawn5/10/2, V0 and NULL spawner, no explicit Full.
/// The borrowed arenas keep this pickup's manager admission before the later
/// gib/effect births; deferring it until the NPC callback returns changes order.
pub(crate) struct PickupAdmission<'a> {
    pub game_update_count: u32,
    pub next_pickup: &'a mut u32,
    pub pickups: &'a mut Vec<Pickup>,
    pub actor_order: &'a mut Vec<ActorRef>,
}

impl PickupAdmission<'_> {
    pub(crate) fn admit_half_heart(
        &mut self,
        seed: u32,
        position: WorldVector,
        room_type: u32,
        scheduler: &mut Scheduler,
    ) -> Result<(), String> {
        //4FF560..4FF566 retains the explicit subtype2 outside Arcade.
        //Room8's descriptor1AE0 rewrites need their own actual owner.
        if room_type == 8 {
            return Err("Gimpy half-heart Arcade subtype owner absent".into());
        }
        let id = *self.next_pickup;
        let next = id.checked_add(1).ok_or("Gimpy pickup identity exhausted")?;
        let mut pickup = Pickup::spawn(id, 10, 2, position, true)?;
        pickup.initialize_spawn_seed(seed)?;
        pickup.birth_game_update = self.game_update_count;
        self.pickups.push(pickup);
        *self.next_pickup = next;
        let actor = ActorRef {
            kind: ActorKind::Pickup,
            id,
        };
        self.actor_order.push(actor);
        scheduler.admit(actor)
    }
}
