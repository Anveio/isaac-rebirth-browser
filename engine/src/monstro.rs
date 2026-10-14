//! Fresh Monstro20/0/0 source AI4ce290 and counted projectile producer48d5c0.
//! Constructors and actors use the shared stream; no captured state enters AI.
use crate::{
    audio::{AudioState, SoundRequest},
    combat, content,
    enemy_projectiles::ProjectileBirth,
    grid::{Collider, CollisionGeometry, Grid},
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    phase::Phase,
    room_spawns::{SpawnCategory, SpawnSelector},
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };
fn white() -> [f32; 4] {
    [1.0; 4]
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MonstroState {
    pub state_frame: u32,
    pub choice: u32,
    pub entity_flags: u32,
    pub flip_x: bool,
    /// Actual fresh ColorMod42f450; later status/champion writers are pending.
    #[serde(default = "white")]
    pub inherited_tint: [f32; 4],
    #[serde(default)]
    pub inherited_offsets: [f32; 3],
    pub birth_game_update: u32,
    pub projectile_birth_count: u32,
    pub last_projectile_batch: Option<ProjectileBatch>,
    /// Actual source event boundaries; their effect actors remain unadmitted.
    pub death_blood_start: bool,
    pub death_explosion: bool,
    pub final_boss_callback_pending: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectileBatch {
    pub count: u32,
    pub target: WorldVector,
    pub parameter_angle: f32,
}

pub(crate) struct World<'a> {
    pub player: WorldVector,
    pub grid: &'a Grid,
    pub audio: &'a mut AudioState,
    pub system_counter: u32,
    pub poofs: crate::familiars::NpcEffectAdmission<'a>,
}

pub fn from_selector(id: u32, selector: &SpawnSelector) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 20
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Monstro admission requires actual bounded authored position and seed".into());
    }
    Ok(Some(Npc {
        id,
        kind: 20,
        variant: 0,
        subtype: 0,
        spawn_seed: selector.spawn_seed,
        position: selector.position,
        presented_position: selector.position,
        velocity: ZERO,
        target: ZERO,
        wander_velocity: ZERO,
        hp: content::NPC_20_BASEHP,
        max_hp: content::NPC_20_BASEHP,
        collision_radius: content::NPC_20_COLLISIONRADIUS,
        collision_mass: content::NPC_20_COLLISIONMASS,
        collision_damage: content::NPC_20_COLLISIONDAMAGE,
        base_friction: content::NPC_20_FRICTION,
        shadow_size: content::NPC_20_SHADOWSIZE,
        grid_collision_class: content::NPC_20_GRID_CLASS,
        entity_collision_class: 4,
        can_shut_doors: content::NPC_20_CAN_SHUT_DOORS,
        boss: content::NPC_20_BOSS,
        state: 1,
        frame_age: 0,
        animation_frame: 0,
        animation: "Walk".into(),
        animation_clock: 0.0,
        animation_playing: false,
        visible: true,
        saved_initialized_state: 0,
        appearance_effect_seed: None,
        death_effect_seed: None,
        dead: false,
        removed: false,
        damage: combat::DamageQueue::default(),
        keeper: None,
        fireplace: None,
        level2_fly: None,
        dip: None,
        monstro: Some(MonstroState {
            state_frame: 0,
            choice: 0,
            entity_flags: 4,
            flip_x: false,
            inherited_tint: white(),
            inherited_offsets: [0.0; 3],
            birth_game_update: 0,
            projectile_birth_count: 0,
            last_projectile_batch: None,
            death_blood_start: false,
            death_explosion: false,
            final_boss_callback_pending: false,
        }),
    }))
}

fn play(npc: &mut Npc, name: &str, force: bool) {
    if force || npc.animation != name {
        npc.animation = name.into();
        npc.animation_frame = 0;
        npc.animation_clock = 0.0;
        npc.animation_playing = true;
    }
}

fn sprite_frames(name: &str) -> u32 {
    match name {
        "Appear" => content::MONSTRO_APPEAR_FRAMES,
        "Death" => content::MONSTRO_DEATH_FRAMES,
        "JumpDown" => content::MONSTRO_JUMPDOWN_FRAMES,
        "JumpUp" => content::MONSTRO_JUMPUP_FRAMES,
        "Taunt" => content::MONSTRO_TAUNT_FRAMES,
        "Walk" => content::MONSTRO_WALK_FRAMES,
        _ => unreachable!("only authored ordinary Monstro clips are admitted"),
    }
}

fn update_sprite(npc: &mut Npc) {
    if !npc.animation_playing {
        return;
    }
    let frames = sprite_frames(&npc.animation);
    let next = npc.animation_clock + 1.0;
    if next.floor() < frames as f32 {
        npc.animation_clock = next;
        npc.animation_frame = next.floor() as u32;
    } else {
        npc.animation_clock = (frames - 1) as f32;
        npc.animation_frame = frames - 1;
        npc.animation_playing = false;
    }
}

fn difference(a: WorldVector, b: WorldVector) -> WorldVector {
    WorldVector {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}

fn divided(vector: WorldVector, denominator: f32) -> WorldVector {
    // Native612b30 divides each component, rather than multiplying a reciprocal.
    WorldVector {
        x: vector.x / denominator,
        y: vector.y / denominator,
    }
}

fn normalized(vector: WorldVector) -> WorldVector {
    let length = (vector.x * vector.x + vector.y * vector.y).sqrt();
    if length > 0.0 {
        // Native612860 ->612820 rounds the reciprocal before multiplying.
        // Explicit component division belongs to the distinct612b30 helper.
        vector * (1.0 / length)
    } else {
        vector
    }
}

fn direction(random: &mut Mt19937) -> WorldVector {
    let angle = (f64::from(random.uniform()) * std::f64::consts::PI * 2.0) as f32;
    WorldVector {
        x: angle.cos(),
        y: angle.sin(),
    }
}

fn sound(world: &mut World<'_>, random: &mut Mt19937, id: u32, volume: f32) {
    world.audio.admit(
        random,
        world.system_counter,
        SoundRequest {
            id,
            volume,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
}

fn snap_goal(npc: &mut Npc, world: &World<'_>) {
    if npc.target == ZERO && world.grid.collision_at(world.player) == 0 {
        if let Some(center) = world
            .grid
            .index(world.player)
            .and_then(|at| world.grid.position(at))
        {
            npc.target = center;
        }
    }
}

fn produce(
    npc: &mut Npc,
    random: &mut Mt19937,
    target: WorldVector,
    count: u32,
) -> Result<Vec<ProjectileBirth>, String> {
    // Default ProjectileParams47eaf0 executes this angle draw, including radial shots.
    let parameter_angle = (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
    let mut shots = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let velocity = if target == ZERO {
            direction(random) * 7.0
        } else {
            let radius = random.uniform();
            let jitter = (direction(random) * radius) * 3.5;
            normalized(difference(target, npc.position)) * 7.0 + jitter
        };
        let spawn_seed = random.next_u32();
        // Actual type9.Init draws first; the producer overwrites its FS afterward.
        let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
        let falling_speed = ((-random.uniform() * 30.0) * 0.8) + 5.0;
        let scale_choice = random.bounded(2);
        let scale_jitter = random.uniform() * 0.05;
        let scale = (scale_choice as f32 * 0.4 + 0.9) + scale_jitter;
        shots.push(ProjectileBirth {
            position: npc.position,
            velocity,
            spawn_seed,
            initial_falling_speed,
            parameter_angle,
            falling_speed,
            falling_acceleration: 0.32,
            scale,
        });
    }
    let state = npc.monstro.as_mut().expect("actual Monstro owns its state");
    state.projectile_birth_count = state
        .projectile_birth_count
        .checked_add(count)
        .ok_or("Monstro projectile birth count exhausted")?;
    state.last_projectile_batch = Some(ProjectileBatch {
        count,
        target,
        parameter_angle,
    });
    Ok(shots)
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut World<'_>,
    friction: &mut f32,
) -> Result<Vec<ProjectileBirth>, String> {
    *friction *= 0.9;
    let age = npc.monstro.as_ref().expect("Monstro state").state_frame;
    let mut shots = Vec::new();
    let mut next_age = age
        .checked_add(1)
        .ok_or("Monstro AI state frame exhausted")?;
    match npc.state {
        0 => npc.state = 3,
        3 => {
            npc.animation = "Walk".into();
            npc.animation_clock = 0.0;
            npc.animation_frame = 0;
            npc.animation_playing = false;
            npc.entity_collision_class = 4;
            let choice = random.bounded(3);
            npc.monstro.as_mut().unwrap().choice = choice;
            match choice {
                0 => {
                    npc.state = 4;
                    sound(world, random, 14, 0.5);
                    play(npc, "Walk", true);
                }
                1 => {
                    npc.state = 8;
                    play(npc, "Taunt", true);
                    npc.monstro.as_mut().unwrap().flip_x = npc.position.x - world.player.x < 0.0;
                }
                2 => {
                    npc.state = 6;
                    sound(world, random, 14, 1.0);
                    play(npc, "JumpUp", true);
                }
                _ => unreachable!("bounded source choice"),
            }
            next_age = 1; // Direct4ce9e7 assignment, independent of incoming age.
        }
        4 => {
            snap_goal(npc, world);
            if npc.target == ZERO {
                *friction *= 0.8;
            } else {
                npc.entity_collision_class = if (8..=23).contains(&age) { 0 } else { 4 };
                if age == 23 {
                    sound(world, random, 48, 0.5);
                }
                if age <= 23 {
                    let delta = difference(npc.target, npc.position);
                    let distance = (delta.x * delta.x + delta.y * delta.y).sqrt();
                    let factor = if distance == 0.0 {
                        1200.0
                    } else {
                        1.2 / distance
                    };
                    npc.velocity = npc.velocity + divided(delta * factor, *friction);
                    *friction *= 0.9;
                    npc.monstro.as_mut().unwrap().flip_x = npc.velocity.x > 0.0;
                } else if age <= 44 {
                    *friction *= 0.8;
                } else {
                    npc.target = ZERO;
                    next_age = 1;
                    if random.bounded(2) == 0 {
                        npc.state = 3;
                    } else {
                        sound(world, random, 14, 0.4);
                        play(npc, "Walk", true);
                    }
                }
            }
        }
        6 => {
            npc.entity_collision_class = 4;
            if age > 10 {
                npc.entity_collision_class = 0;
                npc.target = ZERO;
                npc.state = 7;
                next_age = 1;
                play(npc, "JumpDown", true);
            }
        }
        7 => {
            snap_goal(npc, world);
            if npc.target != ZERO && age <= 31 {
                npc.entity_collision_class = 0;
                let delta = difference(npc.target, npc.position);
                let impulse = normalized(delta) * 0.3 + delta * 0.006;
                npc.velocity = npc.velocity + divided(impulse, *friction);
            } else {
                *friction *= 0.8;
            }
            if age > 73 {
                npc.target = ZERO;
                npc.state = 3;
                next_age = 1;
            } else if age > 32 {
                npc.entity_collision_class = 4;
                if age == 34 {
                    sound(world, random, 48, 1.0);
                    // Room camera shake5cc640 is a separate typed presentation dependency.
                    shots = produce(npc, random, ZERO, 18)?;
                }
            }
        }
        8 => {
            if age == 24 {
                sound(world, random, 16, 1.0);
                shots = produce(npc, random, world.player, 13)?;
            }
            if age > 67 {
                npc.state = 3;
                next_age = 1;
            }
        }
        _ => return Err("ordinary Monstro reached an unsupported source AI state".into()),
    }
    npc.monstro.as_mut().unwrap().state_frame = next_age;
    Ok(shots)
}

fn collide(npc: &mut Npc, grid: &Grid, preview: bool) -> Result<(), String> {
    static COLLIDER: OnceLock<Collider> = OnceLock::new();
    let collider = COLLIDER.get_or_init(|| {
        Collider::new(CollisionGeometry {
            radius: content::NPC_20_COLLISIONRADIUS,
            scale: WorldVector { x: 1.0, y: 1.0 },
            points: content::NPC_20_GRID_POINTS,
        })
        .expect("authored Monstro collision geometry")
    });
    if preview {
        let mut velocity = npc.velocity;
        collider.resolve_nonplayer_ground(
            grid,
            &mut npc.presented_position,
            &mut velocity,
            true,
        )?;
    } else {
        collider.resolve_nonplayer_ground(grid, &mut npc.position, &mut npc.velocity, false)?;
        npc.presented_position = npc.position;
    }
    Ok(())
}

/// Source manager454b10 calls the full grid query after entity contacts.
/// Fast prediction remains in the actual Render/fast callback.
pub(crate) fn post_contact_grid(npc: &mut Npc, grid: &Grid) -> Result<(), String> {
    collide(npc, grid, false)
}

pub(crate) fn advance(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut World<'_>,
) -> Result<Vec<ProjectileBirth>, String> {
    if npc.removed {
        return Ok(Vec::new());
    }
    if phase != Phase::Update {
        npc.presented_position = npc.position + npc.velocity * 0.5;
        collide(npc, world.grid, true)?;
        return Ok(Vec::new());
    }
    let mut friction = npc.base_friction;
    let mut shots = Vec::new();
    if npc.dead {
        if npc.state != 16 {
            npc.state = 16;
            npc.monstro.as_mut().unwrap().state_frame = 0;
        }
        friction *= 0.01;
        let frame = npc.monstro.as_ref().unwrap().state_frame;
        if frame == 0 {
            play(npc, "Death", false);
        }
        let state = npc.monstro.as_mut().unwrap();
        state.state_frame = frame.checked_add(1).ok_or("boss Death frame exhausted")?;
        if npc.animation_frame == content::MONSTRO_BLOODSTART_FRAME {
            state.death_blood_start = true;
        }
        if npc.animation_frame == content::MONSTRO_EXPLOSION_FRAME {
            state.death_explosion = true;
        }
        if state.death_blood_start && !state.death_explosion {
            let (x, y, scale_x, scale_y) = *content::MONSTRO_DEATH_GUIDE
                .get(npc.animation_frame as usize)
                .ok_or("Monstro current authored BloodGuide frame absent")?;
            world.poofs.blood_start(
                crate::blood_effects::Emission {
                    parent: crate::familiars::Parent {
                        actor: crate::actors::ActorRef {
                            kind: crate::actors::ActorKind::Npc,
                            id: npc.id,
                        },
                        entity_type: npc.kind,
                        variant: npc.variant,
                    },
                    position: npc.position,
                    guide: crate::blood_effects::Guide {
                        position: WorldVector { x, y },
                        scale: WorldVector {
                            x: scale_x,
                            y: scale_y,
                        },
                    },
                    tint: state.inherited_tint,
                    offsets: state.inherited_offsets,
                },
                random,
                world.audio,
                world.system_counter,
            )?;
        }
        if state.death_explosion {
            return Err(
                "Monstro actual Explosion2/77/category5 death callback is not admitted".into(),
            );
        }
        if !npc.animation_playing {
            // Source4f2d10 invokes final-boss reward/heart callback before Remove.
            // Keep that unimplemented callback explicit; never use Fly retirement.
            state.final_boss_callback_pending = true;
            npc.removed = true;
            npc.visible = false;
        }
    } else if npc.state == 1 {
        friction *= 0.01;
        match npc.frame_age {
            0 => {
                npc.state = 0;
                ai(npc, random, world, &mut friction)?;
                npc.saved_initialized_state = npc.state;
                npc.state = 1;
                npc.visible = false;
            }
            1 => {
                let seed = random.next_u32();
                npc.appearance_effect_seed = Some(seed);
                world.poofs.admit_npc(npc, 3, seed)?;
            }
            4 => {
                npc.visible = true;
                play(npc, "Appear", true);
            }
            age if age > 19 && (npc.animation != "Appear" || !npc.animation_playing) => {
                npc.state = npc.saved_initialized_state;
                friction = npc.base_friction;
            }
            _ => {}
        }
        npc.frame_age = npc
            .frame_age
            .checked_add(1)
            .ok_or("Monstro appearance counter exhausted")?;
        if npc.state != 1 {
            shots = ai(npc, random, world, &mut friction)?;
        }
    } else {
        shots = ai(npc, random, world, &mut friction)?;
    }
    update_sprite(npc);
    npc.velocity = npc.velocity * friction;
    npc.position = npc.position + npc.velocity;
    npc.presented_position = npc.position;
    if !npc.dead {
        let mut health = npc.health();
        npc.damage.advance(&mut health, phase)?;
        npc.hp = health.hp;
        if npc.hp < 0.0 {
            npc.dead = true;
        }
    }
    if !npc.position.is_finite() || !npc.velocity.is_finite() {
        return Err("Monstro source motion overflowed actual world vectors".into());
    }
    Ok(shots)
}
