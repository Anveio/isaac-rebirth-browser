//! NPC family registry. A family owns construction, the full/intermediate
//! update callback and the post-contact grid query for its authored types.
//! Game owns one shared update context and admits each family's births in
//! producer order right after its callback returns, so shared-MT draws and
//! actor identities keep the source order. Families are never batched: the
//! scheduler still visits one actor callback at a time.
use crate::{
    audio::AudioState,
    enemy_projectiles::ProjectileBirth,
    familiars::NpcEffectAdmission,
    fireplaces::{LootBirth, ParticleBirth},
    grid::{Grid, GridDescriptor},
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    phase::Phase,
    player_items::PlayerItems,
    room_costs::RoomCosts,
    room_spawns::SpawnSelector,
    roundworm::Shot,
};

pub(crate) struct SpawnContext {
    pub stage: u32,
    pub game_update_count: u32,
}

/// The union of the owners any supported NPC callback reads or writes.
pub(crate) struct NpcWorld<'a> {
    pub game_counter: u32,
    pub player: WorldVector,
    pub player_velocity: WorldVector,
    pub grid: &'a Grid,
    pub grid_descriptor: Option<&'a mut GridDescriptor>,
    pub path_costs: Option<&'a mut RoomCosts>,
    /// FreePosition(mode1) occupancy, gathered only for families that ask.
    pub occupied: &'a [WorldVector],
    pub audio: &'a mut AudioState,
    pub system_counter: u32,
    pub player_items: Option<&'a PlayerItems>,
    pub effects: NpcEffectAdmission<'a>,
}

/// Actors an NPC callback creates. Game admits them in this order.
pub(crate) enum Birth {
    Blood(u32),
    Projectile(ProjectileBirth),
    Shot(Shot),
    Ember(ParticleBirth),
    Loot(LootBirth),
}

pub(crate) type Advance =
    fn(&mut Npc, Phase, &mut Mt19937, &mut NpcWorld, &mut Vec<Birth>) -> Result<(), String>;

pub(crate) struct Family {
    /// Authored entity types this family's callbacks own.
    pub kinds: &'static [u32],
    pub spawn: fn(u32, &SpawnSelector, &SpawnContext) -> Result<Option<Npc>, String>,
    pub advance: Advance,
    pub post_contact_grid: fn(&mut Npc, &Grid) -> Result<(), String>,
    /// Actual Entity+318 view of the family's own flag owner.
    pub entity_flags: fn(&Npc) -> u32,
    pub needs_occupancy: bool,
}

/// Families without a subclass flag owner keep Init441c80's cleared flags
/// ORed with4.
pub(crate) fn ordinary_flags(_: &Npc) -> u32 {
    4
}

pub(crate) fn no_grid_query(_: &mut Npc, _: &Grid) -> Result<(), String> {
    Ok(())
}

/// Source class3 clamps the center first. Families with zero grid
/// collision points have an empty sample loop after it.
pub(crate) fn clamp_walls(npc: &mut Npc, grid: &Grid) -> Result<(), String> {
    if npc.grid_collision_class == 3 {
        npc.position = grid.clamp_room(npc.position, 0.0);
        npc.presented_position = npc.position;
    }
    Ok(())
}

/// One line per supported family.
pub(crate) static FAMILIES: &[&Family] = &[
    &crate::npcs::FLY_FAMILY,
    &crate::keepers::FAMILY,
    &crate::monstro::FAMILY,
    &crate::fireplaces::FAMILY,
    &crate::level2_flies::FAMILY,
    &crate::dips::FAMILY,
    &crate::roundworm::FAMILY,
    &crate::horfs::FAMILY,
];

pub(crate) fn family(kind: u32) -> Option<&'static Family> {
    FAMILIES
        .iter()
        .copied()
        .find(|family| family.kinds.contains(&kind))
}

/// The first family whose constructor admits the authored selector.
pub(crate) fn spawn(
    id: u32,
    selector: &SpawnSelector,
    context: &SpawnContext,
) -> Result<Option<Npc>, String> {
    for family in FAMILIES {
        if let Some(npc) = (family.spawn)(id, selector, context)? {
            return Ok(Some(npc));
        }
    }
    Ok(None)
}
