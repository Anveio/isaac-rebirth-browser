//! Larry Jr. 19.0: ai_larry 4f2e20. Every authored segment is its own boss
//! actor; at age 0 each links to an adjacent unlinked segment, so the last
//! segment to initialise in a chain becomes the head. The head walks the
//! room with MoveRandomly; each body follows its parent's breadcrumb trail
//! four entries back, and the tail rolls for poop. Severing a chain
//! (448040, `npcs::unlink_removed`) turns the follower into a new head, and
//! a head without a live child kills itself.
use crate::{
    actors::{ActorKind, ActorRef},
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
    kinds: &[19],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags,
    occupancy: crate::entities::Occupancy::None,
};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };
/// 444900/4448c0 direction index 0 left, 1 up, 2 right, 3 down.
const HEAD_CLIPS: [&str; 4] = ["WalkHeadHori", "WalkHeadUp", "WalkHeadHori", "WalkHeadDown"];
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
}

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    // The Hollow 19.1 and subtypes 1/2 have their own movement, shots and
    // drops; base rooms author 19.0.0 only.
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 19
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Larry Jr. requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(19, 0);
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
    if phase == Phase::Update && npc.dead && !npc.removed && npc.state != 16 {
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
    _: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    let mut state = npc.larry.take().ok_or("Larry Jr. requires its AI state")?;
    let result = match npc.state {
        0 => {
            link(npc, &mut state, world);
            Ok(())
        }
        4 => run(npc, &mut state, random, world, friction),
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
        .filter(|other| other.kind == 19 && !other.removed)
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
    friction: &mut f32,
) -> Result<(), String> {
    if plain_npc::on_frame(npc, world, 10)? {
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
    if npc.parent.is_none() {
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
        // 5cbb30 Hurt(1) on the grid entity 40 ahead only affects poop and
        // TNT, whose Hurt draws are not admitted yet.
    } else {
        follow(npc, &mut target_velocity, world);
        if npc.child.is_none() && world.room_type != 11 {
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
fn follow(npc: &mut Npc, target_velocity: &mut WorldVector, world: &NpcWorld) {
    let Some(entry) = npc
        .parent
        .and_then(|id| world.peers.get(id))
        .and_then(|parent| parent.larry.as_ref())
        .and_then(|parent| parent.trail.get(FOLLOW_DELAY))
        .copied()
    else {
        return;
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
    if npc.child.is_none() {
        clips[1] = "Butt";
    }
    face(npc, clips, entry.velocity);
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
