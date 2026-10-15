//! Authored Keeper17 first-AI initialization and anchored idle: shop Keeper
//! variant0 and secret-room Keeper variant1 share AI48af20.
use crate::entities::{Birth, Family, NpcWorld, SpawnContext};
use crate::{
    content,
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    phase::Phase,
    room_spawns::{SpawnCategory, SpawnSelector},
};
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
        ..Npc::authored(
            id,
            content::entity(17, u32::from(selector.variant)),
            selector,
        )
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
            // 48af8c: variant0 formats "Shopkeeper %u", variant1 "Guy%u".
            keeper.animation = if npc.variant == 0 {
                format!("Shopkeeper {}", npc.subtype)
            } else {
                format!("Guy{}", npc.subtype)
            };
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
        let clip = (npc.subtype - 1) as usize;
        if npc.variant == 0 {
            // Nonlooping authored Shopkeeper clips stop on their final frame.
            let frames = content::SHOPKEEPER_ANIMATION_FRAMES[clip] as f32;
            if keeper.animation_playing {
                keeper.animation_clock += 1.0;
                if keeper.animation_clock >= frames {
                    if content::SHOPKEEPER_ANIMATION_LOOPS[clip] {
                        keeper.animation_clock %= frames;
                    } else {
                        keeper.animation_clock = frames - 1.0;
                        keeper.animation_playing = false;
                    }
                }
            }
        } else {
            let frames = content::KEEPER_ANIMATION_FRAMES[clip];
            keeper.animation_clock = (keeper.animation_clock + 1.0) % frames as f32;
        }
        npc.animation_frame = keeper.animation_clock as u32;
        npc.animation = keeper.animation.clone();
        npc.animation_clock = keeper.animation_clock;
        npc.animation_playing = keeper.animation_playing;
    }
    npc.presented_position = npc.position;
    Ok(())
}

pub(crate) const FAMILY: Family = Family {
    kinds: &[17],
    spawn: spawn_family,
    advance: advance_family,
    post_contact_grid: crate::entities::no_grid_query,
    entity_flags: family_flags,
    occupancy: crate::entities::Occupancy::None,
};

fn spawn_family(
    id: u32,
    selector: &SpawnSelector,
    _: &SpawnContext,
) -> Result<Option<Npc>, String> {
    if selector.category == SpawnCategory::Entity
        && selector.entity_type == 17
        && matches!(selector.variant, 0 | 1)
        && selector.subtype == 0
    {
        return from_selector(id, selector).map(Some);
    }
    Ok(None)
}

fn advance_family(
    npc: &mut Npc,
    phase: Phase,
    _: &mut Mt19937,
    _: &mut NpcWorld,
    _: &mut Vec<Birth>,
) -> Result<(), String> {
    advance(npc, phase)
}

fn family_flags(npc: &Npc) -> u32 {
    npc.keeper.as_ref().map_or(4, |keeper| keeper.entity_flags)
}
