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
    pub fn admit(&mut self, actor: ActorRef) -> Result<(), String> {
        if self.actors.contains(&actor) {
            return Err("actor handle already admitted".into());
        }
        self.actors.push(actor);
        self.identities.push(Identity {
            actor,
            sequence_id: self.room_next_sequence,
        });
        self.room_next_sequence = self.room_next_sequence.wrapping_add(1);
        Ok(())
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
        self.actors.retain(|actor| alive(*actor));
        self.identities
            .retain(|identity| self.actors.contains(&identity.actor));
    }
}
