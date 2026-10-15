//!519FC0 C08=1/Have2: three ordered canonical Tears, independent of stats.
use crate::{
    attack_runtime::{DispatchOutcome, ShootButtons},
    item_pipeline::ItemInputs,
    motion::WorldVector,
    phase::Phase,
    weapon_plan::{ControllerChannel, PrimaryWeaponC08, ResolvedWeaponPlan},
    State,
};

pub(crate) fn selected(state: &State, plan: &ResolvedWeaponPlan) -> bool {
    plan.primary == PrimaryWeaponC08::RegularTears
        && plan.controller == ControllerChannel::RegularShots
        && state
            .player_items
            .as_ref()
            .is_some_and(|owner| owner.owned_counts.get(2).is_some_and(|count| *count > 0))
}

fn frontier(state: &mut State, channel: &str) -> DispatchOutcome {
    state
        .weapon_admission
        .record_unadmitted(state.weapon_cache.primary, channel);
    DispatchOutcome::Unadmitted
}

///42E930 chooses Y on ties, including zero -> down. This is the current
///processed callback direction, rather than Monstro's prior DB4 read.
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

///Pure source launch layout.50A4F0 retains DD4*.6 then*2 before removing
///its backward projection.51F1EA emits centre, positive perp, negative perp.
fn launches(
    position: WorldVector,
    dc4: WorldVector,
    aim: WorldVector,
    entering_velocity: WorldVector,
    shot_speed: f32,
) -> Result<[(WorldVector, WorldVector); 3], String> {
    if !position.is_finite()
        || !dc4.is_finite()
        || !aim.is_finite()
        || !entering_velocity.is_finite()
        || !shot_speed.is_finite()
    {
        return Err("Inner Eye launch owners are nonfinite".into());
    }
    let aim = cardinal(aim);
    let mut inherited = (entering_velocity * 0.6) * 2.0;
    let projection = aim.x * inherited.x + aim.y * inherited.y;
    if projection < 0.0 {
        inherited = WorldVector {
            x: inherited.x - aim.x * projection,
            y: inherited.y - aim.y * projection,
        };
    }
    let base = aim * 10.0 + inherited;
    let positive_velocity = WorldVector {
        x: base.y * 0.05,
        y: (-base.x) * 0.05,
    };
    let positive_position = WorldVector {
        x: base.y * 0.8,
        y: (-base.x) * 0.8,
    };
    let negative_velocity = WorldVector {
        x: (-base.y) * 0.05,
        y: base.x * 0.05,
    };
    let negative_position = WorldVector {
        x: (-base.y) * 0.8,
        y: base.x * 0.8,
    };
    let result = [
        ((position + base) + dc4, base * shot_speed),
        (
            (position + positive_position) + dc4,
            (base + positive_velocity) * shot_speed,
        ),
        (
            (position + negative_position) + dc4,
            (base + negative_velocity) * shot_speed,
        ),
    ];
    if result
        .iter()
        .any(|(origin, velocity)| !origin.is_finite() || !velocity.is_finite())
    {
        return Err("Inner Eye launch overflowed".into());
    }
    Ok(result)
}

pub(crate) fn dispatch_player(
    state: &mut State,
    plan: &ResolvedWeaponPlan,
    input: ShootButtons,
    phase: Phase,
    callback_aim: WorldVector,
) -> Result<DispatchOutcome, String> {
    if !selected(state, plan) || phase == Phase::Skipped {
        return Err("Inner Eye requires actual ordinary C08=1/Have2 callback".into());
    }
    let inputs = ItemInputs::new(
        state
            .player_items
            .as_ref()
            .ok_or("Inner Eye items absent")?,
    )?;
    if state.weapon_admission.source_player_type != Some(0)
        || !inputs.admits_inner_eye_caller()
        || state
            .player_items
            .as_ref()
            .is_none_or(|owner| owner.active_id != 0 || owner.active_state_d80 != 0)
    {
        return Ok(frontier(state, "innerEyeCallerContext"));
    }
    if !input.held() {
        return Ok(DispatchOutcome::Handled);
    }
    if state.fire_cooldown >= 0 {
        //Have69=false leaves stack38=0:51C081 returns via51A060,
        //preserving the actual entering C0C on this held continuation.
        return Ok(DispatchOutcome::Handled);
    }
    if !inputs.admits_inner_eye_kind1_context()
        || inputs.pending_cache().0 & (1 | 2 | 4 | 8 | 32 | 64) != 0
        || state.weapon_cache.flags_c98.0 != 0
    {
        return Ok(frontier(state, "innerEyeKind1Cache"));
    }
    //51CA81 rotates the aim for positive C70. The fresh zero owner survives
    //only the closed2/4 grant/Full/room path; a missing restore is not zero.
    if state.attack_runtime.aim_rotation_c70 != Some(0) {
        return Ok(frontier(state, "innerEyeC70History"));
    }
    let Some(dc4) = state.attack_runtime.player_dc4 else {
        return Ok(frontier(state, "innerEyePlayerDc4History"));
    };
    let layout = launches(
        crate::bombs::player_world(state),
        dc4,
        callback_aim,
        crate::bombs::player_velocity(state),
        plan.source_shot_speed_c7c,
    )?;
    let initial = crate::weapon_plan::resolve_inner_eye_kind1(state)?;
    state
        .next_tear
        .checked_add(3)
        .ok_or("Inner Eye Tear IDs exhausted")?;
    //51C7A7..C7E9 flips D70 once and installs C78/BB0 before emission.
    //There is no ordinary single-shot outer eye Uniform on this branch.
    state.player.eye = state.player.eye.flipped();
    state.fire_cooldown = plan.max_fire_delay;
    state.head_timer = initial.head_timer;
    for (origin, velocity) in layout {
        let birth = crate::weapon_plan::resolve_inner_eye_kind1(state)?;
        crate::tear_factory::fire(state, birth, Some(0), origin, velocity)?;
    }
    //51F3D7→51D005→51C081 takes stack38=0→51A060 here.
    //The Have69/Have316 charge-reset branch is absent; preserve C0C.
    Ok(DispatchOutcome::Handled)
}
