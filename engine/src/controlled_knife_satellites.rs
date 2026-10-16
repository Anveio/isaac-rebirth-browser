//! Actual controlled-Knife 4A0 satellites, not Has118 subtype1 Parent children.
//! Caller permissions for each item/cache/getter remain independent of this
//! concrete actor chain. In particular this does not admit items153/245.
use crate::{
    State,
    actors::{ActorKind, ActorRef},
    knives::{CONTROLLED, ChildChain, Knife},
    phase::Phase,
    player_items::PlayerItems,
};
use std::collections::BTreeSet;

///51B1D8/51D930/51D948/520137: positive Have predicates have precedence;
///the final TEMP245 query tests record presence, independently of its count.
fn requested_count(items: &PlayerItems) -> Result<usize, String> {
    if items.owned_counts.len() != 347 {
        return Err("Knife satellites require all347 actual permanent counts".into());
    }
    Ok(if items.owned_counts[153] > 0 {
        3
    } else if items.owned_counts[2] > 0 {
        2
    } else if items.owned_counts[245] > 0
        || items
            .temporary_collectibles
            .iter()
            .any(|record| record.item_id == 245)
    {
        1
    } else {
        0
    })
}

///Walk actual registered4A0 links in their stored order. Parent498 never
///supplies a chain edge. Unknown/restored roles remain unadmitted, not empty.
pub(crate) fn chain(knives: &[Knife], anchor_id: u32) -> Result<Option<Vec<u32>>, String> {
    let anchor = knives
        .iter()
        .find(|knife| knife.id == anchor_id)
        .ok_or("Knife satellite anchor handle is unresolved")?;
    let parent = ActorRef {
        kind: ActorKind::Knife,
        id: anchor_id,
    };
    let mut link = anchor.child_chain_4a0;
    let mut seen = BTreeSet::from([anchor_id]);
    let mut ids = Vec::new();
    loop {
        let actor = match link {
            ChildChain::Unknown => return Ok(None),
            ChildChain::Empty => return Ok(Some(ids)),
            ChildChain::Linked(actor) => actor,
        };
        if actor.kind != ActorKind::Knife {
            return Ok(None);
        }
        if !seen.insert(actor.id) {
            return Err("Knife4A0 chain contains a cycle".into());
        }
        let child = knives
            .iter()
            .find(|knife| knife.id == actor.id)
            .ok_or("registered Knife4A0 target is unresolved")?;
        if child.subtype != 0 || child.parent != Some(parent) || child.attack_flags.0 != CONTROLLED
        {
            return Ok(None);
        }
        //Inactive/dead are not absence: Remove keeps the registered edge
        //until actual Uninit clears it. This loop follows that real edge.
        ids.push(child.id);
        link = child.child_chain_4a0;
    }
}

///447F40 unregisters the old relation, stores4A0, then registers the new one.
///All admitted endpoints share canonical Knife storage; clear_knife_links is
///the existing Uninit sweep over these exact registered relations.
fn set_child(knives: &mut [Knife], owner_id: u32, target: u32) -> Result<(), String> {
    if !knives.iter().any(|knife| knife.id == target) {
        return Err("Knife4A0 setter target is unresolved".into());
    }
    let owner = knives
        .iter_mut()
        .find(|knife| knife.id == owner_id)
        .ok_or("Knife4A0 setter owner is unresolved")?;
    owner.child_chain_4a0 = ChildChain::Linked(ActorRef {
        kind: ActorKind::Knife,
        id: target,
    });
    Ok(())
}

///51C098..51C6BD: trim actual excess actors, create missing50BDD0 actors,
///link them in order, and call18 for every actual age0 actor. A same-age Fast
///Player call repeats that explicit Full; this is not a first-seen heuristic.
pub(crate) fn synchronize(state: &mut State, anchor_id: u32) -> Result<(), String> {
    let desired = requested_count(
        state
            .player_items
            .as_ref()
            .ok_or("Knife satellite item owner absent")?,
    )?;
    let mut ids = chain(&state.knives, anchor_id)?
        .ok_or("Knife satellite chain changed after caller preflight")?;
    for &id in ids.iter().skip(desired) {
        state
            .knives
            .iter_mut()
            .find(|knife| knife.id == id)
            .ok_or("excess Knife satellite is unresolved")?
            .remove();
    }
    ids.truncate(desired);
    let anchor = state
        .knives
        .iter()
        .find(|knife| knife.id == anchor_id)
        .ok_or("Knife satellite anchor disappeared")?;
    let flags = anchor.attack_flags.0;
    let current_color = anchor.current_color;
    let damage = anchor.collision_damage;
    let player = crate::bombs::player_world(state);
    while ids.len() < desired {
        let id = state.attack_runtime.next_knife;
        let next = id
            .checked_add(1)
            .ok_or("Knife satellite handle exhausted")?;
        let seed = state.boot_lifecycle.shared.next_u32();
        let mut child = Knife::initialize(
            id,
            seed,
            state.game_update_count,
            &mut state.boot_lifecycle.shared,
        );
        child.position = player;
        child.set_parent(Some(ActorRef {
            kind: ActorKind::Knife,
            id: anchor_id,
        }));
        child.render_offset = crate::motion::WorldVector { x: 0.0, y: -10.0 };
        child.raw_7ec = 0.0;
        //50BDD0's single=false never sets318bit200000; the later caller
        //sets bit33 and reapplies current color/damage, before any18 call.
        child.apply_resolved(flags, current_color, damage);
        state.knives.push(child);
        state.attack_runtime.next_knife = next;
        state.actor_scheduler.admit(ActorRef {
            kind: ActorKind::Knife,
            id,
        })?;
        ids.push(id);
    }
    let mut previous = anchor_id;
    for id in ids {
        set_child(&mut state.knives, previous, id)?;
        previous = id;
        if state
            .knives
            .iter()
            .find(|knife| knife.id == id)
            .is_some_and(|knife| knife.birth_game_update == state.game_update_count)
        {
            crate::attack_runtime::advance_actor(state, Phase::Update, id, 4)?;
        }
    }
    //The last surviving old edge is not forcibly cleared after trimming:
    //Remove retains it until its target's real registered Uninit sweep.
    Ok(())
}

///474988..474967 writes each linked actor BEFORE the anchor's empty-room
///return steering. System+2EFA0 is a signed counter, not Game age or phase.
pub(crate) fn form(
    anchor: &Knife,
    child: &mut Knife,
    index: usize,
    count: usize,
    system_counter: u32,
) -> Result<(), String> {
    if count == 0
        || index >= count
        || anchor.attack_flags.0 != CONTROLLED
        || !anchor.position.is_finite()
        || !anchor.velocity.is_finite()
        || !anchor.collision_radius.is_finite()
        || anchor.collision_radius < 0.0
    {
        return Err("Knife satellite formation requires its neutral finite source owners".into());
    }
    let angle = (index as f32 * 360.0) / count as f32 + 3.0 * (system_counter as i32) as f32;
    child.velocity = anchor.velocity;
    child.position =
        anchor.position + crate::knives::degree_direction(angle) * (5.0 * anchor.collision_radius);
    child.sprite.rotation = crate::source_angles::wrap_180(angle - 90.0);
    child.apply_resolved(
        anchor.attack_flags.0,
        anchor.current_color,
        anchor.collision_damage,
    );
    if !child.position.is_finite() || !child.sprite.rotation.is_finite() {
        return Err("Knife satellite formation overflowed source fields".into());
    }
    Ok(())
}

///473F00/473F0E bit33 skips ordinary ParentKnife charge/follow. A knownNULL
///Parent instead calls Remove at47357A; the common registry/Grid/Base tail
///still runs. Subtype1 Fast never calls18 and keeps its separate owner.
pub(crate) fn advance_body(
    knife: &mut Knife,
    phase: Phase,
    game_age: u32,
    orphan: bool,
) -> Result<bool, String> {
    if phase == Phase::Skipped {
        return Ok(false);
    }
    if !matches!(knife.attack_flags.0, 0 | CONTROLLED)
        || (orphan && !(knife.parent.is_none() && knife.parent_null_known))
    {
        return Err("Knife linked body requires its closed flags and actual NULL owner".into());
    }
    if !knife.position.is_finite()
        || !knife.velocity.is_finite()
        || !knife.current_friction.is_finite()
    {
        return Err("Knife linked body requires finite source fields".into());
    }
    if phase == Phase::Interpolate {
        knife.fast = true;
    }
    if !knife.fast {
        knife.current_friction *= f32::from_bits(0x3f33_3333);
    }
    if orphan {
        knife.remove();
    }
    if knife.entity_flags & 0x4000 == 0 && (crate::knives::source_game_counter(game_age) as i32) % 3 == 0 {
        knife.hit_sequences.clear();
    }
    if knife.fast {
        knife.saved_position = knife.position;
        knife.position = knife.position + (knife.velocity * knife.current_friction) * 0.5;
    }
    if !knife.position.is_finite() || !knife.velocity.is_finite() {
        return Err("Knife linked body overflowed source fields".into());
    }
    Ok(!knife.fast)
}
