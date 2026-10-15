//! Literal519FC0 C08=3+Have329 caller for actual Laser7/2/1.
//! It reuses the canonical Laser/C18 services and the subtype1 circular body;
//! ordinary50E1C0's getter, lifetime3, directed Eye and sound are separate.
use crate::{
    actors::{ActorKind, ActorRef},
    attack_runtime::{DispatchOutcome, ShootButtons},
    lasers::{Laser, LaserOrigin},
    motion::WorldVector,
    phase::Phase,
    weapon_plan::{ControllerChannel, PrimaryWeaponC08, ResolvedWeaponPlan},
    State,
};
const PLAYER: ActorRef = ActorRef {
    kind: ActorKind::Player,
    id: 0,
};
fn frontier(state: &mut State, channel: &str) {
    state
        .weapon_admission
        .record_unadmitted(state.weapon_cache.primary, channel);
}

pub(crate) fn dispatch_player(
    state: &mut State,
    plan: &ResolvedWeaponPlan,
    input: ShootButtons,
    _phase: Phase,
    callback_aim: WorldVector,
) -> Result<DispatchOutcome, String> {
    if plan.primary != PrimaryWeaponC08::Technology
        || plan.controller != ControllerChannel::ControlledTechnology
    {
        return Err("controlled Technology requires the actual C08=3/Ludovico channel".into());
    }
    let position = crate::bombs::player_world(state);
    if !position.is_finite()
        || !callback_aim.is_finite()
        || !plan.secondary_color_ccc.is_finite()
        || !plan.properties.collision_damage.is_finite()
        || plan.properties.collision_damage < 0.0
    {
        return Err("controlled Technology caller contains invalid current numeric owners".into());
    }
    let owner = state
        .player_items
        .as_ref()
        .ok_or("controlled Technology current items are absent")?;
    let inputs = crate::item_pipeline::ItemInputs::new(owner)?;
    // 51C245/51D359 copy actual cached properties directly. This selected
    // caller footprint does not invoke the ordinary factory's kind2 getter.
    if state.weapon_admission.source_player_type != Some(0)
        || !inputs.admits_controlled_technology_context()
        || plan.properties.flags.0 != 0
    {
        frontier(state, "controlledTechnologyProperties");
        return Ok(DispatchOutcome::Unadmitted);
    }
    if let Some(actor) = state.attack_runtime.persistent {
        if !crate::attack_runtime::admits_referenced_remove(state, actor)? {
            frontier(state, "controlledTechnologyC18Family");
            return Ok(DispatchOutcome::Unadmitted);
        }
    }
    if state.attack_runtime.persistent.is_none() && !state.weapon_admission.c18_is_known_empty {
        frontier(state, "controlledTechnologyC18History");
        return Ok(DispatchOutcome::Unadmitted);
    }
    let matching = state.attack_runtime.persistent.filter(|actor| {
        actor.kind == ActorKind::Laser
            && state
                .lasers
                .iter()
                .any(|laser| laser.id == actor.id && laser.variant == 2 && laser.subtype == 1)
    });
    if let Some(actor) = matching.filter(|actor| {
        state
            .lasers
            .iter()
            .any(|laser| laser.id == actor.id && !laser.dead)
    }) {
        let count = state.room.manager_counts.countable_total();
        let laser = state
            .lasers
            .iter_mut()
            .find(|laser| laser.id == actor.id)
            .ok_or("controlled Technology C18 is unresolved")?;
        if laser.origin != LaserOrigin::PlayerControlled
            || laser.uninitialized != Some(false)
            || !laser.position.is_finite()
            || !laser.velocity.is_finite()
            || !laser.current_friction.is_finite()
        {
            frontier(state, "controlledTechnologyActorHistory");
            return Ok(DispatchOutcome::Unadmitted);
        }
        crate::controlled_brimstone::steer_reused(
            laser,
            position,
            callback_aim,
            input.held(),
            count,
        )?;
        crate::controlled_brimstone::apply_current(laser, plan);
        return Ok(DispatchOutcome::Handled);
    }
    // A wrong family/variant/subtype receives virtual48. A matching dead
    // actor is replaced without a second Remove. Remove does not clear C18.
    if let Some(previous) = state
        .attack_runtime
        .persistent
        .filter(|actor| Some(*actor) != matching)
    {
        crate::attack_runtime::remove_referenced_actor(state, previous)?;
    }
    let id = state.next_laser;
    let next = id.checked_add(1).ok_or("Laser handle exhausted")?;
    let seed = state.boot_lifecycle.shared.next_u32(); // 51C245.
    let mut laser = Laser::initialize_controlled_technology(id, seed, state.game_update_count);
    laser.position = position;
    laser.spawner_type = 1;
    laser.parent = Some(PLAYER); // 447FC0, independent from Player C18.
    laser.render_offset = crate::motion::WorldVector { x: 0.0, y: -20.0 }; //51C2A4..51A92A.
    laser.render_sort_offset = 3000.0;
    crate::controlled_brimstone::apply_current(&mut laser, plan);
    state.lasers.push(laser);
    state.next_laser = next;
    let actor = ActorRef {
        kind: ActorKind::Laser,
        id,
    };
    state.actor_scheduler.admit(actor)?;
    // 51A400 invokes the full callback BEFORE 519130 binds Player C18.
    crate::controlled_brimstone::advance_actor(state, Phase::Update, id)?;
    crate::attack_runtime::replace_c18(state, Some(actor))?;
    Ok(DispatchOutcome::Handled)
}
