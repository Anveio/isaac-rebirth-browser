//! Hopper 29.0, Trite 29.1 and Flaming Hopper 54.0: ai 4cfcf0 (shared with
//! Leaper 34, Keeper 86 and Ragling 246, not admitted). In state 3 the actor
//! picks a landing cell (the player when in reach, else a random hop), hops
//! there over rocks and pits (grid class 3 after take-off) and lands back on
//! the ground. A Hopper hit by fire Morphs into a Flaming Hopper, which
//! erases fire damage. There is no dead test, so the dead-AI update runs
//! the body once more.
use crate::{
    audio::SoundRequest,
    combat::DamageKind,
    content,
    entities::{Birth, Family, NpcWorld, SpawnContext},
    grid::Grid,
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    phase::Phase,
    plain_npc,
    room_spawns::{SpawnCategory, SpawnSelector},
};
use serde::{Deserialize, Serialize};

pub(crate) const FAMILY: Family = Family {
    kinds: &[29, 54],
    spawn,
    advance,
    post_contact_grid: grid_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HopperState {
    /// Entity_NPC+f58 state counter.
    pub counter: i32,
}

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    let admitted = matches!(
        (selector.entity_type, selector.variant),
        (29, 0 | 1) | (54, 0)
    );
    if selector.category != SpawnCategory::Entity || !admitted || selector.subtype != 0 {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Hopper requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(selector.entity_type.into(), selector.variant.into());
    let mut npc = Npc {
        // Config47f573 SetFrame(default, 0) leaves the default clip stopped.
        animation: default_animation(selector.entity_type.into(), selector.variant.into()).into(),
        actor: config.actor.into(),
        hopper: Some(HopperState { counter: 0 }),
        ..Npc::authored(id, config, selector)
    };
    npc.max_hp = content::initial_hp(config, context.stage);
    npc.hp = npc.max_hp;
    Ok(Some(npc))
}

fn default_animation(kind: u32, variant: u32) -> &'static str {
    if kind == 29 && variant == 0 {
        "Idle"
    } else {
        "Hop"
    }
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

/// The AI writes +340 itself: class 5 walks into rocks and pits, class 3
/// (mid-hop) only meets the walls.
fn grid_collision(npc: &mut Npc, grid: &Grid) -> Result<(), String> {
    if npc.grid_collision_class == 3 {
        npc.grid_collided = false;
        crate::entities::wall_collision(npc, grid)
    } else {
        crate::entities::ground_collision(npc, grid)
    }
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    _: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.state == 0 {
        // 4cfcf0 INIT only chooses state 3.
        npc.state = 3;
        return Ok(());
    }
    burn(npc, random)?;
    if plain_npc::finished(npc) && npc.state == 3 {
        plain_npc::play(npc, "Idle", false);
    }
    let mut reach = 0.0_f32;
    let mut minimum_idle = 0;
    if npc.kind == 29 && npc.variant == 1 {
        reach += random.uniform() * 50.0;
        reach += random.uniform() * 70.0;
        minimum_idle = 15;
    } else if npc.kind == 54 {
        reach += random.uniform() * 50.0;
    }
    let target = world.player;
    let delta = sub(target, npc.position);
    let mut hopper = npc
        .hopper
        .take()
        .ok_or("Hopper requires its +f58/+440 state")?;
    let result = match npc.state {
        3 => idle(
            npc,
            &mut hopper,
            random,
            world,
            target,
            delta,
            reach,
            minimum_idle,
        ),
        4 => {
            hop(npc, &mut hopper, random, world, *friction);
            Ok(())
        }
        state => Err(format!("Hopper state {state} is not admitted")),
    };
    if npc.state == 3 {
        npc.grid_collision_class = 5;
    }
    hopper.counter += 1;
    npc.hopper = Some(hopper);
    *friction *= 0.7;
    result
}

/// 4cfd40: each queued fire hit turns a Hopper into a Flaming Hopper at the
/// same HP ratio; a Flaming Hopper erases them. Only the vector entry is
/// erased (4fd320); the pending total and flags stay.
fn burn(npc: &mut Npc, random: &mut Mt19937) -> Result<(), String> {
    let fire = |kind: DamageKind| kind.flags() & 2 != 0;
    if !npc.damage.events().iter().any(|event| fire(event.kind)) {
        return Ok(());
    }
    if npc.kind == 29 && npc.variant == 0 {
        let ratio = npc.hp / npc.max_hp;
        npc.morph(54, 0, "Hop", random)?;
        npc.hp = ratio * npc.max_hp;
    }
    if npc.kind == 54 {
        npc.damage.erase_events(|event| fire(event.kind));
    }
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "IDLE 4cffe0 reads the reach, target and minimum idle the entry computed"
)]
fn idle(
    npc: &mut Npc,
    hopper: &mut HopperState,
    random: &mut Mt19937,
    world: &NpcWorld,
    target: WorldVector,
    delta: WorldVector,
    reach: f32,
    minimum_idle: i32,
) -> Result<(), String> {
    if minimum_idle > hopper.counter || npc.target != ZERO {
        return Ok(());
    }
    let radius = 1.3 * reach + 130.0;
    let mut destination = if radius * radius > delta.x * delta.x + delta.y * delta.y {
        let mut destination = WorldVector {
            x: target.x + (delta.x * reach) / 200.0,
            y: target.y + (delta.y * reach) / 200.0,
        };
        if npc.kind == 54 {
            destination = destination + world.player_velocity * 5.0;
        }
        destination
    } else {
        npc.position + crate::npcs::direction(random) * (95.0 + reach)
    };
    destination = world.grid.clamp_room(destination, 5.0);
    let collision = world.grid.collision_at(destination);
    // Only Ragling 246 reads this roll.
    random.bounded(3);
    if collision != 0 {
        return Ok(());
    }
    let Some(index) = world.grid.index(destination) else {
        return Ok(());
    };
    // Entity+440 (`npc.target`): the landing cell centre.
    npc.target = world
        .grid
        .position(index)
        .ok_or("Hopper landing cell has no centre")?;
    npc.state = 4;
    plain_npc::play(npc, "Hop", true);
    hopper.counter = 0;
    Ok(())
}

/// HOP 4d02f8. Steering uses the update's base friction (+470 before the
/// tail's ×0.7).
fn hop(
    npc: &mut Npc,
    hopper: &mut HopperState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    friction: f32,
) {
    if hopper.counter == 19 {
        world.audio.admit(
            random,
            world.system_counter,
            SoundRequest {
                id: 69,
                volume: 1.0,
                delay: 2,
                looping: false,
                pitch: 1.0,
            },
        );
    }
    let mut towards = sub(npc.target, npc.position);
    let length = (towards.x * towards.x + towards.y * towards.y).sqrt();
    if hopper.counter <= 4 {
        towards = towards * 0.1;
        npc.grid_collision_class = 5;
    } else {
        npc.grid_collision_class = 3;
    }
    let speed = (npc.velocity.x * npc.velocity.x + npc.velocity.y * npc.velocity.y).sqrt();
    let check = length < 10.0 || (speed < 0.75 && random.bounded(10) == 0);
    if check && hopper.counter > 25 {
        npc.target = ZERO;
        npc.state = 3;
        plain_npc::play(npc, "Idle", false);
        hopper.counter = 0;
    }
    let push = if length < 30.0 {
        // 612860: a positive length scales by its reciprocal; zero stays.
        let norm = (towards.x * towards.x + towards.y * towards.y).sqrt();
        if norm > 0.0 {
            towards * (1.0 / norm)
        } else {
            towards
        }
    } else {
        towards * 0.05
    };
    npc.velocity = npc.velocity
        + WorldVector {
            x: push.x / friction,
            y: push.y / friction,
        };
}

fn sub(a: WorldVector, b: WorldVector) -> WorldVector {
    WorldVector {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}
