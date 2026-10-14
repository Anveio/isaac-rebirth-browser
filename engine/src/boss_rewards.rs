//! Ordinary Monstro's actual Room5c8430 final-boss callback.
//! Generic death effects and the later Room5c9240 pedestal/exit producer have
//! separate owners. This callback never substitutes an effect-event marker.
use crate::{
    actors::{ActorKind, ActorRef, Scheduler},
    grid::Grid,
    mt::Mt19937,
    npcs::Npc,
    pickups::Pickup,
    room_costs::RoomCosts,
};

pub(crate) struct FinalBossAdmission<'a> {
    pub room_type: u32,
    pub normal_difficulty: bool,
    pub game_update_count: u32,
    pub boss_count: &'a mut u32,
    pub next_pickup: &'a mut u32,
    pub pickups: &'a mut Vec<Pickup>,
    pub actor_order: &'a mut Vec<ActorRef>,
    pub music: &'a mut crate::music::MusicState,
    pub grid: &'a Grid,
    pub path_costs: &'a RoomCosts,
    pub bombs: &'a [crate::Bomb],
}

impl FinalBossAdmission<'_> {
    pub fn finish(
        &mut self,
        npc: &mut Npc,
        shared: &mut Mt19937,
        scheduler: &mut Scheduler,
    ) -> Result<(), String> {
        let monstro = npc
            .monstro
            .as_mut()
            .ok_or("final boss requires Monstro owner")?;
        //5c849d and5c84c1 write actual Entity318 and Room2490 before Remove.
        monstro.entity_flags |= 0x100000;
        *self.boss_count = self
            .boss_count
            .checked_sub(1)
            .ok_or("final boss callback without admitted boss count")?;
        if *self.boss_count != 0 {
            return Ok(());
        }
        if self.room_type != 5 {
            return Err("ordinary Monstro final callback requires boss room5".into());
        }
        self.music.play(83);
        self.music.queue(37);
        if !self.normal_difficulty {
            return Ok(()); //5c87c3 difficulty1 bypasses the ordinary heart.
        }
        // Fresh single-player without trinkets:508fe0(22) is Daemon's Tail,
        // not collectible22. Its actual trinket owner remains a separate leaf.
        let seed = monstro.actor_rng.raw();
        let velocity = crate::room_placement::direction(shared) * 7.0;
        //5c815c mode0 checks actual admitted Bomb4/Pickup5/Laser6 members.
        // Player and NPCs do not block this callback. Laser6 has no producer
        // in the current scope, so no invented position enters the list.
        let mut occupied: Vec<_> = self
            .pickups
            .iter()
            .filter(|pickup| !pickup.removed)
            .map(|pickup| pickup.position)
            .collect();
        occupied.extend(
            self.bombs
                .iter()
                .filter(|bomb| bomb.motion.lifetime != crate::bombs::BombLifetime::Removed)
                .map(|bomb| bomb.motion.position),
        );
        let position = crate::room_placement::pickup_position(
            npc.position,
            0.0,
            self.grid,
            self.path_costs,
            &occupied,
            shared,
        )?;
        let subtype = crate::pickups::regular_subtype(10, 0, seed)?;
        let id = *self.next_pickup;
        let next = id.checked_add(1).ok_or("boss heart identity exhausted")?;
        let mut pickup = Pickup::spawn(id, 10, subtype, position, true)?;
        pickup.initialize_spawn_seed(seed)?;
        pickup.velocity = velocity;
        pickup.birth_game_update = self.game_update_count;
        //550acd copies cached1c/20. Neither this caller nor Pickup.Init links
        // Entity.parent498; no immediate Update is invoked by5c889b.
        pickup.spawner_type = npc.kind;
        pickup.spawner_variant = npc.variant;
        let actor = ActorRef {
            kind: ActorKind::Pickup,
            id,
        };
        self.pickups.push(pickup);
        self.actor_order.push(actor);
        *self.next_pickup = next;
        scheduler.admit(actor)
    }
}
