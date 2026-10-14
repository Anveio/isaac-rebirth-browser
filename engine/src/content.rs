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

/// Entity::Init446b03 initial HP: stage is capped at 10 before the later-stage
/// f32 factor, and stage HP adds to the base (a zero stage HP keeps the base).
pub fn initial_hp(config: &EntityConfig, stage: u32) -> f32 {
    let scale = stage.min(10) as f32 * if stage > 4 { 0.8 } else { 1.0 };
    scale * config.stage_hp + config.base_hp
}

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

/// One authored ANM2 clip of an NPC actor.
#[derive(Clone, Copy, Debug)]
pub struct Clip {
    pub actor: &'static str,
    pub name: &'static str,
    pub frames: u32,
    pub looping: bool,
    /// anm2 Triggers: (event name, AtFrame).
    pub events: &'static [(&'static str, u32)],
}

include!(concat!(env!("OUT_DIR"), "/clip_catalog.rs"));

pub fn clip(actor: &str, name: &str) -> Option<&'static Clip> {
    CLIPS
        .iter()
        .find(|clip| clip.actor == actor && clip.name == name)
}

/// One bossportraits.xml VS-screen presentation.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct BossPresentation {
    pub id: u32,
    pub entity_type: u32,
    pub variant: u32,
    pub portrait: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/boss_catalog.rs"));

/// The authored VS-screen presentation id for a boss entity.
pub fn boss_presentation(entity_type: u32, variant: u32) -> Option<u32> {
    BOSS_PRESENTATIONS
        .iter()
        .find(|boss| boss.entity_type == entity_type && boss.variant == variant)
        .map(|boss| boss.id)
}
