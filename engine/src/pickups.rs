//! Concrete authored room pickups. Selector RNG belongs to the spawning caller.
//! Animation and contact hooks are separate so the actor scheduler preserves
//! source admission order, including a pickup collected earlier in its phase.
use crate::{motion::WorldVector, phase::Phase, Inventory, Player};
use serde::{Deserialize, Serialize};
/// Normal Isaac's actual50c880 collection predicate. Supported guarded clips
/// include the actual Hit and UseItem names admitted by50c880/50c780. The Hit
/// callback is composed separately; its name uses this same predicate.
/// Controls7d4 and the legacy queued Pickup guard are not general collection
/// vetoes. Collectible queue admission keeps its separate existing rule.
pub(crate) fn player_can_collect(state: &crate::State) -> bool {
    !state.player.dead
        && state.player_animation.current != Some(crate::player_animation::Clip::Death)
        && (!state.player_animation.guard
            || state.player_animation.current.is_some_and(|clip| {
                matches!(
                    clip.name(),
                    "Hit" | "Pickup" | "LiftItem" | "HideItem" | "UseItem"
                )
            }))
}
use std::collections::{BTreeMap, BTreeSet};

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
    pub collision_points: u16,
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
        // Ordinary Isaac500150: independent local35 rolls retain their order.
        // Collectible/trinket replacements belong to their actual owners.
        10 => {
            let mut selected = if random(2) == 0 { 2 } else { 1 };
            if random(50) == 0 {
                selected = 4;
            }
            if random(10) == 0 {
                selected = if random(20) == 0 { 6 } else { 3 };
            }
            Ok(selected)
        }
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

/// Init can select transient Troll/Megatroll PU subtypes (40/3, 40/5)
/// without an authored pickup actor. Until their first full Update converts
/// them, they use the bomb pickup's temporary visual definition, so every
/// definition lookup for a live pickup resolves its subtype here.
pub(crate) fn definition_subtype(variant: u32, subtype: u32) -> u32 {
    if variant == 40 && matches!(subtype, 3 | 5) {
        1
    } else {
        subtype
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

/// The fresh free regular families own these two source collision fields.
/// Collectible and Chest collision state remains with their existing owners.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegularCollision {
    pub grid_collision_class: u32,
    pub entity_collision_class: u32,
}

/// Owned type5 projection of Room.Save's native 0x48-byte entity record.
/// The browser handle joins the existing mixed order; it is not a source ID.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedRegularPickup {
    pub handle: u32,
    pub variant: u32,
    pub subtype: u32,
    pub position: WorldVector,
    pub spawn_seed: u32,
    pub rng_seed: u32,
    pub charge: i32,
    pub touched: bool,
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
    pub regular_collision: Option<RegularCollision>,
    #[serde(default)]
    pub regular_rng: Option<crate::items::Rng>,
    #[serde(default)]
    pub chest: Option<crate::chests::Chest>,
}

impl Pickup {
    /// Entity.Init441c80 seeds its persistent RNG30 without a shared draw.
    /// The temporary Pickup.Init RNG35 remains a separate subtype owner.
    pub(crate) fn initialize_spawn_seed(&mut self, seed: u32) -> Result<(), String> {
        if seed == 0 {
            return Err("pickup spawn seed must be nonzero".into());
        }
        self.spawn_seed = seed;
        self.regular_rng = self
            .has_regular_physics()
            .then(|| crate::items::Rng::new(seed, [4, 3, 17]));
        Ok(())
    }

    /// Ordinary admitted pickups use the common free ground-motion branch.
    /// Collectible home return and Chest callbacks keep their own owners.
    pub(crate) fn has_regular_physics(&self) -> bool {
        matches!(self.variant, 10 | 20 | 30 | 40)
    }

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
        if selector.entity_type != 5 || !matches!(selector.variant, 10 | 20 | 30 | 40) {
            return Ok(None);
        }
        let variant = u32::from(selector.variant);
        let subtype = regular_subtype(variant, u32::from(selector.subtype), selector.spawn_seed)?;
        if !matches!(
            (variant, subtype),
            (10, 1..=6) | (20, 1..=4) | (30, 1 | 3) | (40, 1 | 2 | 3 | 5)
        ) {
            return Ok(None);
        }
        let mut pickup = Self::spawn(
            id,
            variant,
            definition_subtype(variant, subtype),
            selector.position,
            appear,
        )?;
        pickup.subtype = subtype;
        pickup.initialize_spawn_seed(selector.spawn_seed)?;
        Ok(Some(pickup))
    }

    /// Update506f70 routes these two subtypes before BaseUpdate or animation.
    pub fn bomb_conversion_variant(&self) -> Option<u32> {
        if self.removed || self.variant != 40 {
            return None;
        }
        match self.subtype {
            3 => Some(3),
            5 => Some(4),
            _ => None,
        }
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
            anchor_position: if matches!(variant, 10 | 20 | 30 | 40) {
                WorldVector { x: 0.0, y: 0.0 }
            } else {
                position
            },
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
            regular_collision: matches!(variant, 10 | 20 | 30 | 40).then_some(RegularCollision {
                grid_collision_class: 5,
                entity_collision_class: if animation == "Idle" { 4 } else { 0 },
            }),
            regular_rng: None,
            chest: None,
        })
    }

    /// Base ANM2 playback is admitted only on full nonplayer update phases.
    pub fn advance(&mut self, phase: Phase, game_update: u32) {
        if !self.removed
            && (self.variant == 100 || self.has_regular_physics())
            && phase == Phase::Interpolate
        {
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
        } else if self.has_regular_physics() {
            // Pickup506f9d compares the ordinary zero anchor before Base.
            // The true branch5075d2 copies the current physical position;
            // intermediate prediction never initializes this owner.
            if self.anchor_position.x == 0.0 && self.anchor_position.y == 0.0 {
                self.anchor_position = self.position;
            }
            //506ff9 invokes Base447190 before507029 sets the next turn's
            //temporary friction. Velocity is physical; fast prediction never
            //feeds its presented position back into this full integration.
            self.velocity = self.velocity * self.current_friction;
            self.position = self.position + self.velocity;
            self.presented_position = self.position;
            self.current_friction = self.configured_friction * 0.965;
            //507044 calls5068b0 even for a retained Collect actor. Normal
            //Isaac lacks trinket6, so506b30/506b3d restores class3 to5 here.
            if let Some(collision) = &mut self.regular_collision {
                if collision.grid_collision_class == 3 {
                    collision.grid_collision_class = 5;
                }
            }
            self.project();
        }
        // 442b30 uses the Game clock minus the entity's creation clock. The
        // immediate Room::Update on admission advances ANM2, but does not
        // invent a Game-clock tick or age while the transition is frozen.
        self.age = game_update.saturating_sub(self.birth_game_update);
        if self.animation_playing {
            let clip = definition(self.variant, definition_subtype(self.variant, self.subtype))
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
            let clip = definition(self.variant, definition_subtype(self.variant, self.subtype))
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
            if let Some(collision) = &mut self.regular_collision {
                collision.entity_collision_class = 4;
            }
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
        self.finish_collect();
        true
    }

    /// Free normal-Isaac Heart10 branches504db0..504fdf. The caller owns
    /// actual Player50c880 eligibility and contacts; health capacity is tested
    /// here before mutation. Existing health helpers retain black-slot order.
    pub(crate) fn try_collect_heart(
        &mut self,
        player: &mut Player,
        red_multiplier: i32,
        level_progress: &mut crate::level_progress::LevelProgress,
    ) -> bool {
        if self.variant != 10
            || !self.can_collect
            || self.collected
            || self.removed
            || self.collection_delay != 0
        {
            return false;
        }
        match self.subtype {
            1 | 2 | 5 if player.hp < player.max_hp => {
                let amount = match self.subtype {
                    1 => 2,
                    2 => 1,
                    _ => 4,
                };
                crate::player_items::add_red_hearts(player, amount * red_multiplier);
            }
            3 if i32::from(player.max_hp) + player.soul_hp <= 23 => {
                crate::player_items::add_soul_hearts(player, 2);
            }
            //505828 permits one Eternal even at max containers; a second
            //at that capacity is rejected by5058d6 while the first is held.
            4 if player.max_hp <= 23 || player.eternal_hearts == 0 => {
                crate::player_items::add_eternal_hearts(player, 1);
            }
            6 if i32::from(player.max_hp) + player.soul_hp <= 23
                || (1..player.soul_hp)
                    .step_by(2)
                    .any(|half| player.black_heart_mask & (1 << (half / 2)) == 0) =>
            {
                crate::player_items::add_black_hearts(player, 2);
            }
            _ => return false,
        }
        //504e67 precedes the actual Collect504480 callback.
        level_progress.heart_collected = true;
        self.finish_collect();
        true
    }

    fn finish_collect(&mut self) {
        // Collect504d30 clears physical velocity before playing Collect,
        // changes the ground class to3, then sets the dead31f flag.
        self.velocity = WorldVector { x: 0.0, y: 0.0 };
        if let Some(collision) = &mut self.regular_collision {
            collision.grid_collision_class = 3;
            collision.entity_collision_class = 0;
        }
        self.animation = "Collect".into();
        self.animation_frame = 0.0;
        self.animation_playing = true;
        self.can_collect = false;
        self.collected = true;
    }
}

/// Room.Save5ce3d0 walks the live room list, excludes dead31f, then5c57c0
/// accepts type5. Preserve only the regular families owned by this module.
pub(crate) fn save_regular_records(state: &mut crate::State) -> Result<(), String> {
    let mut records = Vec::new();
    for actor in state.actor_scheduler.full_phase() {
        if actor.kind != crate::actors::ActorKind::Pickup {
            continue;
        }
        let Some(pickup) = state
            .room
            .pickups
            .iter()
            .find(|pickup| pickup.id == actor.id)
        else {
            return Err("scheduled pickup missing at room save".into());
        };
        if !pickup.has_regular_physics() || pickup.collected || pickup.removed {
            continue;
        }
        let rng = pickup
            .regular_rng
            .ok_or("saved regular pickup has no persistent RNG30")?;
        records.push(SavedRegularPickup {
            handle: pickup.id,
            variant: pickup.variant,
            subtype: pickup.subtype,
            position: pickup.position,
            spawn_seed: pickup.spawn_seed,
            rng_seed: rng.state,
            charge: pickup.charge,
            touched: pickup.touched,
        });
    }
    state.room.saved_regular_pickups = records;
    Ok(())
}

/// RoomLoad5cf585 calls5c5860 for Descriptor+70..78 saved records. Each
/// GameSpawn receives stored position+20/seed+2c and explicit zero velocity.
/// The distinct Descriptor+b8 additional-spawn vector is not this owner.
pub(crate) fn readmit_regular_records(state: &mut crate::State) -> Result<Vec<u32>, String> {
    let records = std::mem::take(&mut state.room.saved_regular_pickups);
    let mut admitted = Vec::with_capacity(records.len());
    let mut handles = BTreeSet::new();
    for record in records {
        if !matches!(record.variant, 10 | 20 | 30 | 40) || !handles.insert(record.handle) {
            return Err("invalid saved regular pickup record".into());
        }
        // Saved subtype is concrete. Fresh normal Isaac has no modifier that
        // rerolls it; initializing temporary RNG35 consumes no output here.
        let subtype = regular_subtype(record.variant, record.subtype, record.spawn_seed)?;
        let mut pickup = Pickup::spawn(
            record.handle,
            record.variant,
            definition_subtype(record.variant, subtype),
            record.position,
            false,
        )?;
        pickup.subtype = subtype;
        pickup.initialize_spawn_seed(record.spawn_seed)?;
        pickup.regular_rng = Some(crate::items::Rng::new(record.rng_seed, [4, 3, 17]));
        pickup.birth_game_update = state.game_update_count;
        pickup.charge = record.charge;
        pickup.touched = record.touched;
        admitted.push(pickup);
    }
    let ids = admitted.iter().map(|pickup| pickup.id).collect();
    let mut replacements: BTreeMap<_, _> = admitted
        .into_iter()
        .map(|pickup| (pickup.id, pickup))
        .collect();
    let old_pickups = std::mem::take(&mut state.room.pickups);
    for pickup in old_pickups {
        if !pickup.has_regular_physics() {
            state.room.pickups.push(pickup);
        } else if let Some(restored) = replacements.remove(&pickup.id) {
            state.room.pickups.push(restored);
        }
    }
    if !replacements.is_empty() {
        return Err("saved pickup has no room handle".into());
    }
    state.room.authored_actor_order.retain(|actor| {
        actor.kind != crate::actors::ActorKind::Pickup
            || state
                .room
                .pickups
                .iter()
                .any(|pickup| pickup.id == actor.id)
            || handles.contains(&actor.id)
    });
    Ok(ids)
}

/// Full Pickup Update owns motion/friction; the manager owns its later grid
/// pass. Fast4446a0 predicts and resolves its own ground preview directly.
pub(crate) fn advance_actor(state: &mut crate::State, phase: Phase, id: u32) -> Result<(), String> {
    let Some(index) = state.room.pickups.iter().position(|pickup| pickup.id == id) else {
        return Ok(());
    };
    state.room.pickups[index].advance(phase, state.game_update_count);
    let pickup = &mut state.room.pickups[index];
    if phase != Phase::Interpolate
        || !pickup.has_regular_physics()
        || !has_ground_class(pickup)
        || pickup.removed
    {
        return Ok(());
    }
    let grid = crate::grid::Grid::new(
        state
            .room
            .grid
            .clone()
            .ok_or("admitted pickup requires room grid")?,
    )?;
    resolve_grid(pickup, &grid, true)
}

/// 4545d0 finishes all entity contacts, then454b10 invokes442b90 for the live
/// nonplayer list. New births get this grid pass without a full actor update.
pub(crate) fn full_grid_contacts(state: &mut crate::State) -> Result<(), String> {
    if !state.room.pickups.iter().any(has_ground_class) {
        return Ok(());
    }
    let grid = crate::grid::Grid::new(
        state
            .room
            .grid
            .clone()
            .ok_or("admitted pickup requires room grid")?,
    )?;
    for pickup in &mut state.room.pickups {
        if has_ground_class(pickup) {
            resolve_grid(pickup, &grid, false)?;
        }
    }
    Ok(())
}

fn has_ground_class(pickup: &Pickup) -> bool {
    !pickup.removed
        && pickup
            .regular_collision
            .as_ref()
            .is_some_and(|collision| collision.grid_collision_class == 5)
}

/// Pool+20 is constructed before callbacks at4545d0. Its snapshot excludes
/// actors born during contacts; the later live-list ground pass includes them.
pub(crate) struct ContactCandidates {
    cells: Vec<Vec<u32>>,
}

pub(crate) fn contact_candidates(state: &crate::State) -> Result<ContactCandidates, String> {
    //454381..4543a7 ->454070:80-unit cells,1200x720 extent (15x9).
    let mut cells = vec![Vec::new(); 15 * 9];
    for actor in state.actor_scheduler.full_phase() {
        if actor.kind != crate::actors::ActorKind::Pickup {
            continue;
        }
        let pickup = state
            .room
            .pickups
            .iter()
            .find(|pickup| pickup.id == actor.id)
            .ok_or("pickup contact handle requires its admitted actor")?;
        if !has_entity_contacts(pickup) {
            continue;
        }
        let radius = pickup.collision_radius;
        if !pickup.position.is_finite() || !radius.is_finite() || radius <= 0.0 {
            return Err("pickup contact pool requires finite authored geometry".into());
        }
        //452bd0 uses f32 component division and truncation toward zero, then
        //row-major admission. Each cell retains at most40 actor pointers.
        let left = ((pickup.position.x - radius) / 80.0) as i32;
        let right = ((pickup.position.x + radius) / 80.0) as i32;
        let top = ((pickup.position.y - radius) / 80.0) as i32;
        let bottom = ((pickup.position.y + radius) / 80.0) as i32;
        for row in top..=bottom {
            for column in left..=right {
                let index = i64::from(row) * 15 + i64::from(column);
                if let Some(cell) = usize::try_from(index)
                    .ok()
                    .and_then(|index| cells.get_mut(index))
                {
                    if cell.len() < 40 {
                        cell.push(pickup.id);
                    }
                }
            }
        }
    }
    Ok(ContactCandidates { cells })
}

fn has_entity_contacts(pickup: &Pickup) -> bool {
    !pickup.collected
        && !pickup.removed
        && pickup
            .regular_collision
            .as_ref()
            .is_some_and(|collision| collision.entity_collision_class == 4)
}

///453340 walks cells and their admitted pointer order, deduplicating each
///unordered pair before445be0 ->445710. Free PU callbacks504820 veto neither
///mode1 nor mode0; the common circle kernel owns their physical mass response.
pub(crate) fn nonplayer_contacts(
    state: &mut crate::State,
    candidates: ContactCandidates,
) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for cell in candidates.cells {
        for first in 0..cell.len() {
            for second in first + 1..cell.len() {
                let first_id = cell[first];
                let second_id = cell[second];
                if !seen.insert((first_id.min(second_id), first_id.max(second_id))) {
                    continue;
                }
                let first_index = state
                    .room
                    .pickups
                    .iter()
                    .position(|pickup| pickup.id == first_id)
                    .ok_or("pickup contact candidate must survive until manager cleanup")?;
                let second_index = state
                    .room
                    .pickups
                    .iter()
                    .position(|pickup| pickup.id == second_id)
                    .ok_or("pickup contact candidate must survive until manager cleanup")?;
                let (first, second) = if first_index < second_index {
                    let (left, right) = state.room.pickups.split_at_mut(second_index);
                    (&mut left[first_index], &mut right[0])
                } else {
                    let (left, right) = state.room.pickups.split_at_mut(first_index);
                    (&mut right[0], &mut left[second_index])
                };
                if !has_entity_contacts(first) || !has_entity_contacts(second) {
                    continue;
                }
                let body = |pickup: &Pickup| crate::combat::ContactBody {
                    id: pickup.id,
                    entity_type: 5,
                    position: pickup.position,
                    velocity: pickup.velocity,
                    radius: pickup.collision_radius,
                    mass: pickup.collision_mass,
                };
                let mut a = body(first);
                let mut b = body(second);
                if crate::combat::resolve_pickup_pair(&mut a, &mut b)?.colliding {
                    first.position = a.position;
                    first.presented_position = a.position;
                    first.velocity = a.velocity;
                    first.project();
                    second.position = b.position;
                    second.presented_position = b.position;
                    second.velocity = b.velocity;
                    second.project();
                }
            }
        }
    }
    Ok(())
}

/// Shared authored geometry; family owners choose their collision eligibility.
pub(crate) fn resolve_grid(
    pickup: &mut Pickup,
    grid: &crate::grid::Grid,
    preview: bool,
) -> Result<(), String> {
    let definition = definition(
        pickup.variant,
        definition_subtype(pickup.variant, pickup.subtype),
    )
    .ok_or("moving pickup requires authored collision definition")?;
    let collider = crate::grid::Collider::new(crate::grid::CollisionGeometry {
        radius: pickup.collision_radius,
        scale: WorldVector { x: 1.0, y: 1.0 },
        points: definition.collision_points,
    })?;
    if !preview {
        collider.resolve_nonplayer_ground(
            grid,
            &mut pickup.position,
            &mut pickup.velocity,
            false,
        )?;
        pickup.presented_position = pickup.position;
    } else {
        let mut preview_velocity = pickup.velocity;
        collider.resolve_nonplayer_ground(
            grid,
            &mut pickup.presented_position,
            &mut preview_velocity,
            true,
        )?;
    }
    pickup.project();
    Ok(())
}

/// The real554490 actor circle changes velocity immediately; ordinary Pickup
/// HP0 rejects the later damage virtual. No loot or shared-RNG word is sampled.
pub(crate) fn bomb_hit(
    state: &mut crate::State,
    center: WorldVector,
    blast_radius: f32,
    line_clear: &dyn Fn(WorldVector) -> bool,
) -> Result<(), String> {
    if !center.is_finite() {
        return Err("pickup blast center must be finite".into());
    }
    for pickup in &mut state.room.pickups {
        //Appear has source entity class0; Collect has dead31f. Admitted
        //ordinary families have ground class5 and positive authored mass≤20.
        if !pickup.has_regular_physics()
            || !pickup.can_collect
            || pickup.collected
            || pickup.removed
            || pickup.collision_mass == 0.0
        {
            continue;
        }
        let delta = WorldVector {
            x: pickup.position.x - center.x,
            y: pickup.position.y - center.y,
        };
        let squared = delta.x * delta.x + delta.y * delta.y;
        let radius = blast_radius + pickup.collision_radius;
        if !squared.is_finite() || !radius.is_finite() {
            return Err("pickup blast geometry overflowed".into());
        }
        if squared == 0.0 || squared >= radius * radius || !line_clear(pickup.position) {
            continue;
        }
        let distance = squared.sqrt();
        let impulse = (radius - distance).min(15.0);
        //612b30 divides each f32 component before612a70 multiplies impulse;
        //this differs from Normalize612820's rounded reciprocal multiply.
        let radial = WorldVector {
            x: delta.x / distance,
            y: delta.y / distance,
        } * impulse;
        pickup.velocity = pickup.velocity * 0.5 + radial;
        if !pickup.velocity.is_finite() {
            return Err("pickup blast velocity overflowed".into());
        }
    }
    Ok(())
}
