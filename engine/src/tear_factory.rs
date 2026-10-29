//! Canonical selected5177F0 Tear construction, shared by Player callers.
//! Origin/velocity and the actual kind1 result belong to each caller. Init,
//! shared RNG/audio, properties, immediate Full and registration stay here.
use crate::{
    actors, audio, motion::WorldVector, projectiles, weapon_plan::RegularTearPlan, State, Tear,
};

pub(crate) fn fire(
    state: &mut State,
    plan: RegularTearPlan,
    source_variant: Option<u32>,
    origin: WorldVector,
    velocity: WorldVector,
) -> Result<usize, String> {
    let index = state.tears.len();
    let spawn_seed = state.boot_lifecycle.shared.next_u32();
    // Init computes a falling speed that FireTear overwrites. Its
    // shared draw is still required before sound admission.
    let _initializer_falling_speed = (state.boot_lifecycle.shared.uniform() - 0.5) * 0.2;
    state.audio.admit(
        &mut state.boot_lifecycle.shared,
        state.system_clock.system_counter - 1,
        audio::SoundRequest {
            id: 153,
            volume: 1.0,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
    let final_uniform = state.boot_lifecycle.shared.uniform();
    let mut tear = Tear {
        id: state.next_tear,
        x: 0.0,
        y: 0.0,
        vx: 0.0,
        vy: 0.0,
        age: 0,
        spawn_seed,
        birth_game_update: Some(state.game_update_count),
        source_variant,
        shot_ordinal: Some(state.player.shots_fired),
        controlled: None,
        //5424B1..5424F4 clears the fresh actual receiver-ID tree. Only this
        //independently tracked ballistic family exposes that owner here.
        persistent_contacts: (plan.flags.0 == 0x200 && source_variant == Some(0))
            .then(crate::polyphemus_contact::TearHitRegistry::fresh),
        homing_target: None,
        grid_previous_velocity: None,
        collision_damage: plan.damage,
        base_damage: plan.damage,
        attack_flags: plan.flags,
        source_color: plan.color,
        scale: plan.scale,
        animation_tier: plan.animation_tier,
        collision_radius: plan.collision_radius,
        grid_contact: false,
        grid_death: false,
        motion: projectiles::TearMotion::spawn(origin, velocity, plan.parameters, final_uniform)?,
    };
    tear.project();
    let id = tear.id;
    state.tears.push(tear);
    // Room5C6940 assigns Entity+8 before the factory's immediate Full
    // Update, which takes the homing (542675) and spectral Random draws too.
    state.actor_scheduler.admit(actors::ActorRef {
        kind: actors::ActorKind::Tear,
        id,
    })?;
    crate::tear_homing::update(state, id)?;
    crate::spectral_tear_update(state, id)?;
    state.player.shots_fired = state.player.shots_fired.wrapping_add(1);
    state.next_tear = state.next_tear.checked_add(1).ok_or("tear IDs exhausted")?;
    Ok(index)
}

/// One tear of Familiar fire 467170: its own Random seed (42E8C0),
/// Spawn550A40(2, variant) at the Familiar's position, whose Tear Init
/// 5423D0 draws its discarded falling speed and admits sound 153, then the
/// falling speed Random*0.2 + base (5417F0), damage (vtable+60), flags
/// (541810), scale (541FF0) and the color (vtable+58). Init's height -23
/// stays, the falling acceleration stays 0 and no immediate Update runs.
pub(crate) fn fire_familiar(
    state: &mut State,
    origin: WorldVector,
    velocity: WorldVector,
    damage: f32,
    params: crate::familiar_variants::FireParams,
) -> Result<u32, String> {
    let scale = params.scale;
    let spawn_seed = state.boot_lifecycle.shared.next_u32();
    let _initializer_falling_speed = (state.boot_lifecycle.shared.uniform() - 0.5) * 0.2;
    state.audio.admit(
        &mut state.boot_lifecycle.shared,
        state.system_clock.system_counter - 1,
        audio::SoundRequest {
            id: 153,
            volume: 1.0,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
    let falling_speed = state.boot_lifecycle.shared.uniform() * 0.2 + params.falling_base;
    let scale = scale.max(0.01_f32);
    let mut tear = Tear {
        id: state.next_tear,
        x: 0.0,
        y: 0.0,
        vx: 0.0,
        vy: 0.0,
        age: 0,
        spawn_seed,
        birth_game_update: Some(state.game_update_count),
        source_variant: Some(params.tear_variant),
        shot_ordinal: None,
        controlled: None,
        persistent_contacts: None,
        homing_target: None,
        collision_damage: damage,
        base_damage: damage,
        attack_flags: crate::attack_flags::AttackFlags(params.flags),
        source_color: crate::attack_color::AttackColor {
            tint: params.tint,
            source_middle: [0.0; 4],
            // 42F510: offsets in 1/255 (668980).
            offsets: params
                .offsets
                .map(|offset| offset as f32 * (1.0_f32 / 255.0)),
        },
        scale,
        animation_tier: projectiles::regular_animation_tier(scale)?,
        collision_radius: projectiles::collision_radius(scale, 8.0)?,
        grid_contact: false,
        grid_death: false,
        motion: projectiles::TearMotion::unfired(
            origin,
            velocity,
            projectiles::Parameters {
                height: -23.0,
                falling_speed,
                falling_acceleration: 0.0,
            },
        )?,
    };
    tear.project();
    let id = tear.id;
    state.tears.push(tear);
    state.actor_scheduler.admit(actors::ActorRef {
        kind: actors::ActorKind::Tear,
        id,
    })?;
    state.next_tear = state.next_tear.checked_add(1).ok_or("tear IDs exhausted")?;
    Ok(id)
}
