//! Authored secret Keeper17/1 first-AI initialization and anchored idle.
use crate::{content, motion::WorldVector, npcs::Npc, phase::Phase, room_spawns::SpawnSelector};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KeeperState {
    pub anchor: WorldVector,
    pub entity_flags: u32,
    pub state_frame: u32,
    pub countdown: u32,
    pub animation: String,
    pub animation_clock: f32,
    pub animation_playing: bool,
    pub tint: [f32; 4],
    pub offsets: [f32; 3],
}

pub(crate) fn from_selector(id: u32, selector: &SpawnSelector) -> Result<Npc, String> {
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err(
            "Keeper admission requires bounded authored position and nonzero spawn seed".into(),
        );
    }
    Ok(Npc {
        target: selector.position,
        grid_collision_class: 0,
        entity_collision_class: 1,
        animation_playing: true,
        keeper: Some(KeeperState {
            anchor: selector.position,
            entity_flags: 4,
            state_frame: 0,
            countdown: u32::MAX,
            animation: String::new(),
            animation_clock: 0.0,
            animation_playing: true,
            tint: [1.0; 4],
            offsets: [0.0; 3],
        }),
        ..Npc::authored(id, content::entity(17, 1), selector)
    })
}

/// First-AI48af20 uses spawnSeed directly, not a RNG Next. It clears the
/// appearance flag, selects GuyN, stores the anchor and sets idle state3.
/// Later living updates copy that anchor before native BaseUpdate.
pub(crate) fn advance(npc: &mut Npc, phase: Phase) -> Result<(), String> {
    let keeper = npc
        .keeper
        .as_mut()
        .ok_or("Keeper actor requires its admitted state")?;
    if npc.removed {
        return Ok(());
    }
    if phase == Phase::Update {
        if npc.frame_age == 0 {
            npc.subtype = npc.spawn_seed % 9 + 1;
            npc.grid_collision_class = 0;
            npc.entity_collision_class = 1;
            keeper.entity_flags = (keeper.entity_flags & !4) | 0x11;
            keeper.animation = format!("Guy{}", npc.subtype);
            keeper.tint = [0.2, 0.2, 0.2, 1.0];
            keeper.offsets = [0.2; 3];
            keeper.anchor = npc.position;
            npc.target = npc.position;
            npc.state = 3;
            npc.frame_age = 1;
        }
        npc.position = keeper.anchor;
        // Native Sprite::Update runs once at the admitted full update and
        // once per active full phase; transition freezes retain this clock.
        let frames = content::KEEPER_ANIMATION_FRAMES[(npc.subtype - 1) as usize];
        keeper.animation_clock = (keeper.animation_clock + 1.0) % frames as f32;
        npc.animation_frame = keeper.animation_clock as u32;
        npc.animation = keeper.animation.clone();
        npc.animation_clock = keeper.animation_clock;
        npc.animation_playing = keeper.animation_playing;
    }
    npc.presented_position = npc.position;
    Ok(())
}
