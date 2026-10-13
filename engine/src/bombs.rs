//! Regular planted bombs in the observed fresh Isaac starting room.
//!
//! The original public clock tape observes 120 bomb presentations followed by
//! an explosion on presentation 121. The port's 60 Hz step is calibrated to
//! those presentations; this is not a claim about the native simulation clock.
//! Obstacle damage, crater decals, variants and bomb push physics still need
//! independent original scenarios. Native predicates and player knockback are
//! recorded in docs/bomb-evidence.md.
use crate::{Screen, State};
use serde::{Deserialize, Serialize};

pub const FUSE_TICKS: u32 = 121;
// Explosion is an authored 19-frame animation observed at two presentations
// per animation frame. Its final transparent frames still complete playback.
pub const EXPLOSION_TICKS: u32 = 38;
pub const PLAYER_DAMAGE: u8 = 2;
const WORLD_SCALE: f64 = 0.65;
const BLAST_RADIUS: f32 = 75.0;
const ISAAC_RADIUS: f32 = 10.0;
// Native strict circle comparison includes the player's collision radius.
// The regular bomb's own collision radius 16 is unrelated to blast damage.
pub const PLAYER_DAMAGE_RADIUS: f64 = (BLAST_RADIUS + ISAAC_RADIUS) as f64 * WORLD_SCALE;
pub const DAMAGE_COOLDOWN_TICKS: u32 = 120;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Bomb {
    pub id: u32,
    pub x: f64,
    pub y: f64,
    pub age: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Explosion {
    pub id: u32,
    pub x: f64,
    pub y: f64,
    pub age: u32,
}

/// Called only for a rising edge of the bomb action, never keyboard repetition.
/// Empty inventory and paused/menu input are ordinary no-ops.
pub fn place(state: &mut State) -> Result<(), String> {
    if state.screen != Screen::Playing
        || state.paused
        || state.inventory.bombs == 0
        || state.player.hp == 0
    {
        return Ok(());
    }
    let next = state.next_bomb.checked_add(1).ok_or("bomb IDs exhausted")?;
    state.bombs.push(Bomb {
        id: state.next_bomb,
        x: state.player.x,
        y: state.player.y,
        age: 0,
    });
    state.next_bomb = next;
    state.inventory.bombs -= 1;
    Ok(())
}

/// Advance before player movement in an unpaused gameplay step. A detonation
/// changes velocity before that same step integrates position and applies drag.
/// Health changes occur once at detonation, not while an explosion is drawn.
pub fn tick(state: &mut State) {
    if state.screen != Screen::Playing || state.paused {
        return;
    }
    state.damage_cooldown = state.damage_cooldown.saturating_sub(1);
    for explosion in &mut state.explosions {
        explosion.age += 1;
    }
    state
        .explosions
        .retain(|explosion| explosion.age < EXPLOSION_TICKS);

    let mut survivors = Vec::with_capacity(state.bombs.len());
    for mut bomb in state.bombs.drain(..) {
        bomb.age += 1;
        if bomb.age < FUSE_TICKS {
            survivors.push(bomb);
            continue;
        }
        let dx = ((state.player.x - bomb.x) / WORLD_SCALE) as f32;
        let dy = ((state.player.y - bomb.y) / WORLD_SCALE) as f32;
        let distance_squared = dx * dx + dy * dy;
        let radius = (PLAYER_DAMAGE_RADIUS / WORLD_SCALE) as f32;
        if distance_squared < radius * radius {
            // The native blast applies knockback before attempting damage;
            // immunity prevents health loss, but does not absorb the impulse.
            if distance_squared > 0.0 {
                let distance = distance_squared.sqrt();
                let impulse = (radius - distance).min(15.0);
                let vx = (state.player.vx / WORLD_SCALE) as f32;
                let vy = (state.player.vy / WORLD_SCALE) as f32;
                state.player.vx = f64::from(vx * 0.5 + dx / distance * impulse) * WORLD_SCALE;
                state.player.vy = f64::from(vy * 0.5 + dy / distance * impulse) * WORLD_SCALE;
            }
            if state.damage_cooldown == 0 {
                state.player.hp = state.player.hp.saturating_sub(PLAYER_DAMAGE);
                state.damage_cooldown = DAMAGE_COOLDOWN_TICKS;
            }
        }
        state.explosions.push(Explosion {
            id: bomb.id,
            x: bomb.x,
            y: bomb.y,
            age: 0,
        });
    }
    state.bombs = survivors;
}
