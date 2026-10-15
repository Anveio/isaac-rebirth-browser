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
        source_variant,
        shot_ordinal: Some(state.player.shots_fired),
        controlled: None,
        collision_damage: plan.damage,
        base_damage: plan.damage,
        attack_flags: plan.flags,
        source_color: plan.color,
        scale: plan.scale,
        animation_tier: plan.animation_tier,
        collision_radius: plan.collision_radius,
        grid_contact: false,
        motion: projectiles::TearMotion::spawn(origin, velocity, plan.parameters, final_uniform)?,
    };
    tear.project();
    state.tears.push(tear);
    state.actor_scheduler.admit(actors::ActorRef {
        kind: actors::ActorKind::Tear,
        id: state.next_tear,
    })?;
    state.player.shots_fired = state.player.shots_fired.wrapping_add(1);
    state.next_tear = state.next_tear.checked_add(1).ok_or("tear IDs exhausted")?;
    Ok(index)
}
