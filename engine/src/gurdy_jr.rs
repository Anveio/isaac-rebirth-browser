//! Gurdy Jr. 99.0: ai 4e36c0. From its decision state Gurdy Jr idles,
//! sprays an eight-shot ring or spits a Pooter, and every third decision
//! (always under 100 HP) charges the player: a 4 px/update push while
//! Attack03Start plays, then up to ten Slide clips while it still moves.
//! Past its tenth update in a state it bleeds a red splat every fifth
//! frame. The AI never draws the NPC's own RNG.
use crate::{
    audio::SoundRequest,
    content,
    enemy_projectiles::ProjectileBirth,
    entities::{Birth, Family, NpcWorld, SpawnContext, SpawnedNpc},
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    phase::Phase,
    plain_npc,
    room_spawns::{SpawnCategory, SpawnSelector},
};
use serde::{Deserialize, Serialize};

pub(crate) const FAMILY: Family = Family {
    kinds: &[99],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GurdyJrState {
    /// +f58: updates in the current state.
    pub counter: i32,
    /// +f68: decisions since the last charge.
    pub decisions: i32,
    /// +f8c: the charge phase (0 push, 1 slide, 2 end).
    pub phase: i32,
    /// +f88: slide checks this charge.
    pub slides: i32,
    /// +440: the charge push toward the player.
    pub push: WorldVector,
}

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    // Subtypes 1 (a twin spawned 40 px away) and 2 (Super Pooters, no ring)
    // are authored by no base room and stay out.
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 99
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Gurdy Jr. requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(99, 0);
    let mut npc = Npc {
        animation: "Idle".into(),
        actor: config.actor.into(),
        gurdy_jr: Some(GurdyJrState {
            counter: 0,
            decisions: 0,
            phase: 0,
            slides: 0,
            push: WorldVector { x: 0.0, y: 0.0 },
        }),
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

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    let mut state = npc
        .gurdy_jr
        .take()
        .ok_or("Gurdy Jr. requires its AI state")?;
    let result = step(npc, &mut state, random, world, births, friction);
    npc.gurdy_jr = Some(state);
    result
}

fn step(
    npc: &mut Npc,
    state: &mut GurdyJrState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.state == 0 {
        // 4e36e8: decide on the next update.
        npc.state = 3;
        state.counter = 0;
        return Ok(());
    }
    if npc.dead {
        sound(world, random, 117, 1.2);
    }
    state.counter = state.counter.wrapping_add(1);
    if state.counter > 10 && plain_npc::on_frame(npc, world, 5)? {
        // 4e3878: three floats, the first sizing 480fe0's red splat.
        let size = random.uniform();
        random.uniform();
        random.uniform();
        let seed = random.next_u32();
        let id = world
            .effects
            .admit_blood(seed, npc.position, [1.0; 4], [0.0; 3])?;
        world.effects.set_blood_scale(id, (size + size) + 0.4)?;
        world.effects.advance_blood_initial(id, random)?;
    }
    if npc.state == 8 {
        // 4e38b0: a charge only loses speed on every fifth global frame.
        if world.game_counter.is_multiple_of(5) {
            *friction *= 0.97;
        }
    } else {
        *friction *= 0.8;
    }
    // PORT GAP: fear and confusion (+318 & 0xa00) end a charge early and
    // block a new one; the port admits neither status.
    match npc.state {
        3 => {
            // 4e38f0: every third decision, or any under 100 HP, charges.
            state.decisions = state.decisions.wrapping_add(1);
            if state.decisions > 2 || npc.hp < 100.0 {
                charge(npc, state, world, friction);
            } else {
                // 4e3f50: Idle, Attack02, and Attack01 while at most five
                // countables remain.
                let mut choices = vec![6, 9];
                if world.countable_total <= 5 {
                    choices.push(10);
                }
                let pick = random.bounded(choices.len() as u32) as usize;
                npc.state = choices[pick];
                state.counter = 0;
            }
        }
        6 => {
            plain_npc::play(npc, "Idle", false);
            if plain_npc::finished(npc) {
                npc.state = 3;
            }
        }
        8 => slide(npc, state, friction),
        9 => {
            plain_npc::play(npc, "Attack02", false);
            if state.counter == 7 {
                sound(world, random, 14, 1.0);
            }
            if state.counter == 18 {
                sound(world, random, 48, 1.0);
                sound(world, random, 178, 1.0);
                ring(npc, random, births);
            }
            if plain_npc::finished(npc) {
                npc.state = 3;
            }
        }
        10 => {
            plain_npc::play(npc, "Attack01", false);
            if state.counter == 13 {
                sound(world, random, 265, 1.1);
                spit(npc, random, births);
            }
            if plain_npc::finished(npc) {
                npc.state = 3;
            }
        }
        _ => {}
    }
    Ok(())
}

/// 4e3c28: aim a 4 px push at the player and start the charge.
fn charge(npc: &mut Npc, state: &mut GurdyJrState, world: &NpcWorld, friction: &mut f32) {
    npc.state = 8;
    state.phase = 0;
    state.decisions = 0;
    let away = WorldVector {
        x: npc.position.x - world.player.x,
        y: npc.position.y - world.player.y,
    };
    let distance = (away.x * away.x + away.y * away.y).sqrt();
    *friction *= 0.7;
    plain_npc::play(npc, "Attack03Start", false);
    state.push = away * -(4.0 / distance);
    state.slides = 0;
}

/// 4e39f0: push while Attack03Start plays; once a clip finishes, slide
/// again while faster than 2 px/update for at most ten checks, then end.
fn slide(npc: &mut Npc, state: &mut GurdyJrState, friction: &mut f32) {
    if !plain_npc::finished(npc) {
        if state.phase == 0 {
            *friction *= 0.8;
            npc.velocity = npc.velocity
                + WorldVector {
                    x: state.push.x / *friction,
                    y: state.push.y / *friction,
                };
        }
        return;
    }
    match state.phase {
        0 => state.phase = 1,
        1 => {
            let speed = npc.velocity.x * npc.velocity.x + npc.velocity.y * npc.velocity.y;
            let checks = state.slides;
            if speed > 4.0 && checks <= 9 {
                state.slides = checks + 1;
                plain_npc::play(npc, "Slide", true);
            } else {
                if speed > 4.0 {
                    state.slides = checks + 1;
                }
                state.phase = 2;
                plain_npc::play(npc, "Attack03End", false);
            }
        }
        _ => npc.state = 3,
    }
}

/// 4e4082: default ProjectileParams draw their angle, then FireProjectiles
/// mode 8 (4a04b0) spawns eight shots at speed 10, keeping their Init fall.
fn ring(npc: &Npc, random: &mut Mt19937, births: &mut Vec<Birth>) {
    let parameter_angle = (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
    let straight = 10.0_f32;
    let diagonal = straight * 0.707_106_77;
    let velocities = [
        (straight, 0.0),
        (-straight, 0.0),
        (0.0, straight),
        (0.0, -straight),
        (diagonal, diagonal),
        (-diagonal, diagonal),
        (diagonal, -diagonal),
        (-diagonal, -diagonal),
    ];
    for (x, y) in velocities {
        let spawn_seed = random.next_u32();
        let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
        births.push(Birth::Projectile(ProjectileBirth {
            position: npc.position,
            velocity: WorldVector { x, y },
            spawn_seed,
            initial_falling_speed,
            parameter_angle,
            falling_speed: initial_falling_speed,
            falling_acceleration: 0.0,
            scale: 1.0,
            height: -23.0,
        }));
    }
}

/// 4e3ec3: Spawn550a40 of a Pooter 14.0 at (-30, -30) moving up, with no
/// spawner and its appear kept.
fn spit(npc: &Npc, random: &mut Mt19937, births: &mut Vec<Birth>) {
    let spawn_seed = random.next_u32();
    births.push(Birth::Spawned(SpawnedNpc {
        selector: SpawnSelector {
            group_index: 0,
            grid_index: 0,
            category: SpawnCategory::Entity,
            entity_type: 14,
            variant: 0,
            subtype: 0,
            position: npc.position + WorldVector { x: -30.0, y: -30.0 },
            group_seed: spawn_seed,
            spawn_seed,
        },
        velocity: WorldVector { x: 0.0, y: -10.0 },
        spawner_type: 0,
        boss_spawned: false,
        scale: 1.0,
        health_scale: 1.0,
        max_health_scale: 1.0,
        health_delta: 0.0,
        appear: true,
    }));
}
