//! Literal519FC0 ordinary C08=2 charge/release caller. Laser/Impact storage,
//! initialization and body callbacks remain the shared Laser owner's services.
use crate::{
    actors::ActorRef,
    attack_runtime::{DispatchOutcome, ShootButtons},
    item_pipeline::ItemInputs,
    laser_runtime::OrdinaryBrimBirth,
    motion::WorldVector,
    phase::Phase,
    weapon_plan::{ControllerChannel, PrimaryWeaponC08, ResolvedWeaponPlan},
    State,
};

#[derive(Clone, Copy, Debug, PartialEq)]
enum ChargeStep {
    Hold { next_charge: i32 },
    ShortRelease,
    Release { direction: WorldVector },
}

///Immutable control resolution. No actor, slot, animation, audio or RNG writes
///occur until the concrete adapter consumes the corresponding source step.
fn resolve_charge(
    charge: i32,
    maximum: i32,
    held: bool,
    stored_aim: WorldVector,
    phase: Phase,
) -> Result<ChargeStep, String> {
    if maximum <= 0 || !stored_aim.is_finite() || phase == Phase::Skipped {
        return Err("ordinary Brimstone charge requires actual finite callback inputs".into());
    }
    if held {
        Ok(ChargeStep::Hold {
            next_charge: if phase == Phase::Update {
                charge.wrapping_add(1)
            } else {
                charge
            },
        })
    } else if charge < maximum {
        Ok(ChargeStep::ShortRelease)
    } else {
        //42E930 chooses the largest absolute axis, Y on a tie. A fresh zero
        //DB4 consequently chooses down; it is not a DA0 fallback.
        let direction = if stored_aim.x.abs() > stored_aim.y.abs() {
            WorldVector {
                x: if stored_aim.x < 0.0 { -1.0 } else { 1.0 },
                y: 0.0,
            }
        } else {
            WorldVector {
                x: 0.0,
                y: if stored_aim.y < 0.0 { -1.0 } else { 1.0 },
            }
        };
        Ok(ChargeStep::Release { direction })
    }
}

///50B720(kind4)'s genuine Mouth null, after508820 selects the actual118
///Head*Shoot clip at frame0. These four authored ANM values are source content,
///not recorded native positions. The admitted fresh selected wardrobe has
///unit Sprite/Layer scales;50B1B0's -5 offset subtract/add is exact here.
fn shoot_mouth_offset(direction: WorldVector) -> WorldVector {
    let mouth = if direction.x < 0.0 {
        WorldVector { x: -4.0, y: -13.0 }
    } else if direction.x > 0.0 {
        WorldVector { x: 4.0, y: -13.0 }
    } else if direction.y < 0.0 {
        WorldVector { x: 0.0, y: -12.0 }
    } else {
        WorldVector { x: 0.0, y: -18.0 }
    };
    //42EAD0 divides each source-pixel component by actual66895C.
    WorldVector {
        x: mouth.x / 0.65_f32,
        y: mouth.y / 0.65_f32,
    }
}

fn resolve_kind2_damage(state: &State) -> Result<f32, String> {
    crate::weapon_plan::resolve_neutral_kind2_damage(
        ItemInputs::new(
            state
                .player_items
                .as_ref()
                .ok_or("ordinary Brimstone actual Player items are absent")?,
        )?,
        state.weapon_admission.source_player_type,
        state.player_stats,
    )
}

///Known visual-only callback gaps never replace the independently owned C18
///history. They use the single retained diagnostic and never clear a failure.
fn record_visual_frontier(state: &mut State, channel: &str) {
    state.weapon_admission.unadmitted_controller.get_or_insert(
        crate::weapon_plan::UnadmittedController {
            primary: state.weapon_cache.primary.numeric(),
            channel: channel.into(),
        },
    );
}

pub(crate) fn dispatch_player(
    state: &mut State,
    plan: &ResolvedWeaponPlan,
    input: ShootButtons,
    phase: Phase,
    callback_aim: WorldVector,
) -> Result<DispatchOutcome, String> {
    if plan.primary != PrimaryWeaponC08::Brimstone
        || plan.controller != ControllerChannel::ChargedBrimstone
        || phase == Phase::Skipped
    {
        return Err("ordinary Brimstone requires its actual C08=2 caller".into());
    }
    //Pure numeric validation precedes any frontier diagnostic or effect.
    state.player_stats.validate()?;
    let player_position = crate::bombs::player_world(state);
    if !plan.secondary_color_ccc.is_finite()
        || plan.max_fire_delay <= 0
        || !player_position.is_finite()
    {
        return Err("ordinary Brimstone requires finite actual CCC and positive C78".into());
    }
    let (Some(charge), Some(stored_aim)) = (
        state.attack_runtime.charge_c0c,
        state.attack_runtime.shooting_aim_db4,
    ) else {
        state
            .weapon_admission
            .record_unadmitted(plan.primary, "ordinaryBrimstoneChargeHistory");
        return Ok(DispatchOutcome::Unadmitted);
    };
    let step = resolve_charge(charge, plan.max_fire_delay, input.held(), stored_aim, phase)?;
    if state.attack_runtime.persistent.is_none() && !state.weapon_admission.c18_is_known_empty {
        state
            .weapon_admission
            .record_unadmitted(plan.primary, "ordinaryBrimstonePriorActorHistory");
        return Ok(DispatchOutcome::Unadmitted);
    }
    let items = ItemInputs::new(
        state
            .player_items
            .as_ref()
            .ok_or("ordinary Brimstone items absent")?,
    )?;
    if !crate::weapon_plan::admits_neutral_brimstone_properties(
        items,
        state.weapon_admission.source_player_type,
    ) || !items.positive_owned_count(118)
        || items.primary_permanent_presence(329)?
    {
        state
            .weapon_admission
            .record_unadmitted(plan.primary, "ordinaryBrimstoneProperties");
        return Ok(DispatchOutcome::Unadmitted);
    }
    //50B720's admitted ordinary costume/offset path is nonflying. Read the
    //actual D68/class340 outputs rather than substituting unknown history.
    if state.player.flying != Some(false) || state.player.grid_collision_class != Some(5) {
        state
            .weapon_admission
            .record_unadmitted(plan.primary, "ordinaryBrimstoneFlightHistory");
        return Ok(DispatchOutcome::Unadmitted);
    }
    match step {
        ChargeStep::Hold { next_charge } => {
            //51D110 precedes wardrobe/charge work. Remove does not clear C18;
            //real manager Uninit remains the registered-pointer producer.
            if let Some(actor) = state.attack_runtime.persistent {
                if !matches!(
                    actor.kind,
                    crate::actors::ActorKind::Knife | crate::actors::ActorKind::Laser
                ) {
                    state
                        .weapon_admission
                        .record_unadmitted(plan.primary, "ordinaryBrimstonePriorActorRemove");
                    return Ok(DispatchOutcome::Unadmitted);
                }
                crate::attack_runtime::remove_referenced_actor(state, actor)?;
            }
            if !crate::player_costume118::charge(state, callback_aim, charge, plan.max_fire_delay)?
            {
                record_visual_frontier(state, "ordinaryBrimstoneChargeWardrobe");
            }
            state.attack_runtime.charge_c0c = Some(next_charge);
            state.head_timer = 1; //51C5D4 actual BB0.
        }
        ChargeStep::ShortRelease => {
            if state.attack_runtime.persistent.is_none() {
                //51EFC1 hides13 actual costume slots only in this NULL branch.
                if !crate::player_costume118::hide(state) {
                    record_visual_frontier(state, "ordinaryBrimstoneHideWardrobe");
                }
            }
            state.attack_runtime.charge_c0c = Some(0);
        }
        ChargeStep::Release { direction } => {
            match state.attack_runtime.player_dc4 {
                Some(offset) if offset.is_finite() => {}
                Some(_) => return Err("ordinary Brimstone current Player DC4 is nonfinite".into()),
                None => {
                    state
                        .weapon_admission
                        .record_unadmitted(plan.primary, "ordinaryBrimstoneParentHistory");
                    return Ok(DispatchOutcome::Unadmitted);
                }
            }
            let birth = OrdinaryBrimBirth {
                player_position,
                aim_db4_cardinal: direction,
                render_offset_from_50b720: shoot_mouth_offset(direction),
                flags_c98: plan.properties.flags,
                color_ccc: plan.secondary_color_ccc,
            };
            if !crate::player_costume118::shoot(state, direction)? {
                //The actual Mouth lookup consumes the selected Shoot clip.
                //Unknown wardrobe history cannot supply an invented offset.
                state
                    .weapon_admission
                    .record_unadmitted(plan.primary, "ordinaryBrimstoneMouthHistory");
                return Ok(DispatchOutcome::Unadmitted);
            }
            //RealRaw/Init/P/Parent/lifetime→C98+CCC→kind2 getter. The supplied
            //getter is invoked AFTER Spawn, never eagerly in this plan.
            let actor: ActorRef =
                crate::laser_runtime::admit_ordinary_brim(state, birth, resolve_kind2_damage)?;
            crate::attack_runtime::replace_c18(state, Some(actor))?;
            //5205C1 is an explicit FULL callback even on an outer fast phase.
            crate::laser_runtime::advance_ordinary_brim_initial(state, actor)?;
            //No55/87 extra-emission/proc paths are admitted in this footprint.
            state.attack_runtime.charge_c0c = Some(0);
        }
    }
    Ok(DispatchOutcome::Handled)
}
