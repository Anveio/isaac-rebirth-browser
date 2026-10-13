//! Fresh flags-zero type9 produced by ordinary Round Worm FireProjectiles.
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
    pub parameter_angle: f32,
    pub scale: f32,
    pub collision_radius: f32,
    pub collision_mass: f32,
    pub grid_collision_class: u32,
    pub entity_collision_class: u32,
    pub grid_contact: bool,
    pub full_updates: u32,
    pub animation_frame: u32,
    pub dead: bool,
    pub removed: bool,
    pub impact_seed: Option<u32>,
    pub impact_rotation: Option<u32>,
}

impl EnemyProjectile {
    pub fn birth(id: u32, parent_npc: u32, shot: Shot) -> Self {
        Self {
            id,
            parent_npc,
            spawn_seed: shot.spawn_seed,
            position: shot.position,
            presented_position: shot.position,
            velocity: shot.velocity,
            height: -23.0,
            falling_speed: shot.initial_falling_speed,
            falling_acceleration: 0.0,
            parameter_angle: shot.parameter_angle,
            scale: 1.0,
            collision_radius: content::NPC_9_COLLISIONRADIUS,
            collision_mass: content::NPC_9_COLLISIONMASS,
            grid_collision_class: 4,
            entity_collision_class: 4,
            grid_contact: false,
            full_updates: 0,
            animation_frame: 0,
            dead: false,
            removed: false,
            impact_seed: None,
            impact_rotation: None,
        }
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
        // Fresh variant0 plays authored RegularTear6, then common BaseUpdate.
        self.animation_frame = self
            .animation_frame
            .checked_add(1)
            .ok_or("projectile animation exhausted")?;
        self.position = self.position + self.velocity;
        self.presented_position = self.position;
        if self.dead {
            self.removed = true;
            return Ok(true); // Source impact producer executes after BaseUpdate.
        }
        self.falling_speed = (0.9 * self.falling_speed + 0.1) + self.falling_acceleration;
        self.height += self.falling_speed;
        if self.height < -50.0 {
            self.grid_collision_class = 0;
            self.entity_collision_class = 0;
        } else {
            self.grid_collision_class = 4;
            self.entity_collision_class = 4;
        }
        if self.height > -5.0 || (self.grid_contact && self.height >= -50.0) {
            self.dead = true;
        }
        // Common post-update ground query, read by the following Update.
        self.grid_contact =
            self.grid_collision_class == 4 && grid.contacts_regular_tear(self.position);
        if !self.position.is_finite() || !self.height.is_finite() || !self.falling_speed.is_finite()
        {
            return Err("hostile projectile motion overflowed".into());
        }
        Ok(false)
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
            radius: 10.0,
            scale: WorldVector { x: 1.0, y: 1.0 },
            points: 40,
        })
        .expect("authored player geometry")
    });
    for index in 0..state.enemy_projectiles.len() {
        let projectile = &state.enemy_projectiles[index];
        if projectile.dead || projectile.removed || projectile.entity_collision_class == 0 {
            continue;
        }
        let mut player = combat::ContactBody {
            id: 0,
            entity_type: 1,
            position: bombs::player_world(state),
            velocity: bombs::player_velocity(state),
            radius: 10.0,
            mass: 5.0,
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
        combat::try_damage_player(state, 1, 60);
        state.player.x = f64::from(player.position.x) * 0.65 + 32.0;
        state.player.y = f64::from(player.position.y) * 0.65 - 46.0;
        state.player.vx = f64::from(player.velocity.x) * 0.65;
        state.player.vy = f64::from(player.velocity.y) * 0.65;
        let projectile = &mut state.enemy_projectiles[index];
        projectile.dead = true;
        projectile.position = projectile_body.position;
        projectile.presented_position = projectile_body.position;
        projectile.velocity = projectile_body.velocity;
    }
    Ok(())
}
