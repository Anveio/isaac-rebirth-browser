//! Concrete authored room pickups. Selector RNG belongs to the spawning caller.
//! Animation and contact hooks are separate so the actor scheduler preserves
//! source admission order, including a pickup collected earlier in its phase.
use crate::{motion::WorldVector, phase::Phase, Inventory, Player};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Clip {
    pub name: &'static str,
    pub frames: u32,
    pub looping: bool,
}

#[derive(Debug)]
pub struct Definition {
    pub variant: u32,
    pub subtype: u32,
    pub actor: &'static str,
    pub collision_radius: f32,
    pub collision_mass: f32,
    pub friction: f32,
    pub shadow_size: f64,
    pub clips: &'static [Clip],
}

include!(concat!(env!("OUT_DIR"), "/pickup_catalog.rs"));

/// Fresh ordinary Isaac pickup Init owns a new RNG35 seeded by the entity's
/// spawn seed. Item modifiers and special-room replacements are routed by the
/// caller; zero subtype never means an arbitrary fixed regular subtype.
pub fn regular_subtype(variant: u32, subtype: u32, spawn_seed: u32) -> Result<u32, String> {
    if subtype != 0 {
        return Ok(subtype);
    }
    if spawn_seed == 0 {
        return Err("pickup spawn seed must be nonzero".into());
    }
    let mut state = spawn_seed;
    let mut random = |bound| {
        state = crate::room_spawns::next35(state);
        state % bound
    };
    match variant {
        20 => {
            let mut selected = if random(20) == 0 { 2 } else { 1 };
            if random(100) == 0 {
                selected = 3;
            }
            Ok(selected)
        }
        // Pickup::Init4ffcae: ordinary Isaac owns no Humbling Bundle;
        // Random(50)==0 selects the golden key, never a double key.
        30 => Ok(if random(50) == 0 { 2 } else { 1 }),
        // Pickup::Init4ffd4e: fresh Isaac owns neither Humbling Bundle nor
        // BOGO Bombs. Later Megatroll/Troll draws override the regular/double roll.
        40 => {
            let mut selected = if random(7) == 0 { 2 } else { 1 };
            if random(50) == 0 {
                selected = 5;
            }
            if random(10) == 0 {
                selected = 3;
            }
            Ok(selected)
        }
        _ => Err(format!(
            "pickup selector {variant}/0 requires its owning pool or subtype resolver"
        )),
    }
}

pub fn definition(variant: u32, subtype: u32) -> Option<&'static Definition> {
    DEFINITIONS
        .iter()
        .find(|entry| entry.variant == variant && entry.subtype == subtype)
        .or_else(|| {
            DEFINITIONS
                .iter()
                .find(|entry| entry.variant == variant && entry.subtype == 0)
        })
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Pickup {
    pub id: u32,
    pub spawn_seed: u32,
    pub variant: u32,
    pub subtype: u32,
    pub spritesheet_item_id: u32,
    pub position: WorldVector,
    pub presented_position: WorldVector,
    pub anchor_position: WorldVector,
    pub velocity: WorldVector,
    pub configured_friction: f32,
    pub current_friction: f32,
    pub collision_mass: f32,
    pub x: f64,
    pub y: f64,
    pub actor: String,
    pub animation: String,
    pub animation_frame: f64,
    pub animation_playing: bool,
    pub overlay: Option<String>,
    pub overlay_frame: f32,
    pub overlay_playing: bool,
    pub can_collect: bool,
    pub collected: bool,
    pub removed: bool,
    pub collision_radius: f32,
    pub shadow_size: f64,
    pub collection_delay: u32,
    pub age: u32,
    pub birth_game_update: u32,
    pub charge: i32,
    pub touched: bool,
    #[serde(default)]
    pub chest: Option<crate::chests::Chest>,
}

impl Pickup {
    /// Concrete room admission for the supported fresh normal Isaac families.
    /// Unsupported runtime selectors remain in RoomSpawns for their owner.
    pub fn from_selector(
        id: u32,
        selector: &crate::room_spawns::SpawnSelector,
        appear: bool,
    ) -> Result<Option<Self>, String> {
        if selector.entity_type == 5 && selector.variant == 50 {
            return crate::chests::from_selector(id, selector, appear);
        }
        if selector.entity_type != 5 || !matches!(selector.variant, 20 | 30 | 40) {
            return Ok(None);
        }
        let variant = u32::from(selector.variant);
        let subtype = regular_subtype(variant, u32::from(selector.subtype), selector.spawn_seed)?;
        if !matches!((variant, subtype), (20, 1..=4) | (30, 1 | 3) | (40, 1 | 2)) {
            return Ok(None);
        }
        let mut pickup = Self::spawn(id, variant, subtype, selector.position, appear)?;
        pickup.spawn_seed = selector.spawn_seed;
        Ok(Some(pickup))
    }
    /// Natural collectible Init owns the entity's local RNG35 seed; only its
    /// first raw output is handed to the independent ItemPool shift4 picker.
    pub fn from_collectible_selector(
        id: u32,
        selector: &crate::room_spawns::SpawnSelector,
        pools: &mut crate::items::ItemPools,
        room_type: u32,
        shared: &mut crate::mt::Mt19937,
    ) -> Result<Option<Self>, String> {
        if selector.entity_type != 5 || selector.variant != 100 {
            return Ok(None);
        }
        let subtype = if selector.subtype == 0 {
            let pool = pools.room_pool(room_type, shared);
            pools.collectible(pool, true, crate::room_spawns::next35(selector.spawn_seed))?
        } else {
            u32::from(selector.subtype)
        };
        let config = crate::items::definition(subtype)
            .ok_or("authored collectible subtype has no ItemConfig")?;
        let mut pickup = Self::spawn(id, 100, subtype, selector.position, false)?;
        pickup.spawn_seed = selector.spawn_seed;
        pickup.charge = config.max_charges;
        Ok(Some(pickup))
    }
    /// Concrete subtype and authored world position come from room/loot routing.
    /// A restored pickup starts idle; a fresh ordinary pickup starts appearing.
    pub fn spawn(
        id: u32,
        variant: u32,
        subtype: u32,
        position: WorldVector,
        appear: bool,
    ) -> Result<Self, String> {
        if !position.is_finite() {
            return Err("pickup position must be finite".into());
        }
        let definition = definition(variant, subtype)
            .ok_or_else(|| format!("unresolved pickup selector {variant}/{subtype}"))?;
        let animation = if variant == 100 && subtype == 0 {
            "Empty"
        } else if appear && variant != 100 && variant != 380 {
            "Appear"
        } else {
            "Idle"
        };
        if !definition.clips.iter().any(|clip| clip.name == animation) {
            return Err(format!(
                "pickup {variant}/{subtype} has no {animation} clip"
            ));
        }
        Ok(Self {
            id,
            spawn_seed: 0,
            variant,
            subtype,
            spritesheet_item_id: subtype,
            position,
            presented_position: position,
            anchor_position: position,
            velocity: WorldVector { x: 0.0, y: 0.0 },
            configured_friction: definition.friction,
            current_friction: definition.friction,
            collision_mass: definition.collision_mass,
            x: f64::from(position.x) * 0.65 + 32.0,
            y: f64::from(position.y) * 0.65 - 46.0,
            actor: definition.actor.into(),
            animation: animation.into(),
            animation_frame: 0.0,
            animation_playing: true,
            overlay: (variant == 100).then(|| "Alternates".into()),
            overlay_frame: 0.0,
            overlay_playing: false,
            can_collect: animation == "Idle",
            collected: false,
            removed: false,
            collision_radius: definition.collision_radius,
            shadow_size: definition.shadow_size,
            collection_delay: 0,
            age: 0,
            birth_game_update: 0,
            charge: 0,
            touched: false,
            chest: None,
        })
    }

    /// Base ANM2 playback is admitted only on full nonplayer update phases.
    pub fn advance(&mut self, phase: Phase, game_update: u32) {
        if !self.removed && self.variant == 100 && phase == Phase::Interpolate {
            // 4446a0 backs up the physical448 position, then predicts from
            // stored velocity/current friction. Game550860 restores it before
            // the next full update; this model keeps that physical owner intact.
            self.presented_position = self.position + (self.velocity * self.current_friction) * 0.5;
            self.project();
            return;
        }
        if self.removed || !phase.updates_nonplayers() {
            return;
        }
        if self.variant == 100 {
            // Pickup507170 assigns anchor-position before Base447190 applies
            // current friction to velocity and moves. Base restores configured
            // friction, then507029 multiplies it by the actual source .965.
            self.velocity = WorldVector {
                x: self.anchor_position.x - self.position.x,
                y: self.anchor_position.y - self.position.y,
            } * self.current_friction;
            self.position = self.position + self.velocity;
            self.presented_position = self.position;
            self.current_friction = self.configured_friction * 0.965;
            self.project();
        }
        // 442b30 uses the Game clock minus the entity's creation clock. The
        // immediate Room::Update on admission advances ANM2, but does not
        // invent a Game-clock tick or age while the transition is frozen.
        self.age = game_update.saturating_sub(self.birth_game_update);
        if self.animation_playing {
            let clip = definition(self.variant, self.subtype)
                .expect("admitted pickup definition")
                .clips
                .iter()
                .find(|clip| clip.name == self.animation)
                .expect("admitted pickup clip");
            let next = self.animation_frame as f32 + 1.0;
            if next.floor() >= clip.frames as f32 {
                self.animation_frame = if clip.looping {
                    0.0
                } else {
                    f64::from(clip.frames.saturating_sub(1))
                };
                self.animation_playing = clip.looping;
            } else {
                self.animation_frame = f64::from(next);
            }
        }
        if self.overlay_playing {
            let clip = definition(self.variant, self.subtype)
                .expect("admitted pickup definition")
                .clips
                .iter()
                .find(|clip| Some(clip.name) == self.overlay.as_deref())
                .expect("admitted pickup overlay");
            self.overlay_frame += 1.0;
            if self.overlay_frame >= clip.frames as f32 {
                self.overlay_frame = if clip.looping {
                    0.0
                } else {
                    clip.frames as f32 - 1.0
                };
                self.overlay_playing = clip.looping;
            }
        }
        if !self.animation_playing && self.animation == "Appear" {
            self.animation = "Idle".into();
            self.animation_frame = 0.0;
            self.animation_playing = true;
            self.can_collect = true;
            if let Some(chest) = &mut self.chest {
                // Pickup Update507115/50715a admits the closed Chest's Idle
                // clip and player entity contacts after Appear finishes.
                chest.entity_collision_class = 4;
            }
        } else if !self.animation_playing && self.collected && self.variant != 100 {
            self.removed = true;
        }
        self.collection_delay = self.collection_delay.saturating_sub(1);
    }

    pub fn touches(&self, player: &Player) -> bool {
        let px = ((player.x - 32.0) / 0.65) as f32;
        let py = ((player.y + 46.0) / 0.65) as f32;
        let dx = px - self.position.x;
        let dy = py - self.position.y;
        let radius = self.collision_radius + crate::content::PLAYER_COLLISION_RADIUS_WORLD as f32;
        (dx * dx + dy * dy).sqrt() < radius
    }

    pub(crate) fn project(&mut self) {
        self.x = f64::from(self.presented_position.x) * 0.65 + 32.0;
        self.y = f64::from(self.presented_position.y) * 0.65 - 46.0;
    }

    /// Free regular coins, keys and bombs. Other families keep typed room selectors
    /// until their source collection rules and live gates are admitted.
    pub fn try_collect(&mut self, inventory: &mut Inventory) -> bool {
        if !self.can_collect || self.collected || self.removed || self.collection_delay != 0 {
            return false;
        }
        match (self.variant, self.subtype) {
            (20, 1..=4) => {
                let amount = [1, 5, 10, 2][self.subtype as usize - 1];
                inventory.coins = inventory.coins.saturating_add(amount).min(99);
            }
            (30, 1 | 3) => {
                let amount = if self.subtype == 3 { 2 } else { 1 };
                inventory.keys = inventory.keys.saturating_add(amount).min(99);
            }
            (40, 1 | 2) => {
                inventory.bombs = inventory.bombs.saturating_add(self.subtype).min(99);
            }
            _ => return false,
        }
        self.animation = "Collect".into();
        self.animation_frame = 0.0;
        self.animation_playing = true;
        self.can_collect = false;
        self.collected = true;
        true
    }
}
