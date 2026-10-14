//! Source Effect5 blood particles, distinct from Rock Particle4.
//! Factory seeds and immediate updates belong to the real source callers.
use crate::{
    actors::{ActorKind, ActorRef, Scheduler},
    familiars::{self, Parent, Sprite, SpriteEffect},
    grid::{Collider, CollisionGeometry, Grid},
    items,
    motion::WorldVector,
    phase::Phase,
    State,
};
use serde::{Deserialize, Serialize};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };

/// Effect+7d4/7dc/7e0/7e4 and the actual layer0 rotation.
/// Height is the stored7dc value; Entity.render_offset.y is a distinct owner.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BloodParticle {
    /// GameSpawn550ac8 caches the caller's type/variant in Entity+1c/+20.
    /// The browser caller handle is contextual; Entity.parent+498 stays NULL.
    pub spawner: Option<Parent>,
    pub state: i32,
    pub height: f32,
    pub vertical_velocity: f32,
    pub gravity: f32,
    pub layer_rotation: f32,
    pub grid_contact: bool,
    pub grid_normal: WorldVector,
    pub previous_velocity: WorldVector,
}

/// Actual caller arguments, including its post-factory inherited color copy.
pub struct Birth {
    pub subtype: u32,
    pub seed: u32,
    pub position: WorldVector,
    pub velocity: WorldVector,
    pub spawner: Option<Parent>,
    pub tint: [f32; 4],
    pub offsets: [f32; 3],
}

/// Fresh ordinary nonboss445c60 caller. Status/champion/large and the special
/// category58/63 callbacks have separate producers and are not admitted here.
pub struct OrdinaryDeath {
    pub parent: Parent,
    pub position: WorldVector,
    pub radius: f32,
    pub amount: u32,
    pub flags: u32,
    pub tint: [f32; 4],
    pub offsets: [f32; 3],
}

/// Borrow the existing factory owners at a pre-Base callback boundary.
pub struct Admission<'a> {
    pub game_update_count: u32,
    pub next_effect: &'a mut u32,
    pub effects: &'a mut Vec<SpriteEffect>,
    pub scheduler: &'a mut Scheduler,
}

impl Admission<'_> {
    /// Exact ordinary Stage1 category sampling and explicit initializer order.
    /// The caller invokes this inside its actual lethal Base damage callback.
    pub fn ordinary_death(
        &mut self,
        death: OrdinaryDeath,
        shared: &mut crate::mt::Mt19937,
        audio: &mut crate::audio::AudioState,
        system_counter: u32,
    ) -> Result<(), String> {
        let categories = death.flags & (1 | 2 | 4 | 32);
        if death.flags & !0xf != 0
            || (death.amount > 0 && categories == 0)
            || !death.radius.is_finite()
            || death.radius < 0.0
        {
            return Err("ordinary gibs require actual supported category flags and radius".into());
        }
        //44641d: the positive authored amount selects volume1; a zero-gib
        //caller selects .8. Audio admission remains the sole SFX sampler.
        audio.admit(
            shared,
            system_counter,
            crate::audio::SoundRequest {
                id: 30,
                volume: if death.amount > 0 { 1.0 } else { 0.8 },
                delay: 2,
                looping: false,
                pitch: 1.0,
            },
        );
        let mut counts = [0; 4];
        if death.amount > 0 {
            let budget = death
                .amount
                .checked_add(shared.bounded(2))
                .ok_or("gib budget exhausted")?
                / categories.count_ones()
                + 1;
            //445d39..446418 samples every category count BEFORE any birth.
            if death.flags & 1 != 0 {
                counts[0] = shared.bounded(budget) + shared.bounded(budget);
            }
            if death.flags & 2 != 0 {
                counts[1] = budget / 2 + shared.bounded(budget / 2);
            }
            if death.flags & 4 != 0 {
                counts[2] = shared.bounded(budget);
            }
            if death.flags & 8 != 0 {
                counts[3] = shared.bounded(budget.min(3));
            }
        }
        for (subtype, count) in counts.into_iter().enumerate() {
            for _ in 0..count {
                //445f40->445da8->445dcd->445e22: speed, raw seed,
                //elliptical velocity direction, then independent birth offset.
                let speed = shared.uniform() * 4.0 + 2.0;
                let seed = shared.next_u32();
                let direction = crate::npcs::direction(shared);
                let velocity = WorldVector {
                    x: direction.x,
                    y: direction.y * 0.5,
                } * speed;
                let position =
                    death.position + (crate::npcs::direction(shared) * 0.5) * death.radius;
                let id = self.admit(Birth {
                    subtype: subtype as u32,
                    seed,
                    position,
                    velocity,
                    spawner: Some(death.parent),
                    tint: death.tint,
                    offsets: death.offsets,
                })?;
                //445ed5 copied inherited color before445f32 immediateUpdate.
                self.advance_initial(id, shared)?;
            }
        }
        //44622e/446241: final Blood7 has the actual NPC ground position,
        //zero velocity and NULL parent, then its separate immediate Update.
        let seed = shared.next_u32();
        let mut blood = familiars::BloodAdmission {
            game_update_count: self.game_update_count,
            next_effect: self.next_effect,
            effects: self.effects,
            scheduler: self.scheduler,
        };
        let id = blood.admit(seed, death.position, death.tint, death.offsets)?;
        blood.advance_initial(id, shared)
    }

    /// The real445f32 caller invokes this after inherited color copy.
    /// Other caller sites that omit immediate Update must omit this call.
    pub fn advance_initial(
        &mut self,
        id: u32,
        shared: &mut crate::mt::Mt19937,
    ) -> Result<(), String> {
        let effect = self
            .effects
            .iter_mut()
            .find(|effect| effect.id == id)
            .ok_or("initial blood particle absent")?;
        if effect.variant != 5
            || effect.full_updates != 0
            || effect
                .blood_particle
                .as_ref()
                .is_none_or(|particle| particle.state != 0)
        {
            return Err("blood particle explicit first Update must execute once".into());
        }
        advance_base(effect)?;
        initialize(effect, shared)?;
        finish_update(effect)
    }

    pub fn admit(&mut self, birth: Birth) -> Result<u32, String> {
        if birth.subtype > 3
            || !birth.position.is_finite()
            || !birth.velocity.is_finite()
            || !birth
                .tint
                .iter()
                .chain(&birth.offsets)
                .all(|v| v.is_finite())
        {
            return Err("blood particle requires source subtype0..3 and finite fields".into());
        }
        let config =
            familiars::definition(1000, 5).ok_or("authored Blood Particle config absent")?;
        let id = *self.next_effect;
        *self.next_effect = id.checked_add(1).ok_or("effect handle exhausted")?;
        self.effects.push(SpriteEffect {
            id,
            variant: 5,
            subtype: birth.subtype,
            spawn_seed: birth.seed,
            parent: None,
            birth_game_update: self.game_update_count,
            actor_rng: items::Rng::new(birth.seed, [4, 3, 17]),
            position: birth.position,
            presented_position: birth.position,
            render_offset: ZERO,
            velocity: birth.velocity,
            configured_friction: config.friction,
            current_friction: config.friction,
            collision_radius: config.collision_radius,
            collision_mass: config.collision_mass,
            collision_damage: 22.0,
            hp: config.base_hp,
            max_hp: config.base_hp,
            grid_collision_class: 3,
            entity_collision_class: 0,
            flags: 0,
            visible: true,
            active: true,
            removed: false,
            render_order_offset: 0,
            lifetime_divisor: 0,
            time_to_live: -1,
            raw_7f4: 1,
            raw_7f8: 0.0,
            raw_7d8: false,
            sprite: Sprite::new(config),
            full_updates: 0,
            blood_particle: Some(BloodParticle {
                spawner: birth.spawner,
                state: 0,
                height: 0.0,
                vertical_velocity: 0.0,
                gravity: 0.0,
                layer_rotation: 0.0,
                grid_contact: false,
                grid_normal: ZERO,
                previous_velocity: ZERO,
            }),
        });
        self.scheduler.admit(ActorRef {
            kind: ActorKind::Effect,
            id,
        })?;
        //445ed5 copies the parent's inherited color after GameSpawn/Insert.
        let effect = self.effects.last_mut().expect("new blood particle");
        effect.sprite.tint = birth.tint;
        effect.sprite.offsets = birth.offsets;
        Ok(id)
    }
}

pub fn admit(state: &mut State, birth: Birth) -> Result<u32, String> {
    Admission {
        game_update_count: state.game_update_count,
        next_effect: &mut state.next_effect,
        effects: &mut state.sprite_effects,
        scheduler: &mut state.actor_scheduler,
    }
    .admit(birth)
}

fn collider(effect: &SpriteEffect) -> Result<Collider, String> {
    let config = familiars::definition(1000, 5).ok_or("Blood Particle geometry absent")?;
    Collider::new(CollisionGeometry {
        radius: effect.collision_radius,
        scale: WorldVector { x: 1.0, y: 1.0 },
        points: config.grid_points,
    })
}

/// Full45aa70 begins with Base; first5-specific pose and later child callbacks
/// consume the real shared stream in their source order.
pub fn advance(state: &mut State, phase: Phase, id: u32) -> Result<(), String> {
    let Some(index) = state
        .sprite_effects
        .iter()
        .position(|effect| effect.id == id)
    else {
        return Ok(());
    };
    if state.sprite_effects[index].removed
        || !state.sprite_effects[index].active
        || phase == Phase::Skipped
    {
        return Ok(());
    }
    if phase == Phase::Interpolate {
        let grid = Grid::new(
            state
                .room
                .grid
                .clone()
                .ok_or("particle preview requires actual room grid")?,
        )?;
        let effect = &mut state.sprite_effects[index];
        let mut point = grid.clamp_room(
            effect.position + (effect.velocity * effect.current_friction) * 0.5,
            0.0,
        );
        let mut velocity = effect.velocity;
        collider(effect)?.resolve_nonplayer_walls(&grid, &mut point, &mut velocity, true)?;
        effect.presented_position = point;
        return Ok(());
    }
    // Hold this actor's owned record while its callbacks append real births.
    // Reinsert it at the same vector position; scheduling uses the existing
    // source-order ActorRefs, including the manager's old-tail birth fence.
    let mut effect = state.sprite_effects.remove(index);
    let result = update(state, &mut effect);
    state.sprite_effects.insert(index, effect);
    result
}

fn advance_base(effect: &mut SpriteEffect) -> Result<(), String> {
    let config = familiars::definition(1000, 5).ok_or("Blood Particle animation absent")?;
    effect.full_updates = effect
        .full_updates
        .checked_add(1)
        .ok_or("particle callback count exhausted")?;
    effect.sprite.advance(config)?;
    effect.velocity = effect.velocity * effect.current_friction;
    effect.position = effect.position + effect.velocity;
    effect.presented_position = effect.position;
    effect.current_friction = effect.configured_friction;
    Ok(())
}

fn initialize(effect: &mut SpriteEffect, shared: &mut crate::mt::Mt19937) -> Result<(), String> {
    let source_parent = effect
        .blood_particle
        .as_ref()
        .and_then(|particle| particle.spawner.as_ref())
        .map(|parent| (parent.entity_type, parent.variant));
    let particle = effect
        .blood_particle
        .as_mut()
        .ok_or("particle controller owner absent")?;
    particle.gravity = 1.3;
    particle.height = -23.0;
    if source_parent == Some((1000, 5)) {
        //464e1d: child particles use a stopped authored13-frame pose.
        effect.sprite.animation = "Particle".into();
        effect.sprite.frame = shared.bounded(13) as f32;
        effect.sprite.playing = false;
        particle.vertical_velocity = -(shared.uniform() * 20.0) + 5.0;
    } else {
        particle.layer_rotation = shared.uniform() * 360.0;
        let choices: &[&str] = match effect.subtype {
            0 => &["BloodGib01", "BloodGib02", "BloodGib03"],
            1 => &["Bone02", "Bone01"],
            2 => &["Guts01", "Guts02", "Liver"],
            3 => &["Eye"],
            _ => return Err("unadmitted blood particle subtype".into()),
        };
        let choice = if choices.len() == 1 {
            0
        } else {
            shared.bounded(choices.len() as u32) as usize
        };
        //40fd80 Play(force=false) preserves the current nonloop clip.
        //Choosing the default BloodGib01 keeps Base's already advanced frame.
        if effect.sprite.animation != choices[choice] {
            effect.sprite.animation = choices[choice].into();
            effect.sprite.frame = 0.0;
            effect.sprite.playing = true;
        }
        particle.vertical_velocity = -(shared.uniform() * 30.0) + 5.0;
    }
    particle.state = 1;
    Ok(())
}

fn update(state: &mut State, effect: &mut SpriteEffect) -> Result<(), String> {
    advance_base(effect)?;
    if effect
        .blood_particle
        .as_ref()
        .is_some_and(|particle| particle.state == 0)
    {
        initialize(effect, &mut state.boot_lifecycle.shared)?;
    } else {
        let config = familiars::definition(1000, 5).ok_or("Blood Particle animation absent")?;
        let source_parent = effect
            .blood_particle
            .as_ref()
            .and_then(|particle| particle.spawner.as_ref())
            .map(|parent| (parent.entity_type, parent.variant));
        let particle = effect
            .blood_particle
            .as_mut()
            .ok_or("particle controller owner absent")?;
        particle.vertical_velocity = (0.9 * particle.vertical_velocity + 0.1) + particle.gravity;
        particle.height += particle.vertical_velocity;
        effect.render_offset.y = particle.height;
        if particle.height > -5.0 {
            if source_parent.is_some_and(|parent| parent.0 == 1000) {
                //464024 only retires source frames4..10 directly.
                if (4..=10).contains(&(effect.sprite.frame.floor() as u32)) {
                    effect.active = false;
                    effect.removed = true;
                } else {
                    effect.flags |= 8;
                    effect.raw_7d8 = true;
                }
            } else {
                let clip = config
                    .clips
                    .iter()
                    .find(|clip| clip.name == effect.sprite.animation)
                    .ok_or("particle landing clip absent")?;
                effect.sprite.frame = clip.frames as f32 - 1.0;
                effect.sprite.playing = false;
                effect.velocity = ZERO;
                effect.raw_7d8 = true;
                effect.render_order_offset = 90;
                effect.flags |= 8;
            }
        } else if !source_parent.is_some_and(|parent| parent.0 == 1000) {
            //463406 executes every airborne ordinary-parent full callback.
            if state.boot_lifecycle.shared.bounded(3) == 0 {
                let seed = state.boot_lifecycle.shared.next_u32();
                let blood = familiars::admit_blood_splat(
                    state,
                    seed,
                    effect.position,
                    effect.sprite.tint,
                    effect.sprite.offsets,
                )?;
                let scale = state.boot_lifecycle.shared.uniform() * 0.3 + 0.1;
                state
                    .sprite_effects
                    .iter_mut()
                    .find(|item| item.id == blood)
                    .expect("new particle splat")
                    .sprite
                    .scale = WorldVector { x: scale, y: scale };
                familiars::advance_blood_initial(state, blood)?;
            }
            if (2..=4).contains(&particle.state) {
                let mut emitted = 0;
                //The source reevaluates bounded(state) at4634e4 after EACH
                //birth; a single sampled count would change actual draws.
                while emitted < state.boot_lifecycle.shared.bounded(particle.state as u32) {
                    let seed = state.boot_lifecycle.shared.next_u32();
                    let velocity = crate::npcs::direction(&mut state.boot_lifecycle.shared) * 5.0;
                    admit(
                        state,
                        Birth {
                            subtype: 0,
                            seed,
                            position: effect.position,
                            velocity,
                            spawner: Some(Parent {
                                actor: ActorRef {
                                    kind: ActorKind::Effect,
                                    id: effect.id,
                                },
                                entity_type: 1000,
                                variant: 5,
                            }),
                            tint: effect.sprite.tint,
                            offsets: effect.sprite.offsets,
                        },
                    )?;
                    emitted += 1;
                }
            }
            particle.state = particle
                .state
                .checked_add(1)
                .ok_or("particle state counter exhausted")?;
        }
    }
    finish_update(effect)
}

fn finish_update(effect: &mut SpriteEffect) -> Result<(), String> {
    if effect.time_to_live > 0 {
        effect.time_to_live -= 1;
        if effect.time_to_live == 0 {
            effect.removed = true;
        }
    }
    let particle = effect
        .blood_particle
        .as_ref()
        .ok_or("particle controller owner absent")?;
    if !effect.position.is_finite()
        || !effect.velocity.is_finite()
        || !particle.height.is_finite()
        || !particle.vertical_velocity.is_finite()
    {
        return Err("particle motion overflowed".into());
    }
    Ok(())
}

/// Manager's full class3 ground pass follows actor/background callbacks.
pub fn full_grid_contacts(state: &mut State) -> Result<(), String> {
    if !state.sprite_effects.iter().any(|effect| {
        effect.variant == 5 && effect.active && !effect.removed && effect.flags & 8 == 0
    }) {
        return Ok(());
    }
    let grid = Grid::new(
        state
            .room
            .grid
            .clone()
            .ok_or("particle ground pass requires room grid")?,
    )?;
    for effect in &mut state.sprite_effects {
        if effect.variant != 5 || !effect.active || effect.removed || effect.flags & 8 != 0 {
            continue;
        }
        let geometry = collider(effect)?;
        effect.position = grid.clamp_room(effect.position, 0.0);
        // RoomLoad derives the same exact rectangle used by clamp_room.
        let max_x = 60.0 + f32::from(grid.descriptor().width - 2) * 40.0;
        let max_y = 140.0 + f32::from(grid.descriptor().height - 2) * 40.0;
        let boundary = effect.position.x == 60.0
            || effect.position.x == max_x
            || effect.position.y == 140.0
            || effect.position.y == max_y;
        let contact = geometry.resolve_nonplayer_walls(
            &grid,
            &mut effect.position,
            &mut effect.velocity,
            false,
        )?;
        let particle = effect
            .blood_particle
            .as_mut()
            .ok_or("particle grid owner absent")?;
        particle.grid_contact = boundary || contact.is_some_and(|hit| hit.velocity_changed);
        particle.grid_normal = contact.map_or(ZERO, |hit| hit.normal);
        if let Some(hit) = contact.filter(|hit| hit.velocity_changed) {
            particle.previous_velocity = hit.previous_velocity;
        }
        effect.presented_position = effect.position;
    }
    Ok(())
}
