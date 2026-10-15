//! Dingle261.0: ai_dingle4ad900 on the plain NPC base. It idles, then spits a
//! FireProjectiles mode2 fan, whistles up Dips, dances, or slides at the
//! player three times (dropping a poop on the first slide) and tires. A brown
//! Blood7 splat trails it every fifth frame.
use crate::{
    audio::SoundRequest,
    content,
    enemy_projectiles::ProjectileBirth,
    entities::{Birth, Family, NpcWorld, SpawnContext},
    grid::GridEntity,
    items::Rng,
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    phase::Phase,
    plain_npc,
    room_spawns::{SpawnCategory, SpawnSelector},
};
use serde::{Deserialize, Serialize};

pub(crate) const FAMILY: Family = Family {
    kinds: &[261],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::Items,
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DingleState {
    /// Entity_NPC+f58 state frame counter (also the slide countdown).
    pub counter: i32,
    /// Entity_NPC+f88 slides taken this cycle.
    pub slides: i32,
    /// SplatColor+3cc tint and offsets.
    pub tint: [f32; 4],
    pub offsets: [f32; 3],
    /// Entity+7c0 own RNG (spawn seed, shifts 4/3/17), drawn for the poop seed.
    pub rng: Rng,
}

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 261
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Dingle requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(261, 0);
    let mut npc = Npc {
        // Config47f573 SetFrame(default "Idle", 0) leaves the clip stopped.
        animation: "Idle".into(),
        actor: config.actor.into(),
        dingle: Some(DingleState {
            counter: 0,
            slides: 0,
            tint: [1.0; 4],
            offsets: [0.0; 3],
            rng: Rng::new(selector.spawn_seed, [4, 3, 17]),
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

fn sound(random: &mut Mt19937, world: &mut NpcWorld, id: u32, volume: f32, delay: i32) {
    world.audio.admit(
        random,
        world.system_counter,
        SoundRequest {
            id,
            volume,
            delay,
            looping: false,
            pitch: 1.0,
        },
    );
}

fn normalized(vector: WorldVector) -> WorldVector {
    let length = (vector.x * vector.x + vector.y * vector.y).sqrt();
    if length > 0.0 {
        vector * (1.0 / length)
    } else {
        vector
    }
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    let mut state = npc.dingle.take().ok_or("Dingle requires its AI state")?;
    let result = step(npc, &mut state, random, world, births, friction);
    npc.dingle = Some(state);
    result
}

fn step(
    npc: &mut Npc,
    state: &mut DingleState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.state == 0 {
        // 4ada20: the brown SplatColor; the initializer skips the tail.
        npc.state = 3;
        state.counter = 0;
        state.slides = 0;
        let unit = f32::from_bits(0x3b80_8081);
        state.tint = [0.0, 0.0, 0.0, 255.0 * unit];
        state.offsets = [65.0 * unit, 41.0 * unit, 35.0 * unit];
        return Ok(());
    }
    npc.flip_x = false;
    match npc.state {
        3 => {
            plain_npc::play(npc, "Idle", false);
            let old = state.counter;
            state.counter += 1;
            if old > 30 {
                // 4ae570: a hurt Dingle also attacks one time in ten.
                let attack = random.bounded(20) == 0
                    || (npc.damage.pending_damage() > 0.0 && random.bounded(10) == 0);
                if attack {
                    npc.state = if random.bounded(3) == 0 {
                        9
                    } else if random.bounded(2) == 0 {
                        // 4ae7f6: flags0xa00 never hold for an ordinary Dingle.
                        11
                    } else {
                        10
                    };
                }
            }
        }
        8 => {
            plain_npc::play(npc, "Spit", false);
            let delta = WorldVector {
                x: world.player.x - npc.position.x,
                y: world.player.y - npc.position.y,
            };
            if plain_npc::event_triggered(npc, "Shoot") {
                spit(npc, random, world, births, delta);
            }
            if plain_npc::event_was_triggered(npc, "Shoot") {
                npc.flip_x = delta.x > 0.0;
            }
            if plain_npc::finished(npc) {
                npc.state = 3;
                state.counter = 0;
            }
        }
        9 => {
            plain_npc::play(npc, "Whistle", false);
            sound(random, world, 315, 1.0, 60);
            // Room+16ac + Room+16b0 counts Dingle itself.
            if plain_npc::event_triggered(npc, "Spawn") && world.countable_total <= 4 {
                whistle_dips(npc, random, world, births)?;
            }
            if plain_npc::finished(npc) {
                npc.state = 8;
                state.counter = 0;
            }
        }
        10 | 11 => {
            let (clip, next) = if npc.state == 10 {
                ("Dance", 8)
            } else {
                ("BeforeSlide", 4)
            };
            plain_npc::play(npc, clip, false);
            sound(random, world, 72, 0.6, 60);
            if plain_npc::finished(npc) {
                npc.state = next;
                state.counter = 0;
            }
        }
        4 => slide(npc, state, random, world, friction)?,
        15 => {
            plain_npc::play(npc, "Tired", false);
            let old = state.counter;
            let rest = random.bounded(30) as i32;
            state.counter += 1;
            if old > rest + 60 {
                npc.state = 3;
                state.counter = 0;
            }
        }
        // 4ad972: any other state runs the tail only.
        _ => {}
    }
    // Tail 4ad972: the brown trail, then friction unless (now) sliding.
    if plain_npc::on_frame(npc, world, 5)? {
        splat(npc, state, random, world)?;
    }
    if npc.state != 4 {
        *friction *= 0.7;
    }
    Ok(())
}

/// State4 (4adae0): a countdown between slides, then the slide body 4ae120.
fn slide(
    npc: &mut Npc,
    state: &mut DingleState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    friction: &mut f32,
) -> Result<(), String> {
    if state.counter != 0 {
        *friction *= 0.9;
        state.counter -= 1;
        if state.counter == 0 {
            plain_npc::play(npc, "Slide", true);
        }
        return Ok(());
    }
    plain_npc::play(npc, "Slide", false);
    if plain_npc::event_triggered(npc, "Slide") {
        slide_event(npc, state, random, world)?;
    }
    *friction *= 0.98;
    if plain_npc::event_was_triggered(npc, "Slide") {
        let push = normalized(npc.velocity);
        if *friction != 0.0 {
            npc.velocity = npc.velocity
                + WorldVector {
                    x: push.x / *friction,
                    y: push.y / *friction,
                };
        }
    }
    npc.flip_x = npc.velocity.x > 0.0;
    if plain_npc::on_frame(npc, world, 2)? {
        splat(npc, state, random, world)?;
    }
    if plain_npc::finished(npc) {
        if state.slides > 1 {
            npc.state = 15;
            state.counter = 0;
            state.slides = 0;
        } else {
            // 4ae82e: slide again after a 12..15 update countdown.
            let pause = random.bounded(4) as i32;
            state.slides += 1;
            state.counter = pause + 12;
        }
    }
    Ok(())
}

/// 4ae6b0: lunge at the player; the first slide may drop a poop in its cell.
fn slide_event(
    npc: &mut Npc,
    state: &mut DingleState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
) -> Result<(), String> {
    let toward = normalized(WorldVector {
        x: world.player.x - npc.position.x,
        y: world.player.y - npc.position.y,
    });
    npc.velocity = npc.velocity + toward * 10.0;
    sound(random, world, 301, 1.0, 30);
    if state.slides == 0 && random.bounded(5) == 0 {
        let seed = state.rng.raw();
        let index = world
            .grid
            .index(npc.position)
            .ok_or("Dingle stands outside the room grid")?;
        let costs = world
            .path_costs
            .as_deref_mut()
            .ok_or("Dingle poop requires source Room.Init path costs")?;
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
                .ok_or("Dingle poop requires the room grid")?
                .cells[index]
                .entity = Some(poop);
        }
        sound(random, world, 37, 1.0, 2);
    }
    Ok(())
}

/// 480fe0 then SetColor: a brown Blood7 splat at Dingle, scaled 0.2..0.7.
fn splat(
    npc: &Npc,
    state: &DingleState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
) -> Result<(), String> {
    let scale = random.uniform() * 0.5 + 0.2;
    let seed = random.next_u32();
    let id = world
        .effects
        .admit_blood(seed, npc.position, [1.0; 4], [0.0; 3])?;
    world.effects.set_blood_scale(id, scale)?;
    world.effects.advance_blood_initial(id, random)?;
    world.effects.set_blood_color(id, state.tint, state.offsets)
}

/// 4ae428: default ProjectileParams draw their angle, then FireProjectiles
/// mode2 (49f37d) fans three shots at speed 10. Mode2's extra pass sets
/// falling speed -4 and inherits 0.4 of Dingle's velocity.
fn spit(
    npc: &Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    delta: WorldVector,
) {
    let parameter_angle = (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
    let aim = normalized(delta) * 10.0;
    let (cos, sin, scale, spread) = (0.961_f32, 0.276_f32, 0.9_f32, 1.0_f32);
    let (x, y) = (aim.x, aim.y);
    let velocities = [
        aim,
        WorldVector {
            x: (x * cos) * scale - ((y * sin) * scale) * spread,
            y: ((sin * x) * scale) * spread + (cos * y) * scale,
        },
        WorldVector {
            x: (x * cos) * scale + ((y * sin) * scale) * spread,
            y: (cos * y) * scale - ((sin * x) * scale) * spread,
        },
    ];
    for velocity in velocities {
        let spawn_seed = random.next_u32();
        let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
        births.push(Birth::Projectile(ProjectileBirth {
            position: npc.position,
            velocity: velocity * 1.2 + npc.velocity * 0.4,
            spawn_seed,
            initial_falling_speed,
            parameter_angle,
            falling_speed: -4.0,
            falling_acceleration: 0.1,
            scale: 1.0,
        }));
    }
    sound(random, world, 303, 1.0, 2);
}

/// 4ae010: one Dip at a free spot within 60+ of Dingle, a second on a coin
/// flip. Each seed is drawn before its position.
fn whistle_dips(
    npc: &Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    for index in 0..2 {
        if index == 1 && random.bounded(2) != 0 {
            break;
        }
        let spawn_seed = random.next_u32();
        let position = crate::roundworm::find_free_position(
            npc.position,
            60.0,
            random,
            world.grid,
            world
                .path_costs
                .as_deref()
                .ok_or("Dip spawn requires source Room.Init path costs")?,
            world.occupied,
        )?;
        births.push(Birth::Npc(SpawnSelector {
            group_index: 0,
            grid_index: 0,
            category: SpawnCategory::Entity,
            entity_type: 217,
            variant: 0,
            subtype: 0,
            position,
            group_seed: spawn_seed,
            spawn_seed,
        }));
        if index == 0 {
            sound(random, world, 72, 0.6, 2);
        }
    }
    Ok(())
}
