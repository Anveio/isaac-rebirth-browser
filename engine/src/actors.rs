//! Admission order shared by full updates and intermediate predictions.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ActorKind {
    Player,
    Tear,
    Bomb,
    Pickup,
    Npc,
    Effect,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActorRef {
    pub kind: ActorKind,
    pub id: u32,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Scheduler {
    actors: Vec<ActorRef>,
}

impl Scheduler {
    pub fn admit(&mut self, actor: ActorRef) -> Result<(), String> {
        if self.actors.contains(&actor) {
            return Err("actor handle already admitted".into());
        }
        self.actors.push(actor);
        Ok(())
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
    }
}
