//! Normal Isaac's explicit Damage -> Kill -> Death -> GameOver callbacks.
//! Health is checked only at an admitted damage callback. NO_KILL is an
//! incoming DamageFlags bit; it is not an entity flag or a global HP predicate.
use crate::{audio, bombs, familiars, phase::Phase, player_animation::Clip, State};
use serde::{Deserialize, Serialize};

pub const DAMAGE_NO_KILL: u32 = 1;
pub const DAMAGE_SPIKES: u32 = 0x80;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Request {
    Restart,
    Exit,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GameOver {
    pub state: u8,
    pub damage_flags: u32,
    // Source558b79 SetFrame("Diary",0), not Play; no UI animation timer.
    pub diary_frame: u32,
    pub diary_playing: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayerDeath {
    pub last_damage_flags: u32,
    pub game_over: Option<GameOver>,
    pub system_pending: Option<Request>,
}

/// Caller52b080 invokes virtual40 only after health depletion and its actual
/// NO_KILL branch. Common zero-gib Player4464c0 ->445c60 then emits SFX30 and
/// a real externally seeded Blood7, including its immediate first Update.
pub fn after_damage(state: &mut State, flags: u32) -> Result<(), String> {
    if state.player.dead {
        return Ok(());
    }
    state.player_death.last_damage_flags = flags;
    if state.player.hp != 0 || state.player.soul_hp > 0 {
        return Ok(());
    }
    if flags & DAMAGE_NO_KILL != 0 {
        state.player.hp = state.player.max_hp.min(1);
        return Ok(());
    }
    state.player.dead = true;
    state.audio.admit(
        &mut state.boot_lifecycle.shared,
        state.system_clock.system_counter.saturating_sub(1),
        audio::SoundRequest {
            id: 30,
            volume: 0.8_f32,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
    let seed = state.boot_lifecycle.shared.next_u32();
    let id =
        familiars::admit_blood_splat(state, seed, bombs::player_world(state), [1.0; 4], [0.0; 3])?;
    familiars::advance_blood_initial(state, id)?;
    Ok(())
}

/// The caller already advanced the main ANM through Base447190. Dead Player
/// callback531850 forces Death only on name change and zeroes velocity,
/// and returns before ordinary controls, cooldown and shot/active callbacks.
/// RoomFast5cd920 also invokes virtual18 for Player, with flags318 bit4000:
/// Base447190 skips main ANM advancement, but the dead branch still executes.
pub fn advance(state: &mut State, phase: Phase) -> Result<bool, String> {
    if !state.player.dead {
        return Ok(false);
    }
    if phase != Phase::Skipped {
        // Player531864 calls Base447190 before the dead branch5318a5.
        // Its carried velocity (including a lethal grid impulse) advances
        // position at4471c0..4471e3 before531bb6 clears it. The shared
        // player-update head already applied temporary source friction.
        let position = bombs::player_world(state) + bombs::player_velocity(state);
        state.player.x =
            f64::from(position.x) * crate::motion::WORLD_SCALE + crate::motion::WORLD_ORIGIN_X;
        state.player.y =
            f64::from(position.y) * crate::motion::WORLD_SCALE + crate::motion::WORLD_ORIGIN_Y;
        if state.player_animation.current != Some(Clip::Death) {
            state.player.visible = true;
            crate::player_animation::force_play(state, Clip::Death);
        }
        state.player.vx = 0.0;
        state.player.vy = 0.0;
        if !state.player_animation.is_playing(Clip::Death)
            && state.player_animation.is_finished(Some(Clip::Death))
        {
            state.player.visible = false;
            if state.player_death.game_over.is_none() {
                //52E8EE's Music calls precede Manager5BF8F0; its primary
                //Player arm5BFA15 updates counter9 before Popup558900/Clear.
                state.music.play(93);
                state.music.queue(8);
                crate::persistent_effects::add_counter(state, 9, 1)?;
                state.player_death.game_over = Some(GameOver {
                    state: 2,
                    damage_flags: state.player_death.last_damage_flags,
                    diary_frame: 0,
                    diary_playing: false,
                });
            }
        }
    }
    Ok(true)
}

/// Game54f760 invokes popup557650 on actual released System input, then
/// freezes gameplay while popup state is nonzero. The next System dispatcher
/// consumes the pending request; DOM callbacks do not directly reset a run.
pub fn poll_popup(state: &mut State) {
    if state.player_death.game_over.is_none() {
        return;
    }
    if state.public_actions.menu_confirm {
        state.player_death.system_pending = Some(Request::Restart);
    } else if state.public_actions.menu_back {
        state.player_death.system_pending = Some(Request::Exit);
    }
}
