//! Spider web GridEntity_Web (raw1940, kind10), vtable 0x674790.
//!
//! Update571ab0 runs in the full grid pass. An intact (state0) web acts only
//! when Game+ce9f4 & 3 is zero: it reads player0, skips a player whose grid
//! collision class is 0..3 (444860, flying), and when the strict squared
//! distance to the cell centre is below 441 calls AddSlowing444c40 with
//! duration4, factor0.75 and a 1/1/1.3 tint with offset40. Init571c70 and
//! Update store no Room+e98 path cost, so webs keep cost0.
//!
//! AddSlowing sets Entity+318 flag0x80, adds the duration to +408 and always
//! stores the factor at +490. Base447190 multiplies velocity by the temporary
//! friction +470 before integrating and restores the authored friction (1 for
//! the player). Only a full update counts +408 down, clearing0x80 at zero:
//! RoomFast5cd920 sets Entity+318 bit4000 for the intermediate Player update
//! and 4471e8 then skips every status countdown. Player531850 multiplies +470
//! by sqrt(+490) after control while 0x80 is set in either phase, so a slowed
//! player loses that factor on every following update.
use crate::{bombs, motion::WorldVector, phase::Phase, State};
use serde::{Deserialize, Serialize};

const WEB_KIND: u32 = 10;
const SLOW_DURATION: i32 = 4;
const SLOW_FACTOR: f32 = 0.75;
/// Entity constructor 441e98 stores 0.9 at +490.
const CONSTRUCTED_FACTOR: f32 = 0.9;

/// Player Entity+408 slowing countdown, +490 factor and +470 temporary
/// friction. The 0x80 flag is set exactly while the countdown is positive.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Slowing {
    pub remaining: i32,
    pub factor: f32,
    pub friction: f32,
}

impl Default for Slowing {
    fn default() -> Self {
        Self {
            remaining: 0,
            factor: CONSTRUCTED_FACTOR,
            friction: 1.0,
        }
    }
}

impl Slowing {
    pub fn is_constructed(&self) -> bool {
        *self == Self::default()
    }
}

/// Full grid pass: every intact web applies AddSlowing to a nearby ground player.
pub fn advance(state: &mut State, phase: Phase) -> Result<(), String> {
    if phase != Phase::Update || state.game_update_count.wrapping_add(1) & 3 != 0 {
        return Ok(());
    }
    if state
        .player
        .grid_collision_class
        .is_some_and(|class| class <= 3)
    {
        return Ok(());
    }
    let Some(grid) = state.room.grid.as_ref() else {
        return Ok(());
    };
    let position = bombs::player_world(state);
    let mut applied = 0;
    for (index, cell) in grid.cells.iter().enumerate() {
        if !cell
            .entity
            .as_ref()
            .is_some_and(|entity| entity.kind == WEB_KIND && entity.state == 0)
        {
            continue;
        }
        let center = WorldVector {
            x: 40.0 + (index % usize::from(grid.width)) as f32 * 40.0,
            y: 120.0 + (index / usize::from(grid.width)) as f32 * 40.0,
        };
        let delta = WorldVector {
            x: position.x - center.x,
            y: position.y - center.y,
        };
        let squared = delta.x * delta.x + delta.y * delta.y;
        if !squared.is_finite() {
            return Err("spider web contact requires bounded native world positions".into());
        }
        if squared < 21.0 * 21.0 {
            applied += 1;
        }
    }
    if applied > 0 {
        let slowing = &mut state.player.slowing;
        slowing.remaining = slowing
            .remaining
            .checked_add(SLOW_DURATION * applied)
            .ok_or("spider web slowing countdown overflowed")?;
        slowing.factor = SLOW_FACTOR;
    }
    Ok(())
}

/// Base447190 head for one player update: apply and restore the temporary
/// friction, then count the slowing down on a full update.
pub fn begin_player_update(state: &mut State, phase: Phase) -> Result<(), String> {
    let slowing = state.player.slowing;
    if slowing.friction != 1.0 {
        let velocity = bombs::player_velocity(state) * slowing.friction;
        if !velocity.is_finite() {
            return Err("spider web friction overflowed native velocity".into());
        }
        state.player.vx = f64::from(velocity.x) * 0.65;
        state.player.vy = f64::from(velocity.y) * 0.65;
    }
    let slowing = &mut state.player.slowing;
    slowing.friction = 1.0;
    if phase == Phase::Update && slowing.remaining > 0 {
        slowing.remaining -= 1;
    }
    Ok(())
}

/// Player532042: a slowed player scales next update's friction by sqrt(+490).
pub fn finish_player_update(state: &mut State) {
    let slowing = &mut state.player.slowing;
    if slowing.remaining > 0 {
        slowing.friction *= slowing.factor.sqrt();
    }
}
