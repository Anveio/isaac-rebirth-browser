//! Concrete ordinary Chest actors. Authored selectors stay unchanged; fresh
//! Init's temporary RNG35 is independent of the entity's persistent RNG30.
use crate::{
    actors::{ActorKind, ActorRef, Scheduler},
    audio::SoundRequest,
    grid::Grid,
    items::Rng,
    motion::WorldVector,
    mt::Mt19937,
    phase::Phase,
    pickups::Pickup,
    room::Room,
    room_spawns::SpawnSelector,
    State,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Chest {
    pub rng: Rng,
    pub grid_collision_class: u32,
    pub entity_collision_class: u32,
    pub price: i32,
    pub cooldown: u32,
    /// Derived TryOpen factory inputs, including unsupported loot families.
    /// These are dynamic births, independent of immutable authored STB groups.
    #[serde(default)]
    pub loot_births: Vec<LootBirth>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LootBirth {
    pub variant: u32,
    pub spawn_seed: u32,
    pub position: WorldVector,
    pub velocity: WorldVector,
    /// TryOpen only writes the rare child's sprite scale, not collision scale.
    pub sprite_scale_multiplier: Option<f32>,
}

/// TryOpen500e80 fresh ordinary variant50 loot. Each actual GameSpawn input
/// reaches the callback immediately, before the next parent RNG draw, so its
/// independent Init runs in native order. Item/trinket and stage11 overrides
/// must be routed by the caller rather than silently entering this branch.
pub(crate) fn opening_loot(
    rng: &mut Rng,
    position: WorldVector,
    grid: &Grid,
    shared: &mut Mt19937,
    mut spawn: impl FnMut(LootBirth, &mut Mt19937) -> Result<(), String>,
) -> Result<(), String> {
    if rng.bounded(5) == 0 {
        let velocity = scatter(position, grid, shared);
        let variant = if rng.bounded(2) == 0 { 350 } else { 70 };
        return spawn(
            LootBirth {
                variant,
                spawn_seed: rng.raw(),
                position,
                velocity: velocity * 0.4,
                sprite_scale_multiplier: None,
            },
            shared,
        );
    }
    // 50170f and5027a9 are two independent uniforms. The second is only
    // reached when the first rare normal-Chest branch did not win.
    let rare = if rng.uniform() < 0.01 {
        Some(50)
    } else if rng.uniform() < 0.01 {
        Some(60)
    } else {
        None
    };
    if let Some(variant) = rare {
        return spawn(
            LootBirth {
                variant,
                spawn_seed: rng.raw(),
                position: position + WorldVector { x: 0.0, y: 10.0 },
                velocity: WorldVector { x: 0.0, y: 0.0 },
                sprite_scale_multiplier: Some(0.8),
            },
            shared,
        );
    }
    let mut count = rng.bounded(4).max(2);
    if rng.bounded(3) != 0 {
        // 50292e..502a46 actually scatters before testing modifiers, even
        // when fresh Isaac owns none and this vector produces no child.
        let _modifier_scatter = scatter(position, grid, shared);
    }
    let mut completed = 0;
    while completed < count {
        let category = rng.uniform();
        let variant = if category < 0.35 {
            20
        } else if category < 0.55 {
            10
        } else if category < 0.70 {
            30
        } else {
            40
        };
        let children = if variant == 20 { rng.bounded(3) + 1 } else { 1 };
        for _ in 0..children {
            let velocity = scatter(position, grid, shared);
            spawn(
                LootBirth {
                    variant,
                    spawn_seed: rng.raw(),
                    position,
                    velocity,
                    sprite_scale_multiplier: None,
                },
                shared,
            )?;
        }
        if variant == 10 && rng.bounded(2) == 0 {
            count = count.saturating_sub(1);
        }
        completed += 1;
    }
    Ok(())
}

fn scatter(position: WorldVector, grid: &Grid, shared: &mut Mt19937) -> WorldVector {
    let mut velocity = WorldVector { x: 0.0, y: 0.0 };
    for _ in 0..10 {
        velocity = crate::npcs::direction(shared) * 5.0;
        if grid.scatter_line_clear(position, position + velocity * 7.0) {
            break;
        }
    }
    velocity
}

/// Fresh free normal Chest contact in the full postactor manager pass.
pub(crate) fn try_open(state: &mut State, index: usize) -> Result<bool, String> {
    let Some(pickup) = state.room.pickups.get(index) else {
        return Ok(false);
    };
    let Some(chest) = pickup.chest.as_ref() else {
        return Ok(false);
    };
    if pickup.variant != 50
        || pickup.subtype == 0
        || pickup.removed
        || !pickup.can_collect
        || pickup.collection_delay != 0
        || chest.entity_collision_class != 4
        || chest.price != 0
        || chest.cooldown != 0
        || !pickup.touches(&state.player)
        || state.player.hp == 0
    {
        return Ok(false);
    }
    // This is the recovered unmodified first-Basement branch. Other player
    // modifiers/prices/stage11 and queued presentation contacts retain their
    // actor state until those owning callbacks are admitted.
    let player_items = state
        .player_items
        .as_ref()
        .ok_or("Chest requires Player item state")?;
    if state.stage != 1
        || player_items.collection_delay > 0
        || player_items.pickup_animation_guard
        || player_items.owned_counts.iter().any(|count| *count != 0)
    {
        return Ok(false);
    }
    let position = pickup.position;
    let mut rng = chest.rng;
    let grid = Grid::new(
        state
            .room
            .grid
            .clone()
            .ok_or("Chest requires admitted room grid")?,
    )?;
    let pickup = &mut state.room.pickups[index];
    pickup.subtype = 0;
    pickup.animation = "Open".into();
    pickup.animation_frame = 0.0;
    pickup.animation_playing = true;
    pickup.can_collect = false;
    // 501078 admits SFX22 before the first local loot draw and actual shared
    // scatter. The sound owner consumes its real sample selection, even R1.
    state.audio.admit(
        &mut state.boot_lifecycle.shared,
        state.system_clock.system_counter.wrapping_sub(1),
        SoundRequest {
            id: 22,
            volume: 1.0,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
    let mut births = Vec::new();
    let game_update = state.game_update_count;
    let room = &mut state.room;
    let scheduler = &mut state.actor_scheduler;
    let next_pickup = &mut state.next_pickup;
    opening_loot(
        &mut rng,
        position,
        &grid,
        &mut state.boot_lifecycle.shared,
        |birth, _shared| {
            admit_child(room, scheduler, next_pickup, game_update, &birth)?;
            births.push(birth);
            Ok(())
        },
    )?;
    let chest = state.room.pickups[index]
        .chest
        .as_mut()
        .expect("opening Chest owner");
    chest.rng = rng;
    chest.loot_births.extend(births);
    Ok(true)
}

fn admit_child(
    room: &mut Room,
    scheduler: &mut Scheduler,
    next_pickup: &mut u32,
    game_update: u32,
    birth: &LootBirth,
) -> Result<(), String> {
    // Rare child50/60 sprite scaling, pills and trinkets have separate Init
    // callbacks. Their exact factory inputs stay in Chest.loot_births; they
    // never become invented authored groups or substitute regular actors.
    if birth.sprite_scale_multiplier.is_some() || !matches!(birth.variant, 10 | 20 | 30 | 40) {
        return Ok(());
    }
    let subtype = if birth.variant == 10 {
        crate::fireplaces::loot_subtype(10, birth.spawn_seed)?
    } else {
        crate::pickups::regular_subtype(birth.variant, 0, birth.spawn_seed)?
    };
    if birth.variant == 30 && subtype == 2 {
        return Ok(()); // Actual golden key Init/collection remains unsupported.
    }
    let definition_subtype = if birth.variant == 40 && matches!(subtype, 3 | 5) {
        1
    } else {
        subtype
    };
    let id = *next_pickup;
    let mut pickup = Pickup::spawn(id, birth.variant, definition_subtype, birth.position, true)?;
    pickup.subtype = subtype;
    pickup.initialize_spawn_seed(birth.spawn_seed)?;
    pickup.velocity = birth.velocity;
    pickup.birth_game_update = game_update;
    *next_pickup = id.checked_add(1).ok_or("Chest child identity exhausted")?;
    room.pickups.push(pickup);
    let actor = ActorRef {
        kind: ActorKind::Pickup,
        id,
    };
    room.authored_actor_order.push(actor);
    scheduler.admit(actor)
}

/// Collect504aa7 executes TryOpen before445710's ordinary circle response.
/// The fresh normal50 callback leaves both parent positions/velocities intact,
/// so the existing kernel retains the same pre-callback overlap geometry.
pub(crate) fn player_contact(state: &mut State, index: usize) -> Result<(), String> {
    let pickup = &state.room.pickups[index];
    if state.player.dead
        || pickup.variant != 50
        || pickup.removed
        || pickup
            .chest
            .as_ref()
            .is_none_or(|chest| chest.entity_collision_class != 4 || chest.price != 0)
    {
        return Ok(());
    }
    try_open(state, index)?;
    let grid = Grid::new(
        state
            .room
            .grid
            .clone()
            .ok_or("Chest contact requires admitted room grid")?,
    )?;
    static PLAYER_COLLIDER: std::sync::OnceLock<crate::grid::Collider> = std::sync::OnceLock::new();
    let collider = PLAYER_COLLIDER.get_or_init(|| {
        crate::grid::Collider::new(crate::grid::CollisionGeometry {
            radius: crate::content::PLAYER_COLLISION_RADIUS_WORLD as f32,
            scale: WorldVector { x: 1.0, y: 1.0 },
            points: 40,
        })
        .expect("authored normal Isaac geometry")
    });
    let mut player = crate::combat::ContactBody {
        id: 0,
        entity_type: 1,
        position: crate::bombs::player_world(state),
        velocity: crate::bombs::player_velocity(state),
        radius: crate::content::PLAYER_COLLISION_RADIUS_WORLD as f32,
        mass: 5.0,
    };
    let pickup = &state.room.pickups[index];
    let mut chest = crate::combat::ContactBody {
        id: pickup.id,
        entity_type: 5,
        position: pickup.position,
        velocity: pickup.velocity,
        radius: pickup.collision_radius,
        mass: pickup.collision_mass,
    };
    let position = player.position;
    crate::combat::resolve_player_pickup_contact(&mut player, &mut chest, |shift| {
        collider.blocks_player_shift(&grid, position, shift)
    })?;
    state.player.x = f64::from(player.position.x) * 0.65 + 32.0;
    state.player.y = f64::from(player.position.y) * 0.65 - 46.0;
    state.player.vx = f64::from(player.velocity.x) * 0.65;
    state.player.vy = f64::from(player.velocity.y) * 0.65;
    let pickup = &mut state.room.pickups[index];
    pickup.position = chest.position;
    pickup.presented_position = chest.position;
    pickup.velocity = chest.velocity;
    pickup.project();
    Ok(())
}

/// Chest BaseUpdate owns physical motion before its .8 control and ANM2
/// callback. Interpolation predicts from the retained physical owner.
pub(crate) fn advance_actor(state: &mut State, phase: Phase, id: u32) -> Result<(), String> {
    let Some(index) = state.room.pickups.iter().position(|pickup| pickup.id == id) else {
        return Ok(());
    };
    let pickup = &mut state.room.pickups[index];
    if pickup.removed || phase == Phase::Skipped {
        return Ok(());
    }
    if phase == Phase::Update {
        pickup.velocity = pickup.velocity * pickup.current_friction;
        pickup.position = pickup.position + pickup.velocity;
        pickup.presented_position = pickup.position;
        pickup.velocity = pickup.velocity * 0.8;
        pickup.current_friction = pickup.configured_friction * 0.965;
        pickup.advance(phase, state.game_update_count);
        pickup.project();
        if pickup.animation == "Appear"
            && pickup.animation_frame == f64::from(crate::pickups::CHEST_DROP_SOUND_FRAME)
        {
            state.audio.admit(
                &mut state.boot_lifecycle.shared,
                state.system_clock.system_counter.wrapping_sub(1),
                SoundRequest {
                    id: 21,
                    volume: 1.0,
                    delay: 2,
                    looping: false,
                    pitch: 1.0,
                },
            );
        }
    } else if phase == Phase::Interpolate {
        pickup.presented_position =
            pickup.position + (pickup.velocity * pickup.current_friction) * 0.5;
        let grid = Grid::new(
            state
                .room
                .grid
                .clone()
                .ok_or("Chest requires admitted room grid")?,
        )?;
        crate::pickups::resolve_grid(pickup, &grid, true)?;
    }
    Ok(())
}

pub(crate) fn full_grid_contacts(state: &mut State) -> Result<(), String> {
    if !state
        .room
        .pickups
        .iter()
        .any(|pickup| pickup.chest.is_some() && !pickup.removed)
    {
        return Ok(());
    }
    let grid = Grid::new(
        state
            .room
            .grid
            .clone()
            .ok_or("Chest requires admitted room grid")?,
    )?;
    for pickup in &mut state.room.pickups {
        if pickup.chest.is_some() && !pickup.removed {
            crate::pickups::resolve_grid(pickup, &grid, false)?;
        }
    }
    Ok(())
}

pub(crate) fn from_selector(
    id: u32,
    selector: &SpawnSelector,
    appear: bool,
) -> Result<Option<Pickup>, String> {
    if selector.entity_type != 5 || selector.variant != 50 {
        return Ok(None);
    }
    if selector.spawn_seed == 0 {
        return Err("chest spawn seed must be nonzero".into());
    }
    // Pickup::Init4ffedd/50026a: Random20 is always first; the red Chest
    // replacement bypasses the second Random50. Rare replacement callbacks
    // remain unsupported actors with their original typed selector retained.
    let mut temporary = Rng::new(selector.spawn_seed, [5, 9, 7]);
    if temporary.bounded(20) == 0 || temporary.bounded(50) == 0 {
        return Ok(None);
    }
    // Init4fff38 forces closed subtype1 even when the authored subtype is0.
    let mut pickup = Pickup::spawn(id, 50, 1, selector.position, appear)?;
    pickup.spawn_seed = selector.spawn_seed;
    pickup.chest = Some(Chest {
        rng: Rng::new(selector.spawn_seed, [4, 3, 17]),
        grid_collision_class: 5,
        entity_collision_class: if appear { 0 } else { 4 },
        price: 0,
        cooldown: 0,
        loot_births: Vec::new(),
    });
    Ok(Some(pickup))
}
