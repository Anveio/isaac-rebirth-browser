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
    /// Entity_Knife (type 8), including the real Brim child subtype1.
    Knife,
    /// Entity_Laser (type 7).
    Laser,
    /// Entity_Slot (type 6): machines and beggars, see `slots`.
    Slot,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActorRef {
    pub kind: ActorKind,
    pub id: u32,
}

/// Actual EntityManager dc/e0/e4/e8 (Room16ac/16b0/16b4/16b8).
/// Admission adds pending words; the full pass counts its fenced active actors
/// after their callback, then folds pending into current before contacts.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManagerCounts {
    pub countable_pending: i32,
    pub countable_current: i32,
    pub boss_pending: i32,
    pub boss_current: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FullCounts {
    countable: i32,
    bosses: i32,
}

impl FullCounts {
    pub(crate) fn observe(&mut self, countable: bool, boss_without_final_flag: bool) {
        if countable {
            self.countable = self.countable.wrapping_add(1);
            if boss_without_final_flag {
                self.bosses = self.bosses.wrapping_add(1);
            }
        }
    }
}

impl ManagerCounts {
    ///4524f0->452490 on actual manager admission; virtual70 and78 are distinct.
    pub(crate) fn admit(&mut self, countable: bool, boss: bool) {
        if countable {
            self.countable_pending = self.countable_pending.wrapping_add(1);
            if boss {
                self.boss_pending = self.boss_pending.wrapping_add(1);
            }
        }
    }

    ///454f98..454fc6. Pending admission is not deduplicated against the fenced
    /// callback count: both source words enter the real addition.
    pub(crate) fn finish_full(&mut self, counted: FullCounts) {
        self.countable_current = counted.countable.wrapping_add(self.countable_pending);
        self.boss_current = counted.bosses.wrapping_add(self.boss_pending);
        self.countable_pending = 0;
        self.boss_pending = 0;
    }

    /// Signed current plus pending countable actors: the Room.Update clear
    /// predicate (5d1487) and spawn caps read this sum.
    pub fn countable_total(&self) -> i32 {
        self.countable_pending.wrapping_add(self.countable_current)
    }

    pub(crate) fn boss_total(&self) -> i32 {
        self.boss_pending.wrapping_add(self.boss_current)
    }
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
    /// EntityManager+80 render array of the ordinary actors; Room::Render
    /// draws it in order (see `sort_render`).
    #[serde(default)]
    render: Vec<ActorRef>,
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

    /// One past the largest admitted handle id of `kind`: a fresh handle that
    /// cannot collide with any actor the scheduler still holds.
    pub fn next_free_id(&self, kind: ActorKind) -> Option<u32> {
        self.identities
            .iter()
            .filter(|identity| identity.actor.kind == kind)
            .map(|identity| identity.actor.id)
            .max()
            .map_or(Some(0), |id| id.checked_add(1))
    }

    pub fn sequence_id(&self, actor: ActorRef) -> Option<u32> {
        self.identities
            .iter()
            .find(|identity| identity.actor == actor)
            .map(|identity| identity.sequence_id)
    }

    /// In-place Entity.Init441CFD clears the actual Entity+8 word. Morph
    /// does not call Room5C6940: retain the handle, order and next allocator.
    pub(crate) fn reinitialize_in_place(&mut self, actor: ActorRef) -> Result<(), String> {
        if !self.actors.contains(&actor) {
            return Err("in-place Init requires current manager actor".into());
        }
        let identity = self
            .identities
            .iter_mut()
            .find(|identity| identity.actor == actor)
            .ok_or("in-place Init requires actual identity owner")?;
        identity.sequence_id = 0;
        Ok(())
    }

    /// Room.Init resets its allocator before Game.Spawn restores ordinary
    /// records. Manager453ec0 retains the same Player/Familiar pointers and IDs.
    pub fn clear_room(&mut self) {
        self.clear_room_retaining(|_| false);
    }

    ///453ec0 preserves Player1/Familiar3 and actual type8/subtype0.
    pub(crate) fn clear_room_retaining(&mut self, mut retained: impl FnMut(ActorRef) -> bool) {
        self.retain(|actor| {
            matches!(actor.kind, ActorKind::Player | ActorKind::Familiar) || retained(actor)
        });
        self.room_next_sequence = 0;
        //453ec0 empties the render array and re-appends the retained
        // Player, Familiar and type8/subtype0 actors.
        let actors = &self.actors;
        self.render.retain(|actor| actors.contains(actor));
    }

    /// Maintain the EntityManager+80 render array that Room::Render walks in
    /// order (452850/4528c0 call virtual+28 Render), so a later entry draws
    /// over an earlier one. `keys` holds each ordinary actor's virtual+50
    /// render key; actors without one stay out of the array.
    ///
    /// Insert 4524f0 places a new actor before the first entry whose key is
    /// not below its own, so at equal keys the newer actor draws first.
    /// Manager Update ends (45529a) with 452760: up to three passes swapping
    /// adjacent entries only when the left key is strictly (unsigned) greater,
    /// stopping after a pass with fewer than count/10+1 swaps. Equal keys keep
    /// their order, and an actor whose key fell moves at most one slot left
    /// per pass: entering a room, Isaac (inserted at his departure key) walks
    /// back through the room's actors one slot per update. `passes` is false
    /// for the admitting Room.Update, which inserts without sorting.
    pub(crate) fn sort_render(&mut self, keys: &[(ActorRef, u32)], passes: bool) {
        let key = |actor: ActorRef| {
            keys.iter()
                .find(|(other, _)| *other == actor)
                .map(|(_, key)| *key)
        };
        let actors = &self.actors;
        self.render
            .retain(|actor| actors.contains(actor) && key(*actor).is_some());
        for actor in self.actors.iter().copied() {
            if self.render.contains(&actor) {
                continue;
            }
            let Some(own) = key(actor) else {
                continue;
            };
            let at = self
                .render
                .iter()
                .position(|other| key(*other).is_some_and(|other| other >= own))
                .unwrap_or(self.render.len());
            self.render.insert(at, actor);
        }
        if !passes {
            return;
        }
        let count = u32::try_from(self.render.len()).unwrap_or(u32::MAX);
        for _ in 0..3 {
            let mut swaps = 0u32;
            for index in 1..self.render.len() {
                if key(self.render[index - 1]) > key(self.render[index]) {
                    self.render.swap(index - 1, index);
                    swaps += 1;
                }
            }
            if count / 10 + 1 > swaps {
                break;
            }
        }
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
        let actors = &self.actors;
        self.render.retain(|actor| actors.contains(actor));
    }
}
