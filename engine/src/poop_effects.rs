//! Actual Poop Particle58, Wood Particle27, Blood Particle5 and Poop
//! Explosion43 effect controllers. Caller owns gib/retirement producers and
//! the manager's flags8 paint pass.
use crate::{
    actors::{ActorKind, ActorRef, Scheduler},
    audio::{AudioState, SoundRequest},
    familiars::{self, Parent, Sprite, SpriteEffect},
    grid::{Collider, CollisionGeometry, Grid},
    items,
    motion::WorldVector,
    mt::Mt19937,
};
use serde::{Deserialize, Serialize};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Flight {
    pub height: f32,
    pub vertical_velocity: f32,
    pub gravity: f32,
    pub layer_rotation: f32,
}

/// Init writes state7d4=0; first real58 Update owns the flight-field writes.
/// Unused pre-update pooled7dc/7e0/7e4 bytes are not invented initialized zeros.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum Particle {
    Initializing,
    Flying(Flight),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum Controller {
    Particle58(Particle),
    /// Effect update 45dc20, the rock-debris family (4/27/35) as Wood27.
    Particle27(Particle),
    /// Effect update 45d9b8 (Blood Particle 1000.5, Entity::Die's blood, bone,
    /// gut and eye gibs and their airborne children).
    Particle5(BloodParticle),
    /// Actual7d4: Init0, then the real sampled background frame6..8.
    Explosion43 {
        #[serde(rename = "frameTarget")]
        frame_target: i32,
    },
}

impl Controller {
    pub fn source_state(&self) -> i32 {
        match self {
            Self::Particle58(Particle::Initializing) | Self::Particle27(Particle::Initializing) => {
                0
            }
            Self::Particle58(Particle::Flying(_)) | Self::Particle27(Particle::Flying(_)) => 1,
            Self::Particle5(blood) => blood.stage,
            Self::Explosion43 { frame_target } => *frame_target,
        }
    }

    pub fn layer_rotation(&self) -> Option<f32> {
        match self {
            Self::Particle58(Particle::Flying(flight)) => Some(flight.layer_rotation),
            Self::Particle5(BloodParticle {
                particle: Particle::Flying(flight),
                ..
            }) => Some(flight.layer_rotation),
            _ => None,
        }
    }
}

/// Blood Particle state +7d4 is `stage`: 0 before the first Update, then one
/// more per airborne Update. A particle spawned by another effect (Entity+1c
/// is 1000) takes the child branches.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BloodParticle {
    pub particle: Particle,
    pub subtype: u32,
    pub stage: i32,
}

/// What an airborne ordinary Blood Particle spawns after its own Update.
struct AirborneSpawns {
    position: WorldVector,
    stage: i32,
    spawner: Parent,
}

/// Real Game.Spawn arguments. Cached spawner metadata never links parent498.
pub struct ParticleBirth {
    pub seed: u32,
    pub position: WorldVector,
    pub velocity: WorldVector,
    pub spawner: Option<Parent>,
}

pub struct Admission<'a> {
    pub game_update_count: u32,
    pub next_effect: &'a mut u32,
    pub effects: &'a mut Vec<SpriteEffect>,
    pub scheduler: &'a mut Scheduler,
}

impl Admission<'_> {
    /// 4461db real58 factory. The caller separately invokes445f32 Update.
    /// Category5 bypasses inherited color copy, preserving factory white.
    pub fn admit_particle(&mut self, birth: ParticleBirth) -> Result<u32, String> {
        self.admit(58, Controller::Particle58(Particle::Initializing), birth)
    }

    /// Die 445c60 and the airborne 4634b5 spawn 1000.5 with the gib category
    /// as subtype; the caller separately invokes Update where the source does.
    pub fn admit_blood_particle(
        &mut self,
        birth: ParticleBirth,
        subtype: u32,
    ) -> Result<u32, String> {
        let id = self.admit(
            5,
            Controller::Particle5(BloodParticle {
                particle: Particle::Initializing,
                subtype,
                stage: 0,
            }),
            birth,
        )?;
        self.effects
            .last_mut()
            .expect("admitted blood particle")
            .subtype = subtype;
        Ok(id)
    }

    /// Wood Particle27 through the same Effect::Init 458a70 grid class3.
    pub fn admit_wood_particle(&mut self, birth: ParticleBirth) -> Result<u32, String> {
        self.admit(27, Controller::Particle27(Particle::Initializing), birth)
    }

    /// Tooth Particle35, the same rock-debris controller as Wood27.
    pub fn admit_tooth_particle(&mut self, birth: ParticleBirth) -> Result<u32, String> {
        self.admit(35, Controller::Particle27(Particle::Initializing), birth)
    }

    /// 4fc0f9 real43 factory has V0/spawnerNULL and NO immediate Update.
    pub fn admit_explosion(&mut self, seed: u32, position: WorldVector) -> Result<u32, String> {
        self.admit(
            43,
            Controller::Explosion43 { frame_target: 0 },
            ParticleBirth {
                seed,
                position,
                velocity: ZERO,
                spawner: None,
            },
        )
    }

    fn admit(
        &mut self,
        variant: u32,
        controller: Controller,
        birth: ParticleBirth,
    ) -> Result<u32, String> {
        if !birth.position.is_finite() || !birth.velocity.is_finite() {
            return Err("poop effect factory requires finite source world vectors".into());
        }
        let config = familiars::definition(1000, variant)
            .ok_or("actual Poop/Wood Particle or Poop Explosion catalogue absent")?;
        let id = *self.next_effect;
        *self.next_effect = id.checked_add(1).ok_or("effect handle exhausted")?;
        self.effects.push(SpriteEffect {
            id,
            variant,
            subtype: 0,
            spawn_seed: birth.seed,
            spawner_type: birth.spawner.map_or(0, |caller| caller.entity_type),
            spawner_variant: birth.spawner.map_or(0, |caller| caller.variant),
            parent: None,
            birth_game_update: self.game_update_count,
            // Both Base seed30 writes overwrite these words without Next.
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
            grid_collision_class: if matches!(variant, 5 | 27 | 35 | 58) { 3 } else { 0 },
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
            poop_effect: Some(controller),
            blood_effect: None,
            footprint: false,
            critter: None,
            creep_effect: None,
            fly_effect: None,
            epic_fetus: None,
            explosion_effect: None,
        });
        // Real Room sequence and manager append precede any caller Update.
        self.scheduler.admit(ActorRef {
            kind: ActorKind::Effect,
            id,
        })?;
        Ok(id)
    }

    /// Exact445f32 caller; first58 controller finishes before the next birth.
    pub fn advance_particle_initial(
        &mut self,
        id: u32,
        shared: &mut Mt19937,
    ) -> Result<(), String> {
        let effect = self
            .effects
            .iter_mut()
            .find(|effect| effect.id == id)
            .ok_or("explicit initial58 Update requires an admitted effect")?;
        if !matches!(effect.variant, 5 | 27 | 35 | 58) || effect.full_updates != 0 {
            return Err(
                "explicit initial particle Update must run once on its actual birth".into(),
            );
        }
        base_update(effect)?;
        if effect.variant == 5 {
            // The first Update only picks the clip and launch; it spawns nothing.
            blood_update(effect, shared)?;
        } else {
            particle_update(effect, shared)?;
        }
        finish_update(effect)
    }
}

/// A full Update of Blood Particle 1000.5 with its airborne spawns: on a 1/3
/// roll a small BloodSplat 1000.7 (seed, scale Float*0.3+0.1, immediate
/// Update), then on stages 2..4 children while a fresh Random(stage) exceeds
/// their count (seed, Angle*5 velocity, no immediate Update).
pub(crate) fn advance_blood_particle(state: &mut crate::State, id: u32) -> Result<(), String> {
    let effect = state
        .sprite_effects
        .iter_mut()
        .find(|effect| effect.id == id)
        .ok_or("Blood Particle Update requires an admitted effect")?;
    if !effect.active || effect.removed {
        return Ok(());
    }
    base_update(effect)?;
    let spawns = blood_update(effect, &mut state.boot_lifecycle.shared)?;
    finish_update(effect)?;
    let Some(spawns) = spawns else {
        return Ok(());
    };
    let shared = &mut state.boot_lifecycle.shared;
    if shared.bounded(3) == 0 {
        let seed = state.boot_lifecycle.shared.next_u32();
        let splat = familiars::admit_blood_splat(state, seed, spawns.position, [1.0; 4], [0.0; 3])?;
        let scale = state.boot_lifecycle.shared.uniform() * 0.3 + 0.1;
        familiars::set_blood_scale(state, splat, scale)?;
        familiars::advance_blood_initial(state, splat)?;
    }
    if (2..=4).contains(&spawns.stage) {
        let mut count = 0;
        while count < state.boot_lifecycle.shared.bounded(spawns.stage as u32) {
            let shared = &mut state.boot_lifecycle.shared;
            let seed = shared.next_u32();
            let velocity = crate::npcs::direction(shared) * 5.0;
            Admission {
                game_update_count: state.game_update_count,
                next_effect: &mut state.next_effect,
                effects: &mut state.sprite_effects,
                scheduler: &mut state.actor_scheduler,
            }
            .admit_blood_particle(
                ParticleBirth {
                    seed,
                    position: spawns.position,
                    velocity,
                    spawner: Some(spawns.spawner),
                },
                0,
            )?;
            count += 1;
        }
    }
    Ok(())
}

/// The 45d9b8 controller. Returns the spawn request of an airborne ordinary
/// particle (its stage before the increment).
fn blood_update(
    effect: &mut SpriteEffect,
    shared: &mut Mt19937,
) -> Result<Option<AirborneSpawns>, String> {
    let child = effect.spawner_type == 1000;
    let position = effect.position;
    let spawner = Parent {
        actor: ActorRef {
            kind: ActorKind::Effect,
            id: effect.id,
        },
        entity_type: 1000,
        variant: 5,
    };
    let Some(Controller::Particle5(blood)) = &mut effect.poop_effect else {
        return Err("Blood Particle Update requires its typed controller owner".into());
    };
    match &mut blood.particle {
        Particle::Initializing => {
            let (rotation, vertical_velocity) = if child && effect.spawner_variant == 5 {
                // 464e1d: a child stops on a Random(13) frame of "Particle".
                effect.sprite.animation = "Particle".into();
                effect.sprite.frame = shared.bounded(13) as f32;
                effect.sprite.playing = false;
                (0.0, (-shared.uniform()) * 20.0_f32 + 5.0_f32)
            } else {
                let rotation = shared.uniform() * 360.0_f32;
                let clip = match blood.subtype {
                    0 => ["BloodGib01", "BloodGib02", "BloodGib03"][shared.bounded(3) as usize],
                    1 => ["Bone02", "Bone01"][shared.bounded(2) as usize],
                    2 => ["Guts01", "Guts02", "Liver"][shared.bounded(3) as usize],
                    3 => "Eye",
                    5 => return Err("Blood Particle subtype5 Gib clips are not authored".into()),
                    _ => "BloodGib01",
                };
                // Play(force=false), as for Particle58.
                if effect.sprite.animation != clip {
                    effect.sprite.animation = clip.into();
                    effect.sprite.frame = 0.0;
                    effect.sprite.playing = true;
                }
                (rotation, (-shared.uniform()) * 30.0_f32 + 5.0_f32)
            };
            blood.particle = Particle::Flying(Flight {
                height: -23.0,
                vertical_velocity,
                gravity: 1.3,
                layer_rotation: rotation,
            });
            blood.stage = 1;
            Ok(None)
        }
        Particle::Flying(flight) => {
            flight.vertical_velocity =
                (0.9_f32 * flight.vertical_velocity + 0.1_f32) + flight.gravity;
            flight.height += flight.vertical_velocity;
            effect.render_offset.y = flight.height;
            if flight.height > -5.0 {
                if child {
                    // 464024: frames 4..10 are removed, the rest are painted.
                    if (4..=10).contains(&(effect.sprite.frame as i32)) {
                        effect.active = false;
                        effect.removed = true;
                    } else {
                        effect.raw_7d8 = true;
                        effect.flags |= 8;
                    }
                } else {
                    let config =
                        familiars::definition(1000, 5).ok_or("actual5 landing catalogue absent")?;
                    let clip = config
                        .clips
                        .iter()
                        .find(|clip| clip.name == effect.sprite.animation)
                        .ok_or("5 landing clip absent from authored catalogue")?;
                    effect.sprite.frame = clip.frames as f32 - 1.0;
                    effect.velocity = ZERO;
                    effect.raw_7d8 = true;
                    effect.render_order_offset = 90;
                    effect.flags |= 8;
                }
                return Ok(None);
            }
            if child {
                return Ok(None);
            }
            let stage = blood.stage;
            blood.stage += 1;
            Ok(Some(AirborneSpawns {
                position,
                stage,
                spawner,
            }))
        }
    }
}

/// Game::SpawnParticles 5515b0 (default ColorMod) for the admitted particle
/// variants: per particle its seed, angle and immediate Update. The caller
/// owns the speed argument.
pub(crate) fn spawn_particles(
    state: &mut crate::State,
    variant: u32,
    centre: WorldVector,
    count: u32,
    speed: f32,
) -> Result<(), String> {
    for _ in 0..count {
        let seed = state.boot_lifecycle.shared.next_u32();
        let direction = crate::npcs::direction(&mut state.boot_lifecycle.shared);
        let velocity = WorldVector {
            x: direction.x,
            y: direction.y * 0.5,
        } * speed;
        let mut admission = Admission {
            game_update_count: state.game_update_count,
            next_effect: &mut state.next_effect,
            effects: &mut state.sprite_effects,
            scheduler: &mut state.actor_scheduler,
        };
        let birth = ParticleBirth {
            seed,
            position: centre,
            velocity,
            spawner: None,
        };
        let id = match variant {
            58 => admission.admit_particle(birth)?,
            27 => admission.admit_wood_particle(birth)?,
            35 => admission.admit_tooth_particle(birth)?,
            _ => {
                return Err(format!(
                    "SpawnParticles Effect 1000.{variant} is not admitted"
                ))
            }
        };
        admission.advance_particle_initial(id, &mut state.boot_lifecycle.shared)?;
    }
    Ok(())
}

fn base_update(effect: &mut SpriteEffect) -> Result<(), String> {
    let config = familiars::definition(1000, effect.variant)
        .ok_or("poop effect animation catalogue absent")?;
    effect.full_updates = effect
        .full_updates
        .checked_add(1)
        .ok_or("effect callback count exhausted")?;
    effect.sprite.advance(config)?;
    effect.velocity = effect.velocity * effect.current_friction;
    effect.position = effect.position + effect.velocity;
    effect.presented_position = effect.position;
    effect.current_friction = effect.configured_friction;
    Ok(())
}

fn particle_update(effect: &mut SpriteEffect, shared: &mut Mt19937) -> Result<(), String> {
    if let Some(Controller::Particle27(_)) = &effect.poop_effect {
        return wood_update(effect, shared);
    }
    let Some(Controller::Particle58(particle)) = &mut effect.poop_effect else {
        return Err("58 Update requires its typed controller owner".into());
    };
    match particle {
        Particle::Initializing => {
            // Base has already advanced default Gib01 to frame1.
            let rotation = shared.uniform() * 360.0_f32;
            let clip = ["Gib01", "Gib02", "Gib03", "Gib04", "Gib05"][shared.bounded(5) as usize];
            // Play(force=false) preserves the same already-playing nonloop
            // Gib01 pose. A different authored clip resets to frame0.
            if effect.sprite.animation != clip {
                effect.sprite.animation = clip.into();
                effect.sprite.frame = 0.0;
                effect.sprite.playing = true;
            }
            let vertical_velocity = (-shared.uniform()) * 30.0_f32 + 5.0_f32;
            *particle = Particle::Flying(Flight {
                height: -23.0,
                vertical_velocity,
                gravity: 1.3,
                layer_rotation: rotation,
            });
            // Source first controller leaves Entity.renderOffset.y at0.
        }
        Particle::Flying(flight) => {
            flight.vertical_velocity =
                (0.9_f32 * flight.vertical_velocity + 0.1_f32) + flight.gravity;
            flight.height += flight.vertical_velocity;
            effect.render_offset.y = flight.height;
            if flight.height > -5.0 {
                let config =
                    familiars::definition(1000, 58).ok_or("actual58 landing catalogue absent")?;
                let clip = config
                    .clips
                    .iter()
                    .find(|clip| clip.name == effect.sprite.animation)
                    .ok_or("58 landing clip absent from authored catalogue")?;
                // 40ff50 seeks last pose; preserve actual incoming playing.
                effect.sprite.frame = clip.frames as f32 - 1.0;
                effect.velocity = ZERO;
                effect.raw_7d8 = true;
                effect.render_order_offset = 90;
                effect.flags |= 8;
            }
        }
    }
    Ok(())
}

/// 45dc20 for Wood Particle27. The first controller (460839, 464c5d) sets
/// gravity 1.3 and height -23, draws its rising speed -5 - Float*10, then
/// SetFrame40fef0 stops it on frame 0 of a Random(4) Gib1..Gib4 clip. Later
/// controllers fall like Particle58; once height passes -5 and the current
/// clip IsFinished (always, as SetFrame stopped it) 461f06 parks it for the
/// flags8 paint pass without seeking a frame.
fn wood_update(effect: &mut SpriteEffect, shared: &mut Mt19937) -> Result<(), String> {
    let Some(Controller::Particle27(particle)) = &mut effect.poop_effect else {
        return Err("27 Update requires its typed controller owner".into());
    };
    match particle {
        Particle::Initializing => {
            let vertical_velocity = -5.0_f32 - shared.uniform() * 10.0_f32;
            if effect.variant == 35 {
                // 464efd: ToothGib01..04; one in three stops on frame 0, the
                // rest Play(force = false), keeping a playing ToothGib01.
                let clip = ["ToothGib01", "ToothGib02", "ToothGib03", "ToothGib04"]
                    [shared.bounded(4) as usize];
                let stop = shared.bounded(3) == 0;
                if stop || effect.sprite.animation != clip {
                    effect.sprite.animation = clip.into();
                    effect.sprite.frame = 0.0;
                    effect.sprite.playing = !stop;
                }
            } else {
                let clip = ["Gib1", "Gib2", "Gib3", "Gib4"][shared.bounded(4) as usize];
                effect.sprite.animation = clip.into();
                effect.sprite.frame = 0.0;
                effect.sprite.playing = false;
            }
            *particle = Particle::Flying(Flight {
                height: -23.0,
                vertical_velocity,
                gravity: 1.3,
                layer_rotation: 0.0,
            });
        }
        Particle::Flying(flight) => {
            // 4638b5: a tooth is removed once its clip is finished, then
            // falls on as usual.
            if effect.variant == 35 && !effect.sprite.playing {
                effect.removed = true;
            }
            flight.vertical_velocity =
                (0.9_f32 * flight.vertical_velocity + 0.1_f32) + flight.gravity;
            flight.height += flight.vertical_velocity;
            effect.render_offset.y = flight.height;
            if flight.height > -5.0 && !effect.sprite.playing {
                effect.velocity = ZERO;
                effect.raw_7d8 = true;
                effect.render_order_offset = 90;
                effect.flags |= 8;
            }
        }
    }
    Ok(())
}

fn explosion_update(
    effect: &mut SpriteEffect,
    shared: &mut Mt19937,
    audio: &mut AudioState,
    system_counter: u32,
) -> Result<(), String> {
    let Some(Controller::Explosion43 { frame_target }) = &mut effect.poop_effect else {
        return Err("43 Update requires its typed controller owner".into());
    };
    let first = *frame_target == 0;
    if first {
        // 4609e1: exact separate B2/B2/B3 before current-frame handling.
        effect.sprite.flip_x = shared.bounded(2) == 0;
        effect.sprite.flip_y = shared.bounded(2) == 0;
        *frame_target = shared.bounded(3) as i32 + 6;
    }
    let frame = effect.sprite.frame.floor() as i32;
    // First controller jumps45ac80 directly; later43 dispatch checks frame4.
    if !first && frame == 4 {
        audio.admit(
            shared,
            system_counter,
            SoundRequest {
                id: 237,
                volume: 0.9,
                delay: 2,
                looping: false,
                pitch: 1.0,
            },
        );
    }
    if frame == *frame_target {
        // Every native flags8 site also sets +7d8 (sequence render key).
        effect.raw_7d8 = true;
        effect.flags |= 8;
    }
    if !effect.sprite.playing {
        effect.active = false;
        effect.removed = true;
    }
    Ok(())
}

fn finish_update(effect: &mut SpriteEffect) -> Result<(), String> {
    if effect.time_to_live > 0 {
        effect.time_to_live -= 1;
        if effect.time_to_live == 0 {
            effect.removed = true;
        }
    }
    let finite_flight = match &effect.poop_effect {
        Some(
            Controller::Particle58(Particle::Flying(flight))
            | Controller::Particle27(Particle::Flying(flight))
            | Controller::Particle5(BloodParticle {
                particle: Particle::Flying(flight),
                ..
            }),
        ) => flight.height.is_finite() && flight.vertical_velocity.is_finite(),
        _ => true,
    };
    if !effect.position.is_finite() || !effect.velocity.is_finite() || !finite_flight {
        return Err("poop effect source motion overflowed".into());
    }
    Ok(())
}

/// Caller invokes only on the actual full actor callback; no new manager loop.
pub fn advance_full(
    effect: &mut SpriteEffect,
    shared: &mut Mt19937,
    audio: &mut AudioState,
    system_counter: u32,
) -> Result<(), String> {
    if !effect.active || effect.removed {
        return Ok(());
    }
    base_update(effect)?;
    match effect.variant {
        27 | 35 | 58 => particle_update(effect, shared)?,
        5 => return Err("Blood Particle Update requires its state-level spawn owner".into()),
        43 => explosion_update(effect, shared, audio, system_counter)?,
        _ => return Err("poop effect dispatch requires actual27/35/58/43 variant".into()),
    }
    finish_update(effect)
}

fn particle_collider(effect: &SpriteEffect) -> Result<Collider, String> {
    let config = familiars::definition(1000, effect.variant)
        .ok_or("actual particle radius/grid-point geometry absent")?;
    Collider::new(CollisionGeometry {
        radius: effect.collision_radius,
        scale: WorldVector { x: 1.0, y: 1.0 },
        points: config.grid_points,
    })
}

/// Effect4589f0→4446a0, no vertical/controller/sprite/local/shared draws.
pub fn advance_fast(effect: &mut SpriteEffect, grid: &Grid) -> Result<(), String> {
    if !effect.active || effect.removed {
        return Ok(());
    }
    if effect.flags & 2 != 0 {
        effect.flags &= !2;
        return Ok(());
    }
    let step = WorldVector {
        x: (effect.velocity.x * effect.current_friction) / 2.0,
        y: (effect.velocity.y * effect.current_friction) / 2.0,
    };
    let mut point = effect.position + step;
    if effect.grid_collision_class == 3 {
        let mut velocity = effect.velocity;
        particle_collider(effect)?.resolve_nonplayer_walls(
            grid,
            &mut point,
            &mut velocity,
            true,
        )?;
    }
    effect.presented_position = point;
    Ok(())
}

/// Actual late full grid callback, after the manager's flags8 paint pass.
pub fn full_grid_contact(effect: &mut SpriteEffect, grid: &Grid) -> Result<(), String> {
    if !matches!(effect.variant, 5 | 27 | 35 | 58)
        || !effect.active
        || effect.removed
        || effect.flags & 8 != 0
    {
        return Ok(());
    }
    particle_collider(effect)?.resolve_nonplayer_walls(
        grid,
        &mut effect.position,
        &mut effect.velocity,
        false,
    )?;
    effect.presented_position = effect.position;
    Ok(())
}
