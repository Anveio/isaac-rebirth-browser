//! NPC ShootAngle476120 admission and Entity_Laser callbacks.
//! Extracted from the existing Laser module without changing its field or
//! callback behavior; the sole storage owner remains `State::lasers`.
use crate::{
    actors::{ActorKind, ActorRef},
    audio::SoundRequest,
    combat, content,
    lasers::{Impact, Laser},
    motion::WorldVector,
    phase::Phase,
    State,
};

const IMPACT_ACTOR: &str = "gfx/1000.050_brimstoneimpact.anm2";
/// 68ef44 degrees-to-radians.
const DEGREES_TO_RADIANS: f32 = 0.017_453_292;

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
        x: crate::fmath::cos(radians),
        y: crate::fmath::sin(radians),
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

fn parent_position(
    state: &State,
    parent: Option<ActorRef>,
    presented: bool,
) -> Option<(WorldVector, bool)> {
    let parent = parent.filter(|actor| actor.kind == ActorKind::Npc)?;
    state
        .room
        .npcs
        .iter()
        .find(|npc| npc.id == parent.id && !npc.removed)
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
    let mut laser = Laser::initialize_npc(
        id,
        birth.variant,
        birth.spawn_seed,
        state.game_update_count.wrapping_add(1),
        config,
    );
    laser.parent = Some(ActorRef {
        kind: ActorKind::Npc,
        id: parent,
    });
    laser.spawner_type = parent_type;
    laser.set_npc_positions(birth.position, birth.position, false);
    laser.parent_offset = sub(birth.position, birth.parent_position);
    laser.render_offset = birth.position_offset;
    laser.angle = birth.angle;
    laser.render_sort_offset = birth.depth;
    laser.remaining_updates = birth.timeout;
    laser.endpoint = birth.position;
    // init_laser: the only rotation store for straight lasers.
    laser.sprite.rotation = birth.angle - 90.0;

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
        depth: laser.render_sort_offset - 5.0,
        actor: IMPACT_ACTOR.into(),
        animation: "Start".into(),
        animation_clock: 0.0,
        animation_playing: true,
        dead: false,
        removed: false,
    };
    // fx->Update immediately: Entity::Update then the variant50 tail.
    advance_impact_full(&mut impact, false)?;
    laser.child = Some(ActorRef {
        kind: ActorKind::Effect,
        id: impact_id,
    });
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
    //479B38 clears7F4 after the successful immediate first callback.
    laser.first_update = false;
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
    let (mut normal_position, mut presented_position) = laser.npc_positions();
    let parent = match parent_override {
        Some(position) => Some((position, false)),
        None => parent_position(state, laser.parent, interpolating),
    };
    if let Some((position, _)) = parent {
        let followed = position + laser.parent_offset;
        if interpolating {
            presented_position = followed;
        } else {
            normal_position = followed;
            presented_position = followed;
        }
    }
    let base = if interpolating {
        presented_position
    } else {
        normal_position
    };
    laser.set_npc_positions(normal_position, presented_position, interpolating);
    let start = base + laser.render_offset;
    let heading = direction(laser.angle);
    laser.endpoint = end_point(start, heading, WorldVector { x: 0.0, y: 0.0 }, bounds);
    laser.beam_length = length(sub(laser.endpoint, start));
    if let Some(impact) = &mut laser.impact {
        impact.scale = WorldVector {
            x: laser.sprite.scale.x * 0.5 + 0.5,
            y: laser.sprite.scale.y,
        };
        impact.rotation = 90.0 + laser.angle;
        impact.position = base;
        impact.position_offset =
            sub(base + heading * laser.beam_length, base) + laser.render_offset;
    }
    // 478ad0: a dead parent kills the beam.
    if parent.is_none_or(|(_, dead)| dead) {
        laser.dead = true;
    }
    if laser.dead || laser.contact_disabled {
        laser.sprite.scale.x *= 0.895;
        if 0.08 > laser.sprite.scale.x {
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
    if laser.remaining_updates > 0 && !interpolating {
        laser.remaining_updates -= 1;
        if laser.remaining_updates == 0 {
            laser.contact_disabled = true;
        }
    }
    Ok(())
}

fn remove(laser: &mut Laser) {
    laser.remove();
}

fn hits(laser: &Laser, heading: WorldVector, position: WorldVector, radius: f32) -> bool {
    let delta = sub(position, laser.npc_positions().0);
    let reach = radius + laser.collision_radius;
    let cross = (delta.x * heading.y - heading.x * delta.y).abs();
    if reach <= cross {
        return false;
    }
    let dot = delta.x * heading.x + delta.y * heading.y;
    dot > 0.0 && laser.beam_length > dot
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
        if laser
            .parent
            .is_some_and(|parent| parent.kind == ActorKind::Npc && npc.id == parent.id)
            || npc.kind == laser.spawner_type
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
                    id: laser.parent.ok_or("NPC Laser Parent link absent")?.id,
                    entity_type: laser.spawner_type,
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
    if phase == Phase::Skipped || !state.lasers[index].active {
        return Ok(());
    }
    let bounds = room_bounds(state)?;
    let mut laser = state.lasers[index].clone();
    update(state, &mut laser, bounds, phase == Phase::Interpolate, None)?;
    if phase == Phase::Update && laser.active {
        // Entity::Update447190: sprite advance; zero velocity keeps position.
        let mut playing = true;
        step_clip(
            &laser.actor,
            &laser.sprite.animation,
            &mut laser.sprite.frame,
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
            advance_impact_full(impact, laser_dead || !laser.active)?;
        }
    }
    Ok(())
}

/// Preserve the existing NPC scheduler/storage retirement in both phases.
/// This boundary is not evidence of native full Uninit or weak-link clearing.
pub fn retire(state: &mut State) {
    let mut retired = Vec::new();
    for laser in &state.lasers {
        if !laser.active {
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
        !(!laser.active && laser.impact.as_ref().is_none_or(|impact| impact.removed))
    });
}
