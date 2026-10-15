//! Mutable weapon actor ownership, separate from immutable item evaluation.
use crate::{
    actors::{ActorKind, ActorRef},
    knives::{AttackFlags, Knife, ParentView},
    motion::WorldVector,
    phase::Phase,
    weapon_plan::{ControllerChannel, KnifeContactAdmission, ResolvedWeaponPlan},
    State,
};
use serde::{Deserialize, Serialize};

pub use crate::attack_color::AttackColor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ShootButtons {
    pub(crate) left: bool,
    pub(crate) right: bool,
    pub(crate) up: bool,
    pub(crate) down: bool,
}

impl ShootButtons {
    /// Actual public keyboard actions4/5/6/7 retain independent held values.
    pub(crate) fn read(pressed: &std::collections::BTreeSet<String>) -> Self {
        Self {
            left: pressed.contains("ArrowLeft"),
            right: pressed.contains("ArrowRight"),
            up: pressed.contains("ArrowUp"),
            down: pressed.contains("ArrowDown"),
        }
    }

    pub(crate) fn aim(self) -> WorldVector {
        WorldVector {
            x: -f32::from(u8::from(self.left)) + f32::from(u8::from(self.right)),
            y: -f32::from(u8::from(self.up)) + f32::from(u8::from(self.down)),
        }
    }

    pub(crate) fn held(self) -> bool {
        self.left || self.right || self.up || self.down
    }

    ///53057a normal Knife quantizes to pi/4 before restoring the original
    ///magnitude. Controlled Has329 bypasses this operator entirely.
    pub(crate) fn ordinary_knife_aim(self) -> WorldVector {
        let raw = self.aim();
        let octant =
            (f64::from(crate::fmath::atan2_f32(raw.y, raw.x)) / std::f64::consts::FRAC_PI_4) as f32;
        let rounded = if octant < 0.0 {
            (octant - 0.5).ceil()
        } else {
            (octant + 0.5).floor()
        };
        let angle = (f64::from(rounded) * std::f64::consts::FRAC_PI_4) as f32;
        let (sin, cos) = (crate::fmath::sin(angle), crate::fmath::cos(angle));
        WorldVector { x: cos, y: sin } * crate::knives::squared_length(raw).sqrt()
    }
}

///52F485..4C2 distinguishes a new nonzero processed input from release.
///Canceled held input takes52FA08/612A30: use the negated prior DB4 for this
///callback, retain DB4, and do not manufacture an unknown prior direction.
pub(crate) struct ShootingAimStep {
    pub(crate) callback_aim: WorldVector,
    pub(crate) db4_write: Option<WorldVector>,
}

pub(crate) fn ordinary_shooting_aim(
    processed_aim: WorldVector,
    held: bool,
    prior_db4: Option<WorldVector>,
) -> Result<Option<ShootingAimStep>, String> {
    if !processed_aim.is_finite() {
        return Err("ordinary processed shooting aim is nonfinite".into());
    }
    if processed_aim.x != 0.0 || processed_aim.y != 0.0 {
        Ok(Some(ShootingAimStep {
            callback_aim: processed_aim,
            db4_write: Some(processed_aim),
        }))
    } else if held {
        let Some(prior) = prior_db4 else {
            return Ok(None);
        };
        if !prior.is_finite() {
            return Err("ordinary prior shooting aim DB4 is nonfinite".into());
        }
        //612A30 XORs the sign bit of both components, preserving signed zero.
        Ok(Some(ShootingAimStep {
            callback_aim: WorldVector {
                x: -prior.x,
                y: -prior.y,
            },
            db4_write: None,
        }))
    } else {
        Ok(Some(ShootingAimStep {
            callback_aim: processed_aim,
            db4_write: None,
        }))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DispatchOutcome {
    RegularTear,
    Handled,
    Unadmitted,
}

/// Called only at the source-admitted519fc0 seam. Guard ownership and immutable
/// live controller resolution precede this mutable actor/RNG operation.
pub(crate) fn dispatch_player(
    state: &mut State,
    plan: &ResolvedWeaponPlan,
    input: ShootButtons,
    phase: Phase,
    callback_aim: WorldVector,
) -> Result<DispatchOutcome, String> {
    match plan.controller {
        ControllerChannel::RegularShots => Ok(DispatchOutcome::RegularTear),
        ControllerChannel::OrdinaryKnife => ordinary_knife(state, plan, input, phase, callback_aim),
        ControllerChannel::OrdinaryFetus => {
            crate::dr_fetus::dispatch_player(state, plan, input, phase, callback_aim)
        }
        ControllerChannel::ControlledKnife => {
            controlled_knife(state, plan, input, phase, callback_aim)
        }
        ControllerChannel::ChargedBrimstone => {
            crate::brimstone::dispatch_player(state, plan, input, phase, callback_aim)
        }
        ControllerChannel::ControlledTear => {
            crate::controlled_tear::dispatch(state, input, phase, callback_aim)
        }
        unsupported => {
            let channel = match unsupported {
                ControllerChannel::ControlledBrimstone => "controlledBrimstone",
                ControllerChannel::Unimplemented(_) => "unimplementedPrimary",
                _ => unreachable!("implemented controller matched above"),
            };
            //One bounded first-attempt diagnostic persists for this run. This
            //branch performs no admission, RNG, actor or discharge writes.
            record_frontier(state, channel);
            Ok(DispatchOutcome::Unadmitted)
        }
    }
}

fn record_frontier(state: &mut State, channel: impl AsRef<str>) {
    state
        .weapon_admission
        .record_unadmitted(state.weapon_cache.primary, channel.as_ref());
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct KnifeBirthProperties {
    pub(crate) position: WorldVector,
    pub(crate) owner: ActorRef,
    pub(crate) attack_flags: u64,
    pub(crate) color: AttackColor,
    pub(crate) collision_damage: f32,
}

impl KnifeBirthProperties {
    fn validate_fields(self) -> Result<(), String> {
        if self.owner
            != (ActorRef {
                kind: ActorKind::Player,
                id: 0,
            })
            || !self.position.is_finite()
            || !self.collision_damage.is_finite()
            || self.collision_damage < 0.0
            || !self.color.is_finite()
        {
            return Err("Knife birth requires real finite player-owned values".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AttackRuntime {
    ///519599 initializes signed Player+C0C. Controlled dispatch preserves it;
    ///old/unsupported restore histories never receive an invented zero.
    #[serde(default)]
    pub charge_c0c: Option<i32>,
    ///Actual Player+DA0. Title/unsupported restore does not own a direction.
    #[serde(default)]
    pub head_direction_da0: Option<i32>,
    ///Actual Player+DB4 fresh519902/519915; unsupported histories remain absent.
    #[serde(default)]
    pub shooting_aim_db4: Option<WorldVector>,
    /// Actual 5198DA fresh owner, written by 529750 before 52F370.
    #[serde(default)]
    pub movement_direction_d9c: Option<i32>,
    ///Actual51992B/519937 current Player DC4 vector.
    #[serde(default)]
    pub player_dc4: Option<WorldVector>,
    /// Player+C18 is a registered slot, not an independently recomputed actor.
    pub persistent: Option<ActorRef>,
    /// Actual C20/C28/C30 slots; absence is distinct from Remove requested.
    pub secondary: [Option<ActorRef>; 3],
    pub next_knife: u32,
}

impl AttackRuntime {
    ///519599 belongs to actual fresh Player admission, not the Title placeholder.
    pub(crate) fn fresh_player() -> Self {
        Self {
            charge_c0c: Some(0),
            head_direction_da0: Some(3), //5198e4, independently from C0C.
            movement_direction_d9c: Some(-1),
            shooting_aim_db4: Some(WorldVector { x: 0.0, y: 0.0 }),
            player_dc4: Some(WorldVector { x: 0.0, y: 0.0 }),
            ..Self::default()
        }
    }
}

///Normal52f5b8 head-view producer before519fc0. Zero aim preserves DA0;
///absolute ties take the Y branch. Controlled head-view uses a separate actor
///position producer and is not inferred from this ordinary input path.
pub(crate) fn ordinary_head_direction(state: &mut State, aim: WorldVector) {
    if aim.x != 0.0 || aim.y != 0.0 {
        state.attack_runtime.head_direction_da0 = Some(if aim.x.abs() > aim.y.abs() {
            if aim.x < 0.0 {
                0
            } else {
                2
            }
        } else if aim.y < 0.0 {
            1
        } else {
            3
        });
    }
}

pub(crate) fn update_movement_direction(state: &mut State, controls: bool) -> Result<(), String> {
    if state.weapon_admission.source_player_type != Some(0)
        || !state
            .player_items
            .as_ref()
            .is_some_and(|items| items.active_state_d80 == 0)
    {
        state.attack_runtime.movement_direction_d9c = None;
        return Ok(());
    }
    let scale = state.player_stats.move_speed * 0.25 + 0.75;
    if !scale.is_finite() || scale <= 0.0 {
        return Err("current movement direction requires its actual positive input scale".into());
    }
    let enabled = controls && state.player.controls_enabled && state.player.input_cooldown <= 0;
    let pressed = &state.pressed;
    let x = if enabled {
        f32::from(u8::from(pressed.contains("KeyD")))
            - f32::from(u8::from(pressed.contains("KeyA")))
    } else {
        0.0
    } * scale;
    let y = if enabled {
        f32::from(u8::from(pressed.contains("KeyS")))
            - f32::from(u8::from(pressed.contains("KeyW")))
    } else {
        0.0
    } * scale;
    //52981F initializes -1 each invocation; strict X dominance, then Y.
    state.attack_runtime.movement_direction_d9c = Some(if x.abs() > y.abs() {
        if x < 0.0 {
            0
        } else {
            2
        }
    } else if y < 0.0 {
        1
    } else if y > 0.0 {
        3
    } else {
        -1
    });
    Ok(())
}

pub(crate) fn controlled_head_direction(
    state: &mut State,
    aim: WorldVector,
    held: bool,
) -> Result<(), String> {
    let head_aim = if held {
        if let Some(actor) = state.attack_runtime.persistent {
            if actor.kind != ActorKind::Knife {
                record_frontier(state, "controlledKnifeHeadReference");
                return Ok(());
            }
            let knife = state
                .knives
                .iter()
                .find(|knife| knife.id == actor.id)
                .ok_or("controlled head C18 is unresolved")?;
            let player = crate::bombs::player_world(state);
            let delta = WorldVector {
                x: knife.position.x - player.x,
                y: knife.position.y - player.y,
            };
            if !delta.is_finite() {
                return Err("controlled head vector is nonfinite".into());
            }
            state.head_timer = 10;
            delta
        } else {
            aim
        }
    } else {
        aim
    };
    ordinary_head_direction(state, head_aim);
    Ok(())
}

pub(crate) fn controlled_idle_head(state: &mut State, held: bool) {
    if !held && state.head_timer < 0 {
        if let Some(direction) = state.attack_runtime.movement_direction_d9c {
            state.attack_runtime.head_direction_da0 = Some(match direction {
                0 => 0,
                1 => 1,
                2 => 2,
                _ => 3,
            });
        } else {
            record_frontier(state, "controlledKnifeMovementHistory");
        }
    }
}

fn controlled_actor_frontier(knife: &Knife) -> Result<Option<&'static str>, String> {
    if !knife.position.is_finite()
        || !knife.velocity.is_finite()
        || !knife.current_friction.is_finite()
        || !knife.sprite.rotation.is_finite()
        || !knife.source_color.is_finite()
        || !knife.current_color.is_finite()
        || !knife.collision_damage.is_finite()
    {
        return Err("controlled Knife requires finite actual actor fields".into());
    }
    if knife.variant != 0 || knife.subtype != 0 {
        return Ok(Some("controlledKnifeSubtypeHistory"));
    }
    if knife.parent
        != Some(ActorRef {
            kind: ActorKind::Player,
            id: 0,
        })
    {
        return Ok(Some("controlledKnifeParentHistory"));
    }
    if knife.child_chain_4a0 != crate::knives::ChildChain::Empty {
        return Ok(Some("controlledKnifeChildChainHistory"));
    }
    if knife.attack_flags.0 & (4 | 0x400 | 0x800000) != 0 {
        return Ok(Some("controlledKnifeProperties"));
    }
    Ok(None)
}

fn controlled_knife(
    state: &mut State,
    plan: &ResolvedWeaponPlan,
    input: ShootButtons,
    phase: Phase,
    aim: WorldVector,
) -> Result<DispatchOutcome, String> {
    if plan.primary.numeric() != 4 || phase == Phase::Skipped {
        return Err("controlled Knife requires actual C08=4 callback".into());
    }
    let request = KnifeBirthProperties {
        position: crate::bombs::player_world(state),
        owner: ActorRef {
            kind: ActorKind::Player,
            id: 0,
        },
        attack_flags: plan.properties.flags.0,
        color: plan.properties.color,
        collision_damage: plan.properties.collision_damage,
    };
    request.validate_fields()?;
    if !aim.is_finite() || !plan.source_shot_speed_c7c.is_finite() {
        return Err("controlled steering requires finite current inputs".into());
    }
    let items = crate::item_pipeline::ItemInputs::new(
        state
            .player_items
            .as_ref()
            .ok_or("controlled Knife item owner absent")?,
    )?;
    if !crate::weapon_plan::admits_neutral_knife_properties(
        items,
        state.weapon_admission.source_player_type,
    ) || request.attack_flags & (4 | 0x400 | 0x800000) != 0
    {
        record_frontier(state, "controlledKnifeProperties");
        return Ok(DispatchOutcome::Unadmitted);
    }
    let room_age = if !state.room.loaded {
        -1
    } else if let Some(initial) = state.room.initialization_game_update {
        state.game_update_count.wrapping_sub(initial) as i32
    } else {
        record_frontier(state, "controlledKnifeRoomAgeHistory");
        return Ok(DispatchOutcome::Unadmitted);
    };
    if let Some(actor) = state.attack_runtime.persistent {
        if actor.kind == ActorKind::Knife {
            let knife = state
                .knives
                .iter()
                .find(|knife| knife.id == actor.id)
                .ok_or("controlled C18 is unresolved")?;
            if let Some(channel) = controlled_actor_frontier(knife)? {
                record_frontier(state, channel);
                return Ok(DispatchOutcome::Unadmitted);
            }
        }
    }
    let actor = match state.attack_runtime.persistent {
        Some(actor) if actor.kind == ActorKind::Knife => actor,
        previous => {
            if let Some(previous) = previous {
                remove_referenced_actor(state, previous)?;
            }
            let id = state.attack_runtime.next_knife;
            let next = id
                .checked_add(1)
                .ok_or("controlled Knife handle exhausted")?;
            //50BDD0: SharedRaw -> Init uniform at zeroP -> requestedP/Parent.
            let seed = state.boot_lifecycle.shared.next_u32();
            let mut knife = Knife::initialize(
                id,
                seed,
                state.game_update_count,
                &mut state.boot_lifecycle.shared,
            );
            knife.position = request.position;
            knife.parent = Some(request.owner);
            knife.render_offset = WorldVector { x: 0.0, y: -10.0 };
            knife.apply_resolved(
                request.attack_flags,
                request.color,
                request.collision_damage,
            );
            knife.raw_7ec = 0.0; // The actual caller passes a literal zero angle.
            knife.entity_flags |= 0x200000;
            state.knives.push(knife);
            state.attack_runtime.next_knife = next;
            let actor = ActorRef {
                kind: ActorKind::Knife,
                id,
            };
            state.actor_scheduler.admit(actor)?;
            replace_c18(state, Some(actor))?;
            state
                .knives
                .last_mut()
                .expect("just admitted Knife")
                .apply_resolved(
                    request.attack_flags | crate::knives::CONTROLLED,
                    request.color,
                    request.collision_damage,
                );
            //Actual caller v18 remains full even when birth came from fast.
            advance_actor(state, Phase::Update, id, 4)?;
            actor
        }
    };
    //51B12B occurs after birth's explicit18. Remove retains registered slots.
    for extra in state.attack_runtime.secondary.into_iter().flatten() {
        remove_referenced_actor(state, extra)?;
    }
    let knife = state
        .knives
        .iter_mut()
        .find(|knife| knife.id == actor.id)
        .ok_or("controlled C18 disappeared")?;
    //51B16B common current-field reapply also runs after a newly born actor.
    knife.apply_resolved(
        request.attack_flags | crate::knives::CONTROLLED,
        request.color,
        request.collision_damage,
    );
    if room_age == 0 {
        knife.position = request.position;
    }
    if input.held() && knife.current_friction != 0.0 {
        let aim = if crate::knives::squared_length(aim) > 1.0 {
            crate::knives::normalized(aim)
        } else {
            aim
        };
        let scaled = ((aim * plan.source_shot_speed_c7c) * 2.5) * 0.5;
        knife.velocity = knife.velocity
            + WorldVector {
                x: scaled.x / knife.current_friction,
                y: scaled.y / knife.current_friction,
            };
    }
    if !knife.velocity.is_finite() {
        return Err("controlled steering overflowed actual fields".into());
    }
    Ok(DispatchOutcome::Handled)
}

pub(crate) fn ordinary_brimstone_idle_head(state: &mut State, held: bool) {
    if !held && state.head_timer < 0 && state.attack_runtime.persistent.is_none() {
        if let Some(direction) = state.attack_runtime.movement_direction_d9c {
            state.attack_runtime.head_direction_da0 = Some(match direction {
                0 => 0,
                1 => 1,
                2 => 2,
                _ => 3,
            });
        }
    }
}

///Neutral51d080 ordinary subtype0 caller. Each source family owns its own
///birth/bind/immediate-update order; the controlled adapter is not reused.
fn ordinary_knife(
    state: &mut State,
    plan: &ResolvedWeaponPlan,
    input: ShootButtons,
    phase: Phase,
    callback_aim: WorldVector,
) -> Result<DispatchOutcome, String> {
    if plan.primary.numeric() != 4 || phase == Phase::Skipped {
        return Err("ordinary Knife requires actual C08=4 Player callback".into());
    }
    let request = KnifeBirthProperties {
        position: crate::bombs::player_world(state),
        owner: ActorRef {
            kind: ActorKind::Player,
            id: 0,
        },
        attack_flags: plan.properties.flags.0,
        color: plan.properties.color,
        collision_damage: plan.properties.collision_damage,
    };
    request.validate_fields()?;
    let items = crate::item_pipeline::ItemInputs::new(
        state
            .player_items
            .as_ref()
            .ok_or("ordinary Knife item owner absent")?,
    )?;
    if !crate::weapon_plan::admits_neutral_knife_properties(
        items,
        state.weapon_admission.source_player_type,
    ) || request.attack_flags & (4 | 0x400 | 0x1_0000 | 0x800000) != 0
    {
        record_frontier(state, "ordinaryKnifeProperties");
        return Ok(DispatchOutcome::Unadmitted);
    }
    let (Some(charge), Some(head)) = (
        state.attack_runtime.charge_c0c,
        state.attack_runtime.head_direction_da0,
    ) else {
        //Unsupported continuation histories keep absence observable. Do not
        //manufacture the constructor values or freeze ordinary public ticks.
        record_frontier(state, "ordinaryKnifeChargeHistory");
        return Ok(DispatchOutcome::Unadmitted);
    };
    if !(0..=3).contains(&head) || plan.max_fire_delay <= 0 {
        return Err("ordinary Knife charge/head owner violates source domain".into());
    }
    //51e38c/51f0cd Remove the real three extra slots before C18 lookup.
    for extra in state.attack_runtime.secondary.into_iter().flatten() {
        remove_referenced_actor(state, extra)?;
    }
    let actor = match state.attack_runtime.persistent {
        Some(actor) if actor.kind == ActorKind::Knife => actor,
        previous => {
            if let Some(previous) = previous {
                remove_referenced_actor(state, previous)?;
            }
            let id = state.attack_runtime.next_knife;
            let next_id = id.checked_add(1).ok_or("knife handle exhausted")?;
            let seed = state.boot_lifecycle.shared.next_u32();
            let mut knife = Knife::initialize(
                id,
                seed,
                state.game_update_count,
                &mut state.boot_lifecycle.shared,
            );
            knife.position = request.position;
            knife.parent = Some(request.owner);
            knife.render_offset = WorldVector { x: 0.0, y: -10.0 };
            knife.entity_flags |= 0x200000;
            knife.apply_resolved(
                request.attack_flags,
                request.color,
                request.collision_damage,
            );
            state.knives.push(knife);
            state.attack_runtime.next_knife = next_id;
            let actor = ActorRef {
                kind: ActorKind::Knife,
                id,
            };
            state.actor_scheduler.admit(actor)?;
            replace_c18(state, Some(actor))?;
            advance_actor(state, Phase::Update, id, 4)?;
            //51d104 returns before aim, charge or release handling.
            return Ok(DispatchOutcome::Handled);
        }
    };
    let knife = state
        .knives
        .iter_mut()
        .find(|knife| knife.id == actor.id)
        .ok_or("ordinary Knife C18 reference unresolved")?;
    knife.apply_resolved(
        request.attack_flags,
        request.color,
        request.collision_damage,
    );
    knife.render_offset = WorldVector { x: 0.0, y: -10.0 };
    if input.held() && !knife.raw_814 && phase == Phase::Update {
        state.attack_runtime.charge_c0c = Some(charge.wrapping_add(1));
    }
    if !input.held() && !knife.raw_814 && charge > 0 {
        let maximum = plan.max_fire_delay.wrapping_shl(2);
        let clamped = charge.min(maximum);
        let range = (-plan.source_range_c8c * 10.0).clamp(0.1, 300.0);
        knife.launch_ordinary(clamped as f32 / maximum as f32, range)?;
        //51fb3d belongs after the selected no-extra-emission release branch.
        state.attack_runtime.charge_c0c = Some(0);
    }
    if !knife.raw_814 {
        let mut aim = callback_aim;
        if aim.x == 0.0 && aim.y == 0.0 {
            aim = match head {
                0 => WorldVector { x: -1.0, y: 0.0 },
                1 => WorldVector { x: 0.0, y: -1.0 },
                2 => WorldVector { x: 1.0, y: 0.0 },
                3 => WorldVector { x: 0.0, y: 1.0 },
                _ => unreachable!("DA0 validated above"),
            };
        }
        knife.raw_7e8 = crate::knives::angle_degrees(aim);
        state.head_timer = 7;
    } else {
        //51f805 chooses15 only while near the parent and moving outward.
        state.head_timer = if knife.raw_7f4 < 25.0 && knife.raw_7f8 > 0.0 {
            15
        } else {
            7
        };
    }
    Ok(DispatchOutcome::Handled)
}

///4524f0 selects the persistent manager pool by actual type8/subtype0. This
/// is not the unrelated Entity318 bit written by50bdd0's caller.
pub(crate) fn persists_across_room(actor: ActorRef, knives: &[Knife]) -> bool {
    actor.kind == ActorKind::Knife
        && knives
            .iter()
            .any(|knife| knife.id == actor.id && knife.subtype == 0)
}

pub(crate) fn reset_room_schedule(state: &mut State) {
    //453EC0 Uninit/Remove the transient pool before rebuilding the retained
    //type8/subtype0 pool. Active/dead is not this source pool predicate.
    let transient: Vec<_> = state
        .knives
        .iter()
        .filter(|knife| knife.subtype != 0)
        .map(|knife| ActorRef {
            kind: ActorKind::Knife,
            id: knife.id,
        })
        .collect();
    clear_knife_links(state, &transient);
    state.knives.retain(|knife| knife.subtype == 0);
    let knives = &state.knives;
    state
        .actor_scheduler
        .clear_room_retaining(|actor| persists_across_room(actor, knives));
}

pub(crate) fn remove_referenced_actor(state: &mut State, actor: ActorRef) -> Result<(), String> {
    match actor.kind {
        ActorKind::Knife => {
            let knife = state
                .knives
                .iter_mut()
                .find(|knife| knife.id == actor.id)
                .ok_or("registered knife handle is unresolved")?;
            knife.remove();
        }
        ActorKind::Laser => {
            state
                .lasers
                .iter_mut()
                .find(|laser| laser.id == actor.id)
                .ok_or("registered Laser handle is unresolved")?
                .remove();
        }
        ActorKind::Tear => crate::controlled_tear::remove(state, actor.id)?,
        _ => return Err("referenced attack actor has no admitted Remove callback".into()),
    }
    Ok(())
}

///519130 unregisters the old address before assigning/registering its new
/// target. Typed slots encode that same relation without native addresses.
pub(crate) fn replace_c18(state: &mut State, actor: Option<ActorRef>) -> Result<(), String> {
    if let Some(actor) = actor {
        let admitted = match actor.kind {
            ActorKind::Knife => state.knives.iter().any(|knife| knife.id == actor.id),
            ActorKind::Laser => state.lasers.iter().any(|laser| laser.id == actor.id),
            ActorKind::Tear => state
                .tears
                .iter()
                .any(|tear| tear.id == actor.id && tear.controlled.is_some()),
            _ => false,
        };
        if !admitted {
            return Err("C18 replacement requires an admitted weapon actor".into());
        }
    }
    state.attack_runtime.persistent = actor;
    state.weapon_admission.c18_is_known_empty = actor.is_none();
    Ok(())
}

pub(crate) fn advance_actor(
    state: &mut State,
    phase: Phase,
    id: u32,
    primary_c08: u32,
) -> Result<(), String> {
    if phase == Phase::Skipped {
        return Ok(());
    }
    let Some(index) = state.knives.iter().position(|knife| knife.id == id) else {
        return Err("updated Knife handle is unresolved".into());
    };
    let grid = crate::grid::Grid::new(
        state
            .room
            .grid
            .clone()
            .ok_or("knife requires its admitted actual room grid")?,
    )?;
    if state.knives[index].subtype == 0
        && state.knives[index].attack_flags.0 & crate::knives::CONTROLLED != 0
    {
        let parent = ParentView {
            position: crate::bombs::player_world(state),
            primary_c08,
        };
        if let Some(channel) = controlled_actor_frontier(&state.knives[index])? {
            record_frontier(state, channel);
            // Retain known presentation/tail work while the unmapped actor
            // effect remains an explicit admission failure.
            if phase == Phase::Interpolate {
                let knife = &mut state.knives[index];
                knife.fast = true;
                knife.saved_position = knife.position;
                knife.position = knife.position + (knife.velocity * knife.current_friction) * 0.5;
                if !knife.position.is_finite() {
                    return Err("unmapped controlled fast wrapper overflowed actual fields".into());
                }
                return Ok(());
            }
        } else if !crate::controlled_knife::advance(
            &mut state.knives[index],
            phase,
            parent,
            &grid,
            state.game_update_count,
            state.room.manager_counts.countable_total(),
        )? {
            return Ok(());
        }
    } else if state.knives[index].subtype == 1 {
        if phase == Phase::Interpolate {
            return state.knives[index].brim_child_fast();
        }
        if let Some(actor) = state.knives[index].parent {
            if actor.kind == ActorKind::Knife
                && !state.knives.iter().any(|knife| knife.id == actor.id)
            {
                return Err("Brim Knife linked parent is unresolved".into());
            }
        }
        state.knives[index].brim_child_full(&grid, state.game_update_count)?;
    } else if state.knives[index].subtype == 0
        && state.knives[index].parent
            == Some(ActorRef {
                kind: ActorKind::Player,
                id: 0,
            })
    {
        let parent = ParentView {
            position: crate::bombs::player_world(state),
            primary_c08,
        };
        let parent_has_118 = state
            .player_items
            .as_ref()
            .is_some_and(|owner| owner.owned_counts[118] > 0);
        let needs_child = state.knives[index].begin_ordinary_update(
            phase,
            parent,
            state.game_update_count,
            parent_has_118,
        )?;
        if needs_child {
            // An unconverted actor across an unadmitted history must not
            // invent fresh ordinary children while live329 selects control.
            let controlled = state
                .player_items
                .as_ref()
                .is_some_and(|owner| owner.owned_counts[329] > 0);
            if controlled {
                record_frontier(state, "controlledKnife");
            } else {
                admit_brim_knife_child(state, index, parent.position, &grid)?;
            }
        }
        if !state.knives[index].finish_ordinary_update(parent, state.game_update_count)? {
            return Ok(());
        }
    } else {
        record_frontier(state, "ordinaryKnifeParentHistory");
        return Ok(());
    }
    let receiver = crate::attack_grid::knife_callback(
        &mut state.room,
        &grid,
        state.knives[index].position,
        &mut state.boot_lifecycle.shared,
    )?;
    if let crate::attack_grid::GridCallbackOutcome::UnadmittedKind(kind) = receiver {
        record_frontier(state, format!("ordinaryKnifeGridKind{kind}"));
    }
    state.knives[index].finish_full()?;
    Ok(())
}

///473BCE prepares the parent's actual818 Line BEFORE473D00 Raw, then births
///real8/0/1 with cached spawnerKnife and explicit Parent link afterInit.
fn admit_brim_knife_child(
    state: &mut State,
    parent_index: usize,
    player: WorldVector,
    grid: &crate::grid::Grid,
) -> Result<(), String> {
    let parent = &mut state.knives[parent_index];
    crate::knife_path::rebuild(&mut parent.line, player, parent.raw_7e8, grid)?;
    let position = player + crate::knives::degree_direction(parent.raw_7e8) * 30.0;
    let parent_id = parent.id;
    let parent_variant = parent.variant;
    let color = parent.current_color;
    let damage = parent.collision_damage * 0.3;
    let line = parent.line.clone();
    let line_raw_18 = parent.line_raw_18;
    let id = state.attack_runtime.next_knife;
    let next = id.checked_add(1).ok_or("Knife child handle exhausted")?;
    let seed = state.boot_lifecycle.shared.next_u32();
    let mut child = Knife::initialize(
        id,
        seed,
        state.game_update_count,
        &mut state.boot_lifecycle.shared,
    );
    child.subtype = 1;
    child.position = position;
    child.spawner_type = 8;
    child.spawner_variant = parent_variant;
    child.render_offset = WorldVector { x: 0.0, y: -10.0 };
    child.apply_resolved(0, color, damage);
    child.line = line;
    child.line_raw_18 = line_raw_18;
    child.parent = Some(ActorRef {
        kind: ActorKind::Knife,
        id: parent_id,
    });
    state.knives.push(child);
    state.attack_runtime.next_knife = next;
    state.actor_scheduler.admit(ActorRef {
        kind: ActorKind::Knife,
        id,
    })?;
    let primary = state.weapon_cache.primary.numeric();
    advance_actor(state, Phase::Update, id, primary)
}

pub(crate) fn advance_scheduled_actor(
    state: &mut State,
    phase: Phase,
    id: u32,
    primary_c08: u32,
) -> Result<(), String> {
    //454df0 requires31e for its fenced full callback. RoomFast5cd920 does
    //not apply that guard; Remove and Uninit are distinct source operations.
    let knife = state
        .knives
        .iter()
        .find(|knife| knife.id == id)
        .ok_or("scheduled knife handle is unresolved")?;
    if phase == Phase::Update && !knife.active {
        return Ok(());
    }
    advance_actor(state, phase, id, primary_c08)
}

///448040 clears only slots still registered to this exact retired actor.
/// Replacement has already unregistered its old relation via519130.
pub(crate) fn uninit_retired(state: &mut State) {
    let retired: Vec<_> = state
        .knives
        .iter()
        .filter(|knife| !knife.active)
        .map(|knife| ActorRef {
            kind: ActorKind::Knife,
            id: knife.id,
        })
        .collect();
    clear_knife_links(state, &retired);
}

fn clear_knife_links(state: &mut State, retired: &[ActorRef]) {
    if state
        .attack_runtime
        .persistent
        .is_some_and(|actor| retired.contains(&actor))
    {
        state.attack_runtime.persistent = None;
        state.weapon_admission.c18_is_known_empty = true;
    }
    for slot in &mut state.attack_runtime.secondary {
        if slot.is_some_and(|actor| retired.contains(&actor)) {
            *slot = None;
        }
    }
    for knife in &mut state.knives {
        if retired.contains(
            &(ActorRef {
                kind: ActorKind::Knife,
                id: knife.id,
            }),
        ) || knife.parent.is_some_and(|parent| retired.contains(&parent))
        {
            knife.parent = None;
        }
    }
}

pub(crate) fn retire_full(state: &mut State) {
    uninit_retired(state);
    state.knives.retain(|knife| knife.active);
    //455358..455292 removes the retired persistent pointer from the current
    //manager arrays before contacts read them, not at the end of the tick.
    let knives = &state.knives;
    state.actor_scheduler.retain(|actor| {
        actor.kind != ActorKind::Knife || knives.iter().any(|knife| knife.id == actor.id)
    });
}

pub(crate) fn restore_fast_positions(state: &mut State) {
    for knife in &mut state.knives {
        knife.restore_fast_position();
    }
}

/// Actual circular445710 overlap calls virtual80 before mass separation.
/// Knife474e40 returns true, so its contacts never use the tear-only solver.
/// The caller supplies the real kind4 resolver; it runs after registry
/// insertion, once for each admitted contact, preserving conditional RNG.
pub(crate) fn knife_contacts(
    state: &mut State,
    preflight: impl Fn(&State, bool) -> Result<KnifeContactAdmission, String>,
    mut resolve_kind4: impl FnMut(&mut State) -> Result<f32, String>,
) -> Result<(), String> {
    let knife_handles: Vec<_> = state
        .actor_scheduler
        .full_phase()
        .into_iter()
        .filter(|actor| actor.kind == ActorKind::Knife)
        .collect();
    for actor in knife_handles {
        let own_sequence = state
            .actor_scheduler
            .sequence_id(actor)
            .ok_or("knife contact requires actual room sequence")?;
        let Some(index) = state.knives.iter().position(|knife| knife.id == actor.id) else {
            return Err("scheduled knife contact handle is unresolved".into());
        };
        if !state.knives[index].active
            || state.knives[index].dead
            || state.knives[index].entity_collision_class == 0
        {
            continue;
        }
        let parent_is_player = state.knives[index].parent
            == Some(ActorRef {
                kind: ActorKind::Player,
                id: 0,
            });
        let child_parent = state.knives[index].subtype == 1
            && state.knives[index]
                .parent
                .is_none_or(|actor| actor.kind == ActorKind::Knife);
        if !(state.knives[index].subtype == 0 && parent_is_player || child_parent) {
            record_frontier(state, "ordinaryKnifeParentHistory");
            continue;
        }
        for target_index in 0..state.room.npcs.len() {
            let npc = &state.room.npcs[target_index];
            //4453a0 manager eligibility precedes either actor's virtual80.
            if npc.removed
                || npc.dead
                || npc.entity_collision_class == 0
                || npc.entity_collision_class == 1
            {
                continue;
            }
            //444810 is independent of health and visibility. Fire33 is the
            //literal additional target branch in474e62 after manager guards.
            if npc.kind != 33
                && (!(10..1000).contains(&npc.kind)
                    || npc.raw_494
                    || npc.entity_flags() & 0x10 != 0)
            {
                continue;
            }
            let knife = &state.knives[index];
            if !knife.position.is_finite()
                || !npc.position.is_finite()
                || !knife.collision_radius.is_finite()
                || knife.collision_radius < 0.0
                || !npc.collision_radius.is_finite()
                || npc.collision_radius < 0.0
            {
                return Err("Knife contact requires finite actual geometry".into());
            }
            npc.health().validate()?;
            let delta = WorldVector {
                x: npc.position.x - knife.position.x,
                y: npc.position.y - knife.position.y,
            };
            let radius = knife.collision_radius + npc.collision_radius;
            let distance_squared = crate::knives::squared_length(delta);
            if !radius.is_finite() || !distance_squared.is_finite() {
                return Err("Knife contact geometry overflowed source f32 fields".into());
            }
            if delta.x.abs() > radius || delta.y.abs() > radius || distance_squared.sqrt() >= radius
            {
                continue;
            }
            let target_actor = ActorRef {
                kind: ActorKind::Npc,
                id: npc.id,
            };
            let sequence = state
                .actor_scheduler
                .sequence_id(target_actor)
                .ok_or("knife target requires actual room sequence")?;
            if !state.knives[index].can_admit_hit_sequence(
                state.game_update_count,
                own_sequence,
                sequence,
            ) {
                continue;
            }
            if !state.knives[index].collision_damage.is_finite()
                || state.knives[index].collision_damage < 0.0
            {
                return Err("Knife stored48c damage violates finite source domain".into());
            }
            let admission = preflight(state, parent_is_player)?;
            if admission != KnifeContactAdmission::Mapped {
                let channel = match admission {
                    KnifeContactAdmission::UnmappedBuilderModifiers => "knifeContactModifiers",
                    KnifeContactAdmission::UnknownTransformationHistory => "knifeContactHistory",
                    KnifeContactAdmission::TransformationProc => "knifeContactProc",
                    KnifeContactAdmission::Mapped => unreachable!("mapped contact handled below"),
                };
                record_frontier(state, channel);
                continue;
            }
            state.knives[index].admit_hit_sequence(state.game_update_count, own_sequence, sequence);
            let base_damage = if parent_is_player {
                resolve_kind4(state)?
            } else {
                // Actual473F/474F child contact uses its own stored48C.
                state.knives[index].collision_damage
            };
            let amount = state.knives[index].hit_damage(base_damage)?;
            //475030 calls511d90(firstPlayer,0) after current kind4 resolution
            //and the actor's charge multiplier. Fresh519767 and the selected
            //grant route prove zero; unknown/restored history is not false.
            let npc = &mut state.room.npcs[target_index];
            //4750b2 countdown30 is inert for ordinary flags0 receiver446cf0.
            let health = npc.health();
            npc.damage.enqueue(
                health,
                crate::combat::DamageEvent {
                    amount,
                    kind: crate::combat::DamageKind::Knife,
                    source: crate::combat::DamageSource {
                        id: actor.id,
                        entity_type: 8,
                    },
                },
            )?;
        }
    }
    Ok(())
}

impl Knife {
    pub(crate) fn apply_resolved(&mut self, flags: u64, color: AttackColor, damage: f32) {
        //The actual negative-duration446f00 setter writes permanent3a0,
        //then virtual88 (441c10) copies all11 words to current2d0 immediately.
        //This is a caller operation, not an item-cache broadcast or Base delay.
        self.source_color = color;
        self.current_color = self.source_color;
        self.sprite.tint = self.current_color.tint;
        self.sprite.offsets = self.current_color.offsets;
        self.attack_flags = AttackFlags(flags);
        self.collision_damage = damage;
    }
}

/// Only a new requested primary change performs virtual48. A previously
/// unperformed effect has no recovered prior address and stays pending.
pub(crate) fn complete_primary_cleanup(
    state: &mut State,
    previous: crate::weapon_plan::PrimaryWeaponC08,
) -> Result<bool, String> {
    if state.weapon_admission.pending_primary_cleanup.is_some() {
        record_frontier(state, "weaponPrimaryCleanupHistory");
        return Ok(false);
    }
    if state.weapon_cache.primary == previous {
        return Ok(true);
    }
    match state.attack_runtime.persistent {
        Some(actor)
            if matches!(
                actor.kind,
                ActorKind::Knife | ActorKind::Laser | ActorKind::Tear
            ) =>
        {
            remove_referenced_actor(state, actor)?;
            //Virtual48 preserves the registered C18 until actual Uninit.
            Ok(true)
        }
        None if state.weapon_admission.c18_is_known_empty => Ok(true),
        _ => {
            record_frontier(state, "weaponPrimaryCleanupActor");
            Ok(false)
        }
    }
}

pub(crate) fn contacts(state: &mut State) -> Result<(), String> {
    knife_contacts(
        state,
        |state, parent_is_player| {
            if !parent_is_player {
                return Ok(crate::weapon_plan::admit_knife_transformation_history(
                    state.player.transformation_counts,
                ));
            }
            state.player_stats.validate()?;
            if !state.weapon_cache.color_ca0.is_finite() {
                return Err("Knife contact requires finite actual Player color".into());
            }
            Ok(crate::weapon_plan::admit_neutral_knife_contact(
                crate::item_pipeline::ItemInputs::new(
                    state
                        .player_items
                        .as_ref()
                        .ok_or("Knife contact requires actual first Player items")?,
                )?,
                state.weapon_admission.source_player_type,
                state.player.transformation_counts,
            ))
        },
        |state| {
            crate::weapon_plan::resolve_neutral_kind4_damage(
                crate::item_pipeline::ItemInputs::new(
                    state
                        .player_items
                        .as_ref()
                        .ok_or("Knife contact requires actual first Player items")?,
                )?,
                state.weapon_admission.source_player_type,
                state.player_stats,
            )
        },
    )
}
