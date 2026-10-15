//! Literal519FC0 C08=5/no329 shot caller and50DE10's neutral factory inputs.
//! Canonical Bomb storage, Full/Fast, collision and explosion remain in bombs.
use crate::{
    actors::{ActorKind, ActorRef},
    attack_runtime::{DispatchOutcome, ShootButtons},
    bombs::DrFetusBirth,
    item_pipeline::ItemInputs,
    motion::WorldVector,
    phase::Phase,
    weapon_plan::{ControllerChannel, PrimaryWeaponC08, ResolvedWeaponPlan},
    State,
};

///42E930 retains the dominant axis, taking Y on ties, including zero -> down.
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

pub(crate) fn dispatch_player(
    state: &mut State,
    plan: &ResolvedWeaponPlan,
    input: ShootButtons,
    phase: Phase,
    callback_aim: WorldVector,
) -> Result<DispatchOutcome, String> {
    if plan.primary != PrimaryWeaponC08::DrFetus
        || plan.controller != ControllerChannel::OrdinaryFetus
        || phase == Phase::Skipped
    {
        return Err("Dr. Fetus requires its actual C08=5/no329 caller".into());
    }
    //51AA70 returns before the factory, getter or random draw on these gates.
    //C0C and the registered C18 slot are not shot cadence owners here.
    if !input.held() || state.fire_cooldown >= 0 {
        return Ok(DispatchOutcome::Handled);
    }
    let inputs = ItemInputs::new(
        state
            .player_items
            .as_ref()
            .ok_or("Dr. Fetus Player items absent")?,
    )?;
    if state.weapon_admission.source_player_type != Some(0)
        || !inputs.admits_fetus_kind5_context()
        || inputs.primary_permanent_presence(329)?
    {
        state
            .weapon_admission
            .record_unadmitted(plan.primary, "ordinaryFetusKind5Context");
        return Ok(DispatchOutcome::Unadmitted);
    }
    let Some(dc4) = state.attack_runtime.player_dc4 else {
        state
            .weapon_admission
            .record_unadmitted(plan.primary, "ordinaryFetusPlayerDc4History");
        return Ok(DispatchOutcome::Unadmitted);
    };
    //50DE83 evaluates kind5 BEFORE50DE9D's SharedRaw. This is the current
    //C98/C88/CA0 read, not a broadcast to bombs already alive in the room.
    let properties = crate::weapon_plan::resolve_fetus_kind5(
        inputs,
        state.weapon_admission.source_player_type,
        state.weapon_cache,
        state.player_stats,
    )?;
    if properties.flags.0 != 0 {
        state
            .weapon_admission
            .record_unadmitted(plan.primary, "ordinaryFetusBombModifiers");
        return Ok(DispatchOutcome::Unadmitted);
    }
    let player = crate::bombs::player_world(state);
    //52975E saves DD4 from the velocity entering this Player callback, before
    //the later input acceleration. Dispatch runs at that pre-motion seam.
    let dd4 = crate::bombs::player_velocity(state);
    if !callback_aim.is_finite()
        || !dc4.is_finite()
        || !player.is_finite()
        || !dd4.is_finite()
        || !plan.source_shot_speed_c7c.is_finite()
        || !plan.source_range_c8c.is_finite()
        || plan.max_fire_delay <= 0
    {
        return Err("Dr. Fetus requires actual finite caller vectors and cache fields".into());
    }
    //Each multiplication/addition is rounded separately, following51AAD6,
    //51AAFF,51AB28,51AB46,5204E2 and50DE39..59 respectively.
    let inherited = (dd4 * f32::from_bits(0x3f19_999a)) * 2.0;
    let directed = cardinal(callback_aim) * 12.0;
    let velocity =
        ((directed + inherited) * plan.source_shot_speed_c7c) * ((-plan.source_range_c8c) / 23.75);
    let position = player + dc4;
    let explosion_damage = properties.collision_damage * 3.0;
    if !position.is_finite()
        || !velocity.is_finite()
        || !explosion_damage.is_finite()
        || explosion_damage < 0.0
    {
        return Err("Dr. Fetus factory inputs overflowed".into());
    }
    //Positive i32 C78 times .45 remains within CVTTSS2SI's finite range.
    //Source stores these before the factory. Validation above has no effects.
    let head_timer = ((plan.max_fire_delay as f32) * f32::from_bits(0x3ee6_6666)) as i32;
    state.next_bomb.checked_add(1).ok_or("bomb IDs exhausted")?;
    state.fire_cooldown = plan.max_fire_delay;
    state.head_timer = head_timer.wrapping_add(7);
    let spawn_seed = state.boot_lifecycle.shared.next_u32();
    crate::bombs::admit_dr_fetus(
        state,
        phase,
        DrFetusBirth {
            spawn_seed,
            position,
            velocity,
            explosion_damage,
            flags: properties.flags,
            color: properties.color,
            parent_player: ActorRef {
                kind: ActorKind::Player,
                id: 0,
            },
        },
    )?;
    //520507 returns without519130/C18 binding or an explicit initialFull.
    Ok(DispatchOutcome::Handled)
}
