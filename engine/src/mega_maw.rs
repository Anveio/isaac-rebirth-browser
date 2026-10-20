//! Mega Maw 262.0: ai 4e0110. Mega Maw rises 60 px at INIT and is pinned
//! to that anchor. From Idle it spits a 15-shot volley at a player below
//! it, fires a purple fire ring that orbits it, or, with at most four
//! countables, summons a Hopper that leaps out. Every roll waits on the
//! global frame, twice as often under 30% HP. The AI never draws the NPC's
//! own RNG.
use crate::{
    audio::SoundRequest,
    content,
    enemy_projectiles::{
        ProjectileBirth, ProjectileFlags, FLAG_NO_GRID, FLAG_ORBIT, FLAG_ORBIT_REVERSE,
        FLAG_PARENTED,
    },
    entities::{Birth, Family, NpcWorld, SpawnContext},
    motion::WorldVector,
    mt::Mt19937,
    npcs::{direction, normalized, Npc},
    phase::Phase,
    plain_npc,
    room_spawns::{SpawnCategory, SpawnSelector},
};
use serde::{Deserialize, Serialize};

pub(crate) const FAMILY: Family = Family {
    kinds: &[262],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

/// Purple Fire Projectile 9.2.
const FIRE: u32 = 2;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MegaMawState {
    /// +f88: the Summon phase (0 tell, 1 summon).
    pub phase: i32,
}

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 262
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Mega Maw requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(262, 0);
    let mut npc = Npc {
        animation: "Idle".into(),
        actor: config.actor.into(),
        mega_maw: Some(MegaMawState { phase: 0 }),
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

fn sound(world: &mut NpcWorld, random: &mut Mt19937, id: u32, delay: i32) {
    world.audio.admit(
        random,
        world.system_counter,
        SoundRequest {
            id,
            volume: 1.0,
            delay,
            looping: false,
            pitch: 1.0,
        },
    );
}

/// is_frame444890 with no sequence offset: the global Game+ce9f4 frame.
fn global_frame(world: &NpcWorld, period: u32) -> bool {
    world.game_counter.is_multiple_of(period)
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.state == 0 {
        // 4e01e0: rise 60 px and anchor there.
        npc.position.y -= 60.0;
        npc.state = 3;
        npc.target = npc.position;
        return Ok(());
    }
    // 4e013c: pinned to the anchor.
    *friction *= 0.8;
    npc.velocity = WorldVector {
        x: npc.target.x - npc.position.x,
        y: npc.target.y - npc.position.y,
    };
    let mut state = npc
        .mega_maw
        .take()
        .ok_or("Mega Maw requires its AI state")?;
    let result = step(npc, &mut state, random, world, births);
    npc.mega_maw = Some(state);
    result
}

fn step(
    npc: &mut Npc,
    state: &mut MegaMawState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    match npc.state {
        3 => idle(npc, state, random, world),
        8 => {
            plain_npc::play(npc, "Spit", false);
            sound(world, random, 310, 60);
            if plain_npc::event_triggered(npc, "Shoot") {
                spit(npc, random, world, births);
            }
            if plain_npc::finished(npc) {
                npc.state = 3;
            }
            Ok(())
        }
        9 => {
            plain_npc::play(npc, "FireRing", false);
            if plain_npc::event_triggered(npc, "Shoot") {
                fire_ring(npc, random, world, births);
            }
            if plain_npc::finished(npc) {
                npc.state = 3;
            }
            Ok(())
        }
        12 => summon(npc, state, random, world, births),
        _ => Ok(()),
    }
}

/// 4e05e0: Idle; on the global frame (every 20, or 10 under 30% HP) a
/// player more than 40 px below has a 1-in-4 Spit, then a 1-in-3 FireRing,
/// then, with at most four countables, a 1-in-2 Summon every other period.
fn idle(
    npc: &mut Npc,
    state: &mut MegaMawState,
    random: &mut Mt19937,
    world: &NpcWorld,
) -> Result<(), String> {
    plain_npc::play(npc, "Idle", false);
    let period = if 0.3 > npc.hp / npc.max_hp { 10 } else { 20 };
    if world.player.y - 40.0 > npc.position.y
        && global_frame(world, period)
        && random.bounded(4) == 0
    {
        npc.state = 8;
        state.phase = 0;
        return Ok(());
    }
    if global_frame(world, period) && random.bounded(3) == 0 {
        npc.state = 9;
        state.phase = 0;
        return Ok(());
    }
    if world.countable_total <= 4 && global_frame(world, period * 2) && random.bounded(2) == 0 {
        npc.state = 12;
        state.phase = 0;
    }
    Ok(())
}

/// 4e0ad0: aim at the player (100 px lower unless it is mostly below),
/// then 48d5c0 lobs 15 shots with h = 0 and default params; SFX 114.
fn spit(npc: &Npc, random: &mut Mt19937, world: &mut NpcWorld, births: &mut Vec<Birth>) {
    let mut target = world.player;
    let aim = normalized(WorldVector {
        x: target.x - npc.position.x,
        y: target.y - npc.position.y,
    });
    if 0.3 > aim.y {
        target.y += 100.0;
    }
    let parameter_angle = (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
    for _ in 0..15 {
        let radius = random.uniform();
        let jitter = (direction(random) * radius) * 3.5;
        let velocity = normalized(WorldVector {
            x: target.x - npc.position.x,
            y: target.y - npc.position.y,
        }) * 7.0
            + jitter;
        let spawn_seed = random.next_u32();
        let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
        let falling_speed = ((-random.uniform() * 30.0) * 0.8) + 5.0;
        let scale_choice = random.bounded(2);
        let scale_jitter = random.uniform() * 0.05;
        births.push(Birth::Projectile(ProjectileBirth {
            position: npc.position,
            velocity,
            spawn_seed,
            initial_falling_speed,
            parameter_angle,
            falling_speed,
            falling_acceleration: 0.32,
            scale: (scale_choice as f32 * 0.4 + 0.9) + scale_jitter,
            height: -23.0,
        }));
    }
    sound(world, random, 114, 2);
}

/// 4e0810: SFX 304, then the fire params (height +18.5, falling speed -2,
/// acceleration -0.06, speed x1.05, flags 0x2080, centre Mega Maw, variant
/// 2) and bounded(4) picks the FireProjectiles mode-9 rings.
fn fire_ring(npc: &Npc, random: &mut Mt19937, world: &mut NpcWorld, births: &mut Vec<Birth>) {
    sound(world, random, 304, 2);
    let parameter_angle = (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
    let mut flags = FLAG_PARENTED | FLAG_NO_GRID;
    let mut ring = |flags: u32, speed: f32, count: u32, random: &mut Mt19937| {
        let params = ProjectileFlags {
            flags,
            centre: npc.position,
        };
        // Mode 9 (4a09a8): `count` shots around the circle, the first one
        // step past the params angle, each keeping its Init fall.
        let step = (2.0 * std::f64::consts::PI / f64::from(count)) as f32;
        let mut angle = parameter_angle;
        for _ in 0..count {
            angle += step;
            let velocity = WorldVector {
                x: speed * crate::fmath::cos(angle),
                y: speed * crate::fmath::sin(angle),
            };
            let spawn_seed = random.next_u32();
            let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
            births.push(Birth::FlaggedProjectile(
                ProjectileBirth {
                    position: npc.position,
                    velocity: velocity * 1.05,
                    spawn_seed,
                    initial_falling_speed,
                    parameter_angle,
                    falling_speed: initial_falling_speed + -2.0,
                    falling_acceleration: -0.06,
                    scale: 1.0,
                    height: -23.0 + 18.5,
                },
                FIRE,
                params,
            ));
        }
    };
    match random.bounded(4) {
        0 => ring(flags | FLAG_ORBIT, 10.0, 9, random),
        1 => ring(flags | FLAG_ORBIT_REVERSE, 10.0, 9, random),
        2 => {
            // The params are shared, so the second ring carries both bits.
            flags |= FLAG_ORBIT;
            ring(flags, 10.0, 5, random);
            flags |= FLAG_ORBIT_REVERSE;
            ring(flags, 7.0, 5, random);
        }
        _ => ring(flags, 10.0, 9, random),
    }
}

/// 4e0340: SummonTell (SFX 312), then Summon, whose Shoot event spawns a
/// Hopper 29.0 at Mega Maw that leaps toward a free cell 150 px below.
fn summon(
    npc: &mut Npc,
    state: &mut MegaMawState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    if state.phase == 0 {
        plain_npc::play(npc, "SummonTell", false);
        sound(world, random, 312, 60);
        if plain_npc::finished(npc) {
            state.phase = 1;
        }
        return Ok(());
    }
    if state.phase != 1 {
        return Ok(());
    }
    plain_npc::play(npc, "Summon", false);
    if plain_npc::event_triggered(npc, "Shoot") {
        let spawn_seed = random.next_u32();
        let costs = world
            .path_costs
            .as_deref()
            .ok_or("Mega Maw summon requires source Room.Init path costs")?;
        let target = crate::roundworm::find_free_position(
            npc.position + WorldVector { x: 0.0, y: 150.0 },
            40.0,
            random,
            world.grid,
            costs,
            world.occupied,
        )?;
        births.push(Birth::Hop {
            selector: SpawnSelector {
                group_index: 0,
                grid_index: 0,
                category: SpawnCategory::Entity,
                entity_type: 29,
                variant: 0,
                subtype: 0,
                position: npc.position,
                group_seed: spawn_seed,
                spawn_seed,
            },
            target,
        });
        sound(world, random, 114, 2);
    }
    if plain_npc::finished(npc) {
        npc.state = 3;
        state.phase = 0;
    }
    Ok(())
}
