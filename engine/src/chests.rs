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
    /// Missing opening callbacks never grant a substitute loot outcome.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_frontier: Option<OpenFrontier>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OpenFrontier {
    PlayerContext,
    CollectibleMorph,
    ExplosionGeometry,
    RegularChildInit,
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
    inputs: OpeningInputs,
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
        // when the actual trinket queries are false and no child is made.
        let _modifier_scatter = scatter(position, grid, shared);
    }
    //502B6F..594: actual Have199 doubles the current count. It changes no
    //draw, category thresholds or rare-child early return.
    if inputs.double_loot {
        count += count;
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

pub(crate) fn scatter(position: WorldVector, grid: &Grid, shared: &mut Mt19937) -> WorldVector {
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
    if state
        .room
        .pickups
        .get(index)
        .is_some_and(|pickup| pickup.variant == 60)
    {
        return try_open_locked(state, index);
    }
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
        // Collect5049ca uses50c880 before504aa7 callsTryOpen500e80.
        // A living Soul-only Player remains eligible under that sole owner.
        || !crate::pickups::player_can_collect(state)
    {
        return Ok(false);
    }
    if let Some(coverage) = &mut state.room.ordinary_placement {
        coverage.invalidate(crate::ordinary_placement::Frontier::ChestEffects);
    }
    if let Some(owner) = &mut state.player_items {
        crate::ordinary_placement::remember_owner(
            owner,
            crate::ordinary_placement::Frontier::ChestEffects,
        );
    }
    let inputs = match opening_inputs(state) {
        Ok(inputs) => inputs,
        Err(frontier) => {
            state.room.pickups[index]
                .chest
                .as_mut()
                .expect("normal Chest")
                .open_frontier
                .get_or_insert(frontier);
            return Ok(false);
        }
    };
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
        inputs,
        |birth, _shared| {
            admit_child(
                room,
                scheduler,
                next_pickup,
                game_update,
                &birth,
                inputs.regular_subtypes,
                inputs.pocket_subtypes,
            )?;
            births.push(birth);
            Ok(())
        },
    )?;
    if births.iter().any(|birth| birth.variant == 350) {
        crate::item_queries::note_unrepresented_trinket_spawn(state);
    }
    let chest = state.room.pickups[index]
        .chest
        .as_mut()
        .expect("opening Chest owner");
    chest.rng = rng;
    chest.loot_births.extend(births);
    Ok(true)
}

/// Current direct500E80 operands, independent of cache/Get1/grant admission.
/// Only Have199 changes this selected count operator. Neither TEMP nor active
/// D80 is read by that actual direct5090C0 query.
#[derive(Clone, Copy)]
pub(crate) struct OpeningInputs {
    double_loot: bool,
    regular_subtypes: crate::pickups::RegularSubtypeInputs,
    pocket_subtypes: crate::pickups::PocketSubtypeInputs,
}

///508FE0 first queries21, then one/two actual slots (508FC0 Have139), then
///the queued config only if it is kind2. Both known slots zero close the
///negative query for every count139. Unknown queued config is not empty.
fn empty_trinket_inputs(state: &State) -> bool {
    state
        .player
        .entity_source
        .is_some_and(|source| source.entity_type == 1 && source.variant == 0)
        && state.player_items.as_ref().is_some_and(|items| {
            items.owned_counts.len() == 347
                && items.trinket_slots == [0, 0]
                && items.queue_excludes_trinkets()
        })
}

///Actual60 Init reads LeftHand61 and the existing common free-room owner.
///Stage11 is an OPENING special case, not a fresh Init exclusion.
///Only Room8 has the unresolved regular-child subtype override. Free Shop
///Room2 queries Trinket13, already false through the actual empty operands.
fn fresh_locked_context(state: &State) -> bool {
    state.challenge_id != 33
        && state
            .room
            .descriptor
            .as_ref()
            .is_some_and(|room| room.template.room_type != 8)
        && state.weapon_admission.source_player_type == Some(0)
        && empty_trinket_inputs(state)
}

fn opening_inputs(state: &State) -> Result<OpeningInputs, OpenFrontier> {
    //5010F7 and501327 test exactly stage11; ordinary later stages use the
    //same parent operator. Room8/+1AE0 and Challenge33 price have distinct
    //child factory owners; other actual room types share the free operator.
    if state.stage == 11 || !fresh_locked_context(state) {
        return Err(OpenFrontier::PlayerContext);
    }
    let items = state
        .player_items
        .as_ref()
        .expect("checked actual item owner");
    //The selected heart203 operator now shares the canonical local35 core.
    //Other positive heart modifiers retain their independent frontier;
    //empty trinkets/BFC0/non-Room8 are actual guards above, not inferred.
    if [173usize, 238, 239]
        .into_iter()
        .any(|id| items.owned_counts[id] > 0)
    {
        return Err(OpenFrontier::RegularChildInit);
    }
    Ok(OpeningInputs {
        double_loot: items.owned_counts[199] > 0,
        regular_subtypes: crate::pickups::RegularSubtypeInputs::from_state(state),
        pocket_subtypes: crate::pickups::PocketSubtypeInputs::from_state(state),
    })
}

fn try_open_locked(state: &mut State, index: usize) -> Result<bool, String> {
    let pickup = &state.room.pickups[index];
    let Some(chest) = pickup.chest.as_ref() else {
        return Ok(false);
    };
    if pickup.subtype == 0
        || pickup.removed
        || !pickup.can_collect
        || pickup.collection_delay != 0
        || chest.entity_collision_class != 4
        || chest.price != 0
        || chest.cooldown != 0
        || !pickup.touches(&state.player)
        || !crate::pickups::player_can_collect(state)
    {
        return Ok(false);
    }
    let inputs = match opening_inputs(state) {
        Ok(inputs) => inputs,
        Err(frontier) => {
            state.room.pickups[index]
                .chest
                .as_mut()
                .expect("locked Chest")
                .open_frontier
                .get_or_insert(frontier);
            crate::ordinary_placement::invalidate(
                state,
                crate::ordinary_placement::Frontier::ChestEffects,
            );
            return Ok(false);
        }
    };
    if state.player.golden_key.is_none() {
        state.room.pickups[index]
            .chest
            .as_mut()
            .expect("locked Chest")
            .open_frontier
            .get_or_insert(OpenFrontier::PlayerContext);
        crate::ordinary_placement::invalidate(
            state,
            crate::ordinary_placement::Frontier::ChestEffects,
        );
        return Ok(false);
    }
    // PaperClip19 is independently false in this actual empty trinket/queue
    // context. A failed real TryUseKey cannot reach the loot callback.
    if state.player.golden_key == Some(false) && state.inventory.keys == 0 {
        return Ok(false);
    }
    crate::ordinary_placement::invalidate(state, crate::ordinary_placement::Frontier::ChestEffects);
    open_bounded_chest(state, index, inputs)
}

/// Pickup.Hurt504240 has its own positive damage/flags4/variant51/subtype
/// gates. Its source return is false even after the real opening side effect.
/// The selected empty normal context closes TryOpen independently of key use.
pub(crate) fn hurt_bomb_chest(
    state: &mut State,
    index: usize,
    damage: f32,
    damage_flags: u32,
) -> Result<bool, String> {
    let pickup = &state.room.pickups[index];
    if damage <= 0.0 || damage_flags & 4 == 0 || pickup.variant != 51 || pickup.subtype == 0 {
        return Ok(false);
    }
    if let Some(coverage) = &mut state.room.ordinary_placement {
        coverage.invalidate(crate::ordinary_placement::Frontier::ChestEffects);
    }
    if let Some(owner) = &mut state.player_items {
        crate::ordinary_placement::remember_owner(
            owner,
            crate::ordinary_placement::Frontier::ChestEffects,
        );
    }
    if !damage.is_finite() {
        state.room.pickups[index]
            .chest
            .as_mut()
            .expect("Bomb Chest")
            .open_frontier
            .get_or_insert(OpenFrontier::ExplosionGeometry);
        return Ok(false);
    }
    let inputs = match opening_inputs(state) {
        Ok(inputs) => inputs,
        Err(frontier) => {
            state.room.pickups[index]
                .chest
                .as_mut()
                .expect("Bomb Chest")
                .open_frontier
                .get_or_insert(frontier);
            return Ok(false);
        }
    };
    let _opened = open_bounded_chest(state, index, inputs)?;
    // BFC11/17's extra room callback is outside the selected normalBFC0 path.
    Ok(false)
}

/// Actual51 and60 converge at5010F0. Share the existing R5/pool8/Morph and
/// non-Morph operations; key consumption and SFX156 belong only to60's caller.
fn open_bounded_chest(
    state: &mut State,
    index: usize,
    inputs: OpeningInputs,
) -> Result<bool, String> {
    let pickup = &state.room.pickups[index];
    let Some(chest) = pickup.chest.as_ref() else {
        return Ok(false);
    };
    let spend_key = match pickup.variant {
        51 => false,
        60 => true,
        _ => return Err("bounded Chest opening requires actual51/60".into()),
    };
    let position = pickup.position;
    let mut rng = chest.rng;
    // Capability inspection only. Missing optional weak/pool/manager owners
    // cannot be manufactured from inventory or consume real key/audio/RNG.
    let mut branch = rng;
    let morph = branch.bounded(5) == 0;
    let actor = ActorRef {
        kind: ActorKind::Pickup,
        id: pickup.id,
    };
    if morph
        && (pickup
            .morph_links
            .as_ref()
            .is_none_or(|links| !links.known_empty())
            || state.item_pools.is_none()
            || state.actor_scheduler.sequence_id(actor).is_none()
            || !state.actor_scheduler.full_phase().contains(&actor))
    {
        state.room.pickups[index]
            .chest
            .as_mut()
            .expect("locked Chest")
            .open_frontier
            .get_or_insert(OpenFrontier::CollectibleMorph);
        return Ok(false);
    }
    let grid = Grid::new(
        state
            .room
            .grid
            .clone()
            .ok_or("locked Chest requires room grid")?,
    )?;
    // 5052E1 precedes TryOpen. Golden BD0 uses the same actual key API and
    // never decrements BCC; missing owner was rejected above, not defaulted.
    if spend_key && !state.player.try_use_key(&mut state.inventory) {
        return Ok(false);
    }
    let pickup = &mut state.room.pickups[index];
    pickup.subtype = 0;
    pickup.animation = "Open".into();
    pickup.animation_frame = 0.0;
    pickup.animation_playing = true;
    pickup.can_collect = false;
    // 501078 SFX22 for both; only actual variant60 reaches501576 SFX156.
    let sounds: &[u32] = if spend_key { &[22, 156] } else { &[22] };
    for &id in sounds {
        state.audio.admit(
            &mut state.boot_lifecycle.shared,
            state.system_clock.system_counter.wrapping_sub(1),
            SoundRequest {
                id,
                volume: 1.0,
                delay: 2,
                looping: false,
                pitch: 1.0,
            },
        );
    }
    if morph {
        //501110 R5==0,50113D raw,50115C pool8(decrease=true).
        //The copied capability peek above never advances this actual owner.
        let _morph_roll = rng.bounded(5);
        let subtype = state
            .item_pools
            .as_mut()
            .expect("admitted Morph pool")
            .collectible(8, true, rng.raw())?;
        let pickup = &mut state.room.pickups[index];
        pickup.anchor_position = pickup.position;
        pickup.morph_to_collectible(
            subtype,
            state.game_update_count,
            &mut state.actor_scheduler,
        )?;
        //5011F2..246 computes current ChestP - FIRSTPlayerP. The actual
        //near callback changes only PlayerV; cooldown20 follows every path.
        let player_position = crate::bombs::player_world(state);
        let delta = WorldVector {
            x: position.x - player_position.x,
            y: position.y - player_position.y,
        };
        let distance_squared = delta.x * delta.x + delta.y * delta.y;
        if distance_squared < 1600.0 {
            let amount = (40.0 - distance_squared.sqrt()) * 0.4;
            let push = crate::npcs::normalized(delta) * amount;
            let velocity = crate::bombs::player_velocity(state);
            let velocity = WorldVector {
                x: velocity.x - push.x,
                y: velocity.y - push.y,
            } * 0.5;
            state.player.vx = f64::from(velocity.x) * 0.65;
            state.player.vy = f64::from(velocity.y) * 0.65;
        }
        state.room.pickups[index].collection_delay = 20;
        return Ok(true);
    }
    let mut births = Vec::new();
    let game_update = state.game_update_count;
    let room = &mut state.room;
    let scheduler = &mut state.actor_scheduler;
    let next_pickup = &mut state.next_pickup;
    locked_opening_loot(
        &mut rng,
        position,
        &grid,
        &mut state.boot_lifecycle.shared,
        inputs,
        |birth, _shared| {
            admit_child(
                room,
                scheduler,
                next_pickup,
                game_update,
                &birth,
                inputs.regular_subtypes,
                inputs.pocket_subtypes,
            )?;
            births.push(birth);
            Ok(())
        },
    )?;
    if births.iter().any(|birth| birth.variant == 350) {
        crate::item_queries::note_unrepresented_trinket_spawn(state);
    }
    let chest = state.room.pickups[index]
        .chest
        .as_mut()
        .expect("locked Chest opening");
    chest.rng = rng;
    chest.loot_births.extend(births);
    Ok(true)
}

/// Actual51/60 non-Morph branch. PersistentRNG30, shared scatter and
/// immediate child callbacks keep their independent native ordering.
fn locked_opening_loot(
    rng: &mut Rng,
    position: WorldVector,
    grid: &Grid,
    shared: &mut Mt19937,
    inputs: OpeningInputs,
    mut spawn: impl FnMut(LootBirth, &mut Mt19937) -> Result<(), String>,
) -> Result<(), String> {
    if rng.bounded(5) == 0 {
        return Err("locked Chest collectible Morph unadmitted".into());
    }
    if rng.bounded(5) == 0 {
        let velocity = scatter(position, grid, shared);
        let variant = if rng.bounded(2) == 0 { 350 } else { 300 };
        let seed = rng.raw();
        return spawn(
            LootBirth {
                variant,
                spawn_seed: seed,
                position,
                velocity: velocity * 0.4,
                sprite_scale_multiplier: None,
            },
            shared,
        );
    }
    let mut count = rng.bounded(7).max(2);
    if rng.bounded(3) != 0 {
        let _modifier_scatter = scatter(position, grid, shared);
    }
    //502049..06E: direct Have199 doubles after all actual trinket queries.
    //The selected trinket inputs are false; their earlier scatter is retained.
    if inputs.double_loot {
        count += count;
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
        } else if completed == 0 && category < 0.71 {
            50
        } else if completed == 0 && category < 0.72 {
            60
        } else {
            40
        };
        let children = if variant == 20 { rng.bounded(3) + 1 } else { 1 };
        for _ in 0..children {
            let velocity = scatter(position, grid, shared);
            let seed = rng.raw();
            spawn(
                LootBirth {
                    variant,
                    spawn_seed: seed,
                    position,
                    velocity,
                    sprite_scale_multiplier: matches!(variant, 50 | 60).then_some(0.8),
                },
                shared,
            )?;
        }
        // First-iteration .70/.71 rare children end the actual callback.
        if matches!(variant, 50 | 60) {
            return Ok(());
        }
        if variant == 10 && rng.bounded(2) == 0 {
            count = count.saturating_sub(1);
        }
        completed += 1;
    }
    Ok(())
}

fn admit_child(
    room: &mut Room,
    scheduler: &mut Scheduler,
    next_pickup: &mut u32,
    game_update: u32,
    birth: &LootBirth,
    regular_inputs: crate::pickups::RegularSubtypeInputs,
    pocket_inputs: crate::pickups::PocketSubtypeInputs,
) -> Result<(), String> {
    //A350 child's actual Init/Get is outside the admitted grant-only slice.
    //Port gap: the child is not admitted, its birth stays in the chest's
    //loot record and the caller notes the unrepresented trinket, so the
    //opening continues instead of failing every retry of the command.
    if birth.variant == 350 {
        return Ok(());
    }
    //Rare scaled50/60 retain their independent Init owners.
    //Pocket70/300 now shares the room/reentry/drop resolver; every caller
    //retains its actual seed, scatter velocity and scheduler admission order.
    if birth.sprite_scale_multiplier.is_some()
        || !matches!(birth.variant, 10 | 20 | 30 | 40 | 70 | 300)
    {
        return Ok(());
    }
    let id = *next_pickup;
    let mut pickup = if matches!(birth.variant, 70 | 300) {
        let Some(pickup) = Pickup::from_pocket_parts(
            id,
            birth.variant,
            0,
            birth.spawn_seed,
            birth.position,
            true,
            pocket_inputs,
        )?
        else {
            return Ok(());
        };
        pickup
    } else {
        let Some(subtype) = crate::pickups::regular_subtype_with_inputs(
            birth.variant,
            0,
            birth.spawn_seed,
            regular_inputs,
        )?
        else {
            return Ok(());
        };
        let mut pickup = Pickup::spawn(
            id,
            birth.variant,
            crate::pickups::definition_subtype(birth.variant, subtype),
            birth.position,
            true,
        )?;
        pickup.subtype = subtype;
        pickup.initialize_spawn_seed(birth.spawn_seed)?;
        pickup
    };
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
        || !matches!(pickup.variant, 50 | 51 | 60)
        || pickup.removed
        || pickup
            .chest
            .as_ref()
            .is_none_or(|chest| chest.entity_collision_class != 4 || chest.price != 0)
    {
        return Ok(());
    }
    //445765..77B captures the pre-callback radius sum before Collect/Morph.
    //Current mass and velocity are read only after the real callback returns.
    let contact_radius = pickup.collision_radius;
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
        radius: contact_radius,
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
    from_selector_with_bomb_replacement(id, selector, appear, false)
}

/// The real50 ->51 replacement still passes LeftHand508FE0 at4FFF1E.
/// Existing callers without this operand owner keep their prior frontier;
/// direct51 never reaches that query. No missing trinket history becomes0.
pub(crate) fn from_source_selector(
    id: u32,
    selector: &SpawnSelector,
    appear: bool,
    state: &State,
) -> Result<Option<Pickup>, String> {
    let no_left_hand = empty_trinket_inputs(state);
    from_selector_with_bomb_replacement(id, selector, appear, no_left_hand)
}

fn from_selector_with_bomb_replacement(
    id: u32,
    selector: &SpawnSelector,
    appear: bool,
    no_left_hand: bool,
) -> Result<Option<Pickup>, String> {
    if selector.entity_type != 5 || !matches!(selector.variant, 50 | 51) {
        return Ok(None);
    }
    if selector.spawn_seed == 0 {
        return Err("chest spawn seed must be nonzero".into());
    }
    // Normal50 Init4ffedd/50026a: Random20 is first; the red Chest
    // replacement bypasses the second Random50. Only360 keeps its separate
    // unsupported actor; the actual51 replacement uses the canonical Init.
    let variant = if selector.variant == 50 {
        let mut temporary = Rng::new(selector.spawn_seed, [5, 9, 7]);
        if temporary.bounded(20) == 0 {
            // The real360 replacement keeps its own unimplemented owner.
            return Ok(None);
        }
        if temporary.bounded(50) == 0 {
            if !no_left_hand {
                return Ok(None);
            }
            51
        } else {
            50
        }
    } else {
        // Direct51 goes4FFEC3 ->4FFF38, with no R20/R50/LeftHand query.
        51
    };
    // Init4fff38 forces closed subtype1 even when the authored subtype is0.
    let mut pickup = Pickup::spawn(id, variant, 1, selector.position, appear)?;
    pickup.spawn_seed = selector.spawn_seed;
    pickup.chest = Some(Chest {
        rng: Rng::new(selector.spawn_seed, [4, 3, 17]),
        grid_collision_class: 5,
        entity_collision_class: if appear { 0 } else { 4 },
        price: 0,
        cooldown: 0,
        loot_births: Vec::new(),
        open_frontier: None,
    });
    if variant == 51 {
        // Same no-spawner BaseInit441F1F..35 and registration as fresh60.
        pickup.morph_links = Some(crate::pickups::MorphLinks {
            parent498: None,
            link4a0: None,
            link4a8: None,
        });
    }
    Ok(Some(pickup))
}

/// Init4FFEC5 routes60 straight to the actual LeftHand61 test at4FFF19;
/// it never takes normal50's temporary R20/R50 replacement draws.
pub(crate) fn from_locked_selector(
    id: u32,
    selector: &SpawnSelector,
    appear: bool,
    state: &State,
) -> Result<Option<Pickup>, String> {
    if selector.entity_type != 5 || selector.variant != 60 {
        return Ok(None);
    }
    if !fresh_locked_context(state) {
        return Ok(None);
    }
    if selector.spawn_seed == 0 {
        return Err("locked Chest spawn seed must be nonzero".into());
    }
    let mut pickup = Pickup::spawn(id, 60, 1, selector.position, appear)?;
    pickup.spawn_seed = selector.spawn_seed;
    pickup.chest = Some(Chest {
        rng: Rng::new(selector.spawn_seed, [4, 3, 17]),
        grid_collision_class: 5,
        entity_collision_class: if appear { 0 } else { 4 },
        price: 0,
        cooldown: 0,
        loot_births: Vec::new(),
        open_frontier: None,
    });
    //441F1F..35 clears all three actual weak words; the room no-spawner
    //Game.Spawn wrapper550ADD goes directly to registration without a setter.
    pickup.morph_links = Some(crate::pickups::MorphLinks {
        parent498: None,
        link4a0: None,
        link4a8: None,
    });
    Ok(Some(pickup))
}
