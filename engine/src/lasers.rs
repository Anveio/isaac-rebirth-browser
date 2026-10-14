//! Entity_Laser (type 7) as NPCs fire it: ShootAngle476120 spawns a beam
//! that follows its parent, clips to the room bounds, damages along its
//! length every second update and shrinks away after its timeout. Its Brimstone
//! Impact effect (1000.50) rides the beam end. Source notes: init_laser479490,
//! update_laser478320, CalculateEndPoint476450, damage_entities476d70.
use crate::{
    actors::{ActorKind, ActorRef},
    audio::SoundRequest,
    combat, content,
    motion::WorldVector,
    phase::Phase,
    State,
};
use serde::{Deserialize, Serialize};

const IMPACT_ACTOR: &str = "gfx/1000.050_brimstoneimpact.anm2";
/// 68ef44 degrees-to-radians.
const DEGREES_TO_RADIANS: f32 = 0.017_453_292;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Laser {
    pub id: u32,
    pub variant: u32,
    pub spawn_seed: u32,
    /// Parent NPC (+498) by room NPC id, and its type for the damage filter.
    pub parent: u32,
    pub parent_type: u32,
    pub position: WorldVector,
    pub presented_position: WorldVector,
    /// +7d4 fixed follow offset from the parent.
    pub follow_offset: WorldVector,
    /// +458 PositionOffset.
    pub position_offset: WorldVector,
    /// +7dc angle in degrees.
    pub angle: f32,
    /// +7e4 render depth offset.
    pub depth: f32,
    /// +7f0 remaining timeout.
    pub timeout: i32,
    /// +7f6 shrinking after the timeout.
    pub shrinking: bool,
    /// +7f8 beam length from pos+offset to the endpoint.
    pub length: f32,
    /// +8a0 clipped endpoint.
    pub endpoint: WorldVector,
    /// Sprite +2c4 scale and +2cc rotation.
    pub scale: WorldVector,
    pub rotation: f32,
    /// +42c Game+ce9f4 at Init.
    pub birth_game_update: u32,
    pub collision_radius: f32,
    pub collision_damage: f32,
    pub actor: String,
    pub animation: String,
    pub animation_clock: f32,
    pub dead: bool,
    pub removed: bool,
    pub impact: Option<Impact>,
}

/// Brimstone Impact 1000.50 attached to the beam end.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Impact {
    pub id: u32,
    pub spawn_seed: u32,
    pub position: WorldVector,
    pub position_offset: WorldVector,
    pub scale: WorldVector,
    pub rotation: f32,
    pub depth: f32,
    pub actor: String,
    pub animation: String,
    pub animation_clock: f32,
    pub animation_playing: bool,
    pub dead: bool,
    pub removed: bool,
}

/// A laser chosen inside an NPC's AI, which drew its seed. Admission follows
/// the parent's update, so the impact seed drawn there keeps the source order.
pub(crate) struct LaserBirth {
    pub variant: u32,
    /// The parent's position when its AI fired (before its own motion).
    pub parent_position: WorldVector,
    pub position: WorldVector,
    pub angle: f32,
    pub timeout: i32,
    pub position_offset: WorldVector,
    pub depth: f32,
    pub spawn_seed: u32,
}

fn direction(angle: f32) -> WorldVector {
    let radians = angle * DEGREES_TO_RADIANS;
    WorldVector {
        x: radians.cos(),
        y: radians.sin(),
    }
}

fn sub(a: WorldVector, b: WorldVector) -> WorldVector {
    WorldVector {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}

fn length(vector: WorldVector) -> f32 {
    (vector.x * vector.x + vector.y * vector.y).sqrt()
}

/// 612750: x = max(min(x, high), low) per axis.
fn clamp(point: WorldVector, low: WorldVector, high: WorldVector) -> WorldVector {
    WorldVector {
        x: point.x.min(high.x).max(low.x),
        y: point.y.min(high.y).max(low.y),
    }
}

/// CalculateEndPoint476450: clip start + dir*1000 to the room interior
/// expanded by 20 units; rocks and other grid entities never stop the beam.
fn end_point(
    start: WorldVector,
    heading: WorldVector,
    offset: WorldVector,
    bounds: (WorldVector, WorldVector),
) -> WorldVector {
    let (top_left, bottom_right) = bounds;
    let minimum = WorldVector {
        x: (top_left.x - offset.x) - 20.0,
        y: (top_left.y - offset.y) - 20.0,
    };
    let maximum = WorldVector {
        x: (bottom_right.x - offset.x) + 20.0,
        y: (bottom_right.y - offset.y) + 20.0,
    };
    let point = clamp(
        start,
        WorldVector {
            x: minimum.x + 1.0,
            y: minimum.y + 1.0,
        },
        WorldVector {
            x: maximum.x - 1.0,
            y: maximum.y - 1.0,
        },
    );
    let mut end = point + heading * 1000.0;
    if minimum.x > end.x {
        end.y = ((end.y - point.y) * (minimum.x - point.x)) / (end.x - point.x) + point.y;
        end.x = minimum.x;
    } else if end.x > maximum.x {
        end.y = ((end.y - point.y) * (maximum.x - point.x)) / (end.x - point.x) + point.y;
        end.x = maximum.x;
    }
    if minimum.y > end.y {
        end.x = ((end.x - point.x) * (minimum.y - point.y)) / (end.y - point.y) + point.x;
        end.y = minimum.y;
    } else if end.y > maximum.y {
        end.x = ((end.x - point.x) * (maximum.y - point.y)) / (end.y - point.y) + point.x;
        end.y = maximum.y;
    }
    clamp(end, minimum, maximum)
}

/// Room+1c..+28 interior corners for the admitted grid.
fn room_bounds(state: &State) -> Result<(WorldVector, WorldVector), String> {
    let grid = state
        .room
        .grid
        .as_ref()
        .ok_or("laser requires an admitted grid")?;
    Ok((
        WorldVector { x: 60.0, y: 140.0 },
        WorldVector {
            x: 60.0 + f32::from(grid.width - 2) * 40.0,
            y: 140.0 + f32::from(grid.height - 2) * 40.0,
        },
    ))
}

fn parent_position(state: &State, parent: u32, presented: bool) -> Option<(WorldVector, bool)> {
    state
        .room
        .npcs
        .iter()
        .find(|npc| npc.id == parent && !npc.removed)
        .map(|npc| {
            (
                if presented {
                    npc.presented_position
                } else {
                    npc.position
                },
                npc.dead,
            )
        })
}

/// Advance a looping or one-shot authored clip by one sprite update.
fn step_clip(
    actor: &str,
    animation: &str,
    clock: &mut f32,
    playing: &mut bool,
) -> Result<(), String> {
    if !*playing {
        return Ok(());
    }
    let clip = content::clip(actor, animation)
        .ok_or_else(|| format!("{actor} has no authored clip {animation}"))?;
    let next = *clock + 1.0;
    if next.floor() < clip.frames as f32 {
        *clock = next;
    } else if clip.looping {
        *clock = 0.0;
    } else {
        *clock = (clip.frames - 1) as f32;
        *playing = false;
    }
    Ok(())
}

/// Game::Spawn for the laser and its impact, then the parent's immediate
/// Update (init_laser + update_laser) on the spawn frame.
pub(crate) fn admit(state: &mut State, parent: u32, birth: LaserBirth) -> Result<(), String> {
    let parent_type = state
        .room
        .npcs
        .iter()
        .find(|npc| npc.id == parent)
        .map_or(0, |npc| npc.kind);
    let config = content::entity(7, birth.variant);
    let id = state.next_laser;
    state.next_laser = id.checked_add(1).ok_or("laser identity exhausted")?;
    state.actor_scheduler.admit(ActorRef {
        kind: ActorKind::Laser,
        id,
    })?;
    let mut laser = Laser {
        id,
        variant: birth.variant,
        spawn_seed: birth.spawn_seed,
        parent,
        parent_type,
        position: birth.position,
        presented_position: birth.position,
        follow_offset: sub(birth.position, birth.parent_position),
        position_offset: birth.position_offset,
        angle: birth.angle,
        depth: birth.depth,
        timeout: birth.timeout,
        shrinking: false,
        length: 0.0,
        endpoint: birth.position,
        scale: WorldVector { x: 1.0, y: 1.0 },
        // init_laser: the only rotation store for straight lasers.
        rotation: birth.angle - 90.0,
        birth_game_update: state.game_update_count.wrapping_add(1),
        collision_radius: config.collision_radius,
        collision_damage: config.collision_damage,
        actor: config.actor.into(),
        animation: "LargeRedLaser".into(),
        animation_clock: 0.0,
        dead: false,
        removed: false,
        impact: None,
    };
    // init_laser plays the parent's sound before the impact Spawn.
    state.audio.admit(
        &mut state.boot_lifecycle.shared,
        state.system_clock.system_counter.wrapping_sub(1),
        SoundRequest {
            id: 7,
            volume: 1.0,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
    let impact_seed = state.boot_lifecycle.shared.next_u32();
    let impact_id = state.next_effect;
    state.next_effect = impact_id.checked_add(1).ok_or("effect handle exhausted")?;
    state.actor_scheduler.admit(ActorRef {
        kind: ActorKind::Effect,
        id: impact_id,
    })?;
    let mut impact = Impact {
        id: impact_id,
        spawn_seed: impact_seed,
        position: laser.position,
        position_offset: WorldVector { x: 0.0, y: 0.0 },
        scale: WorldVector { x: 1.0, y: 1.0 },
        rotation: 0.0,
        depth: laser.depth - 5.0,
        actor: IMPACT_ACTOR.into(),
        animation: "Start".into(),
        animation_clock: 0.0,
        animation_playing: true,
        dead: false,
        removed: false,
    };
    // fx->Update immediately: Entity::Update then the variant50 tail.
    advance_impact_full(&mut impact, false)?;
    laser.impact = Some(impact);
    // The spawn-frame Update ran inside the parent's AI, before its motion.
    let bounds = room_bounds(state)?;
    update(
        state,
        &mut laser,
        bounds,
        false,
        Some(birth.parent_position),
    )?;
    state.lasers.push(laser);
    Ok(())
}

fn advance_impact_full(impact: &mut Impact, parent_dead: bool) -> Result<(), String> {
    step_clip(
        &impact.actor,
        &impact.animation,
        &mut impact.animation_clock,
        &mut impact.animation_playing,
    )?;
    if !impact.animation_playing {
        impact.animation = "Loop".into();
        impact.animation_clock = 0.0;
        impact.animation_playing = true;
    }
    if impact.dead {
        impact.removed = true;
    }
    if parent_dead {
        impact.dead = true;
    }
    Ok(())
}

/// update_laser478320. `interpolating` is +322 (a fast-phase call).
fn update(
    state: &mut State,
    laser: &mut Laser,
    bounds: (WorldVector, WorldVector),
    interpolating: bool,
    parent_override: Option<WorldVector>,
) -> Result<(), String> {
    let parent = match parent_override {
        Some(position) => Some((position, false)),
        None => parent_position(state, laser.parent, interpolating),
    };
    if let Some((position, _)) = parent {
        let followed = position + laser.follow_offset;
        if interpolating {
            laser.presented_position = followed;
        } else {
            laser.position = followed;
            laser.presented_position = followed;
        }
    }
    let base = if interpolating {
        laser.presented_position
    } else {
        laser.position
    };
    let start = base + laser.position_offset;
    let heading = direction(laser.angle);
    laser.endpoint = end_point(start, heading, WorldVector { x: 0.0, y: 0.0 }, bounds);
    laser.length = length(sub(laser.endpoint, start));
    if let Some(impact) = &mut laser.impact {
        impact.scale = WorldVector {
            x: laser.scale.x * 0.5 + 0.5,
            y: laser.scale.y,
        };
        impact.rotation = 90.0 + laser.angle;
        impact.position = base;
        impact.position_offset = sub(base + heading * laser.length, base) + laser.position_offset;
    }
    // 478ad0: a dead parent kills the beam.
    if parent.is_none_or(|(_, dead)| dead) {
        laser.dead = true;
    }
    if laser.dead || laser.shrinking {
        laser.scale.x *= 0.895;
        if 0.08 > laser.scale.x {
            remove(laser);
        }
        if laser.dead {
            return Ok(());
        }
    }
    let counter = state.game_update_count.wrapping_add(1);
    if counter.is_multiple_of(2)
        && !interpolating
        && counter.wrapping_sub(laser.birth_game_update) as i32 > 0
    {
        damage(state, laser, heading)?;
    }
    if laser.timeout > 0 && !interpolating {
        laser.timeout -= 1;
        if laser.timeout == 0 {
            laser.shrinking = true;
        }
    }
    Ok(())
}

fn remove(laser: &mut Laser) {
    laser.dead = true;
    laser.removed = true;
}

fn hits(laser: &Laser, heading: WorldVector, position: WorldVector, radius: f32) -> bool {
    let delta = sub(position, laser.position);
    let reach = radius + laser.collision_radius;
    let cross = (delta.x * heading.y - heading.x * delta.y).abs();
    if reach <= cross {
        return false;
    }
    let dot = delta.x * heading.x + delta.y * heading.y;
    dot > 0.0 && laser.length > dot
}

/// damage_entities476d70 for a non-player parent: the player takes 1.0 and
/// every other vulnerable NPC 22, except the parent and its own type.
fn damage(state: &mut State, laser: &Laser, heading: WorldVector) -> Result<(), String> {
    let player = crate::bombs::player_world(state);
    if !state.player.dead
        && hits(
            laser,
            heading,
            player,
            content::PLAYER_COLLISION_RADIUS_WORLD as f32,
        )
    {
        combat::try_damage_player(state, 1, 60, combat::DamageKind::Laser.flags())?;
    }
    for npc in &mut state.room.npcs {
        if npc.id == laser.parent
            || npc.kind == laser.parent_type
            || npc.dead
            || npc.removed
            || !(10..1000).contains(&npc.kind)
            || matches!(npc.entity_collision_class, 0 | 1)
            || npc.max_hp <= 0.0
            || !hits(laser, heading, npc.position, npc.collision_radius)
        {
            continue;
        }
        let health = npc.health();
        npc.damage.enqueue(
            health,
            combat::DamageEvent {
                amount: laser.collision_damage,
                kind: combat::DamageKind::Laser,
                source: combat::DamageSource {
                    id: laser.parent,
                    entity_type: laser.parent_type,
                },
            },
        )?;
    }
    Ok(())
}

/// Scheduler dispatch for the beam (ActorKind::Laser).
pub fn advance_actor(state: &mut State, phase: Phase, id: u32) -> Result<(), String> {
    let Some(index) = state.lasers.iter().position(|laser| laser.id == id) else {
        return Ok(());
    };
    if phase == Phase::Skipped || state.lasers[index].removed {
        return Ok(());
    }
    let bounds = room_bounds(state)?;
    let mut laser = state.lasers[index].clone();
    update(state, &mut laser, bounds, phase == Phase::Interpolate, None)?;
    if phase == Phase::Update && !laser.removed {
        // Entity::Update447190: sprite advance; zero velocity keeps position.
        let mut playing = true;
        step_clip(
            &laser.actor,
            &laser.animation,
            &mut laser.animation_clock,
            &mut playing,
        )?;
    }
    state.lasers[index] = laser;
    Ok(())
}

/// Scheduler dispatch for a beam's impact effect (ActorKind::Effect).
pub fn advance_impact(state: &mut State, phase: Phase, id: u32) -> Result<(), String> {
    if phase != Phase::Update {
        return Ok(());
    }
    for laser in &mut state.lasers {
        let laser_dead = laser.dead;
        if let Some(impact) = laser.impact.as_mut().filter(|impact| impact.id == id) {
            advance_impact_full(impact, laser_dead || laser.removed)?;
        }
    }
    Ok(())
}

/// Retire removed beams and impacts from the scheduler.
pub fn retire(state: &mut State) {
    let mut retired = Vec::new();
    for laser in &state.lasers {
        if laser.removed {
            retired.push(ActorRef {
                kind: ActorKind::Laser,
                id: laser.id,
            });
        }
        if let Some(impact) = laser.impact.as_ref().filter(|impact| impact.removed) {
            retired.push(ActorRef {
                kind: ActorKind::Effect,
                id: impact.id,
            });
        }
    }
    state
        .actor_scheduler
        .retain(|actor| !retired.contains(&actor));
    state.lasers.retain(|laser| {
        !(laser.removed && laser.impact.as_ref().is_none_or(|impact| impact.removed))
    });
}
