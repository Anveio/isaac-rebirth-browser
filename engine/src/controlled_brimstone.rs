//! Player 519FC0's selected controlled Brimstone caller and 477DC0 ring body.
//! The canonical Laser allocator, scheduler, contacts and Uninit remain shared.
use crate::{
    actors::{ActorKind, ActorRef},
    attack_runtime::{DispatchOutcome, ShootButtons},
    grid::Grid,
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
const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };

fn frontier(state: &mut State, channel: &str) {
    state
        .weapon_admission
        .record_unadmitted(state.weapon_cache.primary, channel);
}

/// 51A93A/51C73C copy current C98 into both 7E8 and 818, CCC through v58,
/// and C88 through v60. This caller does not execute a kind2 builder or OR bit33.
pub(crate) fn apply_current(laser: &mut Laser, plan: &ResolvedWeaponPlan) {
    laser.attack_flags = plan.properties.flags;
    laser.line_flags = Some(plan.properties.flags);
    laser.source_color = plan.secondary_color_ccc;
    laser.current_color = laser.source_color;
    laser.sprite.tint = laser.current_color.tint;
    laser.sprite.offsets = laser.current_color.offsets;
    laser.collision_damage = plan.properties.collision_damage;
}

/// 51C6BF applies reuse motion before the current property stores. The held
/// path retains two rounded multiplications; neither path reads C7C or C0C.
pub(crate) fn steer_reused(
    laser: &mut Laser,
    player: WorldVector,
    mut aim: WorldVector,
    held: bool,
    countable_total: i32,
) -> Result<(), String> {
    let push = if held {
        let squared = aim.x * aim.x + aim.y * aim.y;
        if squared > 1.0 {
            aim = aim * (1.0 / squared.sqrt());
        }
        Some((aim * 2.0) * 0.5)
    } else {
        let delta = WorldVector {
            x: player.x - laser.position.x,
            y: player.y - laser.position.y,
        };
        let squared = delta.x * delta.x + delta.y * delta.y;
        if countable_total <= 0 && squared > 10000.0 {
            Some(((delta * (1.0 / squared.sqrt())) * 0.3) * 0.5)
        } else {
            None
        }
    };
    if let Some(push) = push {
        if laser.current_friction != 0.0 {
            laser.velocity = laser.velocity
                + WorldVector {
                    x: push.x / laser.current_friction,
                    y: push.y / laser.current_friction,
                };
        }
    }
    if !laser.velocity.is_finite() {
        return Err("controlled Brimstone reuse motion overflowed actual fields".into());
    }
    Ok(())
}

/// 52F4FB/530340 uses the real C18 displacement for held C08=2 or3/Has329.
/// It changes head presentation only; the callback keeps its processed aim.
pub(crate) fn head_direction(
    state: &mut State,
    aim: WorldVector,
    held: bool,
) -> Result<(), String> {
    let mut head_aim = aim;
    if held {
        if let Some(actor) = state.attack_runtime.persistent {
            let position = match actor.kind {
                ActorKind::Laser => state
                    .lasers
                    .iter()
                    .find(|a| a.id == actor.id)
                    .map(|a| a.position),
                ActorKind::Knife => state
                    .knives
                    .iter()
                    .find(|a| a.id == actor.id)
                    .map(|a| a.position),
                ActorKind::Tear => {
                    let Some(position) =
                        crate::controlled_tear::referenced_position(state, actor.id)?
                    else {
                        frontier(state, "controlledBrimstoneHeadReference");
                        return Ok(());
                    };
                    Some(position)
                }
                _ => {
                    frontier(state, "controlledBrimstoneHeadReference");
                    return Ok(());
                }
            }
            .ok_or("controlled Brimstone head C18 is unresolved")?;
            let player = crate::bombs::player_world(state);
            head_aim = WorldVector {
                x: position.x - player.x,
                y: position.y - player.y,
            };
            if !head_aim.is_finite() {
                return Err("controlled Brimstone head displacement is nonfinite".into());
            }
            state.head_timer = 10;
        }
    }
    crate::attack_runtime::ordinary_head_direction(state, head_aim);
    Ok(())
}

pub(crate) fn dispatch_player(
    state: &mut State,
    plan: &ResolvedWeaponPlan,
    input: ShootButtons,
    _phase: Phase,
    callback_aim: WorldVector,
) -> Result<DispatchOutcome, String> {
    if plan.primary != PrimaryWeaponC08::Brimstone
        || plan.controller != ControllerChannel::ControlledBrimstone
    {
        return Err("controlled Brimstone requires the actual C08=2/Ludovico channel".into());
    }
    let position = crate::bombs::player_world(state);
    if !position.is_finite()
        || !callback_aim.is_finite()
        || !plan.secondary_color_ccc.is_finite()
        || !plan.properties.collision_damage.is_finite()
        || plan.properties.collision_damage < 0.0
    {
        return Err("controlled Brimstone caller contains invalid current numeric owners".into());
    }
    let owner = state
        .player_items
        .as_ref()
        .ok_or("controlled Brimstone current items are absent")?;
    let inputs = crate::item_pipeline::ItemInputs::new(owner)?;
    // This selected neutral footprint is conservative. Direct current property
    // copies do not inherit the ordinary factory's kind2 payload or wardrobe.
    if state.weapon_admission.source_player_type != Some(0)
        || !inputs.admits_neutral_kind2()
        || plan.properties.flags.0 != 0
    {
        frontier(state, "controlledBrimstoneProperties");
        return Ok(DispatchOutcome::Unadmitted);
    }
    if let Some(actor) = state.attack_runtime.persistent {
        if !crate::attack_runtime::admits_referenced_remove(state, actor)? {
            frontier(state, "controlledBrimstoneC18Family");
            return Ok(DispatchOutcome::Unadmitted);
        }
    }
    if state.attack_runtime.persistent.is_none() && !state.weapon_admission.c18_is_known_empty {
        frontier(state, "controlledBrimstoneC18History");
        return Ok(DispatchOutcome::Unadmitted);
    }
    let matching = state.attack_runtime.persistent.filter(|actor| {
        actor.kind == ActorKind::Laser
            && state
                .lasers
                .iter()
                .any(|laser| laser.id == actor.id && laser.variant == 1 && laser.subtype == 1)
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
            .ok_or("controlled Brimstone C18 is unresolved")?;
        if laser.origin != LaserOrigin::PlayerControlled
            || laser.uninitialized != Some(false)
            || !laser.position.is_finite()
            || !laser.velocity.is_finite()
            || !laser.current_friction.is_finite()
        {
            frontier(state, "controlledBrimstoneActorHistory");
            return Ok(DispatchOutcome::Unadmitted);
        }
        steer_reused(laser, position, callback_aim, input.held(), count)?;
        apply_current(laser, plan);
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
    let seed = state.boot_lifecycle.shared.next_u32(); // 51A8AF.
    let mut laser = Laser::initialize(id, 1, seed, state.game_update_count)?;
    laser.position = position;
    laser.spawner_type = 1;
    laser.parent = Some(PLAYER); // 447FC0, independent from Player C18.
    laser.render_offset = WorldVector { x: 0.0, y: -20.0 };
    laser.render_sort_offset = 3000.0;
    apply_current(&mut laser, plan);
    state.lasers.push(laser);
    state.next_laser = next;
    let actor = ActorRef {
        kind: ActorKind::Laser,
        id,
    };
    state.actor_scheduler.admit(actor)?;
    // 51A400 invokes the full callback BEFORE 519130 binds Player C18.
    advance_actor(state, Phase::Update, id)?;
    crate::attack_runtime::replace_c18(state, Some(actor))?;
    Ok(DispatchOutcome::Handled)
}

pub(crate) fn advance_actor(state: &mut State, phase: Phase, id: u32) -> Result<(), String> {
    if phase == Phase::Skipped {
        return Ok(());
    }
    let index = state
        .lasers
        .iter()
        .position(|laser| laser.id == id)
        .ok_or("controlled Brimstone scheduled handle is unresolved")?;
    if phase == Phase::Update && !state.lasers[index].active {
        return Ok(());
    }
    let laser = &state.lasers[index];
    laser.validate_contact_geometry()?;
    if !laser.position.is_finite()
        || !laser.velocity.is_finite()
        || !laser.current_friction.is_finite()
        || !laser.configured_friction.is_finite()
        || !laser.sprite.frame.is_finite()
        || !laser.sprite.speed.is_finite()
        || !laser.sprite.scale.is_finite()
    {
        return Err("controlled Brimstone body contains nonfinite actual owners".into());
    }
    if laser.origin != LaserOrigin::PlayerControlled
        || !matches!(laser.variant, 1 | 2)
        || laser.subtype != 1
        || laser.parent != Some(PLAYER)
        || laser.uninitialized != Some(false)
        || laser.entity_flags.is_none()
        || laser.attack_flags.0 != 0
        || (laser.variant == 1 && laser.child.is_some())
        || laser.rotation_enabled
        || laser.max_distance != 0.0
        || laser.line_controller != 0
    {
        frontier(state, "controlledBrimstoneBodyHistory");
        return Ok(());
    }
    let initial = state.lasers[index].first_update;
    if phase == Phase::Interpolate {
        if !state.lasers[index].begin_fast() {
            return Ok(());
        }
    } else if initial {
        // 476410/4763D8 suppress Player subtype1 audio for both variants.
        // Variant2 nevertheless reaches4796BC's real Effect50/sub0 factory.
        if state.lasers[index].variant == 2 {
            crate::laser_player_runtime::initialize_controlled_technology_impact(state, index)?;
        }
        let laser = &mut state.lasers[index];
        laser.line_enabled = true;
        laser.line.initialize(50, 1);
        laser.line.width = Some(60.0);
        laser.grid_collision_class = 0;
    }
    let laser = &state.lasers[index];
    if !laser.line_enabled
        || laser.line.subdivisions != Some(50)
        || laser.line.mode != Some(1)
        || laser.line.width != Some(60.0)
        || laser.grid_collision_class != 0
    {
        frontier(state, "controlledBrimstoneLineHistory");
        return Ok(());
    }
    let grid = Grid::new(
        state
            .room
            .grid
            .clone()
            .ok_or("controlled Brimstone grid is absent")?,
    )?;
    let age = state.game_update_count;
    let laser = &mut state.lasers[index];
    if !laser.fast {
        laser.current_friction *= 0.8;
    }
    laser.line.begin_curve();
    laser.line.origin = laser.position; // 477E07 precedes the physical clamp.
    laser.line.offset = ZERO;
    laser.position = grid.clamp_room(laser.position, 80.0);
    if laser.variant == 2 {
        // 478120..247: the circular variant2 child consumes the actual Full
        // endpoint before Base, or the current Fast prediction directly.
        if let Some(child) = laser.child {
            let position = if laser.fast {
                laser.position
            } else {
                grid.clamp_room(
                    laser.position + laser.velocity * laser.current_friction,
                    80.0,
                )
            };
            let impact = laser
                .impact
                .as_mut()
                .filter(|impact| child.kind == ActorKind::Effect && impact.id == child.id)
                .ok_or("controlled Technology registered child is unresolved")?;
            impact.position = position;
            impact.source_color = Some(laser.current_color);
            impact.current_color = Some(laser.current_color);
            impact.position_offset = laser.render_offset;
        }
    }
    laser.line.circle(laser.position, 80.0);
    laser.line.resample(0.3)?;
    laser.line.simplify(0.001);
    let contacts_due = !laser.dead
        && !laser.fast
        && laser
            .entity_flags
            .ok_or("controlled Brimstone flags318 history is absent")?
            & 0x4000
            == 0
        && (age as i32) % 2 == 0;
    if contacts_due {
        crate::laser_runtime::run_line_contacts(state, index, &grid)?;
    }
    let laser = &mut state.lasers[index];
    if initial {
        laser.first_update = false;
    } else if !laser.fast {
        laser.finish_full()?;
    }
    Ok(())
}
