//! Regular planted bombs in the observed fresh Isaac starting room.
//!
//! The globally phased source timer and admission order distinguish 120/121
//! presentation births. Native predicates, motion and scheduling evidence are
//! recorded in docs/bomb-evidence.md. Grid destruction, crater rendering and
//! variant callbacks remain separately tracked combat work.
use crate::{
    actors::{ActorKind, ActorRef},
    attack_color::AttackColor,
    attack_flags::AttackFlags,
    motion::WorldVector,
    phase::Phase,
    Screen, State,
};
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
const DR_FETUS_FUSE_UPDATES: u8 = 30;

fn initial_fuse_default() -> u8 {
    INITIAL_FUSE_UPDATES
}

fn is_initial_fuse_default(value: &u8) -> bool {
    *value == INITIAL_FUSE_UPDATES
}
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
    #[serde(
        default = "initial_fuse_default",
        skip_serializing_if = "is_initial_fuse_default"
    )]
    pub initial_fuse_updates: u8,
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
            initial_fuse_updates: INITIAL_FUSE_UPDATES,
            full_updates: 0,
            animation_frame: 0,
            lifetime: BombLifetime::Alive,
        };
        if !player.is_finite() || !velocity.is_finite() || !motion.is_finite() {
            return Err("bomb birth requires finite native vectors".into());
        }
        Ok(motion)
    }

    fn spawn_dr_fetus(position: WorldVector, velocity: WorldVector) -> Result<Self, String> {
        let mut motion = Self::spawn(position, WorldVector { x: 0.0, y: 0.0 })?;
        motion.velocity = velocity;
        motion.initial_fuse_updates = DR_FETUS_FUSE_UPDATES;
        motion.fuse_updates_remaining = DR_FETUS_FUSE_UPDATES;
        if !motion.is_finite() {
            return Err("Dr. Fetus bomb birth requires finite native vectors".into());
        }
        Ok(motion)
    }

    fn is_finite(&self) -> bool {
        self.position.is_finite()
            && self.presented_position.is_finite()
            && self.velocity.is_finite()
    }

    fn validate(&self, corpse_frames: u8) -> Result<(), String> {
        if !self.is_finite() {
            return Err("bomb motion requires finite native vectors".into());
        }
        if !matches!(
            self.initial_fuse_updates,
            INITIAL_FUSE_UPDATES | DR_FETUS_FUSE_UPDATES
        ) {
            return Err("unsupported born bomb fuse".into());
        }
        let valid_clock = match self.lifetime {
            BombLifetime::Alive => {
                self.fuse_updates_remaining <= self.initial_fuse_updates
                    && self.full_updates == self.initial_fuse_updates - self.fuse_updates_remaining
                    && self.animation_frame == self.full_updates.min(PULSE_FRAMES - 1)
            }
            BombLifetime::Exploding => {
                self.fuse_updates_remaining == 0
                    && self.full_updates == self.initial_fuse_updates + 1
                    && self.animation_frame < corpse_frames
            }
            BombLifetime::Removed => {
                self.fuse_updates_remaining == 0
                    && self.full_updates == self.initial_fuse_updates + 1
                    && self.animation_frame == corpse_frames
            }
        };
        if !valid_clock {
            return Err("bomb lifetime and source clocks disagree".into());
        }
        Ok(())
    }

    fn predict(&mut self, phase: Phase, friction: f32) {
        self.presented_position = if phase == Phase::Interpolate {
            // FastUpdate4446a0 rounds velocity*temporary-friction before /2.
            self.position + (self.velocity * friction) * 0.5
        } else {
            self.position
        };
    }

    /// Newly admitted bombs are excluded from the current full update fence.
    /// An intermediate iteration can still predict a bomb born in player Update.
    pub fn present_birth(&mut self, phase: Phase) -> Result<(), String> {
        self.validate(CORPSE_FRAMES)?;
        if self.full_updates != 0 || self.lifetime != BombLifetime::Alive {
            return Err("birth presentation requires a newly admitted bomb".into());
        }
        if phase == Phase::Skipped {
            return Err("bomb birth requires an admitted player phase".into());
        }
        let mut next = *self;
        next.predict(phase, 1.0);
        if !next.is_finite() {
            return Err("bomb birth prediction overflowed".into());
        }
        *self = next;
        Ok(())
    }

    /// Advance an already admitted actor. Return true only on detonation.
    /// Pause is caller-owned: advance the global Clock, but skip this method.
    pub fn advance(&mut self, phase: Phase) -> Result<bool, String> {
        self.advance_variant(phase, 1.0, CORPSE_FRAMES)
    }

    fn advance_variant(
        &mut self,
        phase: Phase,
        friction: f32,
        corpse_frames: u8,
    ) -> Result<bool, String> {
        self.validate(corpse_frames)?;
        if phase == Phase::Skipped || self.lifetime == BombLifetime::Removed {
            return Ok(false);
        }
        let mut next = *self;
        let mut detonated = false;
        if phase == Phase::Update {
            // BaseUpdate447190 applies the prior temporary friction before
            // integration, then restores the authored friction for this turn.
            next.velocity = next.velocity * friction;
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
                    if next.animation_frame >= corpse_frames {
                        next.lifetime = BombLifetime::Removed;
                    }
                }
                BombLifetime::Removed => unreachable!(),
            }
        }
        next.predict(phase, friction);
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
    blast_radius: f32,
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
    let radius = blast_radius + ISAAC_RADIUS;
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
    pub variant: u32,
    /// Player509612 draws the planted bomb's seed; conversions own the
    /// actual shared stream draw made by Pickup::Update.
    pub spawn_seed: Option<u32>,
    pub birth_game_update: u32,
    pub friction: f32,
    pub animation: String,
    pub animation_frame: u32,
    pub animation_playing: bool,
    pub x: f64,
    pub y: f64,
    pub motion: BombMotion,
    /// Bomb+7DC Player-contact countdown of planted and converted bombs (a
    /// shot bomb keeps its own in BombShot). Player509612 placement stores 4,
    /// a Pickup conversion's Init leaves 0; Bomb456C20 resets a positive
    /// value to 4 on contact and Full decrements it (pickup-interactions).
    #[serde(default)]
    pub player_contact_countdown: u8,
    /// Ordinary50DE10 birth metadata. Legacy planted/conversion histories stay separate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shot: Option<BombShot>,
}

/// Born-owned fields of the admitted neutral ordinary Dr. Fetus actor4/0.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BombShot {
    ///Actual fresh Base31E/31F owners; old serialized histories stay unknown.
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub dead: Option<bool>,
    ///Registered Child4A0, separately known from inventory or physical Parent.
    #[serde(default)]
    pub child: Option<ActorRef>,
    #[serde(default)]
    pub child_known: bool,
    pub actor_rng: crate::items::Rng,
    pub spawner_type: u32,
    pub spawner_variant: u32,
    /// Actual50DEDB Spawn subtype argument, owned by this factory.
    pub subtype: u32,
    pub parent: Option<ActorRef>,
    pub attack_flags: AttackFlags,
    pub source_color: AttackColor,
    pub current_color: AttackColor,
    pub explosion_damage: f32,
    pub grid_collision_class: u32,
    pub grid_collided: bool,
    pub shot_marker: bool,
    pub player_contact_countdown: u8,
    ///457CD8 reads a global ordinal whose initial/history owner is not closed.
    pub ordinal_7ec: Option<u32>,
}

impl BombShot {
    fn validate(&self) -> Result<(), String> {
        if self.attack_flags.0 != 0
            || self.spawner_type != 1
            || self.spawner_variant != 0
            || self.subtype != 0
            || self.grid_collision_class != 6
            || !self.shot_marker
            || !self.source_color.is_finite()
            || !self.current_color.is_finite()
            || !self.explosion_damage.is_finite()
            || self.explosion_damage < 0.0
            || self.player_contact_countdown > 4
        {
            return Err("unsupported ordinary Dr. Fetus bomb state".into());
        }
        Ok(())
    }
}

/// Caller resolves actual kind5 parameters and vectors before its SharedRaw seed.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DrFetusBirth {
    pub spawn_seed: u32,
    pub position: WorldVector,
    pub velocity: WorldVector,
    pub explosion_damage: f32,
    pub flags: AttackFlags,
    pub color: AttackColor,
    pub parent_player: ActorRef,
}

///50DE10 stores the real Bomb after Init; no inventory debit, C18 bind or first Full.
pub(crate) fn admit_dr_fetus(
    state: &mut State,
    phase: Phase,
    birth: DrFetusBirth,
) -> Result<u32, String> {
    if phase == Phase::Skipped
        || birth.parent_player
            != (ActorRef {
                kind: ActorKind::Player,
                id: 0,
            })
        || birth.flags.0 != 0
        || !birth.color.is_finite()
        || !birth.explosion_damage.is_finite()
        || birth.explosion_damage < 0.0
    {
        return Err("unsupported ordinary Dr. Fetus bomb birth".into());
    }
    let id = state.next_bomb;
    let next = id.checked_add(1).ok_or("bomb IDs exhausted")?;
    let mut motion = BombMotion::spawn_dr_fetus(birth.position, birth.velocity)?;
    motion.present_birth(phase)?;
    let shot = BombShot {
        active: Some(true),
        dead: Some(false),
        child: None,
        child_known: true,
        actor_rng: crate::items::Rng::new(birth.spawn_seed, [4, 3, 17]),
        spawner_type: 1,
        spawner_variant: 0,
        subtype: 0,
        parent: Some(birth.parent_player),
        attack_flags: birth.flags,
        source_color: birth.color,
        current_color: birth.color,
        explosion_damage: birth.explosion_damage,
        grid_collision_class: 6,
        grid_collided: false,
        shot_marker: true,
        player_contact_countdown: 4,
        ordinal_7ec: None,
    };
    shot.validate()?;
    state.actor_scheduler.admit(ActorRef {
        kind: ActorKind::Bomb,
        id,
    })?;
    state.bombs.push(Bomb {
        id,
        variant: 0,
        spawn_seed: Some(birth.spawn_seed),
        birth_game_update: state.game_update_count,
        friction: 1.0,
        animation: "Pulse".into(),
        animation_frame: 0,
        animation_playing: true,
        x: f64::from(motion.presented_position.x) * WORLD_SCALE + 32.0,
        y: f64::from(motion.presented_position.y) * WORLD_SCALE - 46.0,
        motion,
        player_contact_countdown: 0,
        shot: Some(shot),
    });
    state.next_bomb = next;
    Ok(id)
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Explosion {
    pub id: u32,
    pub x: f64,
    pub y: f64,
    pub animation_frame: u8,
    /// Actual1000/1 owner. This row only projects its frame/light; it never
    /// owns another actor, callback, SFX or RNG draw. Legacy rows stay explicit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_effect: Option<u32>,
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
        // Player5318a5's actual dead branch encloses509540. That action
        // checks controls/input/inventory/cooldown, without a red-health veto.
        || state.player.dead
        || phase == Phase::Skipped
    {
        return Ok(());
    }
    let next = state.next_bomb.checked_add(1).ok_or("bomb IDs exhausted")?;
    // Player509612 draws the Game.Spawn seed after the variant/flag queries.
    let spawn_seed = state.boot_lifecycle.shared.next_u32();
    let mut motion = BombMotion::spawn(player_world(state), player_velocity(state))?;
    motion.present_birth(phase)?;
    state.actor_scheduler.admit(crate::actors::ActorRef {
        kind: crate::actors::ActorKind::Bomb,
        id: state.next_bomb,
    })?;
    state.bombs.push(Bomb {
        id: state.next_bomb,
        variant: 0,
        spawn_seed: Some(spawn_seed),
        birth_game_update: state.game_update_count,
        friction: 1.0,
        animation: "Pulse".into(),
        animation_frame: 0,
        animation_playing: true,
        x: f64::from(motion.presented_position.x) * WORLD_SCALE + 32.0,
        y: f64::from(motion.presented_position.y) * WORLD_SCALE - 46.0,
        motion,
        player_contact_countdown: 4,
        shot: None,
    });
    state.next_bomb = next;
    state.inventory.bombs -= 1;
    state.bomb_placement_cooldown.record_placement();
    Ok(())
}

/// Pickup::Update's Game::Spawn admits a child at the same ground position,
/// with no player parent, no placement cost/cooldown and zero initial velocity.
pub fn admit_conversion(
    state: &mut State,
    variant: u32,
    spawn_seed: u32,
    position: WorldVector,
) -> Result<(), String> {
    if !matches!(variant, 3 | 4) {
        return Err("unsupported pickup bomb conversion".into());
    }
    let next = state.next_bomb.checked_add(1).ok_or("bomb IDs exhausted")?;
    let motion = BombMotion::spawn(position, WorldVector { x: 0.0, y: 0.0 })?;
    state.actor_scheduler.admit(crate::actors::ActorRef {
        kind: crate::actors::ActorKind::Bomb,
        id: state.next_bomb,
    })?;
    state.bombs.push(Bomb {
        id: state.next_bomb,
        variant,
        spawn_seed: Some(spawn_seed),
        birth_game_update: state.game_update_count,
        friction: 1.0,
        animation: "Appear".into(),
        animation_frame: 0,
        animation_playing: true,
        x: f64::from(position.x) * WORLD_SCALE + 32.0,
        y: f64::from(position.y) * WORLD_SCALE - 46.0,
        motion,
        player_contact_countdown: 0,
        shot: None,
    });
    state.next_bomb = next;
    Ok(())
}

/// Room.Save's (5CE2D0) EntitySaveState for a live Bomb4: Game.Spawn's
/// type/variant/subtype/position/seed. 5C5860 respawns it through
/// Game.Spawn, so Init gives it a fresh fuse and clocks.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedBombRecord {
    pub variant: u32,
    pub subtype: u32,
    pub position: WorldVector,
    pub spawn_seed: Option<u32>,
}

/// Live bombs in entity (scheduler) order. Exploding and removed bodies are
/// dead entities, which Room.Save skips.
pub(crate) fn capture_saved_records(state: &State) -> Vec<SavedBombRecord> {
    state
        .actor_scheduler
        .full_phase()
        .into_iter()
        .filter(|actor| actor.kind == ActorKind::Bomb)
        .filter_map(|actor| state.bombs.iter().find(|bomb| bomb.id == actor.id))
        .filter(|bomb| bomb.motion.lifetime == BombLifetime::Alive)
        .map(|bomb| SavedBombRecord {
            variant: bomb.variant,
            subtype: bomb.shot.as_ref().map_or(0, |shot| shot.subtype),
            position: bomb.motion.position,
            spawn_seed: bomb.spawn_seed,
        })
        .collect()
}

pub(crate) fn validate_saved_records(records: &[SavedBombRecord]) -> Result<(), String> {
    if records.len() > 512 || records.iter().any(|record| !record.position.is_finite()) {
        return Err("saved bomb record is outside the supported format".into());
    }
    Ok(())
}

/// 5C5860's Bomb4 case: Game.Spawn at the saved position with the saved
/// seed and no parent, so the bomb starts a new fuse at rest, as a Pickup
/// conversion's Game.Spawn does.
pub(crate) fn restore_saved_records(
    state: &mut State,
    records: &[SavedBombRecord],
) -> Result<(), String> {
    validate_saved_records(records)?;
    for record in records {
        let next = state.next_bomb.checked_add(1).ok_or("bomb IDs exhausted")?;
        let motion = BombMotion::spawn(record.position, WorldVector { x: 0.0, y: 0.0 })?;
        state.actor_scheduler.admit(ActorRef {
            kind: ActorKind::Bomb,
            id: state.next_bomb,
        })?;
        state.bombs.push(Bomb {
            id: state.next_bomb,
            variant: record.variant,
            spawn_seed: record.spawn_seed,
            birth_game_update: state.game_update_count,
            friction: 1.0,
            animation: if matches!(record.variant, 3 | 4) {
                "Appear"
            } else {
                "Pulse"
            }
            .into(),
            animation_frame: 0,
            animation_playing: true,
            x: f64::from(record.position.x) * WORLD_SCALE + 32.0,
            y: f64::from(record.position.y) * WORLD_SCALE - 46.0,
            motion,
            player_contact_countdown: 0,
            shot: None,
        });
        state.next_bomb = next;
    }
    Ok(())
}

fn advance_animation(bomb: &mut Bomb, detonated: bool) {
    if detonated {
        bomb.animation = "Explode".into();
        bomb.animation_frame = 0;
        bomb.animation_playing = true;
        return;
    }
    if !bomb.animation_playing && bomb.motion.lifetime == BombLifetime::Alive {
        bomb.animation = "Pulse".into();
        bomb.animation_frame = 0;
        bomb.animation_playing = true;
    }
    if bomb.animation_playing {
        let frames = match bomb.animation.as_str() {
            "Appear" => 35,
            "Pulse" => 59,
            "Explode" => {
                if matches!(bomb.variant, 3 | 4) {
                    5
                } else {
                    10
                }
            }
            _ => unreachable!("admitted bomb animation"),
        };
        bomb.animation_frame += 1;
        if bomb.animation_frame >= frames {
            bomb.animation_frame = frames - 1;
            bomb.animation_playing = false;
        }
    }
}

/// Keep the existing public explosion/light view linked to its actual Effect1.
/// Its scheduler callback owns animation and SFX; this projection owns neither.
/// Old serialized rows without an owner retain their historical adapter only.
pub fn advance_explosions(state: &mut State, phase: Phase, fence: usize) {
    for explosion in &mut state.explosions {
        if let Some(id) = explosion.source_effect {
            if let Some(effect) = state.sprite_effects.iter().find(|effect| {
                effect.id == id && effect.variant == 1 && effect.explosion_effect.is_some()
            }) {
                explosion.x = f64::from(effect.presented_position.x) * WORLD_SCALE + 32.0;
                explosion.y = f64::from(effect.presented_position.y) * WORLD_SCALE - 46.0;
                explosion.animation_frame = effect.sprite.frame as u8;
            } else {
                // This view never keeps an absent body alive. The actual
                // effect roster and lifecycle remain the authority.
                explosion.animation_frame = 19;
            }
        }
    }
    if phase == Phase::Update {
        for explosion in state.explosions.iter_mut().take(fence) {
            if explosion.source_effect.is_some() {
                continue;
            }
            if explosion.animation_frame == 0 {
                state.audio.admit(
                    &mut state.boot_lifecycle.shared,
                    state.system_clock.system_counter.wrapping_sub(1),
                    crate::audio::SoundRequest {
                        id: 182,
                        volume: 1.0,
                        delay: 2,
                        looping: false,
                        pitch: 1.0,
                    },
                );
            }
            explosion.animation_frame += 1;
        }
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
    if let Some(shot) = &state.bombs[index].shot {
        shot.validate()?;
        //454E8F skips inactive Full callbacks; Fast draw-list calls remain.
        if phase == Phase::Update && shot.active == Some(false) {
            return Ok(());
        }
        if state.bombs[index].variant != 0
            || state.bombs[index].motion.initial_fuse_updates != DR_FETUS_FUSE_UPDATES
        {
            return Err("Dr. Fetus bomb birth configuration changed".into());
        }
    } else if state.bombs[index].motion.initial_fuse_updates != INITIAL_FUSE_UPDATES {
        return Err("legacy bomb born fuse changed".into());
    }
    let corpse_frames = if matches!(state.bombs[index].variant, 3 | 4) {
        5
    } else {
        CORPSE_FRAMES
    };
    let friction = state.bombs[index].friction;
    let mut next_motion = state.bombs[index].motion;
    let detonated = next_motion.advance_variant(phase, friction, corpse_frames)?;
    let shot_detonation = detonated && state.bombs[index].shot.is_some();
    if shot_detonation {
        //45776A calls555300 before457784 publishes31F/Explode. Publish
        //the completed Base/Bomb vectors now, while the actual helper still
        //sees the live source actor. Completed-call clocks publish afterward.
        state.bombs[index].motion.position = next_motion.position;
        state.bombs[index].motion.presented_position = next_motion.presented_position;
        state.bombs[index].motion.velocity = next_motion.velocity;
    } else {
        state.bombs[index].motion = next_motion;
    }
    if phase == Phase::Update {
        state.bombs[index].friction = 1.0;
        //Base already advances Pulse before the source explosion helper;
        //only the new shot's Explode transition is deferred to its real fence.
        advance_animation(&mut state.bombs[index], detonated && !shot_detonation);
        // Update457bb0 targets the nearest player at position + velocity*5.
        // The strict 270-world-unit envelope owns a normalized 1.5 impulse,
        // followed by .8 velocity damping. Early age also sets next-turn drag.
        if state.bombs[index].variant == 4 {
            let target = player_world(state) + player_velocity(state) * 5.0;
            let bomb = &mut state.bombs[index];
            let delta = WorldVector {
                x: target.x - bomb.motion.position.x,
                y: target.y - bomb.motion.position.y,
            };
            let squared = delta.x * delta.x + delta.y * delta.y;
            if squared < 72_900.0 {
                let length = squared.sqrt();
                let direction = if length == 0.0 {
                    WorldVector { x: 0.0, y: 0.0 }
                } else {
                    // Normalize612820 rounds one reciprocal before multiplying
                    // each component; direct component division differs in f32.
                    delta * (1.0 / length)
                };
                bomb.motion.velocity = bomb.motion.velocity + direction * 1.5;
            }
            bomb.motion.velocity = bomb.motion.velocity * 0.8;
            if state
                .game_update_count
                .saturating_sub(bomb.birth_game_update)
                <= 19
            {
                bomb.friction = 0.8;
            }
        }
    }
    if state.bombs[index].shot.is_some() {
        if phase == Phase::Interpolate
            && state.bombs[index].motion.lifetime != BombLifetime::Removed
        {
            let grid = room_grid(state)?;
            resolve_shot_grid(&mut state.bombs[index], &grid, true)?;
        }
        // Full class6 grid correction belongs to454B10 after entity contacts,
        // including births excluded by the current actor-update fence.
    } else if phase == Phase::Interpolate
        && state.bombs[index].motion.lifetime != BombLifetime::Removed
    {
        let grid = room_grid(state)?;
        let motion = &mut state.bombs[index].motion;
        let mut preview_velocity = motion.velocity;
        bomb_collider().resolve_nonplayer_ground(
            &grid,
            &mut motion.presented_position,
            &mut preview_velocity,
            true,
        )?;
    }
    // Bomb457060 calls Base447190 without a full grid query. The current live
    // manager list receives that query at454B10, after every entity contact.
    let position = state.bombs[index].motion.position;
    let presented = state.bombs[index].motion.presented_position;
    state.bombs[index].x = f64::from(presented.x) * WORLD_SCALE + 32.0;
    state.bombs[index].y = f64::from(presented.y) * WORLD_SCALE - 46.0;
    if detonated {
        let damage = state.bombs[index]
            .shot
            .as_ref()
            .map_or(60.0, |shot| shot.explosion_damage);
        let radius = if damage >= 105.0 {
            105.0
        } else if damage > 85.0 {
            90.0
        } else {
            BLAST_RADIUS
        };
        explode(
            state,
            position,
            radius,
            damage,
            crate::combat::DamageSource { id, entity_type: 4 },
        )?;
        //457752 passes the actual Sprite Color2D0, not permanent3A0.
        let color = state.bombs[index]
            .shot
            .as_ref()
            .map_or(AttackColor::WHITE, |shot| shot.current_color);
        spawn_blast_effects(state, id, position, color)?;
        //457784 follows the entire555300 call, including both actual children.
        if shot_detonation {
            state.bombs[index].motion = next_motion;
            advance_animation(&mut state.bombs[index], true);
        }
    }
    if let Some(shot) = &mut state.bombs[index].shot {
        //Explicit Remove may precede natural clocks; never revive that31F.
        if shot.active != Some(false) {
            shot.active = shot
                .active
                .map(|_| next_motion.lifetime != BombLifetime::Removed);
            shot.dead = shot
                .dead
                .map(|_| next_motion.lifetime != BombLifetime::Alive);
        }
    }
    //4575DD..EA follows555300 and the dead/Explode stores. NeverFast.
    if phase == Phase::Update {
        let bomb = &mut state.bombs[index];
        match &mut bomb.shot {
            Some(shot) => {
                shot.player_contact_countdown = shot.player_contact_countdown.saturating_sub(1)
            }
            None => bomb.player_contact_countdown = bomb.player_contact_countdown.saturating_sub(1),
        }
    }
    Ok(())
}

fn room_grid(state: &State) -> Result<crate::grid::Grid, String> {
    state
        .room
        .grid
        .as_ref()
        .map(|descriptor| crate::grid::Grid::new(descriptor.clone()))
        .transpose()
        .map(|grid| grid.unwrap_or_else(crate::grid::Grid::closed_1x1_boundary))
}

fn bomb_collider() -> &'static crate::grid::Collider {
    static COLLIDER: std::sync::OnceLock<crate::grid::Collider> = std::sync::OnceLock::new();
    COLLIDER.get_or_init(|| {
        crate::grid::Collider::new(crate::grid::CollisionGeometry {
            radius: 16.0,
            scale: WorldVector { x: 1.0, y: 1.0 },
            points: 12,
        })
        .expect("authored ordinary bomb geometry is valid")
    })
}

fn project(bomb: &mut Bomb) {
    bomb.x = f64::from(bomb.motion.presented_position.x) * WORLD_SCALE + 32.0;
    bomb.y = f64::from(bomb.motion.presented_position.y) * WORLD_SCALE - 46.0;
}

fn contact_body(bomb: &Bomb) -> crate::combat::ContactBody {
    crate::combat::ContactBody {
        id: bomb.id,
        entity_type: 4,
        position: bomb.motion.position,
        velocity: bomb.motion.velocity,
        radius: 16.0,
        mass: 6.0,
    }
}

fn commit_contact(bomb: &mut Bomb, body: crate::combat::ContactBody) {
    bomb.motion.position = body.position;
    bomb.motion.presented_position = body.position;
    bomb.motion.velocity = body.velocity;
    project(bomb);
}

fn resolve_shot_grid(
    bomb: &mut Bomb,
    grid: &crate::grid::Grid,
    preview: bool,
) -> Result<(), String> {
    bomb.shot
        .as_ref()
        .ok_or("ordinary bomb shot metadata absent")?
        .validate()?;
    let mut position = if preview {
        bomb.motion.presented_position
    } else {
        bomb.motion.position
    };
    position = grid.clamp_room(position, 0.0);
    let collided = bomb_collider()
        .samples()
        .iter()
        .any(|sample| grid.collision_at(position + *sample) > 1);
    let mut velocity = bomb.motion.velocity;
    bomb_collider().resolve_nonplayer_no_pits(grid, &mut position, &mut velocity, preview)?;
    bomb.motion.presented_position = position;
    if !preview {
        bomb.motion.position = position;
        bomb.motion.velocity = velocity;
        bomb.shot
            .as_mut()
            .expect("checked shot owner")
            .grid_collided = collided;
    }
    project(bomb);
    Ok(())
}

///454B10 queries the current live list after entity contacts, including births
///excluded from Full. Dead31F alone does not retire an Exploding bomb.
pub(crate) fn full_grid_contacts(state: &mut State) -> Result<(), String> {
    if !state
        .bombs
        .iter()
        .any(|bomb| bomb.motion.lifetime != BombLifetime::Removed)
    {
        return Ok(());
    }
    let grid = room_grid(state)?;
    for bomb in &mut state.bombs {
        if bomb.motion.lifetime == BombLifetime::Removed {
            continue;
        }
        if bomb.shot.is_some() {
            resolve_shot_grid(bomb, &grid, false)?;
        } else {
            let motion = &mut bomb.motion;
            bomb_collider().resolve_nonplayer_ground(
                &grid,
                &mut motion.position,
                &mut motion.velocity,
                false,
            )?;
            motion.presented_position = motion.position;
            project(bomb);
        }
    }
    Ok(())
}

///4453A0 calls Player509160 first (Bomb fails444810: returnsfalse), then
///Bomb456C20 with flagfalse. Planted, converted and shot bombs share it: a
///positive Bomb+7DC countdown resets to 4 and vetoes the mass response, so
///Isaac does not push the bomb he stands on; at zero the bomb takes
///PlayerV*.4 and the mass response pushes it (pickup-interactions suite).
pub(crate) fn player_contact(
    state: &mut State,
    index: usize,
    grid: &crate::grid::Grid,
) -> Result<(), String> {
    if state.bombs[index].motion.lifetime != BombLifetime::Alive {
        return Ok(());
    }
    static COLLIDER: std::sync::OnceLock<crate::grid::Collider> = std::sync::OnceLock::new();
    let collider = COLLIDER.get_or_init(|| {
        crate::grid::Collider::new(crate::grid::CollisionGeometry {
            radius: 10.0,
            scale: WorldVector { x: 1.0, y: 1.0 },
            points: 40,
        })
        .expect("authored Isaac geometry is valid")
    });
    let mut player = crate::combat::ContactBody {
        id: 0,
        entity_type: 1,
        position: player_world(state),
        velocity: player_velocity(state),
        radius: 10.0,
        mass: 5.0,
    };
    let original = player.position;
    let player_before = player;
    let mut body = contact_body(&state.bombs[index]);
    let bomb = &mut state.bombs[index];
    let countdown = match &mut bomb.shot {
        Some(shot) => {
            shot.validate()?;
            &mut shot.player_contact_countdown
        }
        None => &mut bomb.player_contact_countdown,
    };
    crate::combat::resolve_player_bomb_contact(&mut player, &mut body, countdown, |shift| {
        collider.blocks_player_shift(grid, original, shift)
    })?;
    if player != player_before {
        state.player.x = f64::from(player.position.x) * WORLD_SCALE + 32.0;
        state.player.y = f64::from(player.position.y) * WORLD_SCALE - 46.0;
        state.player.vx = f64::from(player.velocity.x) * WORLD_SCALE;
        state.player.vy = f64::from(player.velocity.y) * WORLD_SCALE;
    }
    commit_contact(&mut state.bombs[index], body);
    Ok(())
}

/// The grid a Player/Bomb contact checks its shift against.
pub(crate) fn player_contact_grid(state: &State) -> Result<crate::grid::Grid, String> {
    room_grid(state)
}

/// One plain (flagless) regular tear against the live planted/converted
/// bombs, after its NPC candidates. Tear545E80's type4 arm: a tear moving away
/// from the bomb (dot(Bomb-Tear, TearV) < 0) at most 2 updates after birth
/// (442B30) passes through; a shot bomb (7E8 marker) vetoes and the tear flies
/// on. Otherwise the tear dies on the bomb (546318) and the mass response
/// pushes it. Returns whether the tear died. 444810 (type 10..999) is false
/// for a bomb, so tear effect flags never apply; the caller admits plain and
/// spectral (bit0) tears, which take this path unchanged. Other flags
/// (piercing, 0x1000, 0x80000...) take other arms and are not admitted here.
pub(crate) fn tear_hit(
    state: &mut State,
    motion: &mut crate::projectiles::TearMotion,
    radius: f32,
    tear_id: u32,
    birth_game_update: Option<u32>,
) -> Result<bool, String> {
    for index in 0..state.bombs.len() {
        let bomb = &state.bombs[index];
        if bomb.shot.is_some() || bomb.motion.lifetime != BombLifetime::Alive {
            continue;
        }
        let mut tear = crate::combat::ContactBody {
            id: tear_id,
            entity_type: 2,
            position: motion.position,
            velocity: motion.velocity,
            radius,
            mass: 8.0,
        };
        let mut body = contact_body(bomb);
        let delta = WorldVector {
            x: body.position.x - tear.position.x,
            y: body.position.y - tear.position.y,
        };
        let receding = delta.x * tear.velocity.x + delta.y * tear.velocity.y < 0.0;
        let before = (tear, body);
        let resolution = crate::combat::resolve_tear_bomb_contact(&mut tear, &mut body)?;
        if !resolution.colliding && (tear, body) == before {
            continue;
        }
        if receding {
            let birth = birth_game_update
                .ok_or("tear/bomb contact of a receding tear requires its birth update")?;
            if state.game_update_count.wrapping_sub(birth) <= 2 {
                continue;
            }
        }
        motion.position = tear.position;
        motion.presented_position = tear.position;
        motion.velocity = tear.velocity;
        commit_contact(&mut state.bombs[index], body);
        return Ok(true);
    }
    Ok(false)
}

///Bomb/Bomb self-pair of the shared NPC/Bomb partition (453340, bucket
///order). Both callbacks run; the first actual shot marker vetoes mass
///response. Legacy no-shot pairs stay untouched.
pub(crate) fn bomb_pair(state: &mut State, first_id: u32, second_id: u32) -> Result<(), String> {
    let (Some(first), Some(second)) = (
        state.bombs.iter().position(|bomb| bomb.id == first_id),
        state.bombs.iter().position(|bomb| bomb.id == second_id),
    ) else {
        return Ok(());
    };
    let (first, second) = if first < second {
        let (left, right) = state.bombs.split_at_mut(second);
        (&mut left[first], &mut right[0])
    } else {
        let (left, right) = state.bombs.split_at_mut(first);
        (&mut right[0], &mut left[second])
    };
    if (first.shot.is_none() && second.shot.is_none())
        || first.motion.lifetime != BombLifetime::Alive
        || second.motion.lifetime != BombLifetime::Alive
    {
        return Ok(());
    }
    if let Some(shot) = &first.shot {
        shot.validate()?;
    }
    if let Some(shot) = &second.shot {
        shot.validate()?;
    }
    let mut a = contact_body(first);
    let mut b = contact_body(second);
    crate::combat::resolve_bomb_pair(
        &mut a,
        &mut b,
        first.shot.as_ref().is_some_and(|shot| shot.shot_marker),
    )?;
    commit_contact(first, a);
    commit_contact(second, b);
    Ok(())
}

/// Bomb555385/5554BB and Rock56C8EC/56CA46 enter the same real
/// crater->record->explosion sequence after damage/nested callbacks.
/// The public row is only a compatibility projection for existing lights.
pub(crate) fn spawn_blast_effects(
    state: &mut State,
    id: u32,
    position: WorldVector,
    color: AttackColor,
) -> Result<(), String> {
    let Some(source_effect) =
        crate::explosion_effects::admit_blast_children(state, position, color)?
    else {
        return Ok(());
    };
    state.explosions.push(Explosion {
        id,
        x: f64::from(position.x) * WORLD_SCALE + 32.0,
        y: f64::from(position.y) * WORLD_SCALE - 46.0,
        animation_frame: 0,
        source_effect: Some(source_effect),
    });
    Ok(())
}

/// Game554490 with the line check on and incoming DamageFlags4. Each circle
/// target must pass Room::CheckLine(mode2, ignoreWalls); a blocked line skips
/// knockback and damage alike. Player damage is 2 when the explosion damage
/// is at least 50, else 1, with immunity of 60 per damage unit.
pub(crate) fn explode(
    state: &mut State,
    position: WorldVector,
    radius: f32,
    damage: f32,
    source: crate::combat::DamageSource,
) -> Result<(), String> {
    explode_with_copied_source(state, position, radius, damage, source, None)
}

///Actual common554490 path; only proven callers carry copied EntityRef words.
pub(crate) fn explode_with_copied_source(
    state: &mut State,
    position: WorldVector,
    radius: f32,
    damage: f32,
    source: crate::combat::DamageSource,
    copied_source: Option<crate::combat::CopiedDamageSource>,
) -> Result<(), String> {
    explode_with_kind(
        state,
        position,
        radius,
        damage,
        source,
        copied_source,
        crate::combat::DamageKind::Bomb,
        true,
    )
}

/// Game554490 with the line check off and DamageFlags4 (Mom's stomp 4d4864).
/// The caller owns the >=100 shake draw that precedes the blast.
pub(crate) fn explode_unchecked(
    state: &mut State,
    position: WorldVector,
    radius: f32,
    damage: f32,
    source: crate::combat::DamageSource,
) -> Result<(), String> {
    explode_with_kind(
        state,
        position,
        radius,
        damage,
        source,
        None,
        crate::combat::DamageKind::Bomb,
        false,
    )
}

/// TNT's caller has no source Entity and disables554490's line-check input.
pub(crate) fn explode_tnt(state: &mut State, position: WorldVector) -> Result<(), String> {
    explode_with_kind(
        state,
        position,
        75.0,
        60.0,
        crate::combat::DamageSource {
            id: 0,
            entity_type: 0,
        },
        None,
        crate::combat::DamageKind::Tnt,
        false,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "each argument is a separate input of the 554490 explosion routine: position, radius, damage, source, copied EntityRef, damage flags and the line-check switch TNT disables"
)]
fn explode_with_kind(
    state: &mut State,
    position: WorldVector,
    radius: f32,
    damage: f32,
    source: crate::combat::DamageSource,
    copied_source: Option<crate::combat::CopiedDamageSource>,
    kind: crate::combat::DamageKind,
    check_line: bool,
) -> Result<(), String> {
    //Unclosed rock/alt-rock destruction can add4/5/6 despite no current body.
    crate::ordinary_placement::invalidate(state, crate::ordinary_placement::Frontier::GridEffects);
    let line = state
        .room
        .grid
        .as_ref()
        .map(|descriptor| crate::grid::Grid::new(descriptor.clone()))
        .transpose()?;
    let clear = |target: WorldVector| {
        !check_line
            || line
                .as_ref()
                .is_none_or(|grid| grid.blast_line_clear(position, target, true))
    };
    crate::npcs::blast_hit_with_kind(
        state,
        position,
        radius,
        damage,
        source,
        copied_source,
        kind,
        &clear,
    )?;
    crate::pickups::bomb_hit(state, position, radius, damage, kind.flags(), &clear)?;
    blast_push_bombs(state, position, radius, source, &clear)?;
    crate::fly_effects::bomb_hit(state, position, radius, clear);
    let player_velocity = if clear(player_world(state)) {
        isaac_blast_velocity(
            player_world(state),
            player_velocity(state),
            position,
            radius,
        )?
    } else {
        None
    };
    if let Some(velocity) = player_velocity {
        state.player.vx = f64::from(velocity.x) * WORLD_SCALE;
        state.player.vy = f64::from(velocity.y) * WORLD_SCALE;
        // Bomb555300 supplies actual incoming DamageFlags4 via
        // r9d to Game554490; its Player virtual38 forwards esi.
        let units = if damage >= 50.0 { PLAYER_DAMAGE } else { 1 };
        crate::combat::try_damage_player(
            state,
            units,
            DAMAGE_COOLDOWN_TICKS / u32::from(PLAYER_DAMAGE) * u32::from(units),
            kind.flags(),
        )?;
    }
    crate::critters::blast(state, position, radius, &clear);
    crate::rock_blast::destroy(state, position, radius)
}

/// 554490's actor circle on the other live bombs (type 4, size 16, mass 6).
/// The exploding bomb itself (the source, 55451F) is skipped. A bomb within
/// blast radius + 16 with a clear line takes velocity*0.5 plus the radial
/// impulse min(radius - distance, 15) (55461F..5547A5; mass 6 takes no mass
/// scaling). Base TakeDamage446CF0 does nothing more to a bomb, so it keeps
/// its own fuse.
fn blast_push_bombs(
    state: &mut State,
    center: WorldVector,
    blast_radius: f32,
    source: crate::combat::DamageSource,
    line_clear: &dyn Fn(WorldVector) -> bool,
) -> Result<(), String> {
    for bomb in &mut state.bombs {
        if bomb.motion.lifetime != BombLifetime::Alive
            || (source.entity_type == 4 && source.id == bomb.id)
        {
            continue;
        }
        let delta = WorldVector {
            x: bomb.motion.position.x - center.x,
            y: bomb.motion.position.y - center.y,
        };
        let squared = delta.x * delta.x + delta.y * delta.y;
        let radius = blast_radius + 16.0;
        if !squared.is_finite() {
            return Err("bomb blast geometry overflowed".into());
        }
        if squared >= radius * radius || !line_clear(bomb.motion.position) {
            continue;
        }
        //554625 skips the impulse at exact zero distance.
        if squared == 0.0 {
            continue;
        }
        let distance = squared.sqrt();
        let impulse = (radius - distance).min(15.0);
        //612B30 divides each component before 612A70 scales by the impulse.
        let radial = WorldVector {
            x: delta.x / distance,
            y: delta.y / distance,
        } * impulse;
        bomb.motion.velocity = bomb.motion.velocity * 0.5 + radial;
        if !bomb.motion.velocity.is_finite() {
            return Err("bomb blast velocity overflowed".into());
        }
    }
    Ok(())
}

///Selected neutral50DE10 actor history. A type4 slot alone admits no restore.
pub(crate) fn admits_registered_shot(bomb: &Bomb) -> Result<bool, String> {
    let Some(shot) = &bomb.shot else {
        return Ok(false);
    };
    shot.validate()?;
    if !bomb.friction.is_finite()
        || !bomb.motion.position.is_finite()
        || !bomb.motion.presented_position.is_finite()
        || !bomb.motion.velocity.is_finite()
    {
        return Err("registered Bomb contains nonfinite actual motion".into());
    }
    Ok(bomb.variant == 0
        && shot.parent
            == Some(ActorRef {
                kind: ActorKind::Player,
                id: 0,
            })
        && shot.active.is_some()
        && shot.dead.is_some()
        && shot.child_known
        && shot.child.is_none())
}

pub(crate) fn admits_referenced_remove(state: &State, id: u32) -> Result<bool, String> {
    let bomb = state
        .bombs
        .iter()
        .find(|bomb| bomb.id == id)
        .ok_or("registered Bomb handle is unresolved")?;
    admits_registered_shot(bomb)
}

///441BB0 changes actual31E/31F only; fuse/age/registered links remain intact.
pub(crate) fn remove_referenced(state: &mut State, id: u32) -> Result<(), String> {
    let bomb = state
        .bombs
        .iter_mut()
        .find(|bomb| bomb.id == id)
        .ok_or("registered Bomb handle is unresolved")?;
    if !admits_registered_shot(bomb)? {
        return Err("registered Bomb Remove history is not admitted".into());
    }
    let shot = bomb.shot.as_mut().expect("validated canonical shot");
    shot.active = Some(false);
    shot.dead = Some(true);
    Ok(())
}

///System550860 restores each draw-list445370 before any Player Full callback.
pub(crate) fn restore_shot_fast_positions(state: &mut State) {
    for bomb in &mut state.bombs {
        if bomb.shot.is_some() {
            bomb.motion.presented_position = bomb.motion.position;
            bomb.x = f64::from(bomb.motion.position.x) * WORLD_SCALE + 32.0;
            bomb.y = f64::from(bomb.motion.position.y) * WORLD_SCALE - 46.0;
        }
    }
}

pub(crate) fn referenced_position(state: &State, id: u32) -> Result<Option<WorldVector>, String> {
    let bomb = state
        .bombs
        .iter()
        .find(|bomb| bomb.id == id)
        .ok_or("registered Bomb handle is unresolved")?;
    Ok(admits_registered_shot(bomb)?.then_some(bomb.motion.presented_position))
}

fn uninit_shot_refs(state: &mut State, retired: &[u32]) {
    let is_retired = |actor: ActorRef| actor.kind == ActorKind::Bomb && retired.contains(&actor.id);
    for bomb in &mut state.bombs {
        if let Some(shot) = &mut bomb.shot {
            if retired.contains(&bomb.id) {
                shot.parent = None;
                shot.child = None;
            } else if shot.child.is_some_and(is_retired) {
                shot.child = None;
            }
        }
    }
    if state.attack_runtime.persistent.is_some_and(is_retired) {
        state.attack_runtime.persistent = None;
        state.weapon_admission.c18_is_known_empty = true;
    }
    for slot in &mut state.attack_runtime.secondary {
        if slot.is_some_and(is_retired) {
            *slot = None;
        }
    }
}

///45543E actual inactive Full Uninit precedes the spatial/contact fence.
pub(crate) fn retire_shots_full(state: &mut State) {
    let retired: Vec<_> = state
        .bombs
        .iter()
        .filter(|bomb| {
            bomb.shot.as_ref().is_some_and(|shot| {
                shot.active == Some(false) || bomb.motion.lifetime == BombLifetime::Removed
            })
        })
        .map(|bomb| bomb.id)
        .collect();
    uninit_shot_refs(state, &retired);
    state.bombs.retain(|bomb| !retired.contains(&bomb.id));
    state
        .actor_scheduler
        .retain(|actor| actor.kind != ActorKind::Bomb || !retired.contains(&actor.id));
}

///453EC0 invokes transient Uninit before Remove while rebuilding the Room.
///The inherited untracked planted/conversion adapter remains separate.
pub(crate) fn reset_room_shots(state: &mut State) {
    let retired: Vec<_> = state
        .bombs
        .iter()
        .filter(|bomb| bomb.shot.is_some())
        .map(|bomb| bomb.id)
        .collect();
    uninit_shot_refs(state, &retired);
    for bomb in &mut state.bombs {
        if retired.contains(&bomb.id) {
            if let Some(shot) = &mut bomb.shot {
                shot.active = Some(false);
                shot.dead = Some(true);
            }
        }
    }
    state.bombs.retain(|bomb| !retired.contains(&bomb.id));
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
