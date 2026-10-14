//! Ordinary244 AI4b4f60, staged generic appearance and source free-position.
use crate::entities::{Birth, Family, NpcWorld, SpawnContext};
use crate::{
    audio::{AudioState, SoundRequest},
    content,
    grid::Grid,
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    phase::Phase,
    room_costs::RoomCosts,
    room_spawns::{SpawnCategory, SpawnSelector},
};
use std::sync::OnceLock;

fn collide(npc: &mut Npc, grid: &Grid, preview: bool) -> Result<(), String> {
    static COLLIDER: OnceLock<crate::grid::Collider> = OnceLock::new();
    let collider = COLLIDER.get_or_init(|| {
        crate::grid::Collider::new(crate::grid::CollisionGeometry {
            radius: content::entity(244, 0).collision_radius,
            scale: WorldVector { x: 1.0, y: 1.0 },
            points: content::entity(244, 0).grid_points,
        })
        .expect("authored Worm collision geometry")
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

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };

pub struct World<'a> {
    pub player: WorldVector,
    pub player_velocity: WorldVector,
    /// Active manager actors selected by FreePosition(mode1): type1,4,5,6,
    /// and numeric10..999. Hidden NPCs still exclude their actual position.
    pub occupied: &'a [WorldVector],
    pub grid: &'a Grid,
    pub path_costs: &'a RoomCosts,
    pub audio: &'a mut AudioState,
    pub system_counter: u32,
}

#[derive(Clone, Copy)]
pub struct Shot {
    pub position: WorldVector,
    pub velocity: WorldVector,
    pub spawn_seed: u32,
    pub initial_falling_speed: f32,
    /// The default ProjectileParams constructor executes this real draw,
    /// even though FireProjectiles(mode0) does not read its stored angle.
    pub parameter_angle: f32,
}

fn direction(random: &mut Mt19937) -> WorldVector {
    let angle = (f64::from(random.uniform()) * std::f64::consts::PI * 2.0) as f32;
    WorldVector {
        x: crate::fmath::cos(angle),
        y: crate::fmath::sin(angle),
    }
}

fn free_position(
    anchor: WorldVector,
    random: &mut Mt19937,
    world: &World<'_>,
) -> Result<WorldVector, String> {
    let mut candidate = ZERO;
    let mut index: Option<usize> = None;
    let mut budget = 100_i32;
    let mut radius = 0.0_f32;
    loop {
        let admissible = world.grid.collision_at(candidate) == 0
            && world.grid.index(candidate).is_none_or(|at| {
                !world.grid.descriptor().cells[at]
                    .entity
                    .as_ref()
                    .is_some_and(|entity| matches!(entity.kind, 17 | 18))
            })
            && index.is_some_and(|at| world.path_costs.entries()[at] <= 599);
        if admissible {
            if budget > 0 {
                return Ok(candidate);
            }
            break;
        }
        budget -= 1;
        if budget == -1 {
            break;
        }
        radius = (if radius <= 500.0 { radius } else { 0.0 }) + 5.0;
        candidate = anchor + direction(random) * radius;
        if world.grid.clamp_room(candidate, 0.0) != candidate {
            continue;
        }
        index = world.grid.index(candidate);
        let Some(at) = index else {
            continue;
        };
        if world.path_costs.get(at)? > 599 {
            continue;
        }
        candidate = world.grid.position(at).expect("valid grid center");
        for actor in world.occupied {
            let delta = WorldVector {
                x: actor.x - candidate.x,
                y: actor.y - candidate.y,
            };
            if (delta.x * delta.x + delta.y * delta.y).sqrt() < 20.0 {
                index = None;
            }
        }
    }
    let original = world.grid.clamp_room(anchor, 20.0);
    Ok(world
        .grid
        .index(original)
        .and_then(|at| world.grid.position(at))
        .expect("clamped room anchor has a grid center"))
}

fn play(npc: &mut Npc, name: &str, force: bool) {
    if force || npc.animation != name {
        npc.animation = name.into();
        npc.animation_clock = 0.0;
        npc.animation_frame = 0;
        npc.animation_playing = true;
    }
}

fn update_sprite(npc: &mut Npc) {
    if !npc.animation_playing {
        return;
    }
    let (frames, looping) = match npc.animation.as_str() {
        "Idle" => (content::WORM_IDLE_FRAMES, true),
        "Appear" => (content::WORM_APPEAR_FRAMES, false),
        "DigIn" => (content::WORM_DIGIN_FRAMES, false),
        "DigOut" => (content::WORM_DIGOUT_FRAMES, false),
        _ => unreachable!("only authored ordinary Worm clips are admitted"),
    };
    let frame = npc.animation_clock + 1.0; // Sprite.Reset40dbca default playback speed1.
    if frame.floor() < frames as f32 {
        npc.animation_clock = frame;
        npc.animation_frame = frame.floor() as u32;
    } else if looping {
        npc.animation_clock = 0.0;
        npc.animation_frame = 0;
    } else {
        npc.animation_clock = (frames - 1) as f32;
        npc.animation_frame = frames - 1;
        npc.animation_playing = false;
    }
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut World<'_>,
    friction: &mut f32,
) -> Result<Option<Shot>, String> {
    let mut shot = None;
    match npc.state {
        0 => {
            npc.state = 6;
            npc.target = npc.position;
            return Ok(None); // Source initializer returns before common drag.
        }
        6 => {
            play(npc, "DigIn", false);
            if !npc.animation_playing {
                npc.state = 4;
                npc.target = ZERO;
                npc.entity_collision_class = 0;
                npc.visible = false;
            }
        }
        4 => {
            let radius = random.uniform() * 250.0 + 60.0;
            let anchor = world
                .grid
                .clamp_room(world.grid.room_center() + direction(random) * radius, 20.0);
            let candidate = free_position(anchor, random, world)?;
            let delta = WorldVector {
                x: candidate.x - world.player.x,
                y: candidate.y - world.player.y,
            };
            let distance = delta.x * delta.x + delta.y * delta.y;
            let accept = random.bounded(2);
            if accept == 0 && distance > 10000.0 && distance < 1_000_000.0 {
                npc.target = candidate;
                npc.position = candidate;
            }
            if npc.target != ZERO {
                npc.state = 8;
                npc.visible = true;
            }
        }
        8 => {
            play(npc, "DigOut", false);
            npc.entity_collision_class = 4;
            if npc.animation_frame == content::WORM_SHOOT_FRAME && npc.animation_playing {
                let target = world.player + world.player_velocity * 10.0;
                if world
                    .grid
                    .projectile_line_clear(npc.position, target, world.path_costs)
                {
                    world.audio.admit(
                        random,
                        world.system_counter,
                        SoundRequest {
                            id: 318,
                            volume: 1.0,
                            delay: 2,
                            looping: false,
                            pitch: 1.0,
                        },
                    );
                    let parameter_angle =
                        (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
                    let delta = WorldVector {
                        x: target.x - npc.position.x,
                        y: target.y - npc.position.y,
                    };
                    let length = (delta.x * delta.x + delta.y * delta.y).sqrt();
                    let velocity = if length > 0.0 {
                        delta * (1.0 / length)
                    } else {
                        delta
                    } * 10.0;
                    let spawn_seed = random.next_u32();
                    let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
                    shot = Some(Shot {
                        position: npc.position,
                        velocity,
                        spawn_seed,
                        initial_falling_speed,
                        parameter_angle,
                    });
                }
            }
            if !npc.animation_playing {
                npc.state = 6;
            }
        }
        _ => return Err("ordinary Round Worm reached an unsupported source AI state".into()),
    }
    if npc.target != ZERO {
        npc.velocity = WorldVector {
            x: npc.target.x - npc.position.x,
            y: npc.target.y - npc.position.y,
        };
    }
    *friction *= 0.8;
    Ok(shot)
}

/// Source manager454b10 calls the full grid query after entity contacts.
/// Fast prediction remains in the actual Render/fast callback.
pub(crate) fn post_contact_grid(npc: &mut Npc, grid: &Grid) -> Result<(), String> {
    collide(npc, grid, false)
}

pub fn advance(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut World<'_>,
) -> Result<Option<Shot>, String> {
    if npc.removed {
        return Ok(None);
    }
    if phase != Phase::Update {
        npc.presented_position = npc.position + npc.velocity * 0.5;
        collide(npc, world.grid, true)?;
        return Ok(None);
    }
    if npc.dead {
        // Nonboss244 uses the ordinary next-full retirement branch4fb652.
        npc.removed = true;
        return Ok(None);
    }
    let mut friction = npc.base_friction;
    let mut shot = None;
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
            1 => npc.appearance_effect_seed = Some(random.next_u32()),
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
            .ok_or("Worm appearance counter exhausted")?;
        if npc.state != 1 {
            shot = ai(npc, random, world, &mut friction)?;
        }
    } else {
        shot = ai(npc, random, world, &mut friction)?;
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
        // The real six-gib producer remains a separate unaccepted FX path;
        // do not substitute the Fly zero-gib BloodSplat seed or a fake draw.
    }
    if !npc.position.is_finite() || !npc.velocity.is_finite() {
        return Err("Worm source motion overflowed".into());
    }
    Ok(shot)
}

fn from_selector(
    id: u32,
    selector: &SpawnSelector,
    _: &SpawnContext,
) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 244
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err(
            "NPC admission requires bounded authored position and nonzero spawn seed".into(),
        );
    }
    Ok(Some(Npc {
        animation: "Idle".into(),
        // NPC config47f573 uses Sprite.SetFrame(default,0), which stops
        // playback. Worm waits for the staged forced Appear at counter4.
        animation_playing: false,
        ..Npc::authored(id, content::entity(244, 0), selector)
    }))
}

pub(crate) const FAMILY: Family = Family {
    kinds: &[244],
    spawn: from_selector,
    advance: advance_family,
    post_contact_grid,
    entity_flags: crate::entities::ordinary_flags,
    needs_occupancy: true,
};

fn advance_family(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    let shot = advance(
        npc,
        phase,
        random,
        &mut World {
            player: world.player,
            player_velocity: world.player_velocity,
            occupied: world.occupied,
            grid: world.grid,
            path_costs: world
                .path_costs
                .as_deref()
                .ok_or("Round Worm requires source Room.Init path costs")?,
            audio: world.audio,
            system_counter: world.system_counter,
        },
    )?;
    births.extend(shot.map(Birth::Shot));
    Ok(())
}
