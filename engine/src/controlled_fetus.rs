//!519FC0 C08=5/Have329: a timed canonical Bomb with held steering.
//!50DE10 owns born properties; reuse never refreshes damage, color or fuse.
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

fn frontier(state: &mut State, channel: &str) -> DispatchOutcome {
    state
        .weapon_admission
        .record_unadmitted(state.weapon_cache.primary, channel);
    DispatchOutcome::Unadmitted
}

pub(crate) fn dispatch_player(
    state: &mut State,
    plan: &ResolvedWeaponPlan,
    input: ShootButtons,
    phase: Phase,
    callback_aim: WorldVector,
) -> Result<DispatchOutcome, String> {
    if plan.primary != PrimaryWeaponC08::DrFetus
        || plan.controller != ControllerChannel::ControlledFetus
        || phase == Phase::Skipped
    {
        return Err("controlled Dr. Fetus requires actual C08=5/Have329".into());
    }
    if !callback_aim.is_finite() || !plan.source_shot_speed_c7c.is_finite() {
        return Err("controlled Dr. Fetus current aim/C7C is nonfinite".into());
    }
    let inputs = ItemInputs::new(
        state
            .player_items
            .as_ref()
            .ok_or("controlled Dr. Fetus items absent")?,
    )?;
    if state.weapon_admission.source_player_type != Some(0)
        || !inputs.admits_controlled_fetus_caller()
    {
        return Ok(frontier(state, "controlledFetusContext"));
    }
    if let Some(previous) = state.attack_runtime.persistent {
        if previous.kind != ActorKind::Bomb {
            if !crate::attack_runtime::admits_referenced_remove(state, previous)? {
                return Ok(frontier(state, "controlledFetusC18Family"));
            }
            //51A691..A9 unregisters C18 BEFORE the old family's virtual48.
            crate::attack_runtime::replace_c18(state, None)?;
            crate::attack_runtime::remove_referenced_actor(state, previous)?;
        }
    } else if !state.weapon_admission.c18_is_known_empty {
        return Ok(frontier(state, "controlledFetusC18History"));
    }
    //51A6AC returns on release. Existing Bomb motion and fuse keep advancing.
    if !input.held() {
        return Ok(DispatchOutcome::Handled);
    }
    if let Some(actor) = state.attack_runtime.persistent {
        let Some(bomb) = state.bombs.iter_mut().find(|bomb| bomb.id == actor.id) else {
            return Err("controlled Dr. Fetus registered Bomb is unresolved".into());
        };
        if !crate::bombs::admits_registered_shot(bomb)? {
            return Ok(frontier(state, "controlledFetusBombHistory"));
        }
        let mut aim = callback_aim;
        let squared = aim.x * aim.x + aim.y * aim.y;
        if squared > 1.0 {
            aim = aim * (1.0 / squared.sqrt());
        }
        //51A70F..51A78C retains three separately rounded multiplications.
        let force = ((aim * plan.source_shot_speed_c7c) * 1.2) * 0.5;
        if bomb.friction != 0.0 {
            bomb.motion.velocity = bomb.motion.velocity
                + WorldVector {
                    x: force.x / bomb.friction,
                    y: force.y / bomb.friction,
                };
        }
        if !bomb.motion.velocity.is_finite() {
            return Err("controlled Dr. Fetus force overflowed Bomb velocity".into());
        }
        //The separate neutral caller capability excludes153/2/trinket2.
        //Its real factory has known NULL Child; no chain is invented here.
        return Ok(DispatchOutcome::Handled);
    }
    if state.fire_cooldown >= 0 {
        return Ok(DispatchOutcome::Handled);
    }
    let inputs = ItemInputs::new(
        state
            .player_items
            .as_ref()
            .ok_or("controlled Dr. Fetus items absent")?,
    )?;
    let properties = crate::weapon_plan::resolve_fetus_kind5(
        inputs,
        state.weapon_admission.source_player_type,
        state.weapon_cache,
        state.player_stats,
    )?;
    if properties.flags.0 != 0 {
        return Ok(frontier(state, "controlledFetusKind5Modifiers"));
    }
    let Some(dc4) = state.attack_runtime.player_dc4 else {
        return Ok(frontier(state, "controlledFetusPlayerDc4History"));
    };
    //520C35 passes PlayerP;50DE22..33 then adds actual PlayerDC4.
    let position = crate::bombs::player_world(state) + dc4;
    let inherited = (crate::bombs::player_velocity(state) * 0.6) * 2.0;
    let cardinal = if callback_aim.x.abs() > callback_aim.y.abs() {
        WorldVector {
            x: if callback_aim.x < 0.0 { -1.0 } else { 1.0 },
            y: 0.0,
        }
    } else {
        WorldVector {
            x: 0.0,
            y: if callback_aim.y < 0.0 { -1.0 } else { 1.0 },
        }
    };
    //520BF8..520C50: extra .5 precedes50DE10's separate range multiplier.
    let velocity = (((cardinal * 12.0 + inherited) * plan.source_shot_speed_c7c) * 0.5)
        * ((-plan.source_range_c8c) / 23.75);
    let damage = properties.collision_damage * 3.0;
    if !position.is_finite()
        || !velocity.is_finite()
        || !damage.is_finite()
        || damage < 0.0
        || plan.max_fire_delay <= 0
    {
        return Err("controlled Dr. Fetus factory inputs are invalid".into());
    }
    state.next_bomb.checked_add(1).ok_or("bomb IDs exhausted")?;
    state.fire_cooldown = plan.max_fire_delay;
    state.head_timer = ((plan.max_fire_delay as f32) * 0.45) as i32;
    state.head_timer = state.head_timer.wrapping_add(7);
    let spawn_seed = state.boot_lifecycle.shared.next_u32();
    let id = crate::bombs::admit_dr_fetus(
        state,
        phase,
        DrFetusBirth {
            spawn_seed,
            position,
            velocity,
            explosion_damage: damage,
            flags: properties.flags,
            color: properties.color,
            parent_player: ActorRef {
                kind: ActorKind::Player,
                id: 0,
            },
        },
    )?;
    //520C62 binds the returned timed Bomb. No explicit initial Full is called.
    crate::attack_runtime::replace_c18(
        state,
        Some(ActorRef {
            kind: ActorKind::Bomb,
            id,
        }),
    )?;
    Ok(DispatchOutcome::Handled)
}
