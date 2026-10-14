//! Fresh214/0/0 source4ef880, generic missing-Appear and prior full grid feedback.
use crate::entities::{Birth, Family, NpcWorld, SpawnContext};
use crate::{
    audio::SoundRequest,
    content,
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
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Level2FlyState {
    pub entity_flags: u32,
    pub birth_game_update: u32,
    pub feedback_velocity: WorldVector,
    pub grid_contact: bool,
    pub grid_normal: WorldVector,
    pub previous_grid_velocity: WorldVector,
}
#[derive(Default)]
pub struct Produced {
    pub blood_seed: Option<u32>,
}
pub fn from_selector(id: u32, selector: &SpawnSelector) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 214
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("lvl2 Fly requires bounded authored position and nonzero source seed".into());
    }
    Ok(Some(Npc {
        animation: "Idle".into(),
        level2_fly: Some(Level2FlyState {
            entity_flags: 4,
            birth_game_update: 0,
            feedback_velocity: ZERO,
            grid_contact: false,
            grid_normal: ZERO,
            previous_grid_velocity: ZERO,
        }),
        ..Npc::authored(id, content::entity(214, 0), selector)
    }))
}
fn difference(a: WorldVector, b: WorldVector) -> WorldVector {
    WorldVector {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}
fn length(v: WorldVector) -> f32 {
    (v.x * v.x + v.y * v.y).sqrt()
}
fn divided(v: WorldVector, d: f32) -> WorldVector {
    WorldVector {
        x: v.x / d,
        y: v.y / d,
    }
}
fn direction(random: &mut Mt19937) -> WorldVector {
    let angle = (f64::from(random.uniform()) * std::f64::consts::PI * 2.0) as f32;
    WorldVector {
        x: crate::fmath::cos(angle),
        y: crate::fmath::sin(angle),
    }
}
fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &NpcWorld<'_>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.state == 0 {
        npc.state = 3;
        npc.grid_collision_class = 3;
        npc.level2_fly.as_mut().unwrap().feedback_velocity = ZERO;
    }
    if npc.state != 3 {
        return Err("lvl2 Fly reached unsupported source AI state".into());
    }
    // Play(false) restarts a stopped looping named clip (40fd80/40fe48).
    if !npc.animation_playing {
        npc.animation = "Idle".into();
        npc.animation_clock = 0.0;
        npc.animation_frame = 0;
        npc.animation_playing = true;
    }
    let counter = world.game_counter;
    if counter.is_multiple_of(3) {
        npc.target = world.player;
    }
    if npc.target != ZERO && counter.is_multiple_of(2) {
        let delta = difference(npc.target, npc.position);
        let tangent = WorldVector {
            x: -delta.y,
            y: delta.x,
        };
        let mut delta = delta + tangent;
        delta.y *= 0.5;
        let distance = length(delta);
        if distance > 25.0 {
            let unit = divided(delta, distance); // Actual612bb0, not reciprocal normalization.
            let impulse = ((distance - 25.0) * 0.08).min(1.2);
            let impulse = impulse + impulse;
            npc.velocity = npc.velocity + divided(unit * impulse, *friction);
        }
    }
    let state = npc.level2_fly.as_mut().unwrap();
    if state.grid_contact && random.bounded(3) == 0 {
        let magnitude = random.uniform() * -1.5 - 2.0;
        let normal_length = length(state.grid_normal);
        state.feedback_velocity = if normal_length > 0.0 {
            state.grid_normal * (magnitude / normal_length)
        } else {
            state.grid_normal
        };
    }
    let feedback = state.feedback_velocity;
    if feedback.x * feedback.x + feedback.y * feedback.y <= 1.0 {
        state.feedback_velocity = ZERO;
    } else {
        if *friction != 0.0 {
            npc.velocity = npc.velocity + divided(feedback, *friction);
        }
        state.feedback_velocity = feedback * 0.9;
    }
    if npc.target == ZERO && random.bounded(10) == 0 {
        let delta = difference(world.player, npc.position);
        let distance = length(delta);
        let unit = if distance > 0.0 {
            delta * (1.0 / distance)
        } else {
            delta
        };
        npc.velocity = npc.velocity + unit * 0.5;
    }
    if counter % 4 == 1 {
        npc.wander_velocity = (npc.wander_velocity + direction(random) * 0.6) * 0.4;
    }
    if counter % 3 == 2 {
        if npc.position.y < 230.0 {
            npc.wander_velocity.y += 0.06
        } else if npc.position.y > 420.0 {
            npc.wander_velocity.y -= 0.06
        }
        if npc.position.x > 540.0 {
            npc.wander_velocity.x -= 0.06
        } else if npc.position.x < 80.0 {
            npc.wander_velocity.x += 0.06
        }
        npc.velocity = npc.velocity + divided(npc.wander_velocity * 2.0, *friction);
        *friction *= 0.6;
    }
    Ok(())
}
fn update_sprite(npc: &mut Npc) {
    if npc.animation_playing {
        npc.animation_clock += 1.0;
        if npc.animation_clock.floor() >= content::LEVEL2_FLY_IDLE_FRAMES as f32 {
            npc.animation_clock = 0.0
        }
        npc.animation_frame = npc.animation_clock.floor() as u32;
    }
}
fn collider() -> &'static Collider {
    static COLLIDER: OnceLock<Collider> = OnceLock::new();
    COLLIDER.get_or_init(|| {
        Collider::new(CollisionGeometry {
            radius: content::entity(214, 0).collision_radius,
            scale: WorldVector { x: 1.0, y: 1.0 },
            points: content::entity(214, 0).grid_points,
        })
        .expect("authored lvl2 Fly collision geometry")
    })
}
pub(crate) fn post_contact_grid(npc: &mut Npc, grid: &Grid) -> Result<(), String> {
    let state = npc.level2_fly.as_mut().unwrap();
    let before = grid.clamp_room(npc.position, 0.0);
    let min = grid.clamp_room(
        WorldVector {
            x: f32::MIN,
            y: f32::MIN,
        },
        0.0,
    );
    let max = grid.clamp_room(
        WorldVector {
            x: f32::MAX,
            y: f32::MAX,
        },
        0.0,
    );
    state.grid_contact =
        before.x == min.x || before.x == max.x || before.y == min.y || before.y == max.y;
    state.grid_normal = ZERO;
    if let Some(contact) =
        collider().resolve_nonplayer_walls(grid, &mut npc.position, &mut npc.velocity, false)?
    {
        state.grid_normal = contact.normal;
        if contact.velocity_changed {
            state.grid_contact = true;
            state.previous_grid_velocity = contact.previous_velocity;
        }
    }
    npc.presented_position = npc.position;
    Ok(())
}
pub(crate) fn advance(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut NpcWorld<'_>,
) -> Result<Produced, String> {
    let mut produced = Produced::default();
    if npc.removed || phase == Phase::Skipped {
        return Ok(produced);
    }
    if phase == Phase::Interpolate {
        npc.presented_position = npc.position + (npc.velocity * npc.base_friction) * 0.5;
        let mut velocity = npc.velocity;
        collider().resolve_nonplayer_walls(
            world.grid,
            &mut npc.presented_position,
            &mut velocity,
            true,
        )?;
        return Ok(produced);
    }
    if npc.dead {
        npc.removed = true;
        return Ok(produced);
    }
    let mut friction = npc.base_friction;
    if npc.state == 1 {
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
                world.effects.admit_npc(npc, 2, seed)?;
            }
            4 => {
                npc.visible = true;
            } // Source Play("Appear",true) cannot select a missing clip.
            age if age > 19 => {
                npc.state = npc.saved_initialized_state;
                friction = npc.base_friction;
            }
            _ => {}
        }
        npc.frame_age = npc
            .frame_age
            .checked_add(1)
            .ok_or("lvl2 Fly appearance counter exhausted")?;
        if npc.state != 1 {
            ai(npc, random, world, &mut friction)?;
        }
    } else {
        ai(npc, random, world, &mut friction)?;
    }
    update_sprite(npc);
    npc.velocity = npc.velocity * friction;
    npc.position = npc.position + npc.velocity;
    npc.presented_position = npc.position;
    let mut health = npc.health();
    npc.damage.advance(&mut health, phase)?;
    npc.hp = health.hp;
    if npc.hp < 0.0 {
        npc.dead = true;
        // Zero-gib445c60 path admits sound before real BloodSplat7 seed.
        world.audio.admit(
            random,
            world.system_counter,
            SoundRequest {
                id: 30,
                volume: 0.8,
                delay: 2,
                looping: false,
                pitch: 1.0,
            },
        );
        let seed = random.next_u32();
        npc.death_effect_seed = Some(seed);
        produced.blood_seed = Some(seed);
    }
    if !npc.position.is_finite() || !npc.velocity.is_finite() {
        return Err("lvl2 Fly source motion overflowed".into());
    }
    Ok(produced)
}

pub(crate) const FAMILY: Family = Family {
    kinds: &[214],
    spawn: spawn_family,
    advance: advance_family,
    post_contact_grid,
    entity_flags: family_flags,
    occupancy: crate::entities::Occupancy::None,
};

fn spawn_family(
    id: u32,
    selector: &SpawnSelector,
    context: &SpawnContext,
) -> Result<Option<Npc>, String> {
    let mut npc = from_selector(id, selector)?;
    if let Some(fly) = npc.as_mut().and_then(|npc| npc.level2_fly.as_mut()) {
        fly.birth_game_update = context.game_update_count;
    }
    Ok(npc)
}

fn advance_family(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    births.extend(
        advance(npc, phase, random, world)?
            .blood_seed
            .map(Birth::Blood),
    );
    Ok(())
}

fn family_flags(npc: &Npc) -> u32 {
    npc.level2_fly.as_ref().map_or(4, |fly| fly.entity_flags)
}
