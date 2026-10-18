//! Mega Fatty 264.0: ai 4dd290. It idles until a roll picks SUCK (it pulls
//! the room's actors in, then pukes a fountain of puke shots), FART (Dips
//! when the room is nearly empty, otherwise a lobbed puke volley, and a hurt
//! within 100 px) or JUMP (it flies at the player's lead and lands with five
//! red creep pools). The AI never draws the NPC's own RNG.
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
    State,
};
use serde::{Deserialize, Serialize};

pub(crate) const FAMILY: Family = Family {
    kinds: &[264],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::Items,
};

/// Projectile 9.3, the puke shot of PUKE and of the FART lob.
const PUKE: u32 = 3;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MegaFattyState {
    /// +f58: only IDLE counts.
    pub counter: i32,
    /// +f78: the jump's lerp parameter.
    pub jump_t: f32,
    /// +f88: 0 waits for the Jump event, 1 moves at once and may land at
    /// EarlyTransition, 2 the clip finished, 3 land early.
    pub jump_mode: i32,
}

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 264
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Mega Fatty requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(264, 0);
    let mut npc = Npc {
        animation: "Idle".into(),
        actor: config.actor.into(),
        mega_fatty: Some(MegaFattyState {
            counter: 0,
            jump_t: 0.0,
            jump_mode: 0,
        }),
        ..Npc::authored(id, config, selector)
    };
    // stageHP 0: 600 on every floor.
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

fn state(npc: &mut Npc) -> &mut MegaFattyState {
    npc.mega_fatty.as_mut().expect("Mega Fatty owns its state")
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

fn sub(a: WorldVector, b: WorldVector) -> WorldVector {
    WorldVector {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}

fn length(vector: WorldVector) -> f32 {
    (vector.x * vector.x + vector.y * vector.y).sqrt()
}

/// 612820: the reciprocal is rounded before it scales the vector.
fn normalized(vector: WorldVector) -> WorldVector {
    let length = length(vector);
    if length > 0.0 {
        vector * (1.0 / length)
    } else {
        vector
    }
}

/// MT.angle 42e8d0: one shared float, rounded to f32 before cos/sin.
fn direction(random: &mut Mt19937) -> WorldVector {
    let angle = (f64::from(random.uniform()) * std::f64::consts::TAU) as f32;
    WorldVector {
        x: crate::fmath::cos(angle),
        y: crate::fmath::sin(angle),
    }
}

/// ProjectileParams+18: (f32)((f64)MT.float·π), the first draw of a volley.
fn parameter_angle(random: &mut Mt19937) -> f32 {
    (f64::from(random.uniform()) * std::f64::consts::PI) as f32
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.state == 0 {
        // INIT 4dd350 also writes +314 = 5.0 (a render offset); no tail.
        npc.state = 3;
        state(npc).counter = 0;
        return Ok(());
    }
    match npc.state {
        3 => idle(npc, random, world),
        15 => {
            plain_npc::play(npc, "Sucking", false);
            if plain_npc::event_was_triggered(npc, "StartSucking")
                && !plain_npc::event_was_triggered(npc, "EndSucking")
            {
                births.push(Birth::Suck(npc.position));
            }
            if plain_npc::finished(npc) {
                npc.state = 8;
            }
        }
        8 => {
            plain_npc::play(npc, "Puking", false);
            if plain_npc::event_triggered(npc, "Shoot") {
                sound(world, random, 311, 1.0);
            }
            if plain_npc::event_was_triggered(npc, "Shoot")
                && !plain_npc::event_was_triggered(npc, "StopShooting")
            {
                births.push(puke(npc, random, world));
            }
            if plain_npc::finished(npc) {
                npc.state = 3;
                state(npc).counter = 0;
            }
        }
        6 => jump(npc, random, world),
        7 => {
            plain_npc::play(npc, "Landing", false);
            npc.entity_collision_class = 4;
            npc.grid_collision_class = 5;
            if plain_npc::event_triggered(npc, "Landed") {
                stomp(npc, random, world, births)?;
            }
            if plain_npc::finished(npc) {
                npc.state = 3;
                state(npc).counter = 0;
            }
        }
        10 => {
            plain_npc::play(npc, "Fart", false);
            if plain_npc::event_triggered(npc, "Fart") {
                fart(npc, random, world, births)?;
            }
            if plain_npc::finished(npc) {
                npc.state = 3;
                state(npc).counter = 0;
            }
        }
        _ => {}
    }
    *friction *= 0.5;
    Ok(())
}

/// IDLE 4dd790: from the 32nd update a 1-in-15 roll (or 1-in-5 more while
/// damage is pending) picks by weight between SUCK, FART, JUMP and nothing.
fn idle(npc: &mut Npc, random: &mut Mt19937, world: &mut NpcWorld) {
    plain_npc::play(npc, "Idle", false);
    let counter = state(npc).counter;
    state(npc).counter = counter.wrapping_add(1);
    if counter <= 30 {
        return;
    }
    if random.bounded(15) != 0 && !(npc.damage.pending_damage() > 0.0 && random.bounded(5) == 0) {
        return;
    }
    // 4dd816 measures the player's absolute position, not its distance.
    let distance = length(world.player);
    let (suck_and_none, fart) = if distance < 120.0 {
        (110, 80)
    } else {
        (90, 60)
    };
    let jump = if distance > 240.0 { 40 } else { 20 };
    let roll = random.bounded(suck_and_none + fart + jump);
    if roll <= 29 {
        npc.state = 15;
    } else if roll < fart {
        npc.state = 10;
        sound(world, random, 302, 1.0);
    } else if roll < fart + jump {
        npc.state = 6;
        npc.target = world.player + world.player_velocity * 10.0;
        let mode = random.bounded(2) as i32;
        let state = state(npc);
        state.jump_t = 0.0;
        state.jump_mode = mode;
    }
}

/// PUKE shot 4dd9f0, FireProjectiles mode 0: a high fountain toward the
/// player, jittered harder as the clip advances and leading its velocity.
fn puke(npc: &Npc, random: &mut Mt19937, world: &NpcWorld) -> Birth {
    let delta = sub(world.player, npc.position);
    let t = ((npc.animation_frame as f32 - 10.0) / 15.0).clamp(0.0, 1.0);
    let spread = 1.0 + (3.0 - 1.0) * t;
    let parameter_angle = parameter_angle(random);
    let scale = random.uniform() + 0.5;
    let radius = random.uniform();
    let jitter = (direction(random) * radius) * spread;
    let velocity = (delta * 0.025 + jitter) + world.player_velocity * 0.6;
    let spawn_seed = random.next_u32();
    let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
    Birth::VariantProjectile(
        ProjectileBirth {
            position: npc.position,
            velocity,
            spawn_seed,
            initial_falling_speed,
            parameter_angle,
            // Params +4 -300, +8 -40, +c 2.0 over 49ec10's -23, -3, 0.1.
            falling_speed: -43.0,
            falling_acceleration: 2.1,
            scale,
            height: -323.0,
        },
        PUKE,
    )
}

/// JUMP 4dd610: lerp toward the target with both collision classes off,
/// landing once within √2 (after the clip in mode 0, at EarlyTransition in
/// mode 1).
fn jump(npc: &mut Npc, random: &mut Mt19937, world: &mut NpcWorld) {
    let mut mode = state(npc).jump_mode;
    let mut landing_test = true;
    if mode <= 1 {
        plain_npc::play(npc, "Jumping", false);
        if plain_npc::event_triggered(npc, "EarlyTransition") && mode == 1 {
            mode = 3;
        } else if plain_npc::finished(npc) {
            mode = 2;
        } else if mode == 0 {
            landing_test = false;
        }
        state(npc).jump_mode = mode;
    }
    if landing_test {
        let delta = sub(npc.position, npc.target);
        if delta.x * delta.x + delta.y * delta.y < 2.0 || mode == 3 {
            npc.state = 7;
        }
    }
    if plain_npc::event_triggered(npc, "Jump") {
        sound(world, random, 303, 1.0);
    }
    if mode != 0 || plain_npc::event_was_triggered(npc, "Jump") {
        let t = (state(npc).jump_t + 0.02).min(1.0);
        state(npc).jump_t = t;
        npc.velocity = sub(npc.target * t + npc.position * (1.0 - t), npc.position);
        npc.entity_collision_class = 0;
        npc.grid_collision_class = 0;
    }
}

/// LANDING "Landed" 4dde10: five size-2 red creep pools within 75 px, the
/// screen shake and the stomp sounds.
fn stomp(
    npc: &Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    for _ in 0..5 {
        let seed = random.next_u32();
        let radius = random.uniform() * 75.0;
        let position = npc.position + direction(random) * radius;
        world.effects.admit_creep(seed, position, (0, 0), 2.0)?;
        sound(world, random, 303, 1.0);
        sound(world, random, 48, 0.8);
    }
    births.push(Birth::Shake(10));
    sound(world, random, 48, 1.0);
    // PORT GAP: Room::SetShockwave 5cc640 is a render-only floor distortion.
    Ok(())
}

/// FART event 4de494: the fart effect, then one or two Dips (a coin flip
/// with at most three countables) or a 30-shot puke lob, then a hurt to a
/// player within 100 px.
fn fart(
    npc: &Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    // PORT GAP: Effect 1000.34 Fart (cosmetic) takes this seed but is not
    // admitted; its Init's FlipX roll and fart sound (458ebd) still run.
    random.next_u32();
    random.bounded(2);
    sound(world, random, 37, 1.0);
    if world.countable_total <= 3 && random.bounded(2) == 0 {
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
        }
    } else {
        let target = WorldVector {
            x: world.player.x,
            y: world.player.y.min(npc.position.y - 80.0),
        };
        lob(npc, random, target, births);
    }
    let delta = sub(world.player, npc.position);
    if delta.x * delta.x + delta.y * delta.y < 10000.0 {
        births.push(Birth::PlayerHurt);
    }
    Ok(())
}

/// 48d5c0 with count 30, h = 15 and the puke params (scale MT.float + 0.5,
/// variant 3): higher, heavier arcs than Monstro's h = 0 volley.
fn lob(npc: &Npc, random: &mut Mt19937, target: WorldVector, births: &mut Vec<Birth>) {
    const HEIGHT: f32 = 15.0;
    let parameter_angle = parameter_angle(random);
    let parameter_scale = random.uniform() + 0.5;
    for _ in 0..30 {
        let radius = random.uniform();
        let jitter = (direction(random) * radius) * 3.5;
        let velocity = normalized(sub(target, npc.position)) * 7.0 + jitter;
        let spawn_seed = random.next_u32();
        let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
        let falling_speed = (((-random.uniform() * 30.0) * 0.8) + 5.0) - HEIGHT;
        let scale_choice = random.bounded(2);
        let scale_jitter = random.uniform() * 0.05;
        let scale = ((scale_choice as f32 * 0.4 + 0.9) + scale_jitter) * parameter_scale;
        births.push(Birth::VariantProjectile(
            ProjectileBirth {
                position: npc.position,
                velocity,
                spawn_seed,
                initial_falling_speed,
                parameter_angle,
                falling_speed,
                falling_acceleration: 0.1 * HEIGHT + 0.32,
                scale,
                height: -23.0,
            },
            PUKE,
        ));
    }
}

/// SUCK 4dd438 over the Room entity list: the player, tears, projectiles,
/// pickups and every live NPC but fireplaces 33 gain
/// `normalize(center - pos)·k / friction`.
/// PORT GAP: the port applies the pull after Mega Fatty's update, owner by
/// owner, with the player's and tears' friction taken as 1.0; the source
/// walks the room list in its order mid-update with each actor's current
/// friction.
pub(crate) fn suck(state: &mut State, center: WorldVector) {
    let pull = |position: WorldVector| {
        let delta = sub(center, position);
        let k = (((600.0 - length(delta)) * 0.9) / 600.0).clamp(0.0, 0.9);
        normalized(delta) * k
    };
    if !state.player.dead {
        let velocity = pull(crate::bombs::player_world(state));
        state.player.vx += f64::from(velocity.x) * crate::motion::WORLD_SCALE;
        state.player.vy += f64::from(velocity.y) * crate::motion::WORLD_SCALE;
    }
    for tear in &mut state.tears {
        tear.motion.velocity = tear.motion.velocity + pull(tear.motion.position);
    }
    for projectile in state
        .enemy_projectiles
        .iter_mut()
        .filter(|projectile| !projectile.dead && !projectile.removed)
    {
        projectile.velocity = projectile.velocity + pull(projectile.position);
    }
    for pickup in state
        .room
        .pickups
        .iter_mut()
        .filter(|pickup| !pickup.removed && !pickup.collected && pickup.current_friction != 0.0)
    {
        pickup.velocity = pickup.velocity + divided(pull(pickup.position), pickup.current_friction);
    }
    for npc in state
        .room
        .npcs
        .iter_mut()
        .filter(|npc| !npc.dead && !npc.removed && npc.kind != 33 && npc.friction != 0.0)
    {
        npc.velocity = npc.velocity + divided(pull(npc.position), npc.friction);
    }
}

/// 612b30 divides each component.
fn divided(vector: WorldVector, denominator: f32) -> WorldVector {
    WorldVector {
        x: vector.x / denominator,
        y: vector.y / denominator,
    }
}
