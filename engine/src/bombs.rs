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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Game};

    fn playing() -> Game {
        playing_for_seed(2184569263)
    }

    fn playing_for_seed(seed: u32) -> Game {
        let mut game = Game::new(seed).unwrap();
        for _ in 0..4 {
            game.command(Command::Key {
                code: "Enter".into(),
                down: true,
            })
            .unwrap();
            game.command(Command::Key {
                code: "Enter".into(),
                down: false,
            })
            .unwrap();
        }
        assert_eq!(game.state.screen, Screen::Playing);
        game
    }

    #[test]
    fn stationary_bomb_consumes_inventory_and_damages_once_on_observed_phase() {
        let mut game = playing();
        game.command(Command::Key {
            code: "KeyE".into(),
            down: true,
        })
        .unwrap();
        assert_eq!(game.state.inventory.bombs, 0);
        assert_eq!(game.state.bombs.len(), 1);
        game.command(Command::Tick {
            count: FUSE_TICKS - 1,
        })
        .unwrap();
        assert_eq!(game.state.player.hp, 6);
        assert_eq!(game.state.bombs.len(), 1);
        assert!(game.state.explosions.is_empty());
        game.command(Command::Tick { count: 1 }).unwrap();
        assert_eq!(game.state.player.hp, 4);
        assert!(game.state.bombs.is_empty());
        assert_eq!(game.state.explosions.len(), 1);
        game.command(Command::Tick {
            count: EXPLOSION_TICKS,
        })
        .unwrap();
        assert_eq!(game.state.player.hp, 4);
        assert!(game.state.explosions.is_empty());
    }

    #[test]
    fn held_action_does_not_spend_more_inventory_and_empty_press_is_safe() {
        let mut game = playing();
        game.state.inventory.bombs = 2;
        for _ in 0..3 {
            game.command(Command::Key {
                code: "KeyE".into(),
                down: true,
            })
            .unwrap();
        }
        assert_eq!(game.state.inventory.bombs, 1);
        assert_eq!(game.state.bombs.len(), 1);
        game.command(Command::Key {
            code: "KeyE".into(),
            down: false,
        })
        .unwrap();
        game.command(Command::Key {
            code: "KeyE".into(),
            down: true,
        })
        .unwrap();
        assert_eq!(game.state.inventory.bombs, 0);
        assert_eq!(game.state.bombs.len(), 2);
        game.command(Command::Key {
            code: "KeyE".into(),
            down: false,
        })
        .unwrap();
        game.command(Command::Key {
            code: "KeyE".into(),
            down: true,
        })
        .unwrap();
        assert_eq!(game.state.bombs.len(), 2);
    }

    #[test]
    fn pause_freezes_fuse_and_blocks_placement() {
        let mut game = playing();
        place(&mut game.state).unwrap();
        game.state.inventory.bombs = 1;
        game.command(Command::Key {
            code: "Escape".into(),
            down: true,
        })
        .unwrap();
        let before = game.state.bombs.clone();
        game.command(Command::Tick {
            count: FUSE_TICKS + 10,
        })
        .unwrap();
        game.command(Command::Key {
            code: "KeyE".into(),
            down: true,
        })
        .unwrap();
        assert_eq!(game.state.bombs, before);
        assert_eq!(game.state.inventory.bombs, 1);
        assert_eq!(game.state.player.hp, 6);
    }

    #[test]
    fn independent_distance_cases_separate_near_damage_from_escape() {
        for (distance, expected_hp) in [(28.0, 4), (46.0, 4), (54.0, 4), (57.0, 6), (86.0, 6)] {
            let mut game = playing();
            place(&mut game.state).unwrap();
            game.state.player.x += distance;
            for _ in 0..FUSE_TICKS {
                tick(&mut game.state);
            }
            assert_eq!(game.state.player.hp, expected_hp, "distance {distance}");
        }
    }

    #[test]
    fn exact_native_contact_boundary_is_safe_and_nearby_points_are_radial() {
        for (dx, dy, expected_hp) in [
            (PLAYER_DAMAGE_RADIUS, 0.0, 6),
            (PLAYER_DAMAGE_RADIUS - 0.001, 0.0, 4),
            (0.0, -PLAYER_DAMAGE_RADIUS, 6),
            (40.0, 35.0, 4),
            (40.0, 40.0, 6),
        ] {
            let mut game = playing();
            place(&mut game.state).unwrap();
            game.state.player.x += dx;
            game.state.player.y += dy;
            for _ in 0..FUSE_TICKS {
                tick(&mut game.state);
            }
            assert_eq!(game.state.player.hp, expected_hp, "distance ({dx}, {dy})");
        }
    }

    #[test]
    fn native_blast_impulse_has_distance_falloff_and_survives_immunity() {
        let mut game = playing();
        place(&mut game.state).unwrap();
        game.state.player.x += 54.0;
        game.state.player.vx = 2.0;
        game.state.bombs[0].age = FUSE_TICKS - 1;
        game.state.damage_cooldown = 10;
        tick(&mut game.state);
        assert_eq!(game.state.player.hp, 6);
        assert!((game.state.player.vx - 2.25).abs() < 0.00001);
        assert_eq!(game.state.player.vy, 0.0);
    }

    #[test]
    fn independent_coasting_blast_integrates_impulse_before_drag() {
        // Fresh source seed3BMBMFTE: complete D27/release30/E tape, then
        // independently observed native positions/velocities at E+121/123/133.
        let mut game = playing_for_seed(3350817387);
        game.command(Command::Key {
            code: "KeyD".into(),
            down: true,
        })
        .unwrap();
        game.command(Command::Tick { count: 27 }).unwrap();
        game.command(Command::Key {
            code: "KeyD".into(),
            down: false,
        })
        .unwrap();
        game.command(Command::Tick { count: 30 }).unwrap();
        let world_x = |game: &Game| (game.state.player.x - 32.0) / WORLD_SCALE;
        assert!((world_x(&game) - 438.4042053222656).abs() < 0.000002);
        game.command(Command::Key {
            code: "KeyE".into(),
            down: true,
        })
        .unwrap();
        game.command(Command::Tick { count: 1 }).unwrap();
        game.command(Command::Key {
            code: "KeyE".into(),
            down: false,
        })
        .unwrap();
        game.command(Command::Tick {
            count: FUSE_TICKS - 2,
        })
        .unwrap();
        assert!((world_x(&game) - 439.18450927734375).abs() < 0.000002);
        assert_eq!(game.state.player.hp, 6);
        for (count, native_x, native_vx) in [
            (1, 454.18450927734375, 13.20511245727539),
            (2, 479.0146179199219, 10.23396110534668),
            (10, 540.6290283203125, 2.8612258434295654),
        ] {
            game.command(Command::Tick { count }).unwrap();
            assert!((world_x(&game) - native_x).abs() < 0.000002);
            assert!((game.state.player.vx / WORLD_SCALE - native_vx).abs() < 0.000002);
            assert_eq!(game.state.player.hp, 4);
        }
    }

    #[test]
    fn observed_damage_countdown_decrements_per_step_and_pause_preserves_it() {
        let mut game = playing();
        place(&mut game.state).unwrap();
        game.command(Command::Tick { count: FUSE_TICKS }).unwrap();
        assert_eq!(game.state.damage_cooldown, 120);
        game.command(Command::Tick { count: 1 }).unwrap();
        assert_eq!(game.state.damage_cooldown, 119);
        game.state.paused = true;
        game.command(Command::Tick { count: 120 }).unwrap();
        assert_eq!(game.state.damage_cooldown, 119);
        game.state.paused = false;
        game.command(Command::Tick { count: 118 }).unwrap();
        assert_eq!(game.state.damage_cooldown, 1);
        game.command(Command::Tick { count: 1 }).unwrap();
        assert_eq!(game.state.damage_cooldown, 0);
    }

    #[test]
    fn health_saturates_and_failed_id_allocation_preserves_inventory() {
        let mut game = playing();
        game.state.player.hp = 1;
        place(&mut game.state).unwrap();
        for _ in 0..FUSE_TICKS {
            tick(&mut game.state);
        }
        assert_eq!(game.state.player.hp, 0);
        let mut game = playing();
        game.state.next_bomb = u32::MAX;
        assert!(place(&mut game.state).is_err());
        assert_eq!(game.state.inventory.bombs, 1);
        assert!(game.state.bombs.is_empty());
    }

    #[test]
    fn bomb_replay_matches_chunked_and_individual_steps() {
        let mut single = playing();
        single
            .command(Command::Key {
                code: "KeyE".into(),
                down: true,
            })
            .unwrap();
        single
            .command(Command::Key {
                code: "KeyE".into(),
                down: false,
            })
            .unwrap();
        let mut chunked = single.clone();
        let ticks = FUSE_TICKS + EXPLOSION_TICKS + 10;
        for _ in 0..ticks {
            single.command(Command::Tick { count: 1 }).unwrap();
        }
        chunked.command(Command::Tick { count: ticks }).unwrap();
        assert_eq!(single.state, chunked.state);
    }

    #[test]
    fn new_run_clears_explosives_and_restores_starting_health_and_inventory() {
        let mut game = playing();
        place(&mut game.state).unwrap();
        game.command(Command::Tick { count: FUSE_TICKS }).unwrap();
        assert_eq!(game.state.player.hp, 4);
        assert_eq!(game.state.explosions.len(), 1);
        game.start_run().unwrap();
        assert!(game.state.bombs.is_empty());
        assert!(game.state.explosions.is_empty());
        assert_eq!(game.state.inventory.bombs, 1);
        assert_eq!(game.state.player.hp, 6);
        assert_eq!(game.state.next_bomb, 1);
        assert_eq!(game.state.damage_cooldown, 0);
    }
}
