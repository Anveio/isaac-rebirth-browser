//! Ordinary 13x7/13x14 doorway admission and descriptor-owned room persistence.
//! The public floor graph and authored doors are runtime inputs; source maps
//! and captured room/actor geometry never enter the production route.
use crate::{motion::WorldVector, phase::Phase, Player, Room, State};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TransitionPhase {
    Out,
    In,
    BossScene,
    BossCleanup,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoomTransition {
    pub from_room: usize,
    pub target_room: usize,
    pub from_slot: u8,
    pub target_slot: u8,
    pub phase: TransitionPhase,
    pub full_updates: u8,
    pub boss_scene: Option<BossScene>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BossScene {
    pub presentation_index: u32,
    pub animation_frame: u32,
    pub animation_playing: bool,
}

/// Authored room shapes whose geometry, doors and CameraStyle1 scrolling are
/// shape-generic: every 1x1 ordinary-family room, and the tall (13x14), wide
/// (26x7) and large (26x14) ordinary type-1 rooms. Their actors follow
/// ordinary admission; special large rooms keep their own gates.
fn admitted_shape(template: &crate::room_templates::AuthoredTemplate) -> bool {
    match (template.width, template.height) {
        // Shop2 admits its room, doors and Shopkeeper17/0; 5/150 shop
        // placeholders and the6/8 slot remain unadmitted selectors. Sacrifice
        // rooms (13) have no Room.Init branch of their own.
        (13, 7) => matches!(template.room_type, 1 | 2 | 4 | 7 | 8 | 10 | 12 | 13),
        (13, 14) | (26, 7) | (26, 14) => template.room_type == 1,
        _ => false,
    }
}

/// A 1x1 boss room (type 5) or miniboss room (type 6) is admitted once a
/// registered family constructs its authored boss. Boss rooms also need a
/// bossportraits.xml presentation for their VS scene; minibosses enter without
/// one. Other authored NPCs follow ordinary-room admission.
fn supported_boss(
    descriptor: &crate::floor::Descriptor,
    special: Option<&[crate::room_assets::RoomAsset]>,
) -> bool {
    let room_type = descriptor.template.room_type;
    if !matches!(room_type, 5 | 6)
        || (descriptor.template.width, descriptor.template.height) != (13, 7)
    {
        return false;
    }
    let Ok(asset) = crate::room_assets::for_descriptor_with_special(descriptor, special) else {
        return false;
    };
    let Ok(spawns) = crate::room_spawns::select(&asset, descriptor.spawn_seed) else {
        return false;
    };
    let context = crate::entities::SpawnContext {
        stage: 1,
        game_update_count: 0,
    };
    spawns.selectors.iter().any(|spawn| {
        matches!(crate::entities::spawn(0, spawn, &context), Ok(Some(npc))
            if npc.boss
                && (room_type == 6
                    || crate::content::boss_presentation(npc.kind, npc.variant).is_some()))
    })
}

/// Whether the player may leave through a doorway into `destination`: an
/// admitted shape, or a supported boss or miniboss room. The single owner of
/// doorway admission, public so tools route by the engine's rule.
pub fn admits(destination: &crate::floor::Descriptor) -> bool {
    admits_with_special(destination, None)
}

fn admits_with_special(
    destination: &crate::floor::Descriptor,
    special: Option<&[crate::room_assets::RoomAsset]>,
) -> bool {
    admitted_shape(&destination.template) || supported_boss(destination, special)
}

/// Game551a10 supplies the room's authored boss presentation. Source
/// TransitionChange starts Scene only after the admitted immediate Room.Update.
pub(crate) fn begin_boss_scene(state: &mut State) {
    let presentation_index = state.room.boss_presentation_index;
    if presentation_index != 0
        && !state.room.combat_clear
        && state.room.npcs.iter().any(|npc| {
            !npc.removed && crate::content::boss_presentation(npc.kind, npc.variant).is_some()
        })
    {
        if let Some(transition) = &mut state.room_transition {
            transition.phase = TransitionPhase::BossScene;
            transition.boss_scene = Some(BossScene {
                presentation_index,
                animation_frame: 0,
                animation_playing: true,
            });
        }
    }
}

fn outward(slot: u8) -> WorldVector {
    match slot % 4 {
        0 => WorldVector { x: -1.0, y: 0.0 },
        1 => WorldVector { x: 0.0, y: -1.0 },
        2 => WorldVector { x: 1.0, y: 0.0 },
        _ => WorldVector { x: 0.0, y: 1.0 },
    }
}

fn world(player: &Player) -> WorldVector {
    WorldVector {
        x: ((player.x - 32.0) / 0.65) as f32,
        y: ((player.y + 46.0) / 0.65) as f32,
    }
}

/// The generated floor grid identifies the precise doorway cell along a
/// rectangular room edge. Extended slots use the second cell on that edge;
/// selecting only the first reciprocal direction loses those connections.
fn slot_cell(descriptor: &crate::floor::Descriptor, slot: u8) -> (i32, i32) {
    let x = (descriptor.grid_index % 13) as i32;
    let y = (descriptor.grid_index / 13) as i32;
    let width = i32::from(descriptor.template.width / 13);
    let height = i32::from(descriptor.template.height / 7);
    match slot {
        0 => (x - 1, y),
        1 => (x, y - 1),
        2 => (x + width, y),
        3 => (x, y + height),
        4 => (x - 1, y + 1),
        5 => (x + 1, y - 1),
        6 => (x + width, y + 1),
        7 => (x + 1, y + height),
        _ => unreachable!("authored rectangle has eight door slots"),
    }
}

/// Boundary collision geometry includes each authored opened doorway. The
/// grid factory replaces this geometry with fully seeded authored objects.
pub(crate) fn doorway_grid(room: &Room) -> crate::grid::Grid {
    let mut descriptor = crate::grid::Grid::closed_1x1_boundary()
        .descriptor()
        .clone();
    for door in &room.appearance.doors {
        if door.state != "Opened" {
            continue;
        }
        let column = ((door.world_position.x - 40.0) / 40.0) as usize;
        let row = ((door.world_position.y - 120.0) / 40.0) as usize;
        let index = column + row * usize::from(descriptor.width);
        if let Some(cell) = descriptor.cells.get_mut(index) {
            cell.entity = Some(crate::grid::GridEntity {
                kind: 16,
                variant: 0,
                state: 2,
                collision_class: 5,
                spawn_seed: 0,
                source_owner: None,
            });
        }
    }
    crate::grid::Grid::new(descriptor).expect("authored ordinary door boundary geometry")
}

/// Room::check_player_enter_door 5c71c0 uses Euclidean distance below 25
/// from the authored grid position displaced outward by 18 world units.
/// This check runs after entities in a full Room update.
pub(crate) fn check_departure(
    state: &mut State,
    special: Option<&[crate::room_assets::RoomAsset]>,
) {
    if state.room_transition.is_some() {
        return;
    }
    let position = world(&state.player);
    for door in &state.room.appearance.doors {
        if door.state != "Opened" {
            continue;
        }
        let target_room = door.target_room as usize;
        let destination = &state.floor.descriptors[target_room];
        if !admits_with_special(destination, special) {
            continue;
        }
        let normal = outward(door.slot);
        let delta_x = (door.world_position.x + normal.x * 18.0) - position.x;
        let delta_y = (door.world_position.y + normal.y * 18.0) - position.y;
        if (delta_x * delta_x + delta_y * delta_y).sqrt() >= 25.0 {
            continue;
        }
        let opposite = (door.slot + 2) % 4;
        let destination_cell = slot_cell(&state.floor.descriptors[state.current_room], door.slot);
        let source_cell = (
            destination_cell.0 - normal.x as i32,
            destination_cell.1 - normal.y as i32,
        );
        let Some(target_slot) =
            destination
                .neighbors
                .iter()
                .enumerate()
                .find_map(|(slot, value)| {
                    (*value == Some(state.current_room)
                        && slot % 4 == usize::from(opposite)
                        && slot_cell(destination, slot as u8) == source_cell)
                        .then_some(slot as u8)
                })
        else {
            continue;
        };
        state.room_transition = Some(RoomTransition {
            from_room: state.current_room,
            target_room,
            from_slot: door.slot,
            target_slot,
            phase: TransitionPhase::Out,
            // Game::Update chooses its transition branch before Room::Update.
            full_updates: 0,
            boss_scene: None,
        });
        state.player.vx = 0.0;
        state.player.vy = 0.0;
        // Transition request5dcfc0 writes Player+7d4 at5dd048. Fast room
        // callbacks stay frozen without rewriting the persistent field.
        state.player.controls_enabled = false;
        break;
    }
}

/// The next full Game update admits a normal style0 door (5ddf20/5de26f).
/// The inbound state3 pre-counter then uses limit7 (5dde10).
/// Native room admission clears velocity and puts the player one 40-unit cell
/// inside the incoming authored doorway. Mutable Room state is restored once.
pub(crate) fn advance_transition(
    state: &mut State,
    phase: Phase,
    special: Option<&[crate::room_assets::RoomAsset]>,
) -> Result<bool, String> {
    let Some(mut transition) = state.room_transition.take() else {
        return Ok(false);
    };
    if !phase.updates_nonplayers() {
        state.room_transition = Some(transition);
        return Ok(true);
    }
    if transition.phase == TransitionPhase::BossScene {
        let scene = transition
            .boss_scene
            .as_mut()
            .expect("actual boss transition owns Scene");
        // Sprite.Update precedes IsFinished(empty-name). Authored nonloop
        // playback stops at its final frame; no fitted transition delay.
        if scene.animation_playing {
            let next = scene.animation_frame + 1;
            if next < crate::content::BOSS_SCENE_FRAMES {
                scene.animation_frame = next;
            } else {
                scene.animation_frame = crate::content::BOSS_SCENE_FRAMES - 1;
                scene.animation_playing = false;
            }
        }
        if !scene.animation_playing {
            transition.phase = TransitionPhase::BossCleanup;
            // Player50e710 sets shooting lock, not damage immunity.
            state.player.shooting_lock = state.player.shooting_lock.max(10);
        }
        state.room_transition = Some(transition);
        return Ok(true);
    }
    if transition.phase == TransitionPhase::BossCleanup {
        // Game owns the real controls-disabled Player.Update at5dde9e before
        // resuming ordinary actors. Removing the transition ends its freeze.
        return Ok(true);
    }
    if transition.phase == TransitionPhase::In && transition.full_updates < 7 {
        transition.full_updates += 1;
        state.room_transition = Some(transition);
        return Ok(true);
    }
    if transition.phase == TransitionPhase::In {
        // The state3 completion branch dispatches one Player::Update at
        // 5dde9e while the room's grid/nonplayer callbacks remain frozen.
        crate::combat::advance_player_immunity(state);
        return Ok(true);
    }
    // ChangeRoom5dd1b0 enables Player+7d4 before550970, whose admitted
    // Room.Update observes it. Game restores the incoming freeze afterward.
    state.player.controls_enabled = true;
    state
        .persisted_rooms
        .insert(state.current_room, state.room.clone());
    state.current_room = transition.target_room;
    state.room = match state.persisted_rooms.remove(&state.current_room) {
        Some(room) => room,
        // Room.Init's pit linking (5cf550) follows its grid construction in
        // admit_room_pickups.
        None => {
            Room::from_floor_descriptor_with_special(&state.floor, state.current_room, special)?
        }
    };
    // Reentry owns a new Room.Init timestamp, independent of saved geometry.
    state.room.initialization_game_update = Some(state.game_update_count);
    state.room.loaded = false;
    state.room.initialize_path_costs();
    state.visited_rooms.insert(state.current_room);
    let door = state
        .room
        .appearance
        .doors
        .iter()
        .find(|door| door.slot == transition.target_slot)
        .expect("reciprocal generated ordinary doorway is authored");
    let normal = outward(transition.target_slot);
    state.player.x = f64::from(door.world_position.x - normal.x * 40.0) * 0.65 + 32.0;
    state.player.y = f64::from(door.world_position.y - normal.y * 40.0) * 0.65 - 46.0;
    state.player.vx = 0.0;
    state.player.vy = 0.0;
    state.tears.clear();
    state.enemy_projectiles.clear();
    state.lasers.clear();
    state.fire_particles.clear();
    state.bombs.clear();
    state.explosions.clear();
    crate::attack_runtime::reset_room_schedule(state);
    transition.phase = TransitionPhase::In;
    transition.full_updates = 0;
    state.room_transition = Some(transition);
    Ok(true)
}
