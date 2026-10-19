//! Chub 28.0: ai 4f7290's variant-0 path (shared with C.H.A.D. 28.1 and The
//! Carrion Queen 28.2, not admitted). Three authored segments link at INIT:
//! each counts the later same-variant segments within 120 px, so the last
//! authored becomes the head. The head walks with MoveRandomly, charges an
//! aligned player, eats a bomb it runs into, and once the room is nearly
//! empty has its tail lay Chargers; the bodies copy points of their parent's
//! 2 px trail. All segments share the lowest HP and die together.
use crate::{
    audio::SoundRequest,
    combat::{DamageEvent, DamageKind, DamageSource},
    content,
    entities::{Birth, Family, NpcWorld, SpawnContext, SpawnedNpc},
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    pathfinder::{self, Pathfinder},
    phase::Phase,
    plain_npc,
    room_spawns::{SpawnCategory, SpawnSelector},
};
use serde::{Deserialize, Serialize};

pub(crate) const FAMILY: Family = Family {
    kinds: &[28],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags,
    occupancy: crate::entities::Occupancy::None,
};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };
/// +318 NO_DEATH_TRIGGER: only the head keeps the OnKill callbacks.
const NO_DEATH_TRIGGER: u32 = 0x40000;
/// Charge state 8, with the head's (hori, down, up) clips.
pub(crate) const CHARGING: u32 = 8;
/// State 15: a swallowed bomb.
pub(crate) const SWALLOWED: u32 = 15;
const NORMAL: [&str; 3] = ["WalkNormalHori", "WalkNormalDown", "WalkNormalUp"];
const ATTACK01: [&str; 3] = ["WalkAttack01Hori", "WalkAttack01Down", "WalkAttackUp"];
const CHUBBY: [&str; 3] = ["WalkChubbyHori", "WalkChubbyDown", "WalkNormalUp"];
const ATTACK02: [&str; 3] = ["WalkAttack02Hori", "WalkAttack02Down", "WalkAttackUp"];
const SAD: [&str; 3] = ["WalkSadHori", "WalkSadDown", "WalkNormalUp"];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChubState {
    /// +f88: 0 head, 1 middle body, 2 tail.
    pub segment: i32,
    /// +f90 PositionTrail, newest first, resampled every 2 px.
    pub trail: Vec<WorldVector>,
    /// +f58: the per-state counter.
    pub counter: i32,
    /// +f78: the charge axis.
    pub direction: WorldVector,
    /// +f80: the swallowed bomb's damage.
    pub bomb_damage: f32,
    /// +f8c: the swallowed bomb's tear flags.
    pub bomb_flags: u32,
    /// Entity+318 flags.
    pub entity_flags: u32,
}

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 28
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Chub requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(28, 0);
    let mut npc = Npc {
        animation: "WalkNormalDown".into(),
        actor: config.actor.into(),
        pathfinder: Some(Pathfinder::new()),
        chub: Some(ChubState {
            segment: 0,
            trail: Vec::new(),
            counter: 0,
            direction: ZERO,
            bomb_damage: 0.0,
            bomb_flags: 0,
            entity_flags: 4,
        }),
        ..Npc::authored(id, config, selector)
    };
    // stageHP 0: 350 per segment on every floor.
    npc.max_hp = content::initial_hp(config, context.stage);
    npc.hp = npc.max_hp;
    Ok(Some(npc))
}

fn entity_flags(npc: &Npc) -> u32 {
    npc.chub.as_ref().map_or(4, |state| state.entity_flags)
}

fn advance(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    plain_npc::advance_dead_ai(npc, phase, random, world, births, ai)
}

fn sound(world: &mut NpcWorld, random: &mut Mt19937, id: u32, volume: f32) {
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

fn minus(a: WorldVector, b: WorldVector) -> WorldVector {
    WorldVector {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}

fn length(vector: WorldVector) -> f32 {
    (vector.x * vector.x + vector.y * vector.y).sqrt()
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    let mut state = npc.chub.take().ok_or("Chub requires its AI state")?;
    let result = step(npc, &mut state, random, world, births, friction);
    npc.chub = Some(state);
    result
}

fn step(
    npc: &mut Npc,
    state: &mut ChubState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.dead {
        // 4f7b48: a dying segment takes its neighbours with it, then the
        // body below runs once more.
        sound(world, random, 117, 0.5);
        for id in [npc.child, npc.parent].into_iter().flatten() {
            if let Some(other) = world.peers.get_mut(id) {
                other.dead = true;
            }
        }
    }
    if npc.state == 0 {
        link(npc, state, world);
        return Ok(());
    }
    match state.segment {
        0 => head(npc, state, random, world, births, friction)?,
        segment => {
            if segment == 1 {
                plain_npc::play(npc, "WalkBody01", false);
            } else if plain_npc::finished(npc) {
                plain_npc::play(npc, "WalkBody02", false);
            }
            match npc.parent.and_then(|id| world.peers.get(id)) {
                // 4f7476: a body without a parent kills itself.
                None => npc.dead = true,
                Some(parent) => {
                    let delay = if segment == 2 { 13 } else { 15 };
                    if let Some(point) = parent.chub.as_ref().and_then(|p| p.trail.get(delay)) {
                        npc.position = *point;
                    }
                }
            }
            if segment == 2 && npc.state == 12 {
                tail_spawn(npc, state, random, world, births, *friction)?;
            }
        }
    }
    trail(npc, state);
    *friction *= 0.8;
    Ok(())
}

/// INIT 4f7970: count the later same-variant segments within 120 px; the
/// count mod 3 is the segment index and the nearest-listed one the parent.
fn link(npc: &mut Npc, state: &mut ChubState, world: &mut NpcWorld) {
    state.entity_flags |= NO_DEATH_TRIGGER;
    npc.state = 4;
    let mut count = 0;
    let mut candidate = None;
    for other in world.peers.after.iter().rev() {
        if other.kind != npc.kind || other.variant != npc.variant || other.removed {
            continue;
        }
        let offset = minus(other.position, npc.position);
        if 14400.0 > offset.x * offset.x + offset.y * offset.y {
            count += 1;
            candidate = Some(other.id);
        }
    }
    state.segment = count % 3;
    if state.segment == 0 {
        plain_npc::play(npc, "AppearHead", false);
        state.entity_flags &= !NO_DEATH_TRIGGER;
        return;
    }
    npc.parent = candidate;
    if let Some(parent) = candidate.and_then(|id| world.peers.get_mut(id)) {
        parent.child = Some(npc.id);
    }
    // PORT GAP: the +460 render order (head 1500, bodies 500 / -500) is not
    // admitted; the bodies' boss byte is cleared.
    npc.boss = false;
    if state.segment == 1 {
        plain_npc::play(npc, "AppearBody01", false);
        npc.shadow_size = 0.34;
    } else {
        plain_npc::play(npc, "AppearBody02", false);
        npc.shadow_size = 0.26;
    }
}

/// HEAD 4f72df: HP sync, the cell probe, the state, then the walk clip.
fn head(
    npc: &mut Npc,
    state: &mut ChubState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    crate::larry::sync_health(npc, world)?;
    let probe = npc.position + crate::npcs::normalized(npc.velocity) * 40.0;
    births.push(Birth::GridHurt {
        position: probe,
        damage: 1,
    });
    let clips = match npc.state {
        4 => {
            walk(npc, state, random, world, friction)?;
            NORMAL
        }
        CHARGING => {
            state.counter += 1;
            let blocked = world.grid.index(probe).is_some_and(|index| {
                world
                    .path_costs
                    .as_deref()
                    .and_then(|costs| costs.get(index).ok())
                    == Some(999)
            });
            if npc.grid_collided || blocked {
                npc.state = 4;
                state.direction = ZERO;
            } else {
                *friction *= 0.9;
                let push = state.direction * 3.7;
                npc.velocity = npc.velocity
                    + WorldVector {
                        x: push.x / *friction,
                        y: push.y / *friction,
                    };
            }
            ATTACK01
        }
        12 => {
            state.counter += 1;
            if state.counter == 5 {
                let tail = npc
                    .child
                    .and_then(|id| world.peers.get(id))
                    .and_then(|body| body.child);
                if let Some(tail) = tail.and_then(|id| world.peers.get_mut(id)) {
                    tail.state = 12;
                    if let Some(owner) = tail.chub.as_mut() {
                        owner.counter = 0;
                    }
                }
            }
            if state.counter <= 10 {
                CHUBBY
            } else {
                if state.counter > 30 {
                    npc.state = 4;
                }
                ATTACK02
            }
        }
        SWALLOWED => {
            swallowed(npc, state, random, world, births, friction)?;
            SAD
        }
        _ => NORMAL,
    };
    animate(npc, clips, world);
    if npc.child.is_none() {
        // A head without a child kills itself.
        npc.dead = true;
    }
    Ok(())
}

/// WALK 4f8374: MoveRandomly at 1.5; after 50 updates charge a player
/// aligned within 20 on an axis and in sight; else 1 in 100 lays enemies
/// while at most 4 countables remain.
fn walk(
    npc: &mut Npc,
    state: &mut ChubState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    friction: &mut f32,
) -> Result<(), String> {
    pathfinder::move_randomly(npc, random, world, 1.5, *friction)?;
    state.counter += 1;
    if state.counter > 50 {
        let lead = random.uniform();
        // PORT GAP: fear and confusion block the charge; the port admits
        // neither status.
        let aim = world.player + (world.player_velocity * lead) * 16.0;
        let delta = minus(aim, npc.position);
        if 20.0 > delta.y.abs() || 20.0 > delta.x.abs() {
            let costs = world
                .path_costs
                .as_deref()
                .ok_or("Chub charge requires source Room.Init path costs")?;
            if world
                .grid
                .charge_line_clear(npc.position, npc.position + delta, costs)
            {
                sound(world, random, 115, 1.0);
                state.direction = crate::gluttony::axis(delta);
                state.counter = 0;
                npc.state = CHARGING;
                if let Some(path) = npc.pathfinder.as_mut() {
                    path.reset();
                }
            }
        }
    }
    if npc.state == 4 && random.bounded(100) == 0 && world.countable_total <= 4 {
        state.counter = 0;
        npc.state = 12;
        if let Some(path) = npc.pathfinder.as_mut() {
            path.reset();
        }
    }
    Ok(())
}

/// State 15 4f7d16: a swallowed bomb rumbles for 35 updates, then bursts
/// inside the head for 1.2x its damage.
fn swallowed(
    npc: &mut Npc,
    state: &mut ChubState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if state.counter == 1 {
        sound(world, random, 117, 0.5);
    }
    *friction *= 0.6;
    state.counter += 1;
    if state.counter <= 35 {
        return Ok(());
    }
    let seed = random.next_u32();
    births.push(Birth::ExplosionEffect {
        seed,
        position: npc.position,
    });
    let health = npc.health();
    npc.damage.enqueue(
        health,
        DamageEvent {
            amount: 1.2 * state.bomb_damage,
            kind: DamageKind::RoomEffect,
            source: DamageSource {
                id: npc.id,
                entity_type: 0,
            },
            copied_source: None,
        },
    )?;
    // PORT GAP: the bomb's poison (0x10), slow (0x8) and freeze (0x20) tear
    // flags are not admitted; plain bombs carry none.
    state.bomb_damage = 0.0;
    state.bomb_flags = 0;
    npc.state = 4;
    state.direction = ZERO;
    Ok(())
}

/// ANIM 4f78d8: the dominant axis picks the clip; a swallowing head holds
/// frame 0 and stills its bodies.
fn animate(npc: &mut Npc, [hori, down, up]: [&str; 3], world: &mut NpcWorld) {
    let name = if npc.velocity.x.abs() > npc.velocity.y.abs() {
        npc.flip_x = npc.velocity.x < 0.0;
        hori
    } else if npc.velocity.y > 0.0 {
        down
    } else {
        up
    };
    if npc.state != SWALLOWED {
        plain_npc::play(npc, name, false);
        return;
    }
    plain_npc::set_frame(npc, name, 0);
    let mut next = npc.child;
    for _ in 0..2 {
        let Some(body) = next.and_then(|id| world.peers.get_mut(id)) else {
            break;
        };
        body.state = SWALLOWED;
        next = body.child;
    }
}

/// TAIL_SPAWN 4f8000: a poop clip toward the parent, then one Charger (or
/// two, 1 in 3) at the cell 5 trail points back.
fn tail_spawn(
    npc: &mut Npc,
    state: &mut ChubState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: f32,
) -> Result<(), String> {
    state.counter += 1;
    if state.counter == 1 {
        sound(world, random, 116, 1.0);
        let delta = npc
            .parent
            .and_then(|id| world.peers.get(id))
            .map_or(ZERO, |parent| minus(parent.position, npc.position));
        if delta.x.abs() > delta.y.abs() {
            npc.flip_x = delta.x > 0.0;
            plain_npc::play(npc, "PoopSide", true);
        } else if delta.y < 0.0 {
            plain_npc::play(npc, "PoopDown", true);
        } else {
            plain_npc::play(npc, "PoopUp", true);
        }
        return Ok(());
    }
    if state.counter != 5 {
        return Ok(());
    }
    let point = state
        .trail
        .get(5)
        .or(state.trail.last())
        .copied()
        .unwrap_or(npc.position);
    let index = world
        .grid
        .index(point)
        .ok_or("Chub tail spawn lies outside the room grid")?;
    let position = world.grid.position(index).ok_or("Chub tail spawn cell")?;
    let selector = |spawn_seed| SpawnSelector {
        group_index: 0,
        grid_index: 0,
        category: SpawnCategory::Entity,
        entity_type: 23,
        variant: 0,
        subtype: 0,
        position,
        group_seed: spawn_seed,
        spawn_seed,
    };
    let charger = |velocity, spawn_seed| {
        Birth::Spawned(SpawnedNpc {
            selector: selector(spawn_seed),
            velocity,
            spawner_type: 0,
            boss_spawned: false,
            scale: 1.0,
            health_scale: 1.0,
            max_health_scale: 1.0,
            health_delta: 0.0,
            appear: true,
        })
    };
    if random.bounded(3) != 0 {
        sound(world, random, 265, 0.7);
    } else {
        sound(world, random, 265, 1.0);
        let seed = random.next_u32();
        births.push(charger(ZERO, seed));
    }
    let seed = random.next_u32();
    let push = minus(position, npc.position) * 0.1;
    births.push(charger(
        WorldVector {
            x: -push.x / friction,
            y: -push.y / friction,
        },
        seed,
    ));
    npc.state = 4;
    Ok(())
}

/// TRAIL 4f755e: resample the path every 2 px from the newest point; keep
/// the 20 newest once it passes 30.
fn trail(npc: &Npc, state: &mut ChubState) {
    if state.trail.is_empty() {
        state.trail.push(npc.position);
    }
    let offset = minus(npc.position, state.trail[0]);
    let mut remaining = length(offset);
    let stride = crate::npcs::normalized(offset) * 2.0;
    while remaining > 2.0 {
        let point = state.trail[0] + stride;
        state.trail.insert(0, point);
        remaining -= 2.0;
    }
    if state.trail.len() > 30 {
        state.trail.truncate(20);
    }
}

/// Collision callback 49b8f0: a charging head bites a Charger or Spider.
pub(crate) fn bites(chub: &Npc, other: &Npc) -> bool {
    chub.kind == 28 && chub.state == CHARGING && matches!(other.kind, 23 | 85)
}

/// Collision callback 49bd60: a charging head eats a bomb it touches.
pub(crate) fn swallow(chub: &mut Npc, damage: f32, flags: u32) -> bool {
    if chub.kind != 28 || chub.parent.is_some() || chub.state != CHARGING {
        return false;
    }
    let Some(state) = chub.chub.as_mut() else {
        return false;
    };
    chub.state = SWALLOWED;
    state.counter = 0;
    state.bomb_flags |= flags;
    state.bomb_damage = damage;
    true
}
