//! Peep 68.0 and its eyes 68.10: ai 4c3150's Peep path (shared with The
//! Bloat 68.1/68.11, not admitted). Peep drifts on a slow persistent wander,
//! jumps at the player and lands with a 12-shot ring, fires an 8-shot ring
//! or spills yellow creep, and leaves a yellow splat every update. Each
//! third of its HP lost pops an invincible eye that bounces diagonally until
//! Peep dies. The AI never draws the NPC's own RNG.
use crate::{
    audio::SoundRequest,
    content,
    enemy_projectiles::ProjectileBirth,
    entities::{Birth, Family, NpcWorld, SpawnContext},
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    phase::Phase,
    plain_npc,
    room_spawns::{SpawnCategory, SpawnSelector},
};
use serde::{Deserialize, Serialize};

pub(crate) const FAMILY: Family = Family {
    kinds: &[68],
    spawn,
    advance,
    post_contact_grid,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };
const EYE: u32 = 10;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PeepState {
    /// +f58: the per-state counter.
    pub counter: i32,
    /// +f88: eyes popped so far.
    pub eyes: i32,
    /// pf+0x10: the wander direction.
    pub direction: WorldVector,
    /// Eye +f78: last update's velocity.
    pub last_velocity: WorldVector,
    /// Eye +f8c: the diagonal jitter in degrees.
    pub jitter: i32,
}

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 68
        || !matches!(selector.variant, 0 | 10)
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Peep requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(68, selector.variant.into());
    let mut npc = Npc {
        animation: "Walk".into(),
        actor: config.actor.into(),
        peep: Some(PeepState {
            counter: 0,
            eyes: 0,
            direction: ZERO,
            last_velocity: ZERO,
            jitter: 0,
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

/// Eyes use grid class 3 (walls and doors only); Peep walks the ground.
fn post_contact_grid(npc: &mut Npc, grid: &crate::grid::Grid) -> Result<(), String> {
    if npc.grid_collision_class == 3 {
        crate::entities::wall_collision(npc, grid)
    } else {
        crate::entities::ground_collision(npc, grid)
    }
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

/// 42e8d0: one MT.float turned into a unit vector.
fn direction(random: &mut Mt19937) -> WorldVector {
    let angle = (f64::from(random.uniform()) * std::f64::consts::PI * 2.0) as f32;
    WorldVector {
        x: crate::fmath::cos(angle),
        y: crate::fmath::sin(angle),
    }
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    let mut state = npc.peep.take().ok_or("Peep requires its AI state")?;
    let result = step(npc, &mut state, random, world, births, friction);
    npc.peep = Some(state);
    result
}

fn step(
    npc: &mut Npc,
    state: &mut PeepState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.state == 0 {
        // 4c316e: an eye is invincible and only collides with walls.
        npc.state = 4;
        if npc.variant == EYE {
            npc.invincible = true;
            npc.grid_collision_class = 3;
        }
        return Ok(());
    }
    if npc.variant == EYE {
        return eye(npc, state, random, world, friction);
    }
    // 2: the ground friction uses the collision class from before this
    // update's state body.
    if npc.entity_collision_class != 0 {
        *friction *= 0.6;
    }
    let target = world.player;
    match npc.state {
        4 => {
            wander(npc, state, random, world, *friction)?;
            plain_npc::play(npc, "Walk", false);
            state.counter = 0;
            if random.bounded(23) == 0 {
                npc.state = 6;
            }
            if random.bounded(20) == 0 {
                npc.state = if random.bounded(2) == 0 { 8 } else { 12 };
            }
        }
        6 => {
            if state.counter == 0 {
                plain_npc::play(npc, "JumpUp", true);
                sound(world, random, 15, 1.0);
            }
            state.counter += 1;
            if plain_npc::finished(npc) {
                state.counter = 0;
                npc.state = 7;
            } else if state.counter >= 30 {
                npc.entity_collision_class = 0;
                if state.counter == 30 {
                    npc.velocity = WorldVector {
                        x: target.x - npc.position.x,
                        y: target.y - npc.position.y,
                    } * 0.033;
                }
            }
        }
        7 => {
            if state.counter == 0 {
                plain_npc::play(npc, "JumpDown", true);
            }
            state.counter += 1;
            if plain_npc::finished(npc) {
                npc.state = 4;
            }
            if state.counter >= 28 {
                npc.entity_collision_class = 4;
                if state.counter == 28 {
                    // PORT GAP: the render-only Room::SetShockwave is not
                    // admitted, nor the shots' yellow colorize.
                    sound(world, random, 48, 1.0);
                    ring(npc, random, births, 12);
                }
            }
        }
        8 => {
            if state.counter == 0 {
                plain_npc::play(npc, "Attack01", true);
                sound(world, random, 15, 1.0);
            }
            state.counter += 1;
            if plain_npc::finished(npc) {
                npc.state = 4;
            }
            if state.counter == 38 {
                sound(world, random, 48, 0.8);
                ring(npc, random, births, 8);
            }
        }
        12 => {
            if state.counter == 0 {
                plain_npc::play(npc, "Attack02", true);
            }
            state.counter += 1;
            if state.counter == 10 {
                sound(world, random, 149, 1.0);
            }
            if plain_npc::finished(npc) {
                npc.state = 4;
            }
            if (31..=39).contains(&state.counter) {
                // 4c3f60: a wasted angle draw, then a pool within 100 px.
                direction(random);
                let seed = random.next_u32();
                let radius = random.uniform() * 100.0;
                let offset = direction(random) * radius;
                // PORT GAP: the pool's immediate first update (its clip
                // roll and contact) runs on the scheduler's next pass.
                world
                    .effects
                    .admit_yellow_creep(seed, npc.position + offset, 2.0)?;
            }
        }
        _ => {}
    }
    tail(npc, state, random, world, births)
}

/// TAIL 4c33d0: the yellow splat every update, then the eyes by HP third.
fn tail(
    npc: &mut Npc,
    state: &mut PeepState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    let scale = random.uniform() * 0.5 + 0.2;
    let seed = random.next_u32();
    let id = world
        .effects
        .admit_blood(seed, npc.position, [1.0; 4], [0.0; 3])?;
    world.effects.set_blood_scale(id, scale)?;
    world.effects.advance_blood_initial(id, random)?;
    let offset = direction(random) * 2.0;
    world
        .effects
        .move_blood(id, npc.position + offset + WorldVector { x: 0.0, y: 8.0 })?;
    let unit = f32::from_bits(0x3b80_8081);
    world
        .effects
        .set_blood_color(id, [0.0, 0.0, 0.0, 0.7], [200.0 * unit, 200.0 * unit, 0.0])?;
    let thirds = ((3.0 * npc.hp) / npc.max_hp) as i32;
    let wanted = (2 - thirds).min(2);
    // PORT GAP: Peep's _b/_c reskins (one, then both eyes missing) are not
    // admitted; the renderer keeps the full sheet.
    while state.eyes < wanted {
        sound(world, random, 237, 1.0);
        state.eyes += 1;
        let spawn_seed = random.next_u32();
        let position = npc.position + direction(random) * 30.0;
        births.push(Birth::Parented(SpawnSelector {
            group_index: 0,
            grid_index: 0,
            category: SpawnCategory::Entity,
            entity_type: 68,
            variant: EYE as u16,
            subtype: 0,
            position,
            group_seed: spawn_seed,
            spawn_seed,
        }));
    }
    Ok(())
}

/// EYE 4c3710: dies with Peep; otherwise a diagonal bounce at 45° ± 15°
/// along the signs of its velocity.
fn eye(
    npc: &mut Npc,
    state: &mut PeepState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    friction: &mut f32,
) -> Result<(), String> {
    plain_npc::play(npc, "Walk", false);
    let parent_alive = npc
        .parent
        .and_then(|id| world.peers.get(id))
        .is_some_and(|parent| !parent.dead);
    if !parent_alive {
        npc.dead = true;
    }
    if random.bounded(4) == 0 {
        let scale = random.uniform() * 0.3 + 0.3;
        let seed = random.next_u32();
        let id = world
            .effects
            .admit_blood(seed, npc.position, [1.0; 4], [0.0; 3])?;
        world.effects.set_blood_scale(id, scale)?;
        world.effects.advance_blood_initial(id, random)?;
    }
    let velocity = npc.velocity;
    let last = state.last_velocity;
    if (velocity.x > 0.0 && last.x <= 0.0) || (velocity.y > 0.0 && last.y <= 0.0) {
        state.jitter = random.bounded(31) as i32 - 15;
    }
    let angle = (f64::from(state.jitter as f32) * std::f64::consts::PI / 180.0
        + std::f64::consts::FRAC_PI_4) as f32;
    let speed = 0.636_396_050_453_186_f64;
    let mut push = WorldVector {
        x: (f64::from(crate::fmath::cos(angle)) * speed) as f32,
        y: (f64::from(crate::fmath::sin(angle)) * speed) as f32,
    };
    if velocity.x <= 0.0 || velocity.x.is_nan() {
        push.x = -push.x;
    }
    if velocity.y <= 0.0 || velocity.y.is_nan() {
        push.y = -push.y;
    }
    *friction *= 0.9;
    npc.velocity = npc.velocity
        + WorldVector {
            x: push.x / *friction,
            y: push.y / *friction,
        };
    state.last_velocity = npc.velocity;
    Ok(())
}

/// FireProjectiles mode 9 (4a09a8): `count` shots at speed 10 around the
/// circle, the first one step past the params angle, keeping their Init fall.
fn ring(npc: &Npc, random: &mut Mt19937, births: &mut Vec<Birth>, count: u32) {
    let parameter_angle = (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
    let step = (2.0 * std::f64::consts::PI / f64::from(count)) as f32;
    let mut angle = parameter_angle;
    for _ in 0..count {
        angle += step;
        let velocity = WorldVector {
            x: 10.0 * crate::fmath::cos(angle),
            y: 10.0 * crate::fmath::sin(angle),
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
            scale: 1.0,
            height: -23.0,
        }));
    }
}

/// 5b6830: a slow wander in a persistent direction that turns 1 time in 30
/// and probes 10 px ahead for obstacles.
fn wander(
    npc: &mut Npc,
    state: &mut PeepState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    friction: f32,
) -> Result<(), String> {
    // PORT GAP: charm (FindGridPath) and fear (EvadeTarget) take over the
    // wander; the port admits neither status.
    if state.direction == ZERO {
        state.direction = direction(random);
    }
    let push = state.direction * 0.5;
    if friction != 0.0 {
        npc.velocity = npc.velocity
            + WorldVector {
                x: push.x / friction,
                y: push.y / friction,
            };
    }
    if world.grid.collision_at(npc.position) != 0 {
        if random.bounded(30) == 0 {
            let costs = world
                .path_costs
                .as_deref()
                .ok_or("Peep wander requires source Room.Init path costs")?;
            let free = crate::roundworm::find_free_position(
                npc.position,
                0.0,
                random,
                world.grid,
                costs,
                world.occupied,
            )?;
            state.direction = crate::npcs::normalized(WorldVector {
                x: free.x - npc.position.x,
                y: free.y - npc.position.y,
            });
        }
        return Ok(());
    }
    if random.bounded(30) == 0 {
        state.direction = crate::npcs::normalized(state.direction + direction(random));
    }
    if npc.grid_collided && plain_npc::on_frame(npc, world, 5)? {
        state.direction = state.direction * -1.0;
    }
    for attempt in 0..6 {
        if world
            .grid
            .collision_at(npc.position + state.direction * 10.0)
            == 0
            || attempt == 5
        {
            return Ok(());
        }
        state.direction = crate::npcs::normalized(state.direction * -1.0 + direction(random));
        npc.velocity = ZERO;
    }
    Ok(())
}
