//! Literal519FC0 C08=3 ordinary Technology caller. Variant2 uses the sole
//! Laser allocator/body; the positive329 controller is a separate frontier.
use crate::{
    attack_runtime::{DispatchOutcome, ShootButtons},
    item_pipeline::ItemInputs,
    laser_runtime::OrdinaryTechnologyBirth,
    motion::WorldVector,
    phase::Phase,
    weapon_plan::{ControllerChannel, PrimaryWeaponC08, ResolvedWeaponPlan},
    State,
};

///Track actual costume registration, rather than reconstructing null priority
///from inventory on restore. Proven null-free acquisitions retain known absence;
///other histories and existing Technology geometry stay unknown after a grant.
pub(crate) fn on_collectible_added(state: &mut State, id: u32) {
    let normal = state.weapon_admission.source_player_type == Some(0);
    let Some(items) = state.player_items.as_mut() else {
        return;
    };
    let fresh_registration = id == 68
        && normal
        && items.technology_eye_registered == Some(false)
        && items.owned_counts[68] == 1
        && items.temporary_collectibles.is_empty()
        && items.trinket_slots == [0, 0]
        && items
            .owned_counts
            .iter()
            .enumerate()
            .all(|(index, &count)| count == 0 || (index == 68 && count == 1));
    //52D196/511060 registers the actual costume, then50EDC0 reselects layers.
    //The1/4/14/143 ANMs contain no Nulls;411490 skin reload changes only sheet
    //paths, so these acquisitions cannot introduce the Technology eye null.
    //This preserves only a known absence, never restored None or Some(true)
    //whose selected-layer geometry needs a separate producer.
    let preserves_absence =
        normal && items.technology_eye_registered == Some(false) && matches!(id, 1 | 4 | 14 | 143);
    items.technology_eye_registered = if fresh_registration {
        Some(true)
    } else if preserves_absence {
        Some(false)
    } else {
        None
    };
}

fn record_frontier(state: &mut State, channel: &str) -> DispatchOutcome {
    state
        .weapon_admission
        .record_unadmitted(state.weapon_cache.primary, channel);
    DispatchOutcome::Unadmitted
}

///42E930 chooses the greatest absolute axis, with Y winning a tie. Even
///canceled held input retains the ordinary callback's actual negated DB4.
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

///50B720(kind0) selects LeftEye;50B1B0 requests the directional Head* clip
///and411110's explicit frame1, independently of the current costume clock.
///These actual068 null frames have unit scales. In fresh normal Isaac the
///scaled(0,-5) subtraction and base layer4 position/addition cancel exactly.
fn eye_offset(aim: WorldVector) -> WorldVector {
    let eye = if aim.x < 0.0 {
        WorldVector { x: -5.0, y: -18.0 }
    } else if aim.x > 0.0 {
        WorldVector { x: 2.0, y: -21.0 }
    } else if aim.y < 0.0 {
        WorldVector { x: -7.0, y: -24.0 }
    } else {
        WorldVector { x: 8.0, y: -19.0 }
    };
    WorldVector {
        x: eye.x / 0.65_f32,
        y: eye.y / 0.65_f32,
    }
}

fn current_kind2_damage(state: &State) -> Result<f32, String> {
    crate::weapon_plan::resolve_neutral_kind2_damage(
        ItemInputs::new(
            state
                .player_items
                .as_ref()
                .ok_or("Technology items absent")?,
        )?,
        state.weapon_admission.source_player_type,
        state.player_stats,
    )
}

pub(crate) fn dispatch_player(
    state: &mut State,
    plan: &ResolvedWeaponPlan,
    input: ShootButtons,
    phase: Phase,
    callback_aim: WorldVector,
) -> Result<DispatchOutcome, String> {
    if plan.primary != PrimaryWeaponC08::Technology
        || plan.controller != ControllerChannel::OrdinaryTechnology
        || phase == Phase::Skipped
    {
        return Err("Technology requires its actual C08=3 ordinary caller".into());
    }
    state.player_stats.validate()?;
    let position = crate::bombs::player_world(state);
    if !callback_aim.is_finite()
        || !position.is_finite()
        || !plan.secondary_color_ccc.is_finite()
        || plan.max_fire_delay <= 0
    {
        return Err("Technology requires finite actual aim/position/CCC and positive C78".into());
    }
    //51B2C3/2CC: no Full-only branch is inserted here. The caller owns its
    //both-phase timers; an admitted FAST callback can also reach this factory.
    if !input.held() || state.fire_cooldown >= 0 {
        return Ok(DispatchOutcome::Handled);
    }
    let owner = state
        .player_items
        .as_ref()
        .ok_or("Technology item owner absent")?;
    let inputs = ItemInputs::new(owner)?;
    if !crate::weapon_plan::admits_neutral_technology_properties(
        inputs,
        state.weapon_admission.source_player_type,
    ) || inputs.primary_permanent_presence(329)?
    {
        return Ok(record_frontier(state, "ordinaryTechnologyProperties"));
    }
    if owner.technology_eye_registered != Some(true) {
        return Ok(record_frontier(state, "ordinaryTechnologyEyeHistory"));
    }
    if state.player.flying != Some(false) || state.player.grid_collision_class != Some(5) {
        return Ok(record_frontier(state, "ordinaryTechnologyFlightHistory"));
    }
    match state.attack_runtime.player_dc4 {
        Some(value) if value.is_finite() => {}
        Some(_) => return Err("Technology current Player DC4 is nonfinite".into()),
        None => return Ok(record_frontier(state, "ordinaryTechnologyParentHistory")),
    }
    let aim = cardinal(callback_aim);
    let offset = eye_offset(aim); //50E211 precedes the476120 Raw consumer.
    state.fire_cooldown = plan.max_fire_delay;
    state.head_timer = (plan.max_fire_delay as f32 * -0.4_f32) as i32 + 7;
    let actor = crate::laser_runtime::admit_ordinary_technology(
        state,
        OrdinaryTechnologyBirth {
            player_position: position,
            aim_cardinal: aim,
            render_offset_from_50b720: offset,
            flags_c98: plan.properties.flags,
            color_ccc: plan.secondary_color_ccc,
        },
        current_kind2_damage,
    )?;
    //51B547→50E1C0→immediate virtual18. Ordinary68 never registers C18.
    crate::laser_runtime::advance_ordinary_technology_initial(state, actor)?;
    Ok(DispatchOutcome::Handled)
}
