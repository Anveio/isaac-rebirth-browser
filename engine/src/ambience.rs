//! Room ambience loops 5c6980, the last call of Room.Update 5d12a0 (5d1866).
//!
//! Each full Room.Update queries the entity partition (4536f0, mask 8) for
//! actors within 450 of the player, widened by each actor's collision radius.
//! Fly-family actors feed the insect swarm loop (SFX 4) and fires (type 33)
//! feed the fire loop (SFX 269). Each loop volume eases halfway (42ef50 with
//! 0.5) toward its target from Room+0x2160 / Room+0x215c, which Room.Init
//! (5c4a00) zeroes. SoundManager 5e1f10 then starts a loop whose volume exceeds
//! 0.01 when it is not playing (594530 Play, delay 2, looping), which draws
//! the shared sample index; a playing loop takes the new volume and pitch, and
//! a loop at or below 0.01 is stopped.
use crate::{audio::SoundRequest, motion::WorldVector, State};
use serde::{Deserialize, Serialize};

const RADIUS: f32 = 450.0;
const THRESHOLD: f32 = 0.01;
const SWARM: u32 = 4;
const FIRE: u32 = 269;
/// The actor types 5c6bed..5c6e48 count as flies.
const FLIES: [u32; 10] = [13, 14, 18, 25, 61, 80, 96, 214, 222, 249];

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Loop {
    pub volume: f32,
    pub pitch: f32,
    pub playing: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Ambience {
    pub fire: Loop,
    pub swarm: Loop,
    ///Port backend dispatch sequence; not an observed native game counter.
    #[serde(default)]
    pub update_sequence: u64,
}

/// Room.Init 5c4a00 zeroes both volumes; playing loops keep playing.
/// Room.Init calls ItemPool 582340 just before it (5CED96), which clears
/// the selected-collectible bytes; every Room.Init site runs both here.
pub(crate) fn reset_room(state: &mut State) {
    if let Some(pools) = state.item_pools.as_mut() {
        pools.reset_selected();
    }
    state.ambience.fire.volume = 0.0;
    state.ambience.swarm.volume = 0.0;
}

/// 5c6980 at the end of a full Room.Update. `paused` is Game 550760: a room or
/// stage transition here (its menu, pause and other inputs are not modeled and
/// are clear in ordinary room play).
pub(crate) fn update(state: &mut State, paused: bool) {
    state.ambience.update_sequence = state
        .ambience
        .update_sequence
        .checked_add(1)
        .expect("ambience backend dispatch sequence exhausted");
    let (fire_target, swarm_target, swarm_pitch) = if paused {
        (0.0, 0.0, 0.0)
    } else {
        targets(state)
    };
    let fire = lerp(state.ambience.fire.volume, fire_target);
    let swarm = lerp(state.ambience.swarm.volume, swarm_target);
    step(state, FIRE, fire, 1.0);
    step(state, SWARM, swarm, swarm_pitch);
}

fn targets(state: &State) -> (f32, f32, f32) {
    let player = crate::bombs::player_world(state);
    let player_velocity = crate::bombs::player_velocity(state);
    let (mut fire, mut swarm, mut pitch, mut flies) = (0.0f32, 0.0f32, 0.0f32, 0u32);
    for npc in &state.room.npcs {
        if npc.removed {
            continue;
        }
        let offset = WorldVector {
            x: npc.position.x - player.x,
            y: npc.position.y - player.y,
        };
        let reach = RADIUS + npc.collision_radius;
        if offset.x * offset.x + offset.y * offset.y >= reach * reach {
            continue;
        }
        let distance = (offset.x * offset.x + offset.y * offset.y).sqrt();
        let falloff = 1.0 - distance / RADIUS;
        if FLIES.contains(&npc.kind) {
            let direction = normalized(offset);
            let toward_player = dot(direction, player_velocity) + 100.0;
            let toward_fly = dot(direction, npc.velocity) + 100.0;
            swarm += falloff * falloff * 0.35;
            pitch += toward_player / toward_fly;
            flies += 1;
        } else if npc.kind == 33 {
            fire += falloff * falloff * 0.6;
        }
    }
    let average = if flies == 0 {
        1.0
    } else {
        pitch / flies as f32
    };
    let t = state.native_game_frame() as f32;
    let wobble =
        (crate::fmath::sin(t / 61.0) + crate::fmath::sin(t / 53.0) + crate::fmath::sin(t / 23.0))
            * 0.02;
    (fire, swarm, wobble + average)
}

fn step(state: &mut State, id: u32, volume: f32, pitch: f32) {
    let current = match id {
        FIRE => &state.ambience.fire,
        _ => &state.ambience.swarm,
    };
    let mut next = Loop {
        volume,
        pitch: current.pitch,
        playing: current.playing,
    };
    if volume > THRESHOLD {
        if next.playing {
            next.pitch = pitch;
        } else if state
            .audio
            .admit(
                &mut state.boot_lifecycle.shared,
                state.system_clock.system_counter.wrapping_sub(1),
                SoundRequest {
                    id,
                    volume,
                    delay: 2,
                    looping: true,
                    pitch,
                },
            )
            .is_some()
        {
            next.playing = true;
            next.pitch = pitch;
        }
    } else {
        next.playing = false;
    }
    match id {
        FIRE => state.ambience.fire = next,
        _ => state.ambience.swarm = next,
    }
}

fn lerp(current: f32, target: f32) -> f32 {
    (target - current) * 0.5 + current
}

fn dot(a: WorldVector, b: WorldVector) -> f32 {
    a.x * b.x + a.y * b.y
}

fn normalized(v: WorldVector) -> WorldVector {
    let length = (v.x * v.x + v.y * v.y).sqrt();
    if length == 0.0 {
        v
    } else {
        WorldVector {
            x: v.x / length,
            y: v.y / length,
        }
    }
}
