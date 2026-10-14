//! Small authored values generated from entities2.xml, the shadow PNG and
//! the reproducibly decoded lightgradient animation frame.
//! The executable embeds the catalog, not a copy of the source asset documents.
include!(concat!(env!("OUT_DIR"), "/shadow_catalog.rs"));

/// One authored entities2.xml row. `grid_class` applies the native mapping
/// (none0, walls3, ground5; absent means ground only with collision points).
/// Identity, actor and portrait fields are for registry and asset consumers.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct EntityConfig {
    pub id: u32,
    pub variant: u32,
    pub subtype: u32,
    pub name: &'static str,
    pub actor: &'static str,
    pub portrait: Option<u32>,
    pub base_hp: f32,
    pub stage_hp: f32,
    pub boss: bool,
    pub champion: bool,
    pub collision_damage: f32,
    pub collision_mass: f32,
    pub collision_radius: f32,
    pub friction: f32,
    pub shadow_size: f32,
    pub grid_points: u16,
    pub grid_class: u32,
    pub can_shut_doors: bool,
    pub gib_amount: u32,
    pub gib_flags: u32,
}

include!(concat!(env!("OUT_DIR"), "/entity_catalog.rs"));

/// The first authored row for (id, variant). Evaluated at compile time where
/// a constant needs it; an unauthored pair is a build error there.
pub const fn entity(id: u32, variant: u32) -> &'static EntityConfig {
    let mut index = 0;
    while index < ENTITIES.len() {
        let config = &ENTITIES[index];
        if config.id == id && config.variant == variant {
            return config;
        }
        index += 1;
    }
    panic!("entity is not authored in entities2.xml");
}
