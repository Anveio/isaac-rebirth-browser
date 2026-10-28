//! Ring Fly 222.0: ai_ring_of_flies 498230 on the plain NPC base.
//!
//! The room's Ring Flies form one ring. The first in the room list (455680
//! type 222) leads: it walks the shared centre (+440) towards the player and
//! breathes the radius (V1.y) between 40 and 70. Every fly copies the
//! leader's centre and radius, takes its slot angle from its index in that
//! list and flies to its point on the rotating ring (V2).
use crate::{
    content,
    entities::{Birth, Family, NpcWorld, SpawnContext},
    motion::WorldVector,
    mt::Mt19937,
    npcs::{normalized, Npc},
    phase::Phase,
    plain_npc,
    room_spawns::{SpawnCategory, SpawnSelector},
};
use serde::{Deserialize, Serialize};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };

/// 6129d0: per-component f32 difference.
fn sub(a: WorldVector, b: WorldVector) -> WorldVector {
    WorldVector {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}

/// Entity_NPC fields the ring AI owns beyond the shared ones. V1 (+f78:
/// slot angle, +f7c: radius) is `Npc::wander_velocity`, the centre +440 is
/// `Npc::target`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RingFlyState {
    /// V2 +f80: this fly's point on the ring.
    pub point: WorldVector,
    /// I1 +f88: the leader's radius grows while set, shrinks while clear.
    pub growing: i32,
    /// I2 +f8c: the leader approaches the player while the squared distance
    /// exceeds its square. Nothing writes it, so it stays 0.
    pub hold_distance: i32,
}

pub(crate) const FAMILY: Family = Family {
    kinds: &[222],
    spawn,
    advance,
    post_contact_grid: crate::entities::clamp_walls,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

fn spawn(id: u32, selector: &SpawnSelector, _: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 222
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Ring Fly requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(222, 0);
    Ok(Some(Npc {
        // Config47f573 SetFrame(default,0) leaves the default clip stopped.
        animation: "Fly".into(),
        actor: config.actor.into(),
        ring_fly: Some(RingFlyState {
            point: ZERO,
            growing: 0,
            hold_distance: 0,
        }),
        ..Npc::authored(id, config, selector)
    }))
}

fn advance(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    plain_npc::advance(npc, phase, random, world, births, ai)
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    _: &mut Vec<Birth>,
    _: &mut f32,
) -> Result<(), String> {
    // 49827f: with Lord of the Flies (511d90 form 1) a living Ring Fly is
    // removed and the player gains a Blue Fly (508740). Blue Flies are not
    // ported, and an unowned counter history cannot rule the form out.
    match world.transformation_counts {
        Some([_, flies]) if flies < 3 => {}
        _ if npc.dead => {}
        _ => {
            return Err(format!(
                "Ring Fly {} needs the Lord of the Flies form (511d90) and Blue Flies (508740)",
                npc.id
            ))
        }
    }
    // 4982c1: with Skatole (5090c0 item 9) it becomes a Fly 13.0 (Morph
    // 480750) with V1 and V2 cleared, and this update ends.
    let skatole = world
        .player_items
        .is_some_and(|items| items.owned_counts.get(9).is_some_and(|count| *count > 0));
    if skatole {
        npc.morph(13, 0, "Fly", random)?;
        npc.wander_velocity = ZERO;
        npc.ring_fly = None;
        return Ok(());
    }
    // 4982fc: the room's type-222 list in room order, this fly included.
    // 455680 keeps dead and removed actors until the update's purge.
    let before = world.peers.before.iter().filter(|other| other.kind == 222);
    let first = before.clone().next().map_or(npc.id, |other| other.id);
    let index = before.count();
    let count = index
        + 1
        + world
            .peers
            .after
            .iter()
            .filter(|other| other.kind == 222)
            .count();
    // 498319: an orphan adopts the list's first fly (itself when first).
    if npc.parent.is_none() {
        npc.parent = Some(first);
    }
    let parent = npc.parent.expect("parented above");
    match npc.state {
        // 498440: no grid collision, the centre from the parent (or its own
        // position when it leads), growing, radius 40, state 4.
        0 => {
            npc.grid_collision_class = 0;
            npc.target = if parent == npc.id {
                npc.position
            } else {
                parent_npc(world, parent)?.target
            };
            let ring = npc.ring_fly.as_mut().ok_or("Ring Fly state is absent")?;
            ring.growing = 1;
            npc.wander_velocity.y = 40.0;
            npc.state = 4;
            Ok(())
        }
        4 => fly(npc, world, parent, index, count),
        _ => Ok(()),
    }
}

/// The parent handle's actor. A parent removed earlier in this update keeps
/// its fields until Removal448040 nulls the handle.
fn parent_npc<'a>(world: &'a NpcWorld, parent: u32) -> Result<&'a Npc, String> {
    world
        .peers
        .iter()
        .find(|other| other.id == parent)
        .ok_or_else(|| "Ring Fly parent handle names no room NPC".into())
}

/// 498530: the state-4 ring.
fn fly(
    npc: &mut Npc,
    world: &NpcWorld,
    parent: u32,
    index: usize,
    count: usize,
) -> Result<(), String> {
    if plain_npc::finished(npc) {
        plain_npc::play(npc, "Fly", false);
    }
    // 4985b4: 480320 reads the player position unless Fear (0x800) or
    // Confusion (0x200) redirect it; neither redirect is ported.
    if npc.status.feared {
        return Err(format!("Ring Fly {} target under Fear (480320)", npc.id));
    }
    let delta = sub(world.player, npc.position);
    if parent == npc.id {
        // 498821: the leader walks the centre 2 towards the player while the
        // squared distance exceeds I2 squared, else 2 away.
        let ring = npc.ring_fly.as_mut().ok_or("Ring Fly state is absent")?;
        let step = normalized(delta) * 2.0;
        let hold = ring.hold_distance.wrapping_mul(ring.hold_distance) as f32;
        if delta.x * delta.x + delta.y * delta.y > hold {
            npc.target = npc.target + step;
        } else {
            npc.target = sub(npc.target, step);
        }
        // 498894: the radius breathes by 0.5 between 40 and 70.
        if ring.growing != 0 {
            npc.wander_velocity.y += 0.5;
            if npc.wander_velocity.y > 70.0 {
                ring.growing = 0;
            }
        } else {
            npc.wander_velocity.y -= 0.5;
            if 40.0 > npc.wander_velocity.y {
                ring.growing = 1;
            }
        }
    } else {
        // 498621: followers take the parent's centre and radius.
        let leader = parent_npc(world, parent)?;
        npc.target = leader.target;
        npc.wander_velocity.y = leader.wander_velocity.y;
    }
    // 498662: the slot angle, then the ring's rotation by the Game frame
    // counter (Game+ce9f4), reduced by fprem, in f64.
    let slot = ((index as f32) as f64 * std::f64::consts::TAU / (count as f32) as f64) as f32;
    npc.wander_velocity.x = slot;
    let turn = ((world.game_counter as f32 * -0.03_f32) as f64) * std::f64::consts::PI;
    let angle = ((turn + slot as f64) % std::f64::consts::TAU) as f32;
    let (sin, cos) = (crate::fmath::sin(angle), crate::fmath::cos(angle));
    let radius = npc.wander_velocity.y;
    let point = npc.target + WorldVector { x: cos, y: sin } * radius;
    npc.ring_fly
        .as_mut()
        .ok_or("Ring Fly state is absent")?
        .point = point;
    // 498794: fly at 6 towards a point more than 10 away, else snap to it.
    let towards = sub(point, npc.position);
    if towards.x * towards.x + towards.y * towards.y > 100.0 {
        npc.velocity = normalized(towards) * 6.0;
    } else {
        npc.velocity = ZERO;
        npc.position = point;
    }
    Ok(())
}
