//! Admission order shared by full updates and intermediate predictions.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ActorKind {
    Player,
    Familiar,
    Tear,
    Bomb,
    Pickup,
    Npc,
    Projectile,
    Effect,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActorRef {
    pub kind: ActorKind,
    pub id: u32,
}

/// Source Entity+8 is assigned by Room5c6940, independently of the browser
/// handle. IDs can coincide after a room reset retains Player/Familiar pointers.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Identity {
    pub actor: ActorRef,
    pub sequence_id: u32,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Scheduler {
    actors: Vec<ActorRef>,
    identities: Vec<Identity>,
    room_next_sequence: u32,
}

impl Scheduler {
    fn allocate_identity(&mut self, actor: ActorRef) -> Result<(), String> {
        if self
            .identities
            .iter()
            .any(|identity| identity.actor == actor)
        {
            return Err("actor handle already admitted".into());
        }
        self.identities.push(Identity {
            actor,
            sequence_id: self.room_next_sequence,
        });
        self.room_next_sequence = self.room_next_sequence.wrapping_add(1);
        Ok(())
    }

    pub fn admit(&mut self, actor: ActorRef) -> Result<(), String> {
        self.allocate_identity(actor)?;
        self.actors.push(actor);
        Ok(())
    }

    /// Room5c6940 assigns Entity+8 before Manager4524f0 tests flags8. These
    /// actual background births reserve an identity but never enter the
    /// ordinary full-update array or live intermediate dispatch.
    pub fn admit_background(&mut self, actor: ActorRef) -> Result<(), String> {
        self.allocate_identity(actor)
    }

    pub fn sequence_id(&self, actor: ActorRef) -> Option<u32> {
        self.identities
            .iter()
            .find(|identity| identity.actor == actor)
            .map(|identity| identity.sequence_id)
    }

    /// Room.Init resets its allocator before Game.Spawn restores ordinary
    /// records. Manager453ec0 retains the same Player/Familiar pointers and IDs.
    pub fn clear_room(&mut self) {
        self.retain(|actor| matches!(actor.kind, ActorKind::Player | ActorKind::Familiar));
        self.room_next_sequence = 0;
    }

    /// The original full loop stops at the last actor present on entry.
    pub fn full_phase(&self) -> Vec<ActorRef> {
        self.actors.clone()
    }

    /// Intermediate dispatch observes additions made during that same phase.
    pub fn get(&self, index: usize) -> Option<ActorRef> {
        self.actors.get(index).copied()
    }

    pub fn retain(&mut self, mut alive: impl FnMut(ActorRef) -> bool) {
        // Background membership is independent of ordinary update membership.
        // Evaluate the actual alive predicate once for every admitted identity.
        self.identities.retain(|identity| alive(identity.actor));
        self.actors.retain(|actor| {
            self.identities
                .iter()
                .any(|identity| identity.actor == *actor)
        });
    }
}
