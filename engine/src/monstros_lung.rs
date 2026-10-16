//!519FC0 C08=7 charge/release and13 canonical5177F0 Blood Tears.
//!C0C, DB4, current cache and shared MT retain their existing owners.
use crate::{
    attack_runtime::{DispatchOutcome, ShootButtons},
    item_pipeline::ItemInputs,
    motion::WorldVector,
    phase::Phase,
    weapon_plan::{ControllerChannel, PrimaryWeaponC08, ResolvedWeaponPlan},
    State,
};

fn frontier(state: &mut State, channel: &str) -> DispatchOutcome {
    state
        .weapon_admission
        .record_unadmitted(state.weapon_cache.primary, channel);
    DispatchOutcome::Unadmitted
}

///42E930 takes Y on ties, including an actual zero DB4 -> down.
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
    if plan.primary != PrimaryWeaponC08::MonstrosLung
        || plan.controller != ControllerChannel::MonstroLung
        || phase == Phase::Skipped
    {
        return Err("Monstro's Lung requires its actual C08=7 caller".into());
    }
    let inputs = ItemInputs::new(
        state
            .player_items
            .as_ref()
            .ok_or("Monstro item owner absent")?,
    )?;
    if state.weapon_admission.source_player_type != Some(0) || !inputs.admits_monstro_caller() {
        return Ok(frontier(state, "monstroLungCallerContext"));
    }
    let (Some(prior_db4), Some(charge)) = (
        state.attack_runtime.shooting_aim_db4,
        state.attack_runtime.charge_c0c,
    ) else {
        return Ok(frontier(state, "monstroLungChargeAimHistory"));
    };
    if !callback_aim.is_finite() || !prior_db4.is_finite() || plan.max_fire_delay <= 0 {
        return Err("Monstro charge requires actual finite aim and positive C78".into());
    }
    //51B9B8 cardinalizes prior DB4 before branching on held input. The outer
    //caller writes the new DB4 only after this callback returns.
    let direction = cardinal(prior_db4);
    if input.held() {
        if charge < plan.max_fire_delay {
            let current = if phase == Phase::Update {
                let current = charge.wrapping_add(1);
                state.attack_runtime.charge_c0c = Some(current);
                current
            } else {
                charge
            };
            let _ = crate::player_costume229::partial_charge(
                state,
                callback_aim,
                current,
                plan.max_fire_delay,
            )?;
        } else {
            let _ = crate::player_costume229::full_charge(state, callback_aim)?;
        }
        //51C5D4/51DEA6 keep BB0=1 for partial and fully charged holds.
        //Visual-only wardrobe frontiers stay in their independent pose owner.
        state.head_timer = 1;
        return Ok(DispatchOutcome::Handled);
    }
    if charge < plan.max_fire_delay {
        state.attack_runtime.charge_c0c = Some(0);
        return Ok(DispatchOutcome::Handled);
    }
    if !inputs.admits_monstro_kind1_context()
        || inputs.pending_cache().0 & (1 | 2 | 4 | 8 | 32 | 64) != 0
        || state.weapon_cache.flags_c98.0 != 0
    {
        return Ok(frontier(state, "monstroLungKind1Cache"));
    }
    let Some(dc4) = state.attack_runtime.player_dc4 else {
        return Ok(frontier(state, "monstroLungPlayerDc4History"));
    };
    let origin = crate::bombs::player_world(state) + dc4;
    let inherited = crate::bombs::player_velocity(state);
    if !origin.is_finite() || !inherited.is_finite() || !dc4.is_finite() {
        return Err("Monstro burst has nonfinite actual Player vectors".into());
    }
    //51BC99..BCE6 adds ten-unit cardinal aim and50A4F0's inherited DD4;
    //51BFA4 multiplies the entire base vector by current C7C.
    let velocity =
        crate::projectiles::stored_velocity(direction, inherited, plan.source_shot_speed_c7c)?;
    let initial = crate::weapon_plan::resolve_monstro_kind1(state)?;
    state
        .next_tear
        .checked_add(13)
        .ok_or("Monstro Tear IDs exhausted")?;
    let _ = crate::player_costume229::shoot(state, direction)?;
    state.fire_cooldown = plan.max_fire_delay;
    state.head_timer = initial.head_timer;
    //Neutral Have153 false gives17-4=13; Have2/TEMP2 and245/TEMP245
    //are independently excluded by this caller's source footprint.
    for _ in 0..13 {
        let uniform = state.boot_lifecycle.shared.uniform();
        //42E8D0 itself consumes the next global Uniform, then source sincosf.
        let angle =
            (f64::from(state.boot_lifecycle.shared.uniform()) * std::f64::consts::PI * 2.0) as f32;
        let scatter = (WorldVector {
            x: crate::fmath::cos(angle),
            y: crate::fmath::sin(angle),
        } * uniform)
            * 3.5;
        //The real kind1 builder is evaluated for each factory invocation,
        //after its spread draws. Later cache operations never rewrite births.
        let birth = crate::weapon_plan::resolve_monstro_kind1(state)?;
        let index = crate::tear_factory::fire(state, birth, Some(1), origin, velocity + scatter)?;
        //These setters run AFTER the factory's immediate Full and subtraction.
        state.tears[index].motion.falling_acceleration =
            0.5 + state.player_stats.tear_falling_acceleration;
        let falling =
            state.player_stats.tear_falling_speed - state.boot_lifecycle.shared.uniform() * 15.0;
        state.tears[index].motion.falling_speed = 5.0 + falling;
        let size_choice = state.boot_lifecycle.shared.bounded(2);
        let size_uniform = state.boot_lifecycle.shared.uniform();
        let tear = &mut state.tears[index];
        let factor = (size_choice as f32 * 0.4 + 0.9) + size_uniform * 0.05;
        tear.scale = (factor * tear.scale).max(0.01);
        //5418E0 shares these tiers/radius for actual variants0 and1; the
        //authored selected animation is BloodTear for variant1.
        tear.animation_tier = crate::projectiles::regular_animation_tier(tear.scale)?;
        tear.collision_radius = crate::projectiles::collision_radius(tear.scale, 8.0)?;
    }
    state.attack_runtime.charge_c0c = Some(0);
    Ok(DispatchOutcome::Handled)
}
