//! Room.Init's ambient critters (5cf64f..5cf9ad): Tiny Bug 1000.21, Tiny Fly
//! 1000.33 and Wall Bug 1000.68. Each is an Effect whose Update 45aa70 runs
//! the common BaseUpdate and then its own AI case. Every random choice below
//! draws the shared MT in source order; the counts use the room's own RNG35.
use crate::{
    actors::{ActorKind, ActorRef},
    familiars::{self, definition, Sprite, SpriteEffect},
    items,
    motion::WorldVector,
    phase::Phase,
    State,
};
use serde::{Deserialize, Serialize};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };
/// 68ef44: degrees to radians.
const RADIANS: f32 = 0.017_453_292;

/// The Effect fields the critter cases own.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Critter {
    /// Effect+7d4: the bugs' run timer; Blood Explosion's sound latch.
    pub timer: i32,
    /// Effect+814: the bugs' heading in degrees.
    pub rotation: f32,
    /// Effect+440: the previous position, or a Wall Bug's (depth, perimeter).
    pub previous: WorldVector,
    /// Entity+31f: the critter has been killed.
    pub dying: bool,
    /// Sprite layer0+34: a killed Tiny Fly's fall offset.
    pub fall: f32,
    /// Spawned by the run loader: the original's first full update after
    /// admission (Game+ce9f4 = 1) has no runtime counterpart, so the next
    /// update replays it first.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub loader_update_pending: bool,
}

/// 6126c0: the unit vector of an angle in degrees.
fn angle_vector(degrees: f32) -> WorldVector {
    let radians = degrees * RADIANS;
    WorldVector {
        x: crate::fmath::cos(radians),
        y: crate::fmath::sin(radians),
    }
}

/// 42edd0: wrap an angle into (-180, 180].
fn wrap_angle(mut angle: f32) -> f32 {
    if angle.abs() > 360.0 {
        angle %= 360.0;
    }
    if angle > 180.0 {
        angle - 360.0
    } else if angle < -180.0 {
        angle + 360.0
    } else {
        angle
    }
}

/// 42eeb0: the shortest-way interpolation between two wrapped angles.
fn lerp_angle(from: f32, to: f32, t: f32) -> f32 {
    let mut from = wrap_angle(from);
    let to = wrap_angle(to);
    let mut delta = to - from;
    if delta.abs() > 180.0 {
        if from < to {
            from += 360.0;
            delta = to - from;
        } else {
            delta = (360.0 + to) - from;
        }
    }
    wrap_angle(delta * t + from)
}

/// 612560: a + (b - a) * t per component.
fn lerp(t: f32, a: WorldVector, b: WorldVector) -> WorldVector {
    WorldVector {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
    }
}

/// 6129d0: a - b per component.
fn minus(a: WorldVector, b: WorldVector) -> WorldVector {
    WorldVector {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}

fn distance_squared(a: WorldVector, b: WorldVector) -> f32 {
    (a.x - b.x) * (a.x - b.x) + (a.y - b.y) * (a.y - b.y)
}

/// AddVelocity 4589a0 and its inlined copies divide by the current friction.
fn add_velocity(effect: &mut SpriteEffect, impulse: WorldVector) {
    let friction = effect.current_friction;
    if friction != 0.0 {
        effect.velocity = effect.velocity
            + WorldVector {
                x: impulse.x / friction,
                y: impulse.y / friction,
            };
    }
}

/// Room+1c..28: the authored interior rectangle in world units.
fn room_rect(state: &State) -> (WorldVector, WorldVector) {
    let (width, height) = state
        .room
        .descriptor
        .as_ref()
        .map_or((13, 7), |descriptor| {
            (descriptor.template.width, descriptor.template.height)
        });
    (
        WorldVector { x: 60.0, y: 140.0 },
        WorldVector {
            x: 60.0 + f32::from(width) * 40.0,
            y: 140.0 + f32::from(height) * 40.0,
        },
    )
}

/// 5c7e50: clamp into the room rectangle inset by a margin on every side.
fn clamp_to_room(state: &State, position: WorldVector, margin: f32) -> WorldVector {
    let (low, high) = room_rect(state);
    WorldVector {
        x: position.x.min(high.x - margin).max(low.x + margin),
        y: position.y.min(high.y - margin).max(low.y + margin),
    }
}

/// 5c6fa0: the grid index of a world position, or -1 outside the grid.
fn grid_index(state: &State, position: WorldVector) -> Option<usize> {
    let grid = state.room.grid.as_ref()?;
    let column = ((position.x - 40.0) / 40.0 + 0.5).floor() as i32;
    let row = ((position.y - 120.0) / 40.0 + 0.5).floor() as i32;
    if column < 0 || row < 0 || column >= i32::from(grid.width) || row >= i32::from(grid.height) {
        return None;
    }
    Some(column as usize + row as usize * usize::from(grid.width))
}

/// 5c7080 -> 5c5af0: the grid collision class at a world position.
fn grid_class(state: &State, position: WorldVector) -> u32 {
    let Some(index) = grid_index(state, position) else {
        return 4;
    };
    let grid = state.room.grid.as_ref().expect("indexed grid");
    if let Some(entity) = grid.cells.get(index).and_then(|cell| cell.entity.as_ref()) {
        return entity.collision_class;
    }
    let cost = state
        .room
        .path_costs
        .as_ref()
        .and_then(|costs| costs.get(index).ok());
    match cost {
        Some(3000) => 1,
        Some(cost) if cost >= 1000 => 3,
        Some(_) => 0,
        None => crate::grid::Grid::new(grid.clone()).map_or(0, |grid| grid.collision(index)),
    }
}

struct Admission {
    variant: u32,
    seed: u32,
    position: WorldVector,
    tint: [f32; 4],
    offsets: [f32; 3],
    critter: bool,
}

/// Plain Game.Spawn of an effect with no subclass controller, such as the
/// Tear.Update 542530 poof. The caller owns the seed and any Init draws.
pub(crate) fn admit_plain(
    state: &mut State,
    variant: u32,
    seed: u32,
    position: WorldVector,
) -> Result<u32, String> {
    admit(
        state,
        Admission {
            variant,
            seed,
            position,
            tint: [1.0; 4],
            offsets: [0.0; 3],
            critter: false,
        },
    )
}

fn admit(state: &mut State, admission: Admission) -> Result<u32, String> {
    let Admission {
        variant,
        seed,
        position,
        tint,
        offsets,
        critter,
    } = admission;
    let config = definition(1000, variant).ok_or("critter effect config absent")?;
    let id = state.next_effect;
    state.next_effect = id.checked_add(1).ok_or("effect handle exhausted")?;
    let mut sprite = Sprite::new(config);
    sprite.tint = tint;
    sprite.offsets = offsets;
    state.sprite_effects.push(SpriteEffect {
        id,
        variant,
        subtype: 0,
        spawn_seed: seed,
        spawner_type: 0,
        spawner_variant: 0,
        parent: None,
        birth_game_update: state.game_update_count,
        actor_rng: items::Rng::new(seed, [4, 3, 17]),
        position,
        presented_position: position,
        render_offset: ZERO,
        velocity: ZERO,
        configured_friction: config.friction,
        current_friction: config.friction,
        collision_radius: config.collision_radius,
        // Effect.Init 458dd8 gives 21/33/68 mass 1 and entity class 4.
        collision_mass: if matches!(variant, 21 | 33 | 68) {
            1.0
        } else {
            config.collision_mass
        },
        collision_damage: 22.0,
        hp: config.base_hp,
        max_hp: config.base_hp,
        grid_collision_class: 0,
        entity_collision_class: if matches!(variant, 21 | 33 | 68) {
            4
        } else {
            0
        },
        flags: 0,
        visible: true,
        active: true,
        removed: false,
        // Effect.Init458da8 places Wall Bugs at render offset 5000.
        render_order_offset: if variant == 68 { 5000 } else { 0 },
        lifetime_divisor: 0,
        time_to_live: -1,
        raw_7f4: 1,
        raw_7f8: 0.0,
        raw_7d8: variant == 68,
        sprite,
        full_updates: 0,
        poop_effect: None,
        blood_effect: None,
        footprint: false,
        critter: critter.then_some(Critter {
            timer: 0,
            rotation: 0.0,
            previous: ZERO,
            dying: false,
            fall: 0.0,
            loader_update_pending: false,
        }),
    });
    state.actor_scheduler.admit(ActorRef {
        kind: ActorKind::Effect,
        id,
    })?;
    Ok(id)
}

/// Room.Init 5cf64f, on the descriptor's first visit after the ambient grid
/// decoration. `counter` is Game+ce9f4 while this Room.Init runs; `loader`
/// marks the run loader's starting room.
pub(crate) fn spawn(state: &mut State, counter: u32, loader: bool) -> Result<(), String> {
    let Some(descriptor) = state.room.descriptor.as_ref() else {
        return Ok(());
    };
    let room_type = descriptor.template.room_type;
    // Room types 5, 9, 18/19 and 21 first run their own Room.Init extras
    // (boss SFX190, arcade, statues, dice floor), then rejoin this spawner.
    let start_room = descriptor.index == state.floor.starting_descriptor;
    let mut rng = items::Rng::new(descriptor.spawn_seed, [5, 9, 7]);
    // Backdrop0x37 (Room+17c8) does not occur in stage1 Basement.
    if rng.bounded(2) != 0 || room_type == 16 {
        return Ok(());
    }
    let owned = |item: usize| {
        state
            .player_items
            .as_ref()
            .is_some_and(|items| items.owned_counts.get(item).is_some_and(|count| *count > 0))
    };
    let (range, multiplier, extra) = if owned(148) || owned(234) {
        (30, 2, 1)
    } else {
        (15, 1, 0)
    };
    let divisor = if start_room { 2 } else { 1 };
    let draw = |rng: &mut items::Rng, bound: u32| {
        let limit = rng.bounded(bound) + 1;
        rng.bounded(limit)
    };
    let bugs = draw(&mut rng, range);
    let flies = draw(&mut rng, multiplier * 10);
    let wall_bugs = draw(&mut rng, multiplier * 10);
    // Three further room draws feed counts used only by other stages.
    let unused = rng.bounded(extra * 10 + 20);
    let unused = rng.bounded(unused + 1);
    rng.bounded(unused + 1);
    // Backdrop1 Basement's Room+1af8 flag admits Wall Bugs.
    let wall_bugs = wall_bugs / divisor;
    let (low, high) = room_rect(state);
    for (variant, count) in [(21, bugs / divisor), (33, flies / divisor)] {
        for _ in 0..count {
            let shared = &mut state.boot_lifecycle.shared;
            let vertical = shared.uniform();
            let horizontal = shared.uniform();
            let position = WorldVector {
                x: horizontal * (high.x - low.x) + low.x,
                y: (high.y - low.y) * vertical + low.y,
            };
            let seed = shared.next_u32();
            let id = admit(
                state,
                Admission {
                    variant,
                    seed,
                    position,
                    tint: [1.0; 4],
                    offsets: [0.0; 3],
                    critter: true,
                },
            )?;
            update(state, id, counter)?;
            effect_mut(state, id)?
                .critter
                .as_mut()
                .expect("critter")
                .loader_update_pending = loader;
        }
    }
    for _ in 0..wall_bugs {
        let seed = state.boot_lifecycle.shared.next_u32();
        let id = admit(
            state,
            Admission {
                variant: 68,
                seed,
                position: ZERO,
                tint: [1.0; 4],
                offsets: [0.0; 3],
                critter: true,
            },
        )?;
        // Effect.Init458da8 draws the wall depth and perimeter coordinate.
        let shared = &mut state.boot_lifecycle.shared;
        let perimeter = shared.uniform();
        let depth = shared.uniform();
        effect_mut(state, id)?
            .critter
            .as_mut()
            .expect("critter")
            .previous = WorldVector {
            x: depth * 40.0,
            y: perimeter * 10000.0,
        };
        update(state, id, counter)?;
        effect_mut(state, id)?
            .critter
            .as_mut()
            .expect("critter")
            .loader_update_pending = loader;
    }
    Ok(())
}

fn effect_mut(state: &mut State, id: u32) -> Result<&mut SpriteEffect, String> {
    state
        .sprite_effects
        .iter_mut()
        .find(|effect| effect.id == id)
        .ok_or_else(|| "critter effect is not admitted".into())
}

/// Scheduler callback for an admitted critter or its Blood Explosion.
pub(crate) fn advance(state: &mut State, phase: Phase, id: u32) -> Result<(), String> {
    match phase {
        Phase::Skipped => Ok(()),
        Phase::Interpolate => {
            let effect = effect_mut(state, id)?;
            if !effect.removed {
                effect.presented_position =
                    effect.position + (effect.velocity * effect.current_friction) * 0.5;
            }
            Ok(())
        }
        // Inside full callbacks Game+ce9f4 reads game_update_count+1.
        Phase::Update => update(state, id, state.game_update_count.wrapping_add(1)),
    }
}

/// The original's first full update after run admission (Game+ce9f4 = 1)
/// precedes the runtime's first one. Replay it for the starting room's
/// critters, in scheduler order, before that runtime update's actor pass.
pub(crate) fn replay_loader_update(state: &mut State) -> Result<(), String> {
    let pending: Vec<u32> = state
        .actor_scheduler
        .full_phase()
        .into_iter()
        .filter(|actor| actor.kind == ActorKind::Effect)
        .filter_map(|actor| {
            state
                .sprite_effects
                .iter()
                .find(|effect| effect.id == actor.id)
                .filter(|effect| {
                    effect
                        .critter
                        .as_ref()
                        .is_some_and(|critter| critter.loader_update_pending)
                })
                .map(|effect| effect.id)
        })
        .collect();
    for id in pending {
        effect_mut(state, id)?
            .critter
            .as_mut()
            .expect("critter")
            .loader_update_pending = false;
        update(state, id, state.game_update_count)?;
    }
    Ok(())
}

/// Effect::Update 45aa70: BaseUpdate 447190, then the variant's AI case.
fn update(state: &mut State, id: u32, counter: u32) -> Result<(), String> {
    let effect = effect_mut(state, id)?;
    if effect.removed {
        return Ok(());
    }
    let variant = effect.variant;
    let config = definition(1000, variant).ok_or("critter effect config absent")?;
    effect.full_updates = effect
        .full_updates
        .checked_add(1)
        .ok_or("effect full callback count exhausted")?;
    if effect.sprite.playing {
        effect.sprite.advance(config)?;
    }
    effect.velocity = effect.velocity * effect.current_friction;
    effect.position = effect.position + effect.velocity;
    effect.presented_position = effect.position;
    effect.current_friction = effect.configured_friction;
    match variant {
        2 => blood_explosion(state, id),
        21 => tiny_bug(state, id),
        33 => tiny_fly(state, id, counter),
        68 => wall_bug(state, id),
        _ => Err("critter variant has no AI case".into()),
    }?;
    let effect = effect_mut(state, id)?;
    if !effect.position.is_finite() || !effect.velocity.is_finite() {
        return Err("critter motion overflowed".into());
    }
    effect.presented_position = effect.position;
    Ok(())
}

/// Remove441bb0 clears active and marks the actor dead.
/// Game554490's effect branch 554e40: live Tiny Bugs and Tiny Flies in the
/// blast query (critter radius 0) whose line passes the 5545d1 check are
/// marked dead without any draw. Wall Bugs fall through to the no-op effect
/// TakeDamage.
pub(crate) fn blast(
    state: &mut State,
    center: WorldVector,
    radius: f32,
    clear: &dyn Fn(WorldVector) -> bool,
) {
    for effect in &mut state.sprite_effects {
        if effect.removed || !matches!(effect.variant, 21 | 33) || effect.critter.is_none() {
            continue;
        }
        let (dx, dy) = (effect.position.x - center.x, effect.position.y - center.y);
        if dx * dx + dy * dy < radius * radius && clear(effect.position) {
            remove(effect);
        }
    }
}

fn remove(effect: &mut SpriteEffect) {
    effect.active = false;
    effect.removed = true;
}

/// A killed critter leaves a Blood7 splat in its own color (SetColor before
/// the immediate Update), then removes itself.
fn splat(state: &mut State, id: u32, scale: Option<f32>, flags: u32) -> Result<(), String> {
    let seed = state.boot_lifecycle.shared.next_u32();
    let effect = effect_mut(state, id)?;
    let (position, tint, offsets) = (effect.position, effect.sprite.tint, effect.sprite.offsets);
    let blood = familiars::admit_blood_splat(state, seed, position, tint, offsets)?;
    if let Some(scale) = scale {
        familiars::set_blood_scale(state, blood, scale)?;
    }
    effect_mut(state, blood)?.flags |= flags;
    familiars::advance_blood_initial(state, blood)?;
    remove(effect_mut(state, id)?);
    Ok(())
}

/// Case 21 (45c090).
fn tiny_bug(state: &mut State, id: u32) -> Result<(), String> {
    let player = crate::bombs::player_world(state);
    let player_velocity = crate::bombs::player_velocity(state);
    let mut effect = effect_mut(state, id)?.clone();
    let mut critter = effect.critter.clone().expect("critter");
    let shared = &mut state.boot_lifecycle.shared;
    effect.current_friction *= 0.8;
    if critter.timer < 1 {
        if shared.bounded(30) == 0 {
            critter.timer = shared.bounded(120) as i32 + 15;
        } else if shared.bounded(5) == 0 {
            let speed = shared.uniform() * 0.5;
            add_velocity(&mut effect, angle_vector(critter.rotation) * speed);
        }
    } else {
        let speed = shared.uniform() * 0.5 + 0.5;
        add_velocity(&mut effect, angle_vector(critter.rotation) * speed);
        critter.timer -= 1;
    }
    let near = distance_squared(effect.position, player);
    if near < 10000.0 && shared.bounded(3) == 0 {
        let mut speed = 1.0 - near / 10000.0;
        speed = speed + speed;
        let away = crate::npcs::normalized(minus(effect.position, player));
        add_velocity(&mut effect, away * speed);
        critter.timer = shared.bounded(100) as i32 + 30;
    }
    critter.rotation = wrap_angle(shared.uniform() * 50.0 + critter.rotation - 25.0);
    effect.position = clamp_to_room(state, effect.position, -2.0);
    if grid_class(state, effect.position) != 0 {
        if let (Some(index), Some(grid)) =
            (grid_index(state, effect.position), state.room.grid.as_ref())
        {
            let width = usize::from(grid.width);
            let cell = WorldVector {
                x: (index % width) as f32 * 40.0 + 40.0,
                y: (index / width) as f32 * 40.0 + 120.0,
            };
            let delta = minus(cell, effect.position);
            if delta.x.abs() <= delta.y.abs() {
                effect.position.y = if delta.y <= 0.0 {
                    20.0 + cell.y
                } else {
                    cell.y - 20.0
                };
                effect.velocity.y *= 0.1;
            } else {
                effect.position.x = if delta.x <= 0.0 {
                    20.0 + cell.x
                } else {
                    cell.x - 20.0
                };
                effect.velocity.x *= 0.1;
            }
            // A Pit (grid type7) returns the bug to its previous position.
            // No supported room admits a Pit object yet.
            let pit = grid
                .cells
                .get(index)
                .and_then(|cell| cell.entity.as_ref())
                .is_some_and(|entity| entity.kind == 7);
            if pit {
                return Err("Tiny Bug pit recovery is outside supported grid objects".into());
            }
        }
    }
    critter.previous = effect.position;
    let killed = critter.dying;
    effect.critter = Some(critter.clone());
    *effect_mut(state, id)? = effect.clone();
    if killed {
        splat(state, id, None, 0)?;
        return Ok(());
    }
    // 46072d: a ground player moving faster than 1 stomps a nearby bug.
    if near < 100.0
        && player_velocity.x * player_velocity.x + player_velocity.y * player_velocity.y > 1.0
    {
        critter.dying = true;
    }
    // A dying tear within 20 units splashes the bug.
    if tears_within(state, effect.position, 1.0)
        .any(|(position, dying)| dying && distance_squared(position, effect.position) < 400.0)
    {
        critter.dying = true;
    }
    effect_mut(state, id)?.critter = Some(critter);
    Ok(())
}

/// QueryRadius 4536f0 over the tear partition: (position, dead) pairs whose
/// collision circle reaches within `radius` of the point.
fn tears_within(
    state: &State,
    point: WorldVector,
    radius: f32,
) -> impl Iterator<Item = (WorldVector, bool)> + '_ {
    state.tears.iter().filter_map(move |tear| {
        if tear.motion.lifetime == crate::projectiles::Lifetime::Removed {
            return None;
        }
        let reach = radius + tear.collision_radius;
        (distance_squared(point, tear.motion.position) < reach * reach).then_some((
            tear.motion.position,
            tear.motion.lifetime == crate::projectiles::Lifetime::Dying,
        ))
    })
}

/// Case 33 (45b790), shared with effect variants 0/8/18.
fn tiny_fly(state: &mut State, id: u32, counter: u32) -> Result<(), String> {
    let player = crate::bombs::player_world(state);
    let sequence = state
        .actor_scheduler
        .sequence_id(ActorRef {
            kind: ActorKind::Effect,
            id,
        })
        .ok_or("critter frame gate requires its manager sequence")?;
    let mut effect = effect_mut(state, id)?.clone();
    let mut critter = effect.critter.clone().expect("critter");
    let toward = minus(player, effect.position);
    let length = (toward.x * toward.x + toward.y * toward.y).sqrt();
    let unit = if length == 0.0 {
        WorldVector {
            x: toward.x / 0.01,
            y: toward.y / 0.01,
        }
    } else {
        WorldVector {
            x: toward.x / length,
            y: toward.y / length,
        }
    };
    effect.current_friction *= 0.98;
    let mut bursts = 0;
    if !critter.dying {
        let shared = &mut state.boot_lifecycle.shared;
        // Effect+498 (a parent target) is never set for room critters.
        if shared.bounded(5) == 0 {
            let speed = shared.uniform() * 0.5;
            add_velocity(&mut effect, unit * speed);
        }
        if counter.wrapping_add(sequence) % 5 == 0 {
            let speed = shared.uniform() * 0.9;
            let direction = crate::npcs::direction(shared);
            add_velocity(&mut effect, direction * speed);
        }
        if length < 60.0 {
            let push = (1.0 - (length * length) / 3600.0) * -0.2;
            add_velocity(&mut effect, unit * push);
        }
        let touched_tear = tears_within(state, effect.position, 0.5).next().is_some();
        let touched_fire = state.room.npcs.iter().any(|npc| {
            npc.kind == 33 && !npc.removed && {
                let reach = 2.0 + npc.collision_radius;
                distance_squared(npc.position, effect.position) < reach * reach
            }
        });
        // Each touching list kills and bursts independently: tears (45b9c4)
        // and Fire33; the knife8 loop (45bb58) has no runtime actors.
        bursts = usize::from(touched_tear) + usize::from(touched_fire);
    } else if critter.fall <= 16.0 {
        critter.fall += 0.25;
        let side = (sequence & 1) as f32 + (sequence & 1) as f32 - 1.0;
        let amount = critter.fall.abs() * 0.0625;
        let wobble = angle_vector(counter as f32 * 15.0 * side) * amount * 3.0;
        effect.position = effect.position + wobble;
        effect.current_friction *= 0.8;
    } else {
        effect.critter = Some(critter);
        *effect_mut(state, id)? = effect;
        return fly_tail(state, id, true);
    }
    effect.critter = Some(critter);
    *effect_mut(state, id)? = effect;
    for _ in 0..bursts {
        // A killed fly bursts into Blood Explosion2 at its own color and
        // stops its sprite while the layer falls.
        let seed = state.boot_lifecycle.shared.next_u32();
        let effect = effect_mut(state, id)?;
        effect.critter.as_mut().expect("critter").dying = true;
        effect.sprite.playing = false;
        let (position, tint, offsets) =
            (effect.position, effect.sprite.tint, effect.sprite.offsets);
        let burst = admit(
            state,
            Admission {
                variant: 2,
                seed,
                position: position + WorldVector { x: 0.0, y: -16.0 },
                tint,
                offsets,
                critter: true,
            },
        )?;
        effect_mut(state, burst)?.sprite.scale = WorldVector { x: 0.3, y: 0.3 };
        update(state, burst, counter)?;
    }
    fly_tail(state, id, false)
}

/// The fly's common tail: optional landing splat, facing, room clamp and the
/// previous position.
fn fly_tail(state: &mut State, id: u32, landed: bool) -> Result<(), String> {
    if landed {
        let seed = state.boot_lifecycle.shared.next_u32();
        let effect = effect_mut(state, id)?;
        let (position, tint, offsets) =
            (effect.position, effect.sprite.tint, effect.sprite.offsets);
        let blood = familiars::admit_blood_splat(state, seed, position, tint, offsets)?;
        familiars::set_blood_scale(state, blood, 0.1)?;
        familiars::advance_blood_initial(state, blood)?;
        remove(effect_mut(state, id)?);
    }
    let position = effect_mut(state, id)?.position;
    let clamped = clamp_to_room(state, position, 2.0);
    let effect = effect_mut(state, id)?;
    let critter = effect.critter.as_mut().expect("critter");
    effect.sprite.flip_x = minus(effect.position, critter.previous).x < 0.0;
    effect.position = clamped;
    critter.previous = clamped;
    Ok(())
}

/// Case 68 (45b130): the bug walks the wall band around the room.
fn wall_bug(state: &mut State, id: u32) -> Result<(), String> {
    let mut effect = effect_mut(state, id)?.clone();
    let mut critter = effect.critter.clone().expect("critter");
    effect.velocity = ZERO;
    let saved = critter.previous;
    let shared = &mut state.boot_lifecycle.shared;
    critter.rotation = wrap_angle(shared.uniform() * 50.0 + critter.rotation - 25.0);
    if critter.timer < 1 {
        if shared.bounded(30) == 0 {
            critter.timer = shared.bounded(120) as i32 + 15;
        } else if shared.bounded(5) == 0 {
            let speed = shared.uniform();
            critter.previous = critter.previous + angle_vector(critter.rotation) * speed;
        }
    } else {
        let speed = shared.uniform() + 1.0;
        critter.previous = critter.previous + angle_vector(critter.rotation) * speed;
        critter.timer -= 1;
    }
    let depth = critter.previous.x;
    effect.position = project_wall(state, &mut critter.previous);
    if depth != critter.previous.x {
        let turn = if depth > 0.0 {
            if critter.rotation > 0.0 {
                90.0
            } else {
                -90.0
            }
        } else if critter.rotation > 0.0 {
            -90.0
        } else {
            90.0
        };
        critter.rotation = lerp_angle(critter.rotation, critter.rotation + turn, 0.2);
    }
    if grid_class(state, effect.position) == 5 {
        let turn = if critter.previous.y > saved.y {
            if critter.rotation > 0.0 {
                90.0
            } else {
                -90.0
            }
        } else if critter.rotation > 0.0 {
            -90.0
        } else {
            90.0
        };
        critter.rotation = lerp_angle(critter.rotation, critter.rotation + turn, 0.2);
        critter.previous = saved;
        effect.position = project_wall(state, &mut critter.previous);
    }
    if grid_class(state, effect.position) == 5 {
        critter.previous.y += 2.0;
    }
    let killed = critter.dying;
    effect.critter = Some(critter.clone());
    *effect_mut(state, id)? = effect.clone();
    if killed {
        splat(state, id, None, 0x2000)?;
        return Ok(());
    }
    if tears_within(state, effect.position, 1.0).any(|(_, dying)| dying) {
        critter.dying = true;
    }
    effect_mut(state, id)?.critter = Some(critter);
    Ok(())
}

/// 45a3e0: map (depth, perimeter) onto the wall band. Depth clamps to
/// [-10, 40]; the perimeter wraps clockwise from the bottom-left corner.
#[expect(
    clippy::manual_clamp,
    reason = "45a3e0 applies max then min; clamp would keep a NaN depth instead"
)]
fn project_wall(state: &State, coordinate: &mut WorldVector) -> WorldVector {
    coordinate.x = coordinate.x.max(-10.0).min(40.0);
    let (low, high) = room_rect(state);
    let width = high.x - low.x;
    let height = high.y - low.y;
    let perimeter = width + width + height + height;
    let mut along = (f64::from(coordinate.y) % f64::from(perimeter)) as f32;
    if along < 0.0 {
        along += perimeter;
    }
    coordinate.y = along;
    let point = |x: f32, y: f32| WorldVector { x, y };
    let (outer_from, outer_to, inner_from, inner_to, offset, length) = if along < height {
        (
            point(low.x - 40.0, high.y + 40.0),
            point(low.x - 40.0, low.y - 40.0),
            point(low.x, high.y),
            point(low.x, low.y),
            along,
            height,
        )
    } else if along < height + width {
        (
            point(low.x - 40.0, low.y - 40.0),
            point(high.x + 40.0, low.y - 40.0),
            point(low.x, low.y),
            point(high.x, low.y),
            along - height,
            width,
        )
    } else if along < width + height + height {
        (
            point(high.x + 40.0, low.y - 40.0),
            point(high.x + 40.0, high.y + 40.0),
            point(high.x, low.y),
            point(high.x, high.y),
            along - (height + width),
            height,
        )
    } else {
        (
            point(high.x + 40.0, high.y + 40.0),
            point(low.x - 40.0, high.y + 40.0),
            point(high.x, high.y),
            point(low.x, high.y),
            along - (width + height + height),
            width,
        )
    };
    let depth = coordinate.x / 40.0;
    let from = lerp(depth, outer_from, inner_from);
    let to = lerp(depth, outer_to, inner_to);
    lerp(offset / length, from, to)
}

/// Case 2 (45ac60): the first Blood Explosion update draws Random(2) flipX
/// and flipY for every layer (4609e1), then Random(3) + 6 into 0x7d4. When
/// the clip frame reaches it, subtype 0 joins the floor pool (0x20008); the
/// common tail retires it when its clip finishes.
fn blood_explosion(state: &mut State, id: u32) -> Result<(), String> {
    let first = effect_mut(state, id)?
        .critter
        .as_ref()
        .expect("critter")
        .timer
        == 0;
    if first {
        let shared = &mut state.boot_lifecycle.shared;
        let flip_x = shared.bounded(2) == 0;
        let flip_y = shared.bounded(2) == 0;
        let timer = shared.bounded(3) as i32 + 6;
        let effect = effect_mut(state, id)?;
        effect.sprite.flip_x = flip_x;
        effect.sprite.flip_y = flip_y;
        effect.critter.as_mut().expect("critter").timer = timer;
    }
    let effect = effect_mut(state, id)?;
    let timer = effect.critter.as_ref().expect("critter").timer;
    if effect.sprite.frame as i32 == timer && effect.subtype == 0 {
        effect.flags |= 0x2_0008;
    }
    if !effect.sprite.playing {
        remove(effect);
    }
    Ok(())
}
