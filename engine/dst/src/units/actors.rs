//! Actor admission order and room-local identities (`engine/src/actors.rs`)
//! against an independent list model.
use crate::harness::Case;
use crate::props;
use rebirth_cleanroom::actors::{ActorKind, ActorRef, Scheduler};
use serde_json::json;

const KINDS: [ActorKind; 9] = [
    ActorKind::Player,
    ActorKind::Familiar,
    ActorKind::Tear,
    ActorKind::Bomb,
    ActorKind::Pickup,
    ActorKind::Npc,
    ActorKind::Projectile,
    ActorKind::Effect,
    ActorKind::Knife,
];

#[derive(Default)]
struct Model {
    actors: Vec<(ActorRef, u32)>,
    next_sequence: u32,
}

pub fn run(case: &mut Case) {
    let e = &mut *case.entropy;
    let mut scheduler = Scheduler::default();
    let mut model = Model::default();
    for _ in 0..(case.steps / 2).max(64) {
        // A small identity space makes duplicate admissions common.
        let actor = ActorRef {
            kind: *e.pick(&KINDS),
            id: e.range(0, 5),
        };
        match e.below(10) {
            0..=5 => {
                let duplicate = model.actors.iter().any(|(known, _)| *known == actor);
                let result = scheduler.admit(actor);
                props::always!(
                    result.is_err() == duplicate,
                    "actors: admission rejects exactly the already-admitted handles",
                    json!({ "actor": actor, "duplicate": duplicate })
                );
                props::sometimes!(duplicate, "actors: a duplicate admission is attempted");
                if !duplicate {
                    model.actors.push((actor, model.next_sequence));
                    model.next_sequence = model.next_sequence.wrapping_add(1);
                }
            }
            6 | 7 => {
                let drop_kind = *e.pick(&KINDS);
                let drop_parity = e.below(2) as u32;
                let keep = |actor: ActorRef| actor.kind != drop_kind || actor.id % 2 != drop_parity;
                scheduler.retain(keep);
                model.actors.retain(|(actor, _)| keep(*actor));
            }
            8 => {
                scheduler.clear_room();
                model.actors.retain(|(actor, _)| {
                    matches!(actor.kind, ActorKind::Player | ActorKind::Familiar)
                });
                model.next_sequence = 0;
                props::sometimes!(
                    !model.actors.is_empty(),
                    "actors: a room reset retains Player or Familiar handles"
                );
            }
            _ => {
                let saved = serde_json::to_value(&scheduler).expect("scheduler serializes");
                let restored: Result<Scheduler, _> = serde_json::from_value(saved);
                props::always!(
                    restored.as_ref().ok() == Some(&scheduler),
                    "actors: the scheduler round-trips through JSON"
                );
            }
        }
        let order: Vec<ActorRef> = model.actors.iter().map(|(actor, _)| *actor).collect();
        let indexed: Vec<ActorRef> = (0..).map_while(|index| scheduler.get(index)).collect();
        props::always!(
            scheduler.full_phase() == order && indexed == order,
            "actors: dispatch order equals admission order minus retired handles",
            json!({ "observed": scheduler.full_phase(), "model": order })
        );
        let identities_match = model
            .actors
            .iter()
            .all(|(actor, sequence)| scheduler.sequence_id(*actor) == Some(*sequence));
        let retired_unknown = KINDS.iter().all(|kind| {
            (0..6).all(|id| {
                let actor = ActorRef { kind: *kind, id };
                order.contains(&actor) || scheduler.sequence_id(actor).is_none()
            })
        });
        props::always!(
            identities_match && retired_unknown,
            "actors: room sequence identities follow admission and reset with the room"
        );
    }
}
