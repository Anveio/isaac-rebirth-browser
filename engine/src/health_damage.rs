//! Normal primary Isaac's health-container branch of Player52b080.
//! Callers admit contact and immunity separately. Soul damage stores BC0
//! directly; it does not run the Soul-grant cap/mask-normalization callback.
use crate::{audio::SoundRequest, combat, items, player_animation::Clip, player_items, State};

const DAMAGE_RED_HEARTS: u32 = 0x20;

/// Apply the already admitted positive integer damage of the ordinary
/// contact, spike, projectile, curse-door and regular-bomb callers.
pub(crate) fn apply_admitted(
    state: &mut State,
    health_units: u8,
    flags: u32,
) -> Result<(), String> {
    // Normal primary Player52c0b6 requests55 before the container dispatch.
    // AudioState owns its real three-sample selection and admission throttle.
    state.audio.admit(
        &mut state.boot_lifecycle.shared,
        state.system_clock.system_counter.saturating_sub(1),
        SoundRequest {
            id: 55,
            volume: 1.0,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
    //52c0e0 Sound55 precedes52c4d7; the flag still precedes health routing.
    state.level_progress.admitted_damage(flags, state.stage);
    let damage = i32::from(health_units);
    let soul = state.player.soul_hp;
    let red = state.player.hp;
    //52be04..52be24: RED_HEARTS falls back to Soul at low red health.
    let red_priority = flags & DAMAGE_RED_HEARTS != 0 && (red > 2 || (red == 2 && damage <= 1));
    if soul > 0 && !red_priority {
        //52c116 walks descending positions before changing BC0. A Black
        //pair triggers only when its odd (last remaining) half is consumed.
        let mut position = soul;
        while position > soul - damage {
            if position > 0 && position & 1 != 0 {
                let bit = 1_u32 << (((position - 1) / 2) as u32 & 31);
                if state.player.black_heart_mask & bit != 0 {
                    state.player.black_heart_mask &= !bit;
                    use_black_heart(state)?;
                }
            }
            position -= 1;
        }
        //52c16e..52c181 consumes min(Soul,damage), with no red spill and
        //no5082d0 mask recoloring or grant-side maximum-health clamp.
        state.player.soul_hp = soul - soul.min(damage);
    } else {
        let mut remaining = damage;
        if state.player.eternal_hearts > 0 {
            remaining = (remaining - state.player.eternal_hearts).max(0);
            state.player.eternal_hearts = 0;
        }
        player_items::add_red_hearts(&mut state.player, -remaining);
        //52be55..52be60 selects52c6cd after508230 even when Eternal
        //absorbed the health amount. RED_HEARTS bypasses this penalty.
        state.level_progress.red_damage(flags);
    }
    //52c668's actual NO_KILL/virtual40 branch remains owned by PlayerDeath.
    Ok(())
}

/// Ordinary52bf96 chooses the Hit branch.52c4f0 force-plays it after health,
/// NO_KILL/Kill and immunity storage, including a newly fatal damage callback.
/// The next actual Player callback then selects Death; this call does not
/// disable controls or zero velocity. Charged49 costume cancellation is a
/// separate active-item callback; the ordinary supported state is D80zero.
pub(crate) fn finish_admitted(state: &mut State) {
    crate::player_animation::force_play(state, Clip::Hit);
    if let Some(owner) = state.player_items.as_mut() {
        owner.active_state_d80 = 0;
    }
}

///52c143 requests actual Info16 before forced Use35 and the Soul store.
/// Normal35 is costume-free and cache0; its zero-cooldown temporary record
/// persists. The shared authored book host freezes room callbacks, so queued
/// target damage waits for that host to resume the actual NPC full callback.
fn use_black_heart(state: &mut State) -> Result<(), String> {
    state.information_animation.request_black_heart();
    let config = items::definition(35).ok_or("Black-heart ItemConfig35 absent")?;
    state
        .player_items
        .as_mut()
        .ok_or("Black-heart Player item owner absent")?
        .add_temporary_collectible(35)?;
    state.audio.admit(
        &mut state.boot_lifecycle.shared,
        state.system_clock.system_counter.saturating_sub(1),
        SoundRequest {
            id: 33,
            volume: 1.0,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
    //523eaa..523f3f visits the actual NPC vector, skips Entity318 bit10
    //and calls virtual38. Flags0 makes countdown30 inert in446cf0.
    for npc in &mut state.room.npcs {
        if !(10..1000).contains(&npc.kind) || npc.entity_flags() & 0x10 != 0 {
            continue;
        }
        npc.damage.enqueue(
            combat::ActorHealth {
                hp: npc.hp,
                max_hp: npc.max_hp,
            },
            combat::DamageEvent {
                amount: 40.0,
                kind: combat::DamageKind::RoomEffect,
                source: combat::DamageSource {
                    id: 0,
                    entity_type: 0,
                },
            },
        )?;
    }
    let owner = state.player_items.as_mut().expect("checked item owner");
    owner.unevaluated_cache_flags |= config.cache_flags;
    owner.active_flag_3076 = true;
    Ok(())
}
