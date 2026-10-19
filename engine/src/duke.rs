//! The Duke of Flies 67.0: ai_duke 4ccbd0. A walled flyer that bounces
//! diagonally at a constant push and, by rolls, belches three orbiting
//! Attack Flies with smoke and tiny flies (Attack01), one big unbound Attack
//! Fly (Attack02), or blows its orbiting flies outward (Attack03). Its dead
//! AI pass scatters 5..14 orphan tiny flies before the boss death.
use crate::{
    audio::SoundRequest,
    content,
    entities::{Birth, Family, NpcWorld, SpawnContext, SpawnedNpc},
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    phase::Phase,
    plain_npc,
    room_spawns::{SpawnCategory, SpawnSelector},
};

pub(crate) const FAMILY: Family = Family {
    kinds: &[67],
    spawn,
    advance,
    post_contact_grid: crate::entities::wall_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };
/// 492c40 counts these as the Duke's flies.
const FLY: u32 = 18;
/// More than this many tiny flies in the room skips Attack01's swarm.
const TINY_FLY_CAP: usize = 24;

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    // The Husk 67.1 needs Spider 85 and the 48d5c0 lob; base boss rooms
    // author 67.0.0 only.
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 67
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err(
            "Duke of Flies requires bounded authored position and nonzero spawn seed".into(),
        );
    }
    let config = content::entity(67, 0);
    let mut npc = Npc {
        // Config47f573 SetFrame(default, 0) leaves the default clip stopped.
        animation: "Walk".into(),
        actor: config.actor.into(),
        ..Npc::authored(id, config, selector)
    };
    npc.max_hp = content::initial_hp(config, context.stage);
    npc.hp = npc.max_hp;
    Ok(Some(npc))
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

fn sound(random: &mut Mt19937, world: &mut NpcWorld, id: u32) {
    world.audio.admit(
        random,
        world.system_counter,
        SoundRequest {
            id,
            volume: 1.0,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    npc.grid_collision_class = 3;
    // 5c7e50: clamp with a 40 top margin, then bounce off an exact bound.
    let (minimum, maximum) = crate::laser_npc_runtime::room_bounds_from_grid(world.grid);
    let top = minimum.y + 40.0;
    npc.position.x = npc.position.x.min(maximum.x).max(minimum.x);
    npc.position.y = npc.position.y.min(maximum.y).max(top);
    if npc.position.x == maximum.x {
        npc.velocity.x = -1.0;
    }
    if npc.position.x == minimum.x {
        npc.velocity.x = 1.0;
    }
    if npc.position.y == maximum.y {
        npc.velocity.y = -1.0;
    }
    if npc.position.y == top {
        npc.velocity.y = 1.0;
    }
    // 4cce70: a constant diagonal push; vel.y in [0, 0.1) counts as up.
    let sx = if npc.velocity.x > 0.0 { 1.0 } else { -1.0 };
    let sy = if 0.1 <= npc.velocity.y { 1.0 } else { -1.0 };
    *friction *= 0.8;
    npc.velocity = npc.velocity
        + WorldVector {
            x: (sx * 0.4) / *friction,
            y: (sy * 0.4) / *friction,
        };
    let flies = world
        .peers
        .iter()
        .filter(|other| other.kind == FLY && other.spawner_type == 67 && !other.removed)
        .count() as i32;
    let total = world.countable_total.max(0);
    match npc.state {
        0 => {
            npc.state = 4;
            tail(npc);
        }
        4 => {
            // 4cd840: each roll stops the chain once its attack starts.
            if random.bounded(40) == 0 && total + flies <= 8 {
                start(npc, 12, "Attack01");
            } else if random.bounded(70) == 0 && total - flies <= 9 {
                start(npc, 13, "Attack02");
            } else if random.bounded(45) == 0 && flies > 4 && total - flies <= 4 {
                start(npc, 8, "Attack03");
            } else {
                let _unused = random.bounded(70);
                plain_npc::play(npc, "Walk", false);
                npc.death_frame = 1;
            }
        }
        12 => {
            if npc.death_frame == 20 {
                attack_flies(npc, random, world, births)?;
            }
            tail(npc);
        }
        13 => {
            if npc.death_frame == 18 {
                sound(random, world, 106);
                let spawn_seed = random.next_u32();
                births.push(Birth::Spawned(SpawnedNpc {
                    selector: selector(npc, 0.0, spawn_seed),
                    velocity: ZERO,
                    spawner_type: 0,
                    boss_spawned: true,
                    scale: 1.25,
                    health_scale: 1.8,
                    max_health_scale: 1.8,
                    health_delta: 0.0,
                    appear: false,
                }));
            }
            tail(npc);
        }
        8 => {
            if npc.death_frame == 17 {
                blow(npc, random, world)?;
            }
            tail(npc);
        }
        _ => tail(npc),
    }
    if npc.dead {
        // 4ccd74: 5..14 orphan tiny flies; the bound is redrawn per test.
        let mut count = 0;
        while count < random.bounded(10) + 5 {
            let seed = random.next_u32();
            let spread = random.uniform() * 50.0;
            let angle = crate::npcs::direction(random);
            world
                .effects
                .admit_tiny_fly(seed, npc.position + angle * spread, ZERO, None)?;
            count += 1;
        }
    }
    Ok(())
}

/// The state counter is Entity_NPC+f58 (shared with the dying frame).
fn start(npc: &mut Npc, state: u32, clip: &str) {
    npc.state = state;
    plain_npc::play(npc, clip, true);
    npc.death_frame = 1;
}

/// 4ccd50: f58 ticks and the attack ends after 39.
fn tail(npc: &mut Npc) {
    npc.death_frame += 1;
    if npc.death_frame > 39 {
        npc.state = 4;
    }
}

fn selector(npc: &Npc, dx: f32, spawn_seed: u32) -> SpawnSelector {
    SpawnSelector {
        group_index: 0,
        grid_index: 0,
        category: SpawnCategory::Entity,
        entity_type: FLY as u16,
        variant: 0,
        subtype: 0,
        position: WorldVector {
            x: npc.position.x + dx,
            y: npc.position.y + 20.0,
        },
        group_seed: spawn_seed,
        spawn_seed,
    }
}

/// 4cd21e at f58 20: grunt, two silent smoke puffs, three orbiting Attack
/// Flies and a parented tiny-fly swarm.
fn attack_flies(
    npc: &Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    sound(random, world, 108);
    for angle in [90.0, -90.0] {
        let seed = random.next_u32();
        world
            .effects
            .admit_duke_smoke(seed, npc.position, angle, random)?;
    }
    for dx in [0.0, 10.0, -10.0] {
        let spawn_seed = random.next_u32();
        births.push(Birth::Spawned(SpawnedNpc {
            selector: selector(npc, dx, spawn_seed),
            velocity: ZERO,
            spawner_type: 67,
            boss_spawned: true,
            scale: 1.0,
            health_scale: 1.0,
            max_health_scale: 1.0,
            health_delta: 0.0,
            appear: false,
        }));
    }
    if world.effects.tiny_fly_count() > TINY_FLY_CAP {
        return Ok(());
    }
    let count = random.bounded(5) + 1;
    let origin = WorldVector {
        x: npc.position.x,
        y: npc.position.y + 20.0,
    };
    for _ in 0..count {
        let seed = random.next_u32();
        let velocity = crate::npcs::direction(random);
        let spread = random.uniform() * 40.0;
        let angle = crate::npcs::direction(random);
        let id =
            world
                .effects
                .admit_tiny_fly(seed, origin + angle * spread, velocity, Some(npc.id))?;
        // The spawner runs the tiny fly's update at once (vtable+0x18).
        let fires: Vec<_> = world
            .peers
            .iter()
            .filter(|other| other.kind == 33)
            .map(|other| (other.position, other.collision_radius))
            .collect();
        let sequence = world
            .effects
            .scheduler
            .sequence_id(crate::actors::ActorRef {
                kind: crate::actors::ActorKind::Effect,
                id,
            })
            .unwrap_or(0);
        let probe = crate::fly_effects::Probe {
            player: world.player,
            game_counter: world.game_counter,
            sequence,
            bounds: crate::laser_npc_runtime::room_bounds_from_grid(world.grid),
            parent: Some((npc.position, npc.collision_radius)),
            tears: world.tear_probe,
            knives: world.knife_probe,
            fires: &fires,
        };
        world.effects.advance_tiny_fly_initial(id, random, &probe)?;
    }
    Ok(())
}

/// 4cd0da at f58 17: every Duke fly is released and pushed out at 18; every
/// tiny fly is pushed out at 3.
fn blow(npc: &Npc, random: &mut Mt19937, world: &mut NpcWorld) -> Result<(), String> {
    sound(random, world, 112);
    let origin = npc.position;
    let push = |position: WorldVector, velocity: &mut WorldVector, friction: f32, strength: f32| {
        let delta = WorldVector {
            x: position.x - origin.x,
            y: position.y - origin.y,
        };
        let squared = delta.x * delta.x + delta.y * delta.y;
        if squared > 0.01 && friction != 0.0 {
            let scaled = delta * (strength / squared.sqrt());
            *velocity = *velocity
                + WorldVector {
                    x: scaled.x / friction,
                    y: scaled.y / friction,
                };
        }
    };
    for fly in world
        .peers
        .before
        .iter_mut()
        .chain(world.peers.after.iter_mut())
        .filter(|other| other.kind == FLY && other.spawner_type == 67 && !other.removed)
    {
        fly.spawner_type = 0;
        let friction = fly.friction;
        push(fly.position, &mut fly.velocity, friction, 18.0);
    }
    world
        .effects
        .push_tiny_flies(|position, velocity, friction| push(position, velocity, friction, 3.0));
    Ok(())
}
