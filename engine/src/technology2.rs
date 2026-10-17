//! Actual secondary Technology2 C40 and the independent ordinary Tear caller.
//! Shared stores/factories own actors; current cache never rewrites beam births.
use crate::{
    actors::ActorKind,
    attack_runtime::{DispatchOutcome, ShootButtons, Technology2Slot},
    item_pipeline::ItemInputs,
    lasers::LaserOrigin,
    motion::WorldVector,
    phase::Phase,
    projectiles::Eye,
    weapon_plan::{ControllerChannel, PrimaryWeaponC08, ResolvedWeaponPlan},
    State,
};

pub(crate) fn selected(state: &State) -> bool {
    state.weapon_cache.primary == PrimaryWeaponC08::RegularTears
        && state
            .player_items
            .as_ref()
            .is_some_and(|owner| owner.owned_counts.get(152).is_some_and(|count| *count > 0))
}

///Historical observations do not veto an actual body/birth collector. A
///diagnostic record created for an old restore still has unknown pointer state.
pub(crate) fn diagnostic(state: &mut State, channel: &str) {
    state
        .attack_runtime
        .technology2_c40
        .get_or_insert_with(Technology2Slot::default)
        .first_diagnostic
        .get_or_insert_with(|| channel.into());
}

pub(crate) fn invalidate_history(state: &mut State, channel: &str) {
    diagnostic(state, channel);
    if let Some(slot) = &mut state.attack_runtime.technology2_c40 {
        slot.pointer_known = false;
        slot.actor = None;
    }
}

pub(crate) fn primary_frontier(state: &mut State, channel: &str) -> DispatchOutcome {
    //An unrepresented primary property does not invent a C40 transition or
    //invalidate C18. The independent secondary callback has already run.
    diagnostic(state, channel);
    DispatchOutcome::Unadmitted
}

///42E930 selects Y on ties. Zero uses down only after a source nonzero gate
///where required; it is never substituted for missing DB4 history.
fn cardinal(aim: WorldVector) -> WorldVector {
    if aim.x.abs() > aim.y.abs() {
        WorldVector {
            x: if aim.x < 0.0 { -1.0 } else { 1.0 },
            y: 0.0,
        }
    } else {
        WorldVector {
            x: 0.0,
            y: if aim.y < 0.0 { -1.0 } else { 1.0 },
        }
    }
}

///Actual50B720(kind1) selects RightEye via directional Head*/explicitframe1.
///152 owns slot7; actual4/169 each own only null-free head slot4.
///Their real registrations preserve this selected RightEye and the fresh
///normal unit-scale/base-layer geometry; they never heal unknown history.
fn right_eye(state: &State, direction: WorldVector) -> Option<WorldVector> {
    let owner = state.player_items.as_ref()?;
    let history = owner.costume_registrations.as_ref()?;
    if !history.configuration_ids.contains(&152)
        || !history
            .configuration_ids
            .iter()
            .all(|id| matches!(id, 4 | 152 | 169))
        || state.player.flying != Some(false)
        || state.player.grid_collision_class != Some(5)
    {
        return None;
    }
    let point = if direction.x < 0.0 {
        WorldVector { x: -8.0, y: -17.0 }
    } else if direction.x > 0.0 {
        WorldVector { x: 8.0, y: -17.0 }
    } else if direction.y < 0.0 {
        WorldVector { x: 6.0, y: -17.0 }
    } else {
        WorldVector { x: -7.0, y: -17.0 }
    };
    //Actual fresh normal Sprite unit scales/baseLayer4 cancel the(0,-5)
    //terms;42EAD0 divides the selected authored point by binary32 .65.
    Some(WorldVector {
        x: point.x / 0.65_f32,
        y: point.y / 0.65_f32,
    })
}

///Actual52F370 input/clock result kept outside the independent primary
///getter. A pending kind1/cache frontier must not suppress a retained C40.
pub(crate) struct Callback {
    pub(crate) buttons: ShootButtons,
    pub(crate) aim: WorldVector,
    pub(crate) db4_write: Option<WorldVector>,
}

pub(crate) fn prepare_callback(
    state: &mut State,
    phase: Phase,
    controls_enabled: bool,
) -> Result<Option<Callback>, String> {
    let blocked = !controls_enabled
        || state.player_animation.guard
        || state
            .player_items
            .as_ref()
            .is_some_and(|owner| owner.pickup_animation_guard);
    let buttons = if blocked {
        ShootButtons {
            left: false,
            right: false,
            up: false,
            down: false,
        }
    } else {
        ShootButtons::read(state)
    };
    let step = crate::attack_runtime::ordinary_shooting_aim(
        buttons.aim(),
        buttons.held(),
        state.attack_runtime.shooting_aim_db4,
    )?;
    let Some(mut step) = step else {
        diagnostic(state, "technology2ShootingAimHistory");
        return Ok(None);
    };
    //52F6B7/52F8F0 normalizes the processed callback/DB4 vector before
    //52FE28 separately applies42E930 to the secondary's stackD0 vector.
    if crate::knives::squared_length(step.callback_aim) > 1.0 {
        step.callback_aim = crate::knives::normalized(step.callback_aim);
        if step.db4_write.is_some() {
            step.db4_write = Some(step.callback_aim);
        }
    }
    crate::attack_runtime::ordinary_head_direction(state, step.callback_aim);
    if phase == Phase::Update {
        if state.fire_cooldown >= 0 {
            state.fire_cooldown -= 1;
        }
        if state.head_timer >= 0 {
            state.head_timer -= 1;
        }
    }
    secondary(state, buttons, phase, step.callback_aim)?;
    Ok(Some(Callback {
        buttons,
        aim: step.callback_aim,
        db4_write: step.db4_write,
    }))
}

///52F700..52F713 follows519FC0. C08=1 has no C18-dependent idle test.
pub(crate) fn idle_head(state: &mut State, held: bool) {
    if !held && state.head_timer < 0 {
        if let Some(direction) = state.attack_runtime.movement_direction_d9c {
            state.attack_runtime.head_direction_da0 = Some(match direction {
                0 => 0,
                1 => 1,
                2 => 2,
                _ => 3,
            });
        } else {
            diagnostic(state, "technology2MovementHistory");
        }
    }
}

///52FE20 normalD80=0 processes this effect BEFORE519FC0 and its RNG.
///52FE28 cardinalizes stack160 into stackD0 before530A41/530F7F; the
///primary independently cardinalizes its callback at51A448.
///The explicit Have118 suppression is independent of primary selection.
pub(crate) fn secondary(
    state: &mut State,
    input: ShootButtons,
    phase: Phase,
    callback_aim: WorldVector,
) -> Result<(), String> {
    if phase == Phase::Skipped {
        return Ok(());
    }
    let owner = state
        .player_items
        .as_ref()
        .ok_or("Technology2 items absent")?;
    let inputs = ItemInputs::new(owner)?;
    if !inputs.positive_owned_count(152)
        || inputs.positive_owned_count(118)
        || !state.player.can_shoot
    {
        return Ok(());
    }
    if state.weapon_admission.source_player_type != Some(0)
        || !inputs.admits_technology2_secondary()
        || owner.active_id != 0
        || owner.active_state_d80 != 0
    {
        invalidate_history(state, "technology2SecondaryContext");
        return Ok(());
    }
    if !callback_aim.is_finite() {
        return Err("Technology2 current processed aim is nonfinite".into());
    }
    let Some(slot) = state.attack_runtime.technology2_c40.as_ref() else {
        diagnostic(state, "technology2C40Unknown");
        return Ok(());
    };
    if !slot.pointer_known {
        diagnostic(state, "technology2C40Unknown");
        return Ok(());
    }
    let actor = slot.actor;
    if callback_aim.x == 0.0 && callback_aim.y == 0.0 {
        //5311C8 checks actual held input separately. Remove does not clear
        //the registered weak pointer; Full BaseUninit owns that transition.
        if !input.held() {
            if let Some(actor) = actor {
                let laser = state
                    .lasers
                    .iter_mut()
                    .find(|laser| {
                        actor.kind == ActorKind::Laser
                            && laser.id == actor.id
                            && laser.origin == LaserOrigin::PlayerTechnology2
                    })
                    .ok_or("Technology2 C40 released identity is unresolved")?;
                if !laser.dead {
                    laser.remove();
                }
            }
        }
        return Ok(());
    }
    let direction = cardinal(callback_aim);
    let Some(offset) = right_eye(state, direction) else {
        //An unknown steering offset does not own the registered weak pointer.
        //Only an unrepresented possible birth from knownNULL can change C40.
        if actor.is_none() {
            invalidate_history(state, "technology2RightEyeBirthHistory");
        } else {
            diagnostic(state, "technology2RightEyeSteeringHistory");
        }
        return Ok(());
    };
    if let Some(actor) = actor {
        let laser = state
            .lasers
            .iter_mut()
            .find(|laser| {
                actor.kind == ActorKind::Laser
                    && laser.id == actor.id
                    && laser.origin == LaserOrigin::PlayerTechnology2
            })
            .ok_or("Technology2 C40 current identity is unresolved")?;
        if phase == Phase::Update {
            //530FAD excludes FAST steering.4760F0 retains old7DC in7E0
            //and writes Sprite rotation as the actual new angle minus90.
            let angle = (crate::fmath::atan2(f64::from(direction.y), f64::from(direction.x))
                as f32)
                * f32::from_bits(0x4265_2ee0);
            laser.previous_angle = Some(laser.angle);
            laser.angle = angle;
            laser.sprite.rotation = angle - 90.0;
            laser.render_offset = offset;
            laser.render_sort_offset = if direction.y > -0.5 && direction.x > -0.5 {
                3000.0
            } else {
                -10.0
            };
        }
        //5311A4 follows both the Full steering and FAST continuation.
        state.head_timer = 5;
        return Ok(());
    }
    let owner = state
        .player_items
        .as_ref()
        .ok_or("Technology2 birth items absent")?;
    let inputs = ItemInputs::new(owner)?;
    //Mode1 copies actual currentC98&~1. Bit9 has no thin-body/status
    //operator; its born7E8/818 and currentGet3 readsets remain independent.
    if inputs.pending_cache().0 & (1 | 32 | 64) != 0
        || !matches!(state.weapon_cache.flags_c98.0, 0 | 0x200)
    {
        invalidate_history(state, "technology2BirthCache");
        return Ok(());
    }
    //53161E→531645→531652→531660→53166B: one actual canonical factory,
    //then currentC88*.2, TTL=-1, registered C40, and immediate Full.
    let actor = crate::laser_player_runtime::admit_technology2(
        state,
        crate::laser_player_runtime::Technology2Birth {
            player_position: crate::bombs::player_world(state),
            aim: direction,
            render_offset_from_50b720: offset,
        },
    )?;
    let laser = state
        .lasers
        .iter_mut()
        .find(|laser| laser.id == actor.id)
        .ok_or("Technology2 born actor is absent")?;
    laser.collision_damage = state.player_stats.damage * 0.2_f32;
    laser.remaining_updates = -1;
    //No newest-beam/count reconstruction: this is the actual519130 event.
    let slot = state
        .attack_runtime
        .technology2_c40
        .as_mut()
        .ok_or("Technology2 actual C40 owner disappeared during factory")?;
    slot.actor = Some(actor);
    crate::laser_runtime::advance_ordinary_technology_initial(state, actor)?;
    //5311A4 changes BB0 after both existing/born paths. The normal source
    //continuation then evaluates the primary callback below.
    state.head_timer = 5;
    Ok(())
}

///Independent519FC0 ordinary Tear cadence. Positive152 emits only the
///negative D70 half; skipped cycles consume no Tear/eye/audio random draws.
pub(crate) fn primary(
    state: &mut State,
    plan: &ResolvedWeaponPlan,
    input: ShootButtons,
    phase: Phase,
    callback_aim: WorldVector,
) -> Result<DispatchOutcome, String> {
    if !selected(state)
        || plan.controller != ControllerChannel::RegularShots
        || phase == Phase::Skipped
    {
        return Err("Technology2 primary requires actual C08=1 callback".into());
    }
    let owner = state
        .player_items
        .as_ref()
        .ok_or("Technology2 primary items absent")?;
    let inputs = ItemInputs::new(owner)?;
    if state.weapon_admission.source_player_type != Some(0)
        || !inputs.admits_technology2_primary()
        || owner.active_id != 0
        || owner.active_state_d80 != 0
    {
        return Ok(primary_frontier(state, "technology2PrimaryContext"));
    }
    if !input.held() || state.fire_cooldown >= 0 {
        return Ok(DispatchOutcome::Handled);
    }
    let initial = crate::weapon_plan::resolve_technology2_kind1(state)?;
    state.player.eye = state.player.eye.flipped();
    state.fire_cooldown = plan.max_fire_delay;
    state.head_timer = initial.head_timer;
    if state.player.eye != Eye::Negative {
        return Ok(DispatchOutcome::Handled);
    }
    //51DC9A fixes the actual firing eye negative. C70 is read only on
    //this emitting half, never synthesized from owned152 counts.
    if state.attack_runtime.aim_rotation_c70 != Some(0) {
        return Ok(primary_frontier(state, "technology2PrimaryC70History"));
    }
    let Some(dc4) = state.attack_runtime.player_dc4 else {
        return Ok(primary_frontier(state, "technology2PrimaryDc4History"));
    };
    if !callback_aim.is_finite() || !dc4.is_finite() {
        return Err("Technology2 primary vector owner is nonfinite".into());
    }
    let aim = cardinal(callback_aim);
    //50A4F0 rounds DD4*.6 then*2 before removing its backward projection.
    let mut inherited = (crate::bombs::player_velocity(state) * 0.6_f32) * 2.0_f32;
    let projection = aim.x * inherited.x + aim.y * inherited.y;
    if projection < 0.0 {
        inherited = WorldVector {
            x: inherited.x - aim.x * projection,
            y: inherited.y - aim.y * projection,
        };
    }
    let base = aim * 10.0 + inherited;
    let uniform = state.boot_lifecycle.shared.uniform();
    //52068A always consumes the eye draw, using unscaled baseV. Only after
    //that draw does currentHave169 select currentPlayerP at520703..734.
    //5177F0 then addsDC4. This primary override never touches C40's RightEye.
    let origin = crate::projectiles::eye_origin(
        crate::bombs::player_world(state),
        base,
        state.player.eye,
        uniform,
    )?;
    let origin = if inputs.positive_owned_count(169) {
        crate::bombs::player_world(state)
    } else {
        origin
    } + dc4;
    let velocity = base * plan.source_shot_speed_c7c;
    let birth = crate::weapon_plan::resolve_technology2_kind1(state)?;
    crate::tear_factory::fire(state, birth, Some(0), origin, velocity)?;
    //Have69/316=false leaves stack38=0→51C081→51A060: preserve C0C.
    Ok(DispatchOutcome::Handled)
}
