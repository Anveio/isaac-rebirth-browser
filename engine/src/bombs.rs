//! Regular planted bombs in the observed fresh Isaac starting room.
//!
//! The globally phased source timer and admission order distinguish 120/121
//! presentation births. Native predicates, motion and scheduling evidence are
//! recorded in docs/bomb-evidence.md. Grid destruction, crater rendering and
//! variant callbacks remain separately tracked combat work.
use crate::{motion::WorldVector, phase::Phase, Screen, State};
use serde::{Deserialize, Serialize};

// Explosion is an authored 19-frame animation observed at two presentations
// per animation frame. Its final transparent frames still complete playback.
pub const PLAYER_DAMAGE: u8 = 2;
const WORLD_SCALE: f64 = 0.65;
const BLAST_RADIUS: f32 = 75.0;
const ISAAC_RADIUS: f32 = 10.0;
// Native strict circle comparison includes the player's collision radius.
// The regular bomb's own collision radius 16 is unrelated to blast damage.
pub const PLAYER_DAMAGE_RADIUS: f64 = (BLAST_RADIUS + ISAAC_RADIUS) as f64 * WORLD_SCALE;
pub const DAMAGE_COOLDOWN_TICKS: u32 = 120;

pub const INITIAL_FUSE_UPDATES: u8 = 59;
pub const PLACEMENT_COOLDOWN_UPDATES: u8 = 30;
const PULSE_FRAMES: u8 = 59;
const CORPSE_FRAMES: u8 = 10;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BombLifetime {
    Alive,
    Exploding,
    Removed,
}

/// Source bomb state in native world units. The caller owns room/grid contacts,
/// actor admission, shared random draws and blast damage/effect callbacks.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BombMotion {
    pub position: WorldVector,
    pub presented_position: WorldVector,
    pub velocity: WorldVector,
    pub fuse_updates_remaining: u8,
    pub full_updates: u8,
    pub animation_frame: u8,
    pub lifetime: BombLifetime,
}

impl BombMotion {
    /// DropBomb runs after the player's position/control update and copies
    /// that position with ten percent of the current native player velocity.
    pub fn spawn(player: WorldVector, velocity: WorldVector) -> Result<Self, String> {
        let motion = Self {
            position: player,
            presented_position: player,
            velocity: velocity * 0.1,
            fuse_updates_remaining: INITIAL_FUSE_UPDATES,
            full_updates: 0,
            animation_frame: 0,
            lifetime: BombLifetime::Alive,
        };
        if !player.is_finite() || !velocity.is_finite() || !motion.is_finite() {
            return Err("bomb birth requires finite native vectors".into());
        }
        Ok(motion)
    }

    fn is_finite(&self) -> bool {
        self.position.is_finite()
            && self.presented_position.is_finite()
            && self.velocity.is_finite()
    }

    fn validate(&self) -> Result<(), String> {
        if !self.is_finite() {
            return Err("bomb motion requires finite native vectors".into());
        }
        let valid_clock = match self.lifetime {
            BombLifetime::Alive => {
                self.fuse_updates_remaining <= INITIAL_FUSE_UPDATES
                    && self.full_updates == INITIAL_FUSE_UPDATES - self.fuse_updates_remaining
                    && self.animation_frame == self.full_updates.min(PULSE_FRAMES - 1)
            }
            BombLifetime::Exploding => {
                self.fuse_updates_remaining == 0
                    && self.full_updates == INITIAL_FUSE_UPDATES + 1
                    && self.animation_frame < CORPSE_FRAMES
            }
            BombLifetime::Removed => {
                self.fuse_updates_remaining == 0
                    && self.full_updates == INITIAL_FUSE_UPDATES + 1
                    && self.animation_frame == CORPSE_FRAMES
            }
        };
        if !valid_clock {
            return Err("bomb lifetime and source clocks disagree".into());
        }
        Ok(())
    }

    fn predict(&mut self, phase: Phase) {
        self.presented_position = if phase == Phase::Interpolate {
            self.position + self.velocity * 0.5
        } else {
            self.position
        };
    }

    /// Newly admitted bombs are excluded from the current full update fence.
    /// An intermediate iteration can still predict a bomb born in player Update.
    pub fn present_birth(&mut self, phase: Phase) -> Result<(), String> {
        self.validate()?;
        if self.full_updates != 0 || self.lifetime != BombLifetime::Alive {
            return Err("birth presentation requires a newly admitted bomb".into());
        }
        if phase == Phase::Skipped {
            return Err("bomb birth requires an admitted player phase".into());
        }
        let mut next = *self;
        next.predict(phase);
        if !next.is_finite() {
            return Err("bomb birth prediction overflowed".into());
        }
        *self = next;
        Ok(())
    }

    /// Advance an already admitted actor. Return true only on detonation.
    /// Pause is caller-owned: advance the global Clock, but skip this method.
    pub fn advance(&mut self, phase: Phase) -> Result<bool, String> {
        self.validate()?;
        if phase == Phase::Skipped || self.lifetime == BombLifetime::Removed {
            return Ok(false);
        }
        let mut next = *self;
        let mut detonated = false;
        if phase == Phase::Update {
            next.position = next.position + next.velocity;
            // BombUpdate rounds the .95 multiply before its low-speed .5.
            next.velocity = next.velocity * 0.95;
            if next.velocity.x * next.velocity.x + next.velocity.y * next.velocity.y < 1.0 {
                next.velocity = next.velocity * 0.5;
            }
            match next.lifetime {
                BombLifetime::Alive => {
                    next.full_updates = next.full_updates.saturating_add(1);
                    if next.fuse_updates_remaining == 0 {
                        next.lifetime = BombLifetime::Exploding;
                        next.animation_frame = 0;
                        detonated = true;
                    } else {
                        next.fuse_updates_remaining -= 1;
                        next.animation_frame = (next.animation_frame + 1).min(PULSE_FRAMES - 1);
                    }
                }
                BombLifetime::Exploding => {
                    next.animation_frame += 1;
                    if next.animation_frame >= CORPSE_FRAMES {
                        next.lifetime = BombLifetime::Removed;
                    }
                }
                BombLifetime::Removed => unreachable!(),
            }
        }
        next.predict(phase);
        if !next.is_finite() {
            return Err("bomb motion or prediction overflowed".into());
        }
        *self = next;
        Ok(detonated)
    }
}

/// The native placement countdown skips intermediate player updates. A full
/// update that decrements a positive countdown returns before admitting input.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BombPlacementCooldown {
    pub remaining_updates: u8,
}

impl BombPlacementCooldown {
    pub fn advance(&mut self, phase: Phase) -> bool {
        if phase == Phase::Skipped {
            return false;
        }
        if self.remaining_updates > 0 {
            if phase == Phase::Update {
                self.remaining_updates -= 1;
            }
            return false;
        }
        true
    }

    /// Call only after successful inventory/ID/random admission and spawn.
    pub fn record_placement(&mut self) {
        self.remaining_updates = PLACEMENT_COOLDOWN_UPDATES;
    }
}

/// Fresh Isaac's strict regular blast envelope and radial velocity change.
/// Some signals a damage contact; exact zero separation preserves velocity.
pub fn isaac_blast_velocity(
    player: WorldVector,
    velocity: WorldVector,
    bomb: WorldVector,
) -> Result<Option<WorldVector>, String> {
    if !player.is_finite() || !velocity.is_finite() || !bomb.is_finite() {
        return Err("blast contact requires finite native vectors".into());
    }
    let delta = WorldVector {
        x: player.x - bomb.x,
        y: player.y - bomb.y,
    };
    let squared = delta.x * delta.x + delta.y * delta.y;
    if !squared.is_finite() {
        return Err("blast distance overflowed".into());
    }
    let radius = BLAST_RADIUS + ISAAC_RADIUS;
    if squared >= radius * radius {
        return Ok(None);
    }
    if squared == 0.0 {
        return Ok(Some(velocity));
    }
    let distance = squared.sqrt();
    let impulse = (radius - distance).min(15.0);
    let result = WorldVector {
        x: velocity.x * 0.5 + delta.x / distance * impulse,
        y: velocity.y * 0.5 + delta.y / distance * impulse,
    };
    if !result.is_finite() {
        return Err("blast impulse overflowed".into());
    }
    Ok(Some(result))
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Bomb {
    pub id: u32,
    pub x: f64,
    pub y: f64,
    pub motion: BombMotion,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Explosion {
    pub id: u32,
    pub x: f64,
    pub y: f64,
    pub animation_frame: u8,
}

pub fn player_world(state: &State) -> WorldVector {
    WorldVector {
        x: ((state.player.x - 32.0) / WORLD_SCALE) as f32,
        y: ((state.player.y + 46.0) / WORLD_SCALE) as f32,
    }
}

pub fn player_velocity(state: &State) -> WorldVector {
    WorldVector {
        x: (state.player.vx / WORLD_SCALE) as f32,
        y: (state.player.vy / WORLD_SCALE) as f32,
    }
}

/// Admit the rising-edge action inside player Update, after source movement.
/// The caller has already advanced the full-only placement cooldown.
pub fn place(state: &mut State, phase: Phase) -> Result<(), String> {
    if state.screen != Screen::Playing
        || state.paused
        || state.inventory.bombs == 0
        || state.player.hp == 0
        || phase == Phase::Skipped
    {
        return Ok(());
    }
    let next = state.next_bomb.checked_add(1).ok_or("bomb IDs exhausted")?;
    let mut motion = BombMotion::spawn(player_world(state), player_velocity(state))?;
    motion.present_birth(phase)?;
    state.actor_scheduler.admit(crate::actors::ActorRef {
        kind: crate::actors::ActorKind::Bomb,
        id: state.next_bomb,
    })?;
    state.bombs.push(Bomb {
        id: state.next_bomb,
        x: f64::from(motion.presented_position.x) * WORLD_SCALE + 32.0,
        y: f64::from(motion.presented_position.y) * WORLD_SCALE - 46.0,
        motion,
    });
    state.next_bomb = next;
    state.inventory.bombs -= 1;
    state.bomb_placement_cooldown.record_placement();
    Ok(())
}

/// Existing explosions advance only on full phases. Snapshot their count before
/// actor dispatch so the effect created by this phase starts at authored frame0.
pub fn advance_explosions(state: &mut State, phase: Phase, fence: usize) {
    if phase != Phase::Update {
        return;
    }
    for explosion in state.explosions.iter_mut().take(fence) {
        explosion.animation_frame += 1;
    }
    state
        .explosions
        .retain(|explosion| explosion.animation_frame < 19);
}

/// Dispatch one admitted bomb at its position in the actor list. A player
/// admitted earlier has already integrated this full phase's previous velocity.
pub fn advance_actor(state: &mut State, phase: Phase, id: u32) -> Result<(), String> {
    let Some(index) = state.bombs.iter().position(|bomb| bomb.id == id) else {
        return Ok(());
    };
    let detonated = state.bombs[index].motion.advance(phase)?;
    if state.bombs[index].motion.lifetime == BombLifetime::Alive {
        use std::sync::OnceLock;
        static COLLIDER: OnceLock<crate::grid::Collider> = OnceLock::new();
        let grid = state
            .room
            .grid
            .as_ref()
            .map(|descriptor| crate::grid::Grid::new(descriptor.clone()))
            .transpose()?
            .unwrap_or_else(crate::grid::Grid::closed_1x1_boundary);
        let collider = COLLIDER.get_or_init(|| {
            crate::grid::Collider::new(crate::grid::CollisionGeometry {
                radius: 16.0,
                scale: WorldVector { x: 1.0, y: 1.0 },
                points: 12,
            })
            .expect("authored regular bomb collision geometry is valid")
        });
        let motion = &mut state.bombs[index].motion;
        if phase == Phase::Update {
            collider.resolve_nonplayer_ground(
                &grid,
                &mut motion.position,
                &mut motion.velocity,
                false,
            )?;
            motion.presented_position = motion.position;
        } else if phase == Phase::Interpolate {
            let mut preview_velocity = motion.velocity;
            collider.resolve_nonplayer_ground(
                &grid,
                &mut motion.presented_position,
                &mut preview_velocity,
                true,
            )?;
        }
    }
    let position = state.bombs[index].motion.position;
    let presented = state.bombs[index].motion.presented_position;
    state.bombs[index].x = f64::from(presented.x) * WORLD_SCALE + 32.0;
    state.bombs[index].y = f64::from(presented.y) * WORLD_SCALE - 46.0;
    if detonated {
        if let Some(velocity) =
            isaac_blast_velocity(player_world(state), player_velocity(state), position)?
        {
            state.player.vx = f64::from(velocity.x) * WORLD_SCALE;
            state.player.vy = f64::from(velocity.y) * WORLD_SCALE;
            if state.damage_cooldown == 0 {
                state.player.hp = state.player.hp.saturating_sub(PLAYER_DAMAGE);
                state.damage_cooldown = DAMAGE_COOLDOWN_TICKS;
            }
        }
        state.explosions.push(Explosion {
            id,
            x: f64::from(position.x) * WORLD_SCALE + 32.0,
            y: f64::from(position.y) * WORLD_SCALE - 46.0,
            animation_frame: 0,
        });
    }
    Ok(())
}

pub fn retire_removed(state: &mut State) {
    state
        .bombs
        .retain(|bomb| bomb.motion.lifetime != BombLifetime::Removed);
    let bombs = &state.bombs;
    state.actor_scheduler.retain(|actor| {
        actor.kind != crate::actors::ActorKind::Bomb || bombs.iter().any(|bomb| bomb.id == actor.id)
    });
}
