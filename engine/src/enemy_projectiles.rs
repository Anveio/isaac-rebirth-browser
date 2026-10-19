//! Fresh flags-zero type9 from actual Round Worm and Monstro producers.
//! Ground interpolation uses BaseRender4446a0; vertical motion is full-only.
use crate::{
    audio::SoundRequest,
    bombs, combat, content,
    grid::{Collider, CollisionGeometry, Grid},
    motion::WorldVector,
    phase::Phase,
    roundworm::Shot,
    State,
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Actual initializer and post-spawn producer fields. A constructor draw can
/// be overwritten by the producer without being omitted from the shared stream.
#[derive(Clone, Copy)]
pub struct ProjectileBirth {
    pub position: WorldVector,
    pub velocity: WorldVector,
    pub spawn_seed: u32,
    pub initial_falling_speed: f32,
    pub parameter_angle: f32,
    pub falling_speed: f32,
    pub falling_acceleration: f32,
    pub scale: f32,
    /// +7d4 after FireProjectiles: Init's -23 plus ProjectileParams+4.
    pub height: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnemyProjectile {
    pub id: u32,
    pub parent_npc: u32,
    pub spawn_seed: u32,
    pub position: WorldVector,
    pub presented_position: WorldVector,
    pub velocity: WorldVector,
    pub height: f32,
    pub falling_speed: f32,
    pub falling_acceleration: f32,
    pub initializer_falling_speed: f32,
    pub source_parameter1: f32,
    pub source_parameter2: f32,
    pub parameter_angle: f32,
    pub scale: f32,
    pub collision_radius: f32,
    pub collision_mass: f32,
    pub grid_collision_class: u32,
    pub entity_collision_class: u32,
    pub grid_contact: bool,
    pub full_updates: u32,
    pub animation_frame: u32,
    pub animation: Option<String>,
    pub animation_clock: f32,
    pub animation_playing: bool,
    pub dead: bool,
    pub accepted_player_hit: bool,
    pub removed: bool,
    pub impact_seed: Option<u32>,
    pub impact_rotation: Option<u32>,
    /// EntityRef +8: the spawner's entity type (0 unknown).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub spawner_type: u32,
    /// Projectile 9.v: 0 blood, 3 puke (ProjectileParams+74).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub variant: u32,
    /// +7f4 ProjectileFlags, OR'd in from ProjectileParams+54 (49ee5d).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub flags: u32,
    /// +440 from ProjectileParams+60 (49ee8c): the orbit centre.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orbit_centre: Option<WorldVector>,
}

/// The purple fire projectile 9.2 (Mega Maw's FireRing).
pub const PURPLE_FIRE: u32 = 2;

/// Parented to its spawner (49efef SetParent447fc0).
pub const FLAG_PARENTED: u32 = 0x80;
/// Orbits +440 one way (5377e8).
pub const FLAG_ORBIT: u32 = 0x800;
/// Orbits +440 the other way (537837 +7f5 & 0x10).
pub const FLAG_ORBIT_REVERSE: u32 = 0x1000;
/// Never collides with the grid (5371b0, 538050).
pub const FLAG_NO_GRID: u32 = 0x2000;
/// The ProjectileFlags the type-9 update admits; any other bit is refused.
pub const ADMITTED_FLAGS: u32 = FLAG_PARENTED | FLAG_ORBIT | FLAG_ORBIT_REVERSE | FLAG_NO_GRID;

/// A FireProjectiles shot whose ProjectileParams carry flags (+54) and a
/// centre (+60).
#[derive(Clone, Copy)]
pub struct ProjectileFlags {
    pub flags: u32,
    pub centre: WorldVector,
}

impl ProjectileFlags {
    /// Unadmitted bits are refused rather than silently ignored, so no
    /// producer inherits motion the update does not implement.
    pub fn admitted(self) -> Result<Self, String> {
        let unadmitted = self.flags & !ADMITTED_FLAGS;
        if unadmitted != 0 {
            return Err(format!("unadmitted ProjectileFlags {unadmitted:#x}"));
        }
        Ok(self)
    }
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

impl EnemyProjectile {
    pub fn birth(id: u32, parent_npc: u32, shot: Shot) -> Self {
        // Worm FireProjectiles(mode0) stores these post-spawn overrides.
        Self::birth_with_parameters(
            id,
            parent_npc,
            ProjectileBirth {
                position: shot.position,
                velocity: shot.velocity,
                spawn_seed: shot.spawn_seed,
                initial_falling_speed: shot.initial_falling_speed,
                parameter_angle: shot.parameter_angle,
                falling_speed: -3.0,
                falling_acceleration: 0.1,
                scale: 1.0,
                height: -23.0,
            },
        )
    }

    pub fn birth_with_parameters(id: u32, parent_npc: u32, shot: ProjectileBirth) -> Self {
        Self {
            id,
            parent_npc,
            spawn_seed: shot.spawn_seed,
            position: shot.position,
            presented_position: shot.position,
            velocity: shot.velocity,
            height: shot.height,
            falling_speed: shot.falling_speed,
            falling_acceleration: shot.falling_acceleration,
            initializer_falling_speed: shot.initial_falling_speed,
            source_parameter1: 1.0,
            source_parameter2: 0.005,
            parameter_angle: shot.parameter_angle,
            scale: shot.scale,
            collision_radius: content::entity(9, 0).collision_radius,
            collision_mass: content::entity(9, 0).collision_mass,
            grid_collision_class: 4,
            entity_collision_class: 4,
            grid_contact: false,
            full_updates: 0,
            animation_frame: 0,
            animation: None,
            animation_clock: -1.0,
            animation_playing: false,
            dead: false,
            accepted_player_hit: false,
            removed: false,
            impact_seed: None,
            impact_rotation: None,
            spawner_type: 0,
            variant: 0,
            flags: 0,
            orbit_centre: None,
        }
    }

    /// 49ee5d/49ee8c: the flags and centre a FireProjectiles shot copies
    /// from its ProjectileParams (checked by `ProjectileFlags::admitted`).
    pub fn with_flags(mut self, params: ProjectileFlags) -> Self {
        self.flags = params.flags;
        self.orbit_centre = Some(params.centre);
        self
    }

    pub fn advance(&mut self, phase: Phase, grid: &Grid) -> Result<bool, String> {
        if self.removed {
            return Ok(false);
        }
        if phase != Phase::Update {
            self.presented_position = self.position + self.velocity * 0.5;
            return Ok(false);
        }
        self.full_updates = self
            .full_updates
            .checked_add(1)
            .ok_or("projectile update count exhausted")?;
        // Source536e37 selects a regular clip from actual scale every full;
        // Worm scale1 remains6. The purple fire shot 9.2 plays its looping
        // Flickering clip instead (5370f3).
        let name = if self.variant == PURPLE_FIRE {
            "Flickering".to_string()
        } else {
            let size = (self.scale * 3.0 + 3.0).trunc().min(13.0) as u32;
            format!("RegularTear{size}")
        };
        let frames = content::clip(content::entity(9, self.variant).actor, &name)
            .ok_or("hostile projectile scale requires an unadmitted authored clip")?
            .frames;
        self.animation = Some(name);
        self.animation_frame = self.full_updates % frames;
        self.animation_clock = self.animation_frame as f32;
        self.animation_playing = true;
        self.position = self.position + self.velocity;
        self.presented_position = self.position;
        if self.dead {
            self.removed = true;
            return Ok(true); // Source impact producer executes after BaseUpdate.
        }
        if self.flags & (FLAG_ORBIT | FLAG_ORBIT_REVERSE) != 0 {
            self.orbit()?;
        }
        self.falling_speed = (0.9 * self.falling_speed + 0.1) + self.falling_acceleration;
        self.height += self.falling_speed;
        if self.height < -50.0 {
            self.grid_collision_class = 0;
            self.entity_collision_class = 0;
        } else {
            // 538050: a no-grid shot keeps grid class 0.
            self.grid_collision_class = if self.flags & FLAG_NO_GRID != 0 { 0 } else { 4 };
            self.entity_collision_class = 4;
        }
        if self.height > -5.0 || (self.grid_contact && self.height >= -50.0) {
            self.dead = true;
        }
        // Common post-update ground query, read by the following Update.
        self.grid_contact =
            self.grid_collision_class == 4 && grid.contacts_enemy_projectile(self.position);
        if !self.position.is_finite() || !self.height.is_finite() || !self.falling_speed.is_finite()
        {
            return Err("hostile projectile motion overflowed".into());
        }
        Ok(false)
    }
}

impl EnemyProjectile {
    /// 5377e8: after BaseUpdate, steer along the tangent of the centre
    /// offset plus half of it, keeping the speed. FLAG_ORBIT_REVERSE turns
    /// the tangent round (538a90).
    fn orbit(&mut self) -> Result<(), String> {
        let centre = self
            .orbit_centre
            .ok_or("orbiting projectile requires its ProjectileParams centre")?;
        let offset = WorldVector {
            x: self.position.x - centre.x,
            y: self.position.y - centre.y,
        };
        let mut tangent = WorldVector {
            x: -offset.y,
            y: offset.x,
        };
        if self.flags & FLAG_ORBIT_REVERSE != 0 {
            tangent = tangent * -1.0;
        }
        let speed = (self.velocity.x * self.velocity.x + self.velocity.y * self.velocity.y).sqrt();
        self.velocity = crate::npcs::normalized(offset * 0.5 + tangent) * speed;
        Ok(())
    }
}

pub fn advance_actor(state: &mut State, phase: Phase, id: u32) -> Result<(), String> {
    let grid = Grid::new(
        state
            .room
            .grid
            .clone()
            .ok_or("hostile projectile requires admitted grid")?,
    )?;
    if let Some(projectile) = state
        .enemy_projectiles
        .iter_mut()
        .find(|projectile| projectile.id == id)
    {
        if projectile.advance(phase, &grid)? {
            if projectile.variant == PURPLE_FIRE {
                // 538b8c: a dead 9.2 spawns a tinted Poof 1000.15 at -10 px
                // and plays no sound. PORT GAP: as for the other impacts,
                // the effect itself is not constructed; only its seed is
                // drawn.
                projectile.impact_seed = Some(state.boot_lifecycle.shared.next_u32());
                return Ok(());
            }
            // Dead variant0 emits real effect11 seed then bounded4 rotation,
            // then SFX150 (wall) or258 (ground). FX rendering/constructor
            // consumers remain a separately documented stream gap.
            projectile.impact_seed = Some(state.boot_lifecycle.shared.next_u32());
            projectile.impact_rotation = Some(state.boot_lifecycle.shared.bounded(4) * 90);
            state.audio.admit(
                &mut state.boot_lifecycle.shared,
                state.system_clock.system_counter.wrapping_sub(1),
                SoundRequest {
                    id: if projectile.height > -5.0 { 258 } else { 150 },
                    volume: 1.0,
                    delay: 2,
                    looping: false,
                    pitch: 1.0,
                },
            );
        }
    }
    Ok(())
}

pub fn player_contacts(state: &mut State) -> Result<(), String> {
    let grid = Grid::new(
        state
            .room
            .grid
            .clone()
            .ok_or("projectile contact requires admitted grid")?,
    )?;
    static COLLIDER: OnceLock<Collider> = OnceLock::new();
    let collider = COLLIDER.get_or_init(|| {
        Collider::new(CollisionGeometry {
            radius: content::PLAYER_COLLISION_RADIUS_WORLD as f32,
            scale: WorldVector { x: 1.0, y: 1.0 },
            points: 40,
        })
        .expect("authored player geometry")
    });
    for index in 0..state.enemy_projectiles.len() {
        //4453a0 rejects either actual dead31f before the next callback.
        if state.player.dead {
            break;
        }
        let projectile = &state.enemy_projectiles[index];
        if projectile.dead || projectile.removed || projectile.entity_collision_class == 0 {
            continue;
        }
        let mut player = combat::ContactBody {
            id: 0,
            entity_type: 1,
            position: bombs::player_world(state),
            velocity: bombs::player_velocity(state),
            radius: content::PLAYER_COLLISION_RADIUS_WORLD as f32,
            mass: content::PLAYER_COLLISION_MASS_WORLD as f32,
        };
        let mut projectile_body = combat::ContactBody {
            id: projectile.id,
            entity_type: 9,
            position: projectile.position,
            velocity: projectile.velocity,
            radius: projectile.collision_radius,
            mass: projectile.collision_mass,
        };
        let original = player.position;
        let result = combat::resolve_player_projectile_contact(
            &mut player,
            &mut projectile_body,
            |shift| collider.blocks_player_shift(&grid, original, shift),
        )?;
        if !result.colliding {
            continue;
        }
        let source = Some(state.enemy_projectiles[index].spawner_type);
        let accepted = combat::try_damage_player_from(state, 1, 60, 0, source)?;
        state.player.x = f64::from(player.position.x) * 0.65 + 32.0;
        state.player.y = f64::from(player.position.y) * 0.65 - 46.0;
        state.player.vx = f64::from(player.velocity.x) * 0.65;
        state.player.vy = f64::from(player.velocity.y) * 0.65;
        let projectile = &mut state.enemy_projectiles[index];
        projectile.dead = true;
        projectile.accepted_player_hit = accepted;
        projectile.position = projectile_body.position;
        projectile.presented_position = projectile_body.position;
        projectile.velocity = projectile_body.velocity;
    }
    Ok(())
}
