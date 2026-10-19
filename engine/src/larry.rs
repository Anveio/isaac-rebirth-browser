//! Larry Jr. 19.0: ai_larry 4f2e20. Every authored segment is its own boss
//! actor; at age 0 each links to an adjacent unlinked segment, so the last
//! segment to initialise in a chain becomes the head. The head walks the
//! room with MoveRandomly; each body follows its parent's breadcrumb trail
//! four entries back, and the tail rolls for poop. Severing a chain
//! (448040, `npcs::unlink_removed`) turns the follower into a new head, and
//! a head without a live child kills itself.
//!
//! Grub 239.0 shares ai_larry: its chain pools HP, the head charges the
//! player along an axis, and the first death kills the whole chain and
//! spawns two to five Chargers, Spitties or Maggots at shuffled segments.
use crate::{
    actors::{ActorKind, ActorRef},
    audio::SoundRequest,
    combat::{DamageEvent, DamageKind, DamageSource},
    content,
    entities::{Birth, Family, NpcWorld, SpawnContext},
    grid::GridEntity,
    items::Rng,
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
    kinds: &[19, 239],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags,
    occupancy: crate::entities::Occupancy::None,
};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };
/// 444900/4448c0 direction index 0 left, 1 up, 2 right, 3 down.
const HEAD_CLIPS: [&str; 4] = ["WalkHeadHori", "WalkHeadUp", "WalkHeadHori", "WalkHeadDown"];
/// 66ed40: a charging Grub head.
const ATTACK_CLIPS: [&str; 4] = [
    "AttackHeadHori",
    "AttackHeadUp",
    "AttackHeadHori",
    "AttackHeadDown",
];
/// 4f3d1b death rolls bounded(3): Charger, Spitty, Maggot.
const GRUB_SPAWNS: [u16; 3] = [23, 31, 21];
const BODY_CLIPS: [&str; 4] = ["WalkBodyHori", "WalkBodyUp", "WalkBodyHori", "WalkBodyDown"];
/// Larry Jr. bodies read the parent's trail four entries back.
const FOLLOW_DELAY: usize = 4;
/// The trail is only read at FOLLOW_DELAY, so older entries are dropped.
const TRAIL_LIMIT: usize = 8;
/// +318 NO_DEATH_TRIGGER: per-player OnKill callbacks are skipped.
const NO_DEATH_TRIGGER: u32 = 0x40000;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrailEntry {
    pub position: WorldVector,
    pub velocity: WorldVector,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LarryState {
    /// +f90/+fe0 PositionTrail and VelocityTrail, newest first.
    pub trail: Vec<TrailEntry>,
    /// +1058: the original head's manager sequence.
    pub group: u32,
    /// Entity+7c0 own RNG, drawn for the tail poop seed.
    pub rng: Rng,
    /// Entity+318 flags.
    pub entity_flags: u32,
    /// Grub +f58 charge cooldown, counting up to 51 while walking.
    #[serde(default)]
    pub charge_counter: i32,
    /// Grub +f88: charging along `npc.target` (+440).
    #[serde(default)]
    pub charging: bool,
    /// Grub +f8c: this chain's death was processed.
    #[serde(default)]
    pub death_processed: bool,
}

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    // The Hollow 19.1 and subtypes 1/2 have their own movement, shots and
    // drops; base rooms author 19.0.0 and 239.0.0 only.
    if selector.category != SpawnCategory::Entity
        || !matches!(selector.entity_type, 19 | 239)
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Larry Jr. requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(selector.entity_type.into(), 0);
    let mut npc = Npc {
        // Config47f573 SetFrame(default, 0) leaves the default clip stopped.
        animation: "WalkHeadHori".into(),
        actor: config.actor.into(),
        // Every segment owns a pathfinder: a severed body becomes a head.
        pathfinder: Some(Pathfinder::new()),
        larry: Some(LarryState {
            trail: Vec::new(),
            group: 0,
            rng: Rng::new(selector.spawn_seed, [4, 3, 17]),
            entity_flags: 4,
            charge_counter: 0,
            charging: false,
            death_processed: false,
        }),
        ..Npc::authored(id, config, selector)
    };
    npc.max_hp = content::initial_hp(config, context.stage);
    npc.hp = npc.max_hp;
    Ok(Some(npc))
}

fn entity_flags(npc: &Npc) -> u32 {
    npc.larry.as_ref().map_or(4, |state| state.entity_flags)
}

fn advance(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    if phase == Phase::Update && npc.dead && !npc.removed && npc.kind == 239 {
        // 4f2e56: Entity_NPC::Update runs the AI once more with dead set.
        grub_death(npc, random, world, births)?;
    }
    if phase == Phase::Update && npc.dead && !npc.removed && npc.kind == 19 && npc.state != 16 {
        // 4f3760: Entity_NPC::Update runs the AI once more with dead set
        // before the boss enters state16. A dead head heals its child.
        if let (None, Some(child)) = (npc.parent, npc.child) {
            if let Some(child) = world.peers.get_mut(child) {
                if child.hp > 0.0 {
                    child.hp = (child.hp + 25.0).min(child.max_hp);
                }
            }
        }
    }
    plain_npc::advance(npc, phase, random, world, births, ai)
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    let mut state = npc.larry.take().ok_or("Larry Jr. requires its AI state")?;
    let result = match npc.state {
        0 => {
            link(npc, &mut state, world);
            Ok(())
        }
        4 => run(npc, &mut state, random, world, births, friction),
        _ => Ok(()),
    };
    npc.larry = Some(state);
    result
}

/// 444900/4448c0 with FlipX for index 0. A zero vector faces down.
fn face(npc: &mut Npc, clips: [&str; 4], toward: WorldVector) {
    let index = if toward.x.abs() > toward.y.abs() {
        if toward.x >= 0.0 {
            2
        } else {
            0
        }
    } else if toward.y >= 0.0 {
        3
    } else {
        1
    };
    plain_npc::play(npc, clips[index], false);
    npc.flip_x = index == 0;
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

fn sequence(world: &NpcWorld, id: u32) -> u32 {
    world
        .effects
        .scheduler
        .sequence_id(ActorRef {
            kind: ActorKind::Npc,
            id,
        })
        .unwrap_or(0)
}

/// State0 4f2f60: scan the same-type list from its end and parent to every
/// qualifying segment, so the lowest-index candidate within 42 wins.
fn link(npc: &mut Npc, state: &mut LarryState, world: &mut NpcWorld) {
    state.entity_flags |= NO_DEATH_TRIGGER;
    npc.grid_collision_class = 5;
    let candidates: Vec<u32> = world
        .peers
        .iter()
        .filter(|other| other.kind == npc.kind && !other.removed)
        .map(|other| other.id)
        .collect();
    npc.parent = None;
    for &id in candidates.iter().rev() {
        let Some(other) = world.peers.get(id) else {
            continue;
        };
        let offset = minus(other.position, npc.position);
        if other.parent == Some(npc.id)
            || other.child.is_some()
            || length(offset) >= 42.0
            || descends_from(world, id, npc.id)
        {
            continue;
        }
        npc.parent = Some(id);
    }
    if let Some(parent) = npc.parent {
        if let Some(parent) = world.peers.get_mut(parent) {
            parent.child = Some(npc.id);
        }
        plain_npc::play(npc, "WalkBodyHori", false);
        if npc.kind == 239 {
            pool_health(npc, world);
        }
    } else {
        plain_npc::play(npc, "WalkHeadDown", false);
        let group = sequence(world, npc.id);
        let mut next = npc.child;
        let mut steps = 0;
        while let Some(id) = next {
            let Some(segment) = world.peers.get_mut(id) else {
                break;
            };
            if let Some(owner) = segment.larry.as_mut() {
                owner.group = group;
            }
            next = segment.child;
            steps += 1;
            if steps > candidates.len() {
                break;
            }
        }
        state.group = group;
    }
    state.trail.insert(
        0,
        TrailEntry {
            position: npc.position,
            velocity: npc.velocity,
        },
    );
    npc.state = 4;
}

/// Whether `ancestor` appears on `id`'s parent chain (cycle-guarded).
fn descends_from(world: &NpcWorld, id: u32, ancestor: u32) -> bool {
    let mut next = world.peers.get(id).and_then(|npc| npc.parent);
    let mut steps = 0;
    while let Some(current) = next {
        if current == ancestor {
            return true;
        }
        steps += 1;
        if steps > world.peers.before.len() + world.peers.after.len() {
            return false;
        }
        next = world.peers.get(current).and_then(|npc| npc.parent);
    }
    false
}

fn run(
    npc: &mut Npc,
    state: &mut LarryState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.kind == 19 && plain_npc::on_frame(npc, world, 10)? {
        // 480fe0 without SetColor: a red Blood7 splat scaled 0.3..0.7.
        let scale = random.uniform() * 0.4 + 0.3;
        let seed = random.next_u32();
        let id = world
            .effects
            .admit_blood(seed, npc.position, [1.0; 4], [0.0; 3])?;
        world.effects.set_blood_scale(id, scale)?;
        world.effects.advance_blood_initial(id, random)?;
    }
    let mut target_velocity = ZERO;
    if npc.parent.is_none() && npc.kind == 239 {
        sync_health(npc, world)?;
        grub_head(npc, state, random, world, friction)?;
        let clips = if state.charging {
            ATTACK_CLIPS
        } else {
            HEAD_CLIPS
        };
        face(npc, clips, npc.velocity);
        target_velocity = npc.velocity;
    } else if npc.parent.is_none() {
        pathfinder::move_randomly(npc, random, world, 2.0, *friction)?;
        face(npc, HEAD_CLIPS, npc.velocity);
        target_velocity = npc.velocity;
        let alive_child = npc
            .child
            .and_then(|id| world.peers.get(id))
            .is_some_and(|child| !child.dead);
        if !alive_child {
            // 47fc80 Kill: no TakeDamage, so no death sound or blood.
            npc.dead = true;
            release_death_trigger(state, world);
        }
        // L35d1: Room5cbb30 Hurt(1) on the cell 40 ahead (612860 leaves a
        // zero velocity at the head). Only poop and TNT take it; the head
        // draws nothing after this, so the room applies it as a birth.
        births.push(Birth::GridHurt {
            position: npc.position + crate::npcs::normalized(npc.velocity) * 40.0,
            damage: 1,
        });
    } else {
        let followed = follow(npc, &mut target_velocity, world);
        if followed && npc.kind == 19 && npc.child.is_none() && world.room_type != 11 {
            tail_poop(npc, state, random, world)?;
        }
    }
    let moved = state
        .trail
        .first()
        .is_none_or(|last| length(minus(last.position, npc.position)) > 3.0);
    if moved {
        state.trail.insert(
            0,
            TrailEntry {
                position: npc.position,
                velocity: target_velocity,
            },
        );
        state.trail.truncate(TRAIL_LIMIT);
    }
    *friction *= 0.8;
    Ok(())
}

/// 4f37e4: snap to the parent's delayed breadcrumb within 10, otherwise
/// step 15 toward it. The parent's trail is live: an earlier-listed parent
/// has already pushed this update.
/// Returns false when the parent's trail is still shorter than the delay:
/// D.2 then jumps straight to the tail (E), skipping the poop roll too.
fn follow(npc: &mut Npc, target_velocity: &mut WorldVector, world: &NpcWorld) -> bool {
    let Some(entry) = npc
        .parent
        .and_then(|id| world.peers.get(id))
        .and_then(|parent| parent.larry.as_ref())
        .and_then(|parent| parent.trail.get(FOLLOW_DELAY))
        .copied()
    else {
        return false;
    };
    *target_velocity = entry.velocity;
    let offset = minus(entry.position, npc.position);
    let distance = length(offset);
    if distance < 10.0 {
        npc.position = entry.position;
        npc.velocity = entry.velocity;
    } else {
        npc.position = npc.position + offset * (1.0 / distance) * 15.0;
    }
    let mut clips = BODY_CLIPS;
    // 4f46e0: Grub tails keep WalkBodyUp.
    if npc.child.is_none() && npc.kind == 19 {
        clips[1] = "Butt";
    }
    face(npc, clips, entry.velocity);
    true
}

/// 4f4613: the tail rolls bounded(100); a damaged tail also passes on
/// bounded(20). Room5c5070 then places poop unless a non-poop grid entity
/// occupies the cell (checked before the own-RNG seed draw).
fn tail_poop(
    npc: &mut Npc,
    state: &mut LarryState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
) -> Result<(), String> {
    if random.bounded(100) != 0 && !(npc.damage.pending_damage() > 0.0 && random.bounded(20) == 0) {
        return Ok(());
    }
    let index = world.grid.index(npc.position);
    if let Some(index) = index {
        let blocked = world
            .grid_descriptor
            .as_deref()
            .and_then(|grid| grid.cells.get(index))
            .and_then(|cell| cell.entity.as_ref())
            .is_some_and(|entity| entity.kind != 14 && entity.kind != 1);
        if blocked {
            return Ok(());
        }
    }
    let seed = state.rng.raw();
    let Some(index) = index else {
        return Ok(());
    };
    let costs = world
        .path_costs
        .as_deref_mut()
        .ok_or("Larry Jr. poop requires source Room.Init path costs")?;
    if let Some(mut poop) =
        GridEntity::spawn(14, 0, seed, random, world.game_counter, costs, index)?
    {
        // Poop.Init569600: no collision while a walking player stands on it.
        let cell = world.grid.position(index).expect("valid grid center");
        let (dx, dy) = (world.player.x - cell.x, world.player.y - cell.y);
        if dx * dx + dy * dy < 400.0 {
            poop.collision_class = 0;
        }
        world
            .grid_descriptor
            .as_deref_mut()
            .ok_or("Larry Jr. poop requires the room grid")?
            .cells[index]
            .entity = Some(poop);
    }
    Ok(())
}

/// 492df0: when every segment of the group is dead and this one is listed
/// first, it clears NO_DEATH_TRIGGER so OnKill fires once for the chain.
fn release_death_trigger(state: &mut LarryState, world: &NpcWorld) {
    let in_group = |other: &&Npc| {
        (10..1000).contains(&other.kind)
            && !other.removed
            && other
                .larry
                .as_ref()
                .is_some_and(|owner| owner.group == state.group)
    };
    let before: Vec<&Npc> = world.peers.before.iter().filter(in_group).collect();
    let after: Vec<&Npc> = world.peers.after.iter().filter(in_group).collect();
    if before.is_empty() && after.iter().all(|other| other.dead) {
        state.entity_flags &= !NO_DEATH_TRIGGER;
    }
}

/// 4f3bb8: a linking Grub takes max(child, parent) + its own max HP and
/// writes it along the whole chain; max HP stays per segment.
fn pool_health(npc: &mut Npc, world: &mut NpcWorld) {
    let child = npc
        .child
        .and_then(|id| world.peers.get(id))
        .map_or(0.0, |child| child.hp);
    let parent = npc
        .parent
        .and_then(|id| world.peers.get(id))
        .map_or(0.0, |parent| parent.hp);
    // maxss: the child's value only when strictly greater.
    let pooled = if child > parent { child } else { parent };
    npc.hp = pooled + npc.max_hp;
    let limit = world.peers.before.len() + world.peers.after.len();
    for start in [npc.parent, npc.child] {
        let upward = start == npc.parent;
        let mut next = start;
        let mut steps = 0;
        while let Some(id) = next {
            let Some(segment) = world.peers.get_mut(id) else {
                break;
            };
            segment.hp = npc.hp;
            next = if upward {
                segment.parent
            } else {
                segment.child
            };
            steps += 1;
            if steps > limit {
                break;
            }
        }
    }
}

/// The chain below `npc` in child order (cycle-guarded).
fn descendants(npc: &Npc, world: &NpcWorld) -> Vec<u32> {
    let limit = world.peers.before.len() + world.peers.after.len();
    let mut chain = Vec::new();
    let mut next = npc.child;
    while let Some(id) = next {
        if chain.len() > limit || chain.contains(&id) {
            break;
        }
        chain.push(id);
        next = world.peers.get(id).and_then(|segment| segment.child);
    }
    chain
}

/// 4f43d9: the head queues damage on every segment above the chain's
/// lowest HP after pending damage. TakeDamage only queues; no draws.
pub(crate) fn sync_health(npc: &mut Npc, world: &mut NpcWorld) -> Result<(), String> {
    let chain = descendants(npc, world);
    let mut lowest = npc.hp - npc.damage.pending_damage();
    for &id in &chain {
        if let Some(segment) = world.peers.get(id) {
            let remaining = segment.hp - segment.damage.pending_damage();
            lowest = if remaining < lowest {
                remaining
            } else {
                lowest
            };
        }
    }
    let source = DamageSource {
        id: npc.id,
        entity_type: npc.kind,
    };
    let queue = |segment: &mut Npc| -> Result<(), String> {
        if segment.hp - segment.damage.pending_damage() > lowest {
            let health = segment.health();
            segment.damage.enqueue(
                health,
                DamageEvent {
                    amount: segment.hp - lowest,
                    kind: DamageKind::HealthSync,
                    source,
                    copied_source: None,
                },
            )?;
        }
        Ok(())
    };
    queue(npc)?;
    for id in chain {
        if let Some(segment) = world.peers.get_mut(id) {
            queue(segment)?;
        }
    }
    Ok(())
}

/// 4f4681: walk with MoveRandomly; after 50 walking updates look for the
/// player aligned within 20 on an axis and in sight, then charge at 3.7.
fn grub_head(
    npc: &mut Npc,
    state: &mut LarryState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    friction: &mut f32,
) -> Result<(), String> {
    if state.charging {
        if npc.grid_collided {
            state.charging = false;
            npc.target = ZERO;
        } else {
            *friction *= 0.9;
            let push = npc.target * 3.7;
            npc.velocity = npc.velocity
                + WorldVector {
                    x: push.x / *friction,
                    y: push.y / *friction,
                };
        }
        return Ok(());
    }
    pathfinder::move_randomly(npc, random, world, 2.0, *friction)?;
    if state.charge_counter <= 50 {
        state.charge_counter += 1;
        return Ok(());
    }
    let lead = random.uniform();
    let predicted = world.player + (world.player_velocity * lead) * 16.0;
    let delta = minus(predicted, npc.position);
    if !(20.0 > delta.y.abs() || 20.0 > delta.x.abs()) {
        return Ok(());
    }
    let end = npc.position + delta;
    let costs = world
        .path_costs
        .as_deref()
        .ok_or("Grub charge requires source Room.Init path costs")?;
    if !world.grid.charge_line_clear(npc.position, end, costs) {
        return Ok(());
    }
    world.audio.admit(
        random,
        world.system_counter,
        SoundRequest {
            id: 115,
            volume: 1.0,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
    npc.target = crate::gluttony::axis(delta);
    state.charge_counter = 0;
    state.charging = true;
    if let Some(path) = npc.pathfinder.as_mut() {
        path.target = ZERO;
    }
    Ok(())
}

/// 4f2e56: the first dead Grub to update kills every linked segment, then
/// shuffles their positions and spawns 2..5 rolls of Charger, Spitty or
/// Maggot at the collision-free ones.
fn grub_death(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    let mut state = npc.larry.take().ok_or("Grub requires its AI state")?;
    let result = (|| {
        if state.death_processed {
            return Ok(());
        }
        state.death_processed = true;
        let mut positions = vec![npc.position];
        let mut members = descendants(npc, world);
        let limit = world.peers.before.len() + world.peers.after.len();
        let mut next = npc.parent;
        while let Some(id) = next {
            if members.len() > limit || members.contains(&id) {
                break;
            }
            members.push(id);
            next = world.peers.get(id).and_then(|segment| segment.parent);
        }
        for id in members {
            let Some(segment) = world.peers.get_mut(id) else {
                continue;
            };
            let position = segment.position;
            positions.push(position);
            if let Some(owner) = segment.larry.as_mut() {
                owner.death_processed = true;
            }
            if segment.dead {
                continue;
            }
            segment.dead = true;
            let seed = death_burst(position, random, world)?;
            if let Some(segment) = world.peers.get_mut(id) {
                segment.death_effect_seed = Some(seed);
            }
        }
        for index in (1..positions.len()).rev() {
            let other = random.bounded(index as u32 + 1) as usize;
            positions.swap(index, other);
        }
        let tries = random.bounded(4) as usize + 2;
        for &position in positions.iter().take(tries) {
            if world.grid.collision_at(position) != 0 {
                continue;
            }
            let roll = random.bounded(3) as usize;
            let spawn_seed = random.next_u32();
            births.push(Birth::Npc(SpawnSelector {
                group_index: 0,
                grid_index: 0,
                category: SpawnCategory::Entity,
                entity_type: GRUB_SPAWNS[roll],
                variant: 0,
                subtype: 0,
                position,
                group_seed: spawn_seed,
                spawn_seed,
            }));
        }
        state.entity_flags &= !NO_DEATH_TRIGGER;
        Ok(())
    })();
    npc.larry = Some(state);
    result
}

/// 47fc80 Kill of a living non-boss segment: 445c60 runs at once (the
/// death sound and Blood splat the plain NPC death admits; gibs pending).
fn death_burst(
    position: WorldVector,
    random: &mut Mt19937,
    world: &mut NpcWorld,
) -> Result<u32, String> {
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
    let id = world
        .effects
        .admit_blood(seed, position, [1.0; 4], [0.0; 3])?;
    world.effects.advance_blood_initial(id, random)?;
    Ok(seed)
}
