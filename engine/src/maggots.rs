//! Maggot 21.0, Charger 23.0, Spitty 31.0 and Gurgling 237.0/237.1:
//! ai_maggot_leech 4c6520.
//! Ground walkers that wander with MoveRandomly and mark their cell with
//! path cost 900. A Charger rushes along an axis toward the player's
//! predicted position; a Spitty stops to spit one cardinal shot. A Gurgling
//! charges like a Charger, skids to a Stop that fires a six-shot ring
//! (237.0) or releases Attack Flies (the 237.1 boss), and splats blood
//! while moving fast.
use crate::{
    audio::SoundRequest,
    content,
    enemy_projectiles::ProjectileBirth,
    entities::{Birth, Family, NpcWorld, SpawnContext},
    gluttony::axis,
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
    kinds: &[21, 23, 31, 237],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };
const WALK_CLIPS: [&str; 3] = ["Move Hori", "Move Up", "Move Down"];
const ATTACK_CLIPS: [&str; 3] = ["Attack Hori", "Attack Up", "Attack Down"];
/// t170/t190, indexed up, down, left, right.
const GURGLING_WALK: [&str; 4] = ["WalkVert", "WalkVert", "WalkLeft", "WalkRight"];
const GURGLING_CHARGE: [&str; 4] = ["ChargeUp", "ChargeDown", "ChargeLeft", "ChargeRight"];
/// 42e9e0 dir4: left, up, right, down.
const DIRECTIONS: [WorldVector; 4] = [
    WorldVector { x: -1.0, y: 0.0 },
    WorldVector { x: 0.0, y: -1.0 },
    WorldVector { x: 1.0, y: 0.0 },
    WorldVector { x: 0.0, y: 1.0 },
];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MaggotState {
    /// Entity_NPC+f58, incremented at the end of every non-init update.
    pub counter: i32,
    /// +f78 Charger charge vector (axis * 1.3).
    pub charge: WorldVector,
    /// +f88 Spitty attack direction index latched at the spit start.
    pub attack_direction: usize,
}

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || !matches!(
            (selector.entity_type, selector.variant),
            (21 | 23 | 31 | 237, 0) | (237, 1)
        )
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err(
            "Maggot family requires bounded authored position and nonzero spawn seed".into(),
        );
    }
    let config = content::entity(selector.entity_type.into(), selector.variant.into());
    let mut npc = Npc {
        // Config47f573 SetFrame(default, 0) leaves the default clip stopped.
        animation: if selector.entity_type == 237 {
            "WalkVert"
        } else {
            "Move Hori"
        }
        .into(),
        actor: config.actor.into(),
        pathfinder: Some(Pathfinder::new()),
        maggot: Some(MaggotState {
            counter: 0,
            charge: ZERO,
            attack_direction: 0,
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
    plain_npc::advance(npc, phase, random, world, births, ai)
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.state == 0 {
        npc.state = 4;
        return Ok(());
    }
    let mut state = npc
        .maggot
        .take()
        .ok_or("Maggot family requires its AI state")?;
    let result = update(npc, &mut state, random, world, births, friction);
    npc.maggot = Some(state);
    result
}

fn sound(random: &mut Mt19937, world: &mut NpcWorld, id: u32, delay: i32) {
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

fn update(
    npc: &mut Npc,
    state: &mut MaggotState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    let speed = match npc.kind {
        21 => 0.75,
        23 | 237 => 0.6,
        _ => 0.72,
    };
    let index = world
        .grid
        .index(npc.position)
        .ok_or("Maggot family walker left the room grid")?;
    world
        .path_costs
        .as_deref_mut()
        .ok_or("Maggot family requires source Room.Init path costs")?
        .store(index, 900)?;
    // 4c67a6: a Gurgling has left/right clips and never touches FlipX.
    let gurgling = if npc.velocity.x.abs() > npc.velocity.y.abs() {
        if npc.velocity.x >= 0.0 {
            3
        } else {
            2
        }
    } else if npc.velocity.y >= 0.0 {
        1
    } else {
        0
    };
    // 4c67a1: strict horizontal test sets FlipX; vertical leaves it alone.
    let direction = if npc.kind == 237 {
        0
    } else if npc.velocity.x.abs() > npc.velocity.y.abs() {
        npc.flip_x = 0.0 > npc.velocity.x;
        0
    } else if npc.velocity.y >= 0.0 {
        2
    } else {
        1
    };
    match npc.state {
        4 => {
            if npc.kind == 237 {
                plain_npc::play(npc, GURGLING_WALK[gurgling], false);
            } else {
                plain_npc::play(npc, WALK_CLIPS[direction], false);
            }
            pathfinder::move_randomly(npc, random, world, speed, *friction)?;
            *friction *= 0.85;
            npc.velocity.y *= 0.82 / *friction;
            match npc.kind {
                23 | 237 => charge_check(npc, state, random, world, index)?,
                // 4c7850: bounded(35) every walking update.
                31 if random.bounded(35) == 0 => {
                    npc.state = 9;
                    state.counter = 0;
                    plain_npc::play(npc, ATTACK_CLIPS[direction], true);
                    state.attack_direction = direction;
                }
                _ => {}
            }
        }
        8 => {
            if npc.grid_collided {
                // 4c6c80: the previous grid pass hit a solid; stop at once.
                if let Some(path) = npc.pathfinder.as_mut() {
                    path.target = ZERO;
                }
                if npc.kind != 237 {
                    npc.state = 4;
                    state.counter += 1;
                    return Ok(());
                }
                // 4c7966: a Gurgling skids into Stop and runs the tail.
                plain_npc::play(npc, "Stop", false);
                npc.state = 3;
                return tail(npc, state, random, world, births);
            }
            *friction *= if npc.kind == 23 { 0.8 } else { 0.9 };
            npc.velocity = npc.velocity
                + WorldVector {
                    x: state.charge.x / *friction,
                    y: state.charge.y / *friction,
                };
            // Pull toward the occupied cell's centre along the travel axis.
            let centre = world.grid.position(index).expect("valid grid center");
            if npc.velocity.x.abs() > npc.velocity.y.abs() {
                npc.position.x = 0.9 * npc.position.x + centre.x * 0.1;
            } else {
                npc.position.y = 0.9 * npc.position.y + centre.y * 0.1;
            }
            if npc.kind == 237 {
                plain_npc::play(npc, GURGLING_CHARGE[gurgling], false);
            } else {
                plain_npc::play(npc, ATTACK_CLIPS[direction], false);
            }
        }
        3 => {
            *friction *= 0.85;
            if npc.animation_frame == 10 {
                if npc.variant == 0 {
                    ring(npc, random, world, births);
                } else {
                    release_flies(npc, random, births);
                }
            }
            if plain_npc::finished(npc) {
                npc.state = 4;
            }
        }
        9 => {
            *friction *= 0.85;
            if state.counter == 8 {
                spit(npc, state, random, world, births);
            } else if state.counter > 22 {
                npc.state = 4;
            }
        }
        _ => {}
    }
    tail(npc, state, random, world, births)
}

/// 4c67ee: the counter ticks; a fast Gurgling splats blood on every fifth
/// game update (no sequence offset) and, unless it is the boss walking,
/// leaves red creep.
fn tail(
    npc: &mut Npc,
    state: &mut MaggotState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    state.counter += 1;
    let speed = npc.velocity.x * npc.velocity.x + npc.velocity.y * npc.velocity.y;
    if npc.kind != 237 || speed <= 9.0 || !world.game_counter.is_multiple_of(5) {
        return Ok(());
    }
    let scale = random.uniform() * 0.39 + 0.2;
    let seed = random.next_u32();
    let id = world
        .effects
        .admit_blood(seed, npc.position, [1.0; 4], [0.0; 3])?;
    world.effects.set_blood_scale(id, scale)?;
    world.effects.advance_blood_initial(id, random)?;
    if npc.variant == 0 || npc.state == 8 {
        let seed = random.next_u32();
        let creep = world
            .effects
            .admit_creep(seed, npc.position, (npc.kind, npc.variant), 1.23)?;
        if world
            .effects
            .advance_creep_initial(creep, random, world.game_counter, world.player)?
        {
            births.push(Birth::CreepHit);
        }
        // SetColor copies the Gurgling's colour; alpha 1 unless champion.
        // Entity+3a0 is the base colour, not the timed hit flash.
        let color = npc.base_color();
        let mut tint = color.tint;
        if npc.champion.is_none() {
            tint[3] = 1.0;
        }
        world.effects.set_creep_color(creep, tint, color.offset)?;
    }
    Ok(())
}

/// 4c7190: on is_frame(4) aim at the player's position led by its velocity;
/// rush when aligned within 20 on an axis, under 200 away and in sight.
fn charge_check(
    npc: &mut Npc,
    state: &mut MaggotState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    index: usize,
) -> Result<(), String> {
    if !plain_npc::on_frame(npc, world, 4)? {
        return Ok(());
    }
    let centre = world.grid.position(index).expect("valid grid center");
    let lead = random.uniform();
    let predicted = world.player + (world.player_velocity * lead) * 16.0;
    let delta = WorldVector {
        x: predicted.x - centre.x,
        y: predicted.y - centre.y,
    };
    if !(20.0 > delta.x.abs() || 20.0 > delta.y.abs()) {
        return Ok(());
    }
    if delta.x * delta.x + delta.y * delta.y >= 40000.0 {
        return Ok(());
    }
    let costs = world
        .path_costs
        .as_deref()
        .ok_or("Charger requires source Room.Init path costs")?;
    if !world.grid.charge_line_clear(centre, predicted, costs) {
        return Ok(());
    }
    sound(random, world, if npc.kind == 237 { 57 } else { 224 }, 20);
    state.charge = axis(delta) * 1.3;
    npc.state = 8;
    Ok(())
}

/// 4c6f40: two spit sounds, then one mode0 shot along a cardinal direction.
fn spit(
    npc: &Npc,
    state: &MaggotState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) {
    sound(random, world, 178, 2);
    sound(random, world, 319, 2);
    let parameter_angle = (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
    let heading = match state.attack_direction {
        0 if npc.flip_x => 0,
        0 => 2,
        1 => 1,
        _ => 3,
    };
    let spawn_seed = random.next_u32();
    let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
    births.push(Birth::Projectile(ProjectileBirth {
        position: npc.position,
        velocity: DIRECTIONS[heading] * 8.0,
        spawn_seed,
        initial_falling_speed,
        parameter_angle,
        falling_speed: -3.0_f32 + -0.08_f32,
        falling_acceleration: 0.1,
        scale: if npc.champion.is_some() { 1.4 } else { 1.0 },
        height: -23.0 + 14.0,
    }));
}

/// 4c762c: FireProjectiles mode 9, a six-shot ring at 7.5 that keeps the
/// projectile Init falling speed.
fn ring(npc: &Npc, random: &mut Mt19937, world: &mut NpcWorld, births: &mut Vec<Birth>) {
    sound(random, world, 178, 2);
    let parameter_angle = (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
    let step = (2.0 * std::f64::consts::PI / 6.0) as f32;
    let mut angle = parameter_angle;
    for _ in 0..6 {
        angle += step;
        let velocity = WorldVector {
            x: 7.5 * crate::fmath::cos(angle),
            y: 7.5 * crate::fmath::sin(angle),
        };
        let spawn_seed = random.next_u32();
        let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
        births.push(Birth::Projectile(ProjectileBirth {
            position: npc.position,
            velocity,
            spawn_seed,
            initial_falling_speed,
            parameter_angle,
            falling_speed: initial_falling_speed,
            falling_acceleration: 0.0,
            scale: if npc.champion.is_some() { 1.4 } else { 1.0 },
            height: -23.0,
        }));
    }
}

/// 4c7615: up to two Attack Flies 40 out along random angles, flung at 2
/// with APPEAR cleared; the bound is redrawn on every test.
fn release_flies(npc: &Npc, random: &mut Mt19937, births: &mut Vec<Birth>) {
    let mut count = 0;
    while count < random.bounded(3) {
        let direction = crate::npcs::direction(random);
        count += 1;
        let spawn_seed = random.next_u32();
        births.push(Birth::FlungNpc(
            SpawnSelector {
                group_index: 0,
                grid_index: 0,
                category: SpawnCategory::Entity,
                entity_type: 18,
                variant: 0,
                subtype: 0,
                position: npc.position + direction * 40.0,
                group_seed: spawn_seed,
                spawn_seed,
            },
            direction * 2.0,
        ));
    }
}
