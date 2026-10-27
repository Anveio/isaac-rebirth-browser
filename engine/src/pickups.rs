//! Concrete authored room pickups. Selector RNG belongs to the spawning caller.
//! Animation and contact hooks are separate so the actor scheduler preserves
//! source admission order, including a pickup collected earlier in its phase.
use crate::{motion::WorldVector, phase::Phase, Inventory, Player};
use serde::{Deserialize, Serialize};
///Pickup.Collect504820's AL result controls the later445710 mass response.
///Failure to acquire an item is not itself a physics decision: AD0 returns1,
///while a free normal-Player UseItem refusal returns0. Unknown callback
///inputs are separate from either source return value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlayerContactDecision {
    NotReached,
    AllowPhysicalResponse,
    VetoPhysicalResponse,
    Unowned,
}
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
    /// Frame of the clip's authored DropSound trigger, if any.
    pub drop_sound: Option<u32>,
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
#[derive(Debug)]
pub struct PocketCardDefinition {
    pub name: &'static str,
    pub description: &'static str,
}
#[derive(Debug)]
pub struct PocketPillDefinition {
    pub name: &'static str,
}
include!(concat!(env!("OUT_DIR"), "/card_catalog.rs"));

pub(crate) fn card_definition(id: u32) -> Option<&'static PocketCardDefinition> {
    CARD_DEFINITIONS.get(id as usize)
}

///Actual57BF80 config lookup after final effect resolution.
pub(crate) fn pill_definition(effect: u32) -> Option<&'static PocketPillDefinition> {
    PILL_DEFINITIONS.get(effect as usize)
}

/// GetCard5817A0 uses an independent temporary RNG35. Unlock filtering keeps
/// ascending configuration order and falls back once to all forty card IDs.
/// Neither the run ItemPool state nor the shared MT stream is sampled.
pub(crate) fn random_card(
    seed: u32,
    achievements: &[bool],
    suits: bool,
    runes: bool,
) -> Result<u32, String> {
    if seed == 0 || achievements.len() != 179 {
        return Err("GetCard requires its actual nonzero seed and 179 achievement bytes".into());
    }
    let mut rng = crate::items::Rng::new(seed, [5, 9, 7]);
    let (first, last) = if rng.bounded(25) == 0 {
        (36, 40)
    } else if rng.bounded(10) == 0 && runes {
        (28, 35)
    } else if rng.bounded(5) == 0 && suits {
        (23, 27)
    } else {
        (1, 22)
    };
    let eligible = |id: &u32| {
        let achievement = CARD_ACHIEVEMENTS[*id as usize];
        achievement < 0 || achievements.get(achievement as usize).copied() == Some(true)
    };
    let mut candidates: Vec<u32> = (first..=last).filter(eligible).collect();
    if candidates.is_empty() {
        candidates = (1..=40).filter(eligible).collect();
    }
    if candidates.is_empty() {
        return Err("GetCard has no unlocked authored card".into());
    }
    Ok(candidates[rng.bounded(candidates.len() as u32) as usize])
}

///Independent actual Have operands and the selected profile's unlock bytes.
///The same Init core serves room admission, chest children and later drops;
///a missing unrelated property never vetoes an explicit resolved family.
#[derive(Clone, Copy)]
pub(crate) struct PocketSubtypeInputs {
    little_baggy: Option<bool>,
    starter_deck: Option<bool>,
    achievements: Option<[bool; 179]>,
}
impl PocketSubtypeInputs {
    pub(crate) fn from_state(state: &crate::State) -> Self {
        let items = state
            .player
            .entity_source
            .filter(|source| source.entity_type == 1 && source.variant == 0)
            .and(state.player_items.as_ref());
        Self {
            little_baggy: items
                .and_then(|owner| owner.owned_counts.get(252))
                .map(|count| *count > 0),
            starter_deck: items
                .and_then(|owner| owner.owned_counts.get(251))
                .map(|count| *count > 0),
            achievements: state
                .profiles
                .get(usize::from(state.profile).wrapping_sub(1))
                .and_then(|profile| profile.achievements.as_slice().try_into().ok()),
        }
    }
}

///The actual preconversion runs once, even for explicit subtypes. Card300
///reads252 then251 only on its positive branch; Pill70 reads251 directly.
///The conversion jumps to the selected resolver without restarting the
///modifier prelude. Unknown reached inputs never become neutral counts.
fn source_pocket_selection(
    variant: u32,
    subtype: u32,
    spawn_seed: u32,
    inputs: PocketSubtypeInputs,
) -> Result<Option<(u32, u32)>, String> {
    let mut variant = variant;
    let mut subtype = subtype;
    if variant == 300 {
        let Some(baggy) = inputs.little_baggy else {
            return Ok(None);
        };
        if baggy {
            let Some(deck) = inputs.starter_deck else {
                return Ok(None);
            };
            if !deck {
                variant = 70;
                subtype = 0;
            }
        }
    } else if variant == 70 {
        let Some(deck) = inputs.starter_deck else {
            return Ok(None);
        };
        if deck {
            variant = 300;
            subtype = 0;
        }
    } else {
        return Ok(None);
    }
    if subtype == 0 {
        if spawn_seed == 0 {
            return Err("Pocket Init requires its actual nonzero spawn seed".into());
        }
        let raw = crate::room_spawns::next35(spawn_seed);
        subtype = if variant == 70 {
            //581950 is incoming unsigned seed%9+1 and owns no RNG.
            raw % 9 + 1
        } else {
            let Some(achievements) = inputs.achievements else {
                return Ok(None);
            };
            random_card(raw, &achievements, true, true)?
        };
    }
    Ok(match (variant, subtype) {
        (70, 1..=9) | (300, 1..=40) => Some((variant, subtype)),
        _ => None,
    })
}

/// Direct Pickup.Init4ff1e0 Have operands. Each property is independent:
/// missing actual Player/count ownership is unknown, never a neutral count.
#[derive(Clone, Copy)]
pub(crate) struct RegularSubtypeInputs {
    humbling_bundle: Option<bool>,
    bogo_bombs: Option<bool>,
    // The heart property additionally owns its separate negative operands.
    heart_humbling_bundle: Option<bool>,
}
impl RegularSubtypeInputs {
    const NEUTRAL: Self = Self {
        humbling_bundle: Some(false),
        bogo_bombs: Some(false),
        heart_humbling_bundle: Some(false),
    };

    pub(crate) fn from_state(state: &crate::State) -> Self {
        let items = state
            .player
            .entity_source
            .filter(|source| source.entity_type == 1 && source.variant == 0)
            .and(state.player_items.as_ref());
        let humbling_bundle = items
            .and_then(|items| items.owned_counts.get(203))
            .map(|count| *count > 0);
        //500550..638/50045f..53c/500691..6e1: these actual negative
        //operands skip only their own extra heart draws/replacements.
        //Room8/+1ae0 and Challenge33 pricing retain separate owners.
        let heart_humbling_bundle = items
            .filter(|items| {
                state.weapon_admission.source_player_type == Some(0)
                    && state.challenge_id != 33
                    && state
                        .room
                        .descriptor
                        .as_ref()
                        .is_some_and(|room| room.template.room_type != 8)
                    && [173usize, 238, 239]
                        .into_iter()
                        .all(|id| items.owned_counts.get(id).is_some_and(|count| *count <= 0))
                    && [22, 38, 18, 17].into_iter().all(|id| {
                        crate::item_queries::AppearanceQueries::new(
                            items,
                            state.weapon_cache.primary,
                            state.player.entity_source,
                            None,
                        )
                        .has_trinket(id)
                            == Some(false)
                    })
            })
            .and(humbling_bundle);
        Self {
            humbling_bundle,
            bogo_bombs: items
                .and_then(|items| items.owned_counts.get(250))
                .map(|count| *count > 0),
            heart_humbling_bundle,
        }
    }
}

/// Existing neutral callers retain their original input role. Source-aware
/// factories below supply the actual current Have inputs to the same core.
pub fn regular_subtype(variant: u32, subtype: u32, spawn_seed: u32) -> Result<u32, String> {
    regular_subtype_with_inputs(variant, subtype, spawn_seed, RegularSubtypeInputs::NEUTRAL)?
        .ok_or_else(|| "neutral regular subtype inputs unavailable".into())
}

/// Init owns local RNG35, distinct from the entity's persistent RNG30.
/// None means a reached Have input is unowned; no body/ID/Init is fabricated.
pub(crate) fn regular_subtype_with_inputs(
    variant: u32,
    subtype: u32,
    spawn_seed: u32,
    inputs: RegularSubtypeInputs,
) -> Result<Option<u32>, String> {
    //4FF32B..342 passes Battery90 subtype through without a draw.
    if variant == 90 {
        return Ok(Some(subtype));
    }
    if subtype != 0 {
        if variant == 20 && subtype == 1 {
            //5002fe..33b also transforms an explicitly authored penny.
            return Ok(inputs.humbling_bundle.map(|have| if have { 4 } else { 1 }));
        }
        return Ok(Some(subtype));
    }
    if spawn_seed == 0 {
        return Err("pickup spawn seed must be nonzero".into());
    }
    let mut state = spawn_seed;
    let mut random = |bound| {
        state = crate::room_spawns::next35(state);
        state % bound
    };
    let selected = match variant {
        //500150..1ff: Have203 selects5 before the same local35 rolls.
        //The independent heart input closes only its actual negative
        //173/238/239/trinket/BFC0/non-Room8 footprint above.
        10 => {
            let Some(humbling) = inputs.heart_humbling_bundle else {
                return Ok(None);
            };
            let mut selected = if humbling { 5 } else { 1 };
            if random(2) == 0 {
                selected = 2;
            }
            if random(50) == 0 {
                selected = 4;
            }
            if random(10) == 0 {
                selected = if random(20) == 0 { 6 } else { 3 };
            }
            selected
        }
        20 => {
            let mut selected = if random(20) == 0 { 2 } else { 1 };
            if random(100) == 0 {
                selected = 3;
            }
            if selected == 1 {
                let Some(have) = inputs.humbling_bundle else {
                    return Ok(None);
                };
                if have {
                    selected = 4;
                }
            }
            selected
        }
        30 => {
            let Some(have) = inputs.humbling_bundle else {
                return Ok(None);
            };
            let mut selected = if have { 3 } else { 1 };
            if random(50) == 0 {
                selected = 2;
            }
            selected
        }
        40 => {
            let Some(humbling) = inputs.humbling_bundle else {
                return Ok(None);
            };
            //500409..435 reaches Have250 only after Have203 returned false.
            let double = if humbling {
                true
            } else {
                let Some(have) = inputs.bogo_bombs else {
                    return Ok(None);
                };
                have
            };
            let mut selected = if double { 2 } else { 1 };
            if random(7) == 0 {
                selected = 2;
            }
            if random(50) == 0 {
                selected = 5;
            }
            if random(10) == 0 {
                selected = 3;
            }
            selected
        }
        _ => {
            return Err(format!(
                "pickup selector {variant}/0 requires its owning pool or subtype resolver"
            ))
        }
    };
    Ok(Some(selected))
}

/// Init can select transient Troll/Megatroll PU subtypes (40/3, 40/5)
/// without an authored pickup actor. Until their first full Update converts
/// them, they use the bomb pickup's temporary visual definition, so every
/// definition lookup for a live pickup resolves its subtype here.
pub(crate) fn definition_subtype(variant: u32, subtype: u32) -> u32 {
    match (variant, subtype) {
        (40, 3 | 5) | (300, 1..=22) => 1,
        (300, 23..=27 | 36..=40) => 2,
        (300, 28..=31) => 3,
        (300, 32..=35) => 4,
        _ => subtype,
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
    ///Opaque native ABD byte copied by Room.Save; legacy absence is unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub save_byte_abd: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shop_price: Option<crate::shop::SavedPrice>,
}

/// Actual weak words copied by Pickup::Morph500A8F..AAE/500BB5..BD4.
/// Fresh supported Init owns NULL; an old restore has no invented owner.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MorphLinks {
    pub parent498: Option<crate::actors::ActorRef>,
    pub link4a0: Option<crate::actors::ActorRef>,
    pub link4a8: Option<crate::actors::ActorRef>,
}
impl MorphLinks {
    pub(crate) fn known_empty(&self) -> bool {
        self.parent498.is_none() && self.link4a0.is_none() && self.link4a8.is_none()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Pickup {
    ///Actual Entity1C/20, independent of weak parent498.
    ///An unowned/legacy caller never acquires an invented null spawner.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_spawner: Option<SourceSpawner>,
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
    ///Distinct signed AD0 pickup-contact cooldown, not the ACC delay above.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pocket_contact_delay: Option<i32>,
    pub age: u32,
    pub birth_game_update: u32,
    pub charge: i32,
    pub touched: bool,
    ///Opaque native ABD byte copied by Room.Save; legacy absence is unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub save_byte_abd: Option<u8>,
    #[serde(default)]
    pub regular_collision: Option<RegularCollision>,
    #[serde(default)]
    pub regular_rng: Option<crate::items::Rng>,
    #[serde(default)]
    pub chest: Option<crate::chests::Chest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shop_price: Option<crate::shop::Price>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub morph_links: Option<MorphLinks>,
    /// Pickup+AD4 door kick: the free active swap (50631A) sets 1 when a
    /// room door is within 70 of the Player; the next full update (5071A9)
    /// adds the kick and sets 2; the one after (5075E4) re-anchors and clears.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub door_kick_ad4: u8,
}

fn is_zero_u8(value: &u8) -> bool {
    *value == 0
}

///Game550ACD..ADA copies these words only for a nonnull spawner.
///Base441EE8/EF supplies zeros for an actual null-spawner birth.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceSpawner {
    pub entity_type: u32,
    pub variant: u32,
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
        matches!(self.variant, 10 | 20 | 30 | 40 | 70 | 90 | 300)
    }

    ///Free natural Card/Pill Init uses the canonical preconversion and pool
    ///resolver. A card keeps its actual ID after Base selects its art family.
    pub(crate) fn from_pocket_selector(
        id: u32,
        selector: &crate::room_spawns::SpawnSelector,
        appear: bool,
        state: &crate::State,
    ) -> Result<Option<Self>, String> {
        if selector.entity_type != 5
            || !matches!(selector.variant, 70 | 300)
            || state.challenge_id == 33
        {
            return Ok(None);
        }
        Self::from_pocket_parts(
            id,
            u32::from(selector.variant),
            u32::from(selector.subtype),
            selector.spawn_seed,
            selector.position,
            appear,
            PocketSubtypeInputs::from_state(state),
        )
    }

    ///Actual Game.Spawn inputs reach this same resolver without manufacturing
    ///an authored Room selector or consuming any extra shared seed/draw.
    pub(crate) fn from_pocket_parts(
        id: u32,
        variant: u32,
        subtype: u32,
        spawn_seed: u32,
        position: WorldVector,
        appear: bool,
        inputs: PocketSubtypeInputs,
    ) -> Result<Option<Self>, String> {
        let Some((variant, subtype)) =
            source_pocket_selection(variant, subtype, spawn_seed, inputs)?
        else {
            return Ok(None);
        };
        let mut pickup = Self::spawn(id, variant, subtype, position, appear)?;
        pickup.initialize_spawn_seed(spawn_seed)?;
        Ok(Some(pickup))
    }

    /// Concrete room admission for the supported fresh normal Isaac families.
    /// Unsupported runtime selectors remain in RoomSpawns for their owner.
    pub fn from_selector(
        id: u32,
        selector: &crate::room_spawns::SpawnSelector,
        appear: bool,
    ) -> Result<Option<Self>, String> {
        if selector.entity_type == 5 && matches!(selector.variant, 50 | 51) {
            return crate::chests::from_selector(id, selector, appear);
        }
        if selector.entity_type != 5 || !matches!(selector.variant, 10 | 20 | 30 | 40) {
            return Ok(None);
        }
        Self::from_regular_selector(id, selector, appear, RegularSubtypeInputs::NEUTRAL)
    }

    /// Current authored10/20/30/40 Init uses actual FIRSTPlayer operands.
    /// Other families retain their own existing factory/context contracts.
    pub(crate) fn from_source_selector(
        id: u32,
        selector: &crate::room_spawns::SpawnSelector,
        appear: bool,
        state: &crate::State,
    ) -> Result<Option<Self>, String> {
        if selector.entity_type == 5 && selector.variant == 90 {
            //Challenge33 has its separate pricing owner, as for other families.
            if state.challenge_id == 33 {
                return Ok(None);
            }
            let mut pickup = Self::spawn(
                id,
                90,
                u32::from(selector.subtype),
                selector.position,
                appear,
            )?;
            pickup.initialize_spawn_seed(selector.spawn_seed)?;
            return Ok(Some(pickup));
        }
        if selector.entity_type != 5 || !matches!(selector.variant, 10 | 20 | 30 | 40) {
            return Self::from_selector(id, selector, appear);
        }
        //4ff740 Challenge33 pricing precedes the free pickup postlude.
        //Unavailable pricing stays local to this actual selected factory.
        if state.challenge_id == 33 {
            return Ok(None);
        }
        //4ff560/500240 applies Room8 overrides even to explicit hearts.
        //Other explicit heart subtypes do not read the random-heart inputs.
        if selector.variant == 10
            && state
                .room
                .descriptor
                .as_ref()
                .is_none_or(|room| room.template.room_type == 8)
        {
            return Ok(None);
        }
        Self::from_regular_selector(
            id,
            selector,
            appear,
            RegularSubtypeInputs::from_state(state),
        )
    }

    fn from_regular_selector(
        id: u32,
        selector: &crate::room_spawns::SpawnSelector,
        appear: bool,
        inputs: RegularSubtypeInputs,
    ) -> Result<Option<Self>, String> {
        let variant = u32::from(selector.variant);
        let Some(subtype) = regular_subtype_with_inputs(
            variant,
            u32::from(selector.subtype),
            selector.spawn_seed,
            inputs,
        )?
        else {
            return Ok(None);
        };
        if !matches!(
            (variant, subtype),
            (10, 1..=6) | (20, 1..=4) | (30, 1..=3) | (40, 1 | 2 | 3 | 5)
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
    /// Transient subtypes (Troll and Megatroll Bombs) take their temporary
    /// visual definition through `definition_subtype`, so no birth site can
    /// spawn one without it.
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
        let visual = definition_subtype(variant, subtype);
        let definition = definition(variant, visual)
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
            source_spawner: None,
            id,
            spawn_seed: 0,
            variant,
            subtype,
            spritesheet_item_id: visual,
            position,
            presented_position: position,
            anchor_position: if matches!(variant, 10 | 20 | 30 | 40 | 70 | 90 | 300) {
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
            pocket_contact_delay: Some(0),
            age: 0,
            birth_game_update: 0,
            charge: 0,
            touched: false,
            //Pickup.Init4FF4E1 owns this exact zero, independently of type/SKU.
            save_byte_abd: Some(0),
            regular_collision: matches!(variant, 10 | 20 | 30 | 40 | 70 | 90 | 300).then_some(
                RegularCollision {
                    grid_collision_class: 5,
                    entity_collision_class: if animation == "Idle" { 4 } else { 0 },
                },
            ),
            regular_rng: None,
            chest: None,
            shop_price: None,
            morph_links: None,
            door_kick_ad4: 0,
        })
    }

    /// Selected free in-place Morph500A10 to an already chosen collectible.
    /// It is not Game.Spawn: no new handle, manager slot or Room sequence.
    pub(crate) fn morph_to_collectible(
        &mut self,
        subtype: u32,
        game_update: u32,
        scheduler: &mut crate::actors::Scheduler,
    ) -> Result<(), String> {
        let links = self
            .morph_links
            .clone()
            .ok_or("Morph weak-word owner missing")?;
        if !links.known_empty() {
            return Err("Morph positive weak-link lifetime owner unadmitted".into());
        }
        let config = crate::items::definition(subtype).ok_or("Morph collectible config absent")?;
        if self.spawn_seed == 0 || !self.position.is_finite() || !self.velocity.is_finite() {
            return Err("Morph requires actual seed and finite saved vectors".into());
        }
        //500A2A..56 local RNG7 advances the OLD spawnSeed exactly once;
        //the subsequent actual Init resets persistent RNG30 to this new seed.
        let seed = crate::items::Rng::new(self.spawn_seed, [1, 21, 20]).raw();
        let mut next = Self::spawn(self.id, 100, subtype, self.position, false)?;
        next.spawn_seed = seed;
        next.regular_rng = Some(crate::items::Rng::new(seed, [4, 3, 17]));
        next.anchor_position = self.anchor_position;
        next.presented_position = self.presented_position;
        next.velocity = self.velocity;
        next.touched = self.touched;
        //Morph500AC8/ACF saves ABD across virtual Init,500BE6/BEB restores it.
        next.save_byte_abd = self.save_byte_abd;
        next.charge = config.max_charges;
        next.birth_game_update = game_update;
        next.morph_links = Some(links);
        //TryOpen501178..1B0 selects actual locked-Chest Alternates frame4.
        next.overlay_frame = 4.0;
        next.project();
        scheduler.reinitialize_in_place(crate::actors::ActorRef {
            kind: crate::actors::ActorKind::Pickup,
            id: self.id,
        })?;
        *self = next;
        Ok(())
    }

    /// The D6 reroll Morph500A10(5,100,0,keepPrice1) of a free pedestal:
    /// the caller derives `seed` (500A2A..56 advances the old seed once) and
    /// selects `subtype` with Init's 4FF6B8 local draw. Morph restores the
    /// 498 links, ABC/ABD/AC8 and replays the Alternates overlay at its
    /// saved frame (500ADD..B35, 500C4B); AB8 is the config charge for an
    /// active (500D50).
    pub(crate) fn morph_rerolled(
        &mut self,
        seed: u32,
        subtype: u32,
        game_update: u32,
        scheduler: &mut crate::actors::Scheduler,
    ) -> Result<(), String> {
        if !self.position.is_finite() || !self.velocity.is_finite() {
            return Err("reroll Morph requires finite saved vectors".into());
        }
        let config = crate::items::definition(subtype).ok_or("Morph collectible config absent")?;
        let mut next = Self::spawn(self.id, 100, subtype, self.position, false)?;
        next.source_spawner = self.source_spawner;
        next.spawn_seed = seed;
        next.regular_rng = Some(crate::items::Rng::new(seed, [4, 3, 17]));
        next.anchor_position = self.anchor_position;
        next.presented_position = self.presented_position;
        next.velocity = self.velocity;
        next.touched = self.touched;
        next.save_byte_abd = self.save_byte_abd;
        next.morph_links = self.morph_links.clone();
        if self.overlay.as_deref() == Some("Alternates") {
            next.overlay_frame = self.overlay_frame;
        }
        next.birth_game_update = game_update;
        next.charge = if config.kind == 3 {
            config.max_charges
        } else {
            0
        };
        next.project();
        scheduler.reinitialize_in_place(crate::actors::ActorRef {
            kind: crate::actors::ActorKind::Pickup,
            id: self.id,
        })?;
        *self = next;
        Ok(())
    }

    /// Free active swap 50621D: Morph500A10(5,100,old,keepPrice0) re-Inits
    /// the touched pedestal in place as the dropped active. Morph clears AB8;
    /// the caller then stores the dropped active's charge. Position, velocity,
    /// ABC/ABD and the Alternates overlay frame survive; age restarts at Init.
    pub(crate) fn morph_swapped_active(
        &mut self,
        subtype: u32,
        charge: i32,
        game_update: u32,
        scheduler: &mut crate::actors::Scheduler,
    ) -> Result<(), String> {
        if self.spawn_seed == 0 || !self.position.is_finite() || !self.velocity.is_finite() {
            return Err("active swap Morph requires actual seed and finite saved vectors".into());
        }
        //500A2A..56: the local RNG7 advances the old seed once (no shared MT).
        let seed = crate::items::Rng::new(self.spawn_seed, [1, 21, 20]).raw();
        let mut next = Self::spawn(self.id, 100, subtype, self.position, false)?;
        next.source_spawner = self.source_spawner;
        next.spawn_seed = seed;
        next.regular_rng = Some(crate::items::Rng::new(seed, [4, 3, 17]));
        next.anchor_position = self.anchor_position;
        next.presented_position = self.presented_position;
        next.velocity = self.velocity;
        next.save_byte_abd = self.save_byte_abd;
        next.morph_links = self.morph_links.clone();
        next.overlay_frame = self.overlay_frame;
        next.birth_game_update = game_update;
        next.charge = charge;
        //5060F6 marks the dropped active touched before replaying Idle.
        next.touched = true;
        next.project();
        scheduler.reinitialize_in_place(crate::actors::ActorRef {
            kind: crate::actors::ActorKind::Pickup,
            id: self.id,
        })?;
        *self = next;
        Ok(())
    }

    /// A free Morph500A10(5, variant, 0, keepPrice1) into a pickup the caller
    /// built by the new variant's Init from the advanced seed (500A2A..56).
    /// Morph keeps the handle, position, velocity, ABC/ABD, the spawner and
    /// the weak links; age restarts at the new Init.
    pub(crate) fn morph_into(
        &mut self,
        mut next: Pickup,
        game_update: u32,
        scheduler: &mut crate::actors::Scheduler,
    ) -> Result<(), String> {
        if !self.position.is_finite() || !self.velocity.is_finite() {
            return Err("Morph requires finite saved vectors".into());
        }
        next.id = self.id;
        next.source_spawner = self.source_spawner;
        next.position = self.position;
        next.anchor_position = self.anchor_position;
        next.presented_position = self.presented_position;
        next.velocity = self.velocity;
        next.touched = self.touched;
        next.save_byte_abd = self.save_byte_abd;
        next.morph_links = self.morph_links.clone();
        next.birth_game_update = game_update;
        next.project();
        scheduler.reinitialize_in_place(crate::actors::ActorRef {
            kind: crate::actors::ActorKind::Pickup,
            id: self.id,
        })?;
        *self = next;
        Ok(())
    }

    /// Base ANM2 playback is admitted only on full nonplayer update phases.
    pub fn advance(&mut self, phase: Phase, game_update: u32, door_kick: WorldVector) {
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
        if self.has_regular_physics()
            && self.anchor_position.x == 0.0
            && self.anchor_position.y == 0.0
        {
            self.anchor_position = self.position;
        }
        //506FAA/5075E4: the update after a door kick re-anchors in place.
        if self.door_kick_ad4 == 2 {
            self.anchor_position = self.position;
            self.door_kick_ad4 = 0;
        }
        if self.variant == 100
            || self
                .shop_price
                .as_ref()
                .is_some_and(|price| price.value != 0)
        {
            // Pickup506FC3 routes every nonzero AC0 here, not only100.
            // Pickup507170 assigns anchor-position before Base447190 applies
            // current friction to velocity and moves. Base restores configured
            // friction, then507029 multiplies it by the actual source .965.
            let mut pull = WorldVector {
                x: self.anchor_position.x - self.position.x,
                y: self.anchor_position.y - self.position.y,
            };
            //5071A9..507316 adds the caller's door kick once, then AD4=2.
            if self.door_kick_ad4 == 1 {
                pull = pull + door_kick;
                self.door_kick_ad4 = 2;
            }
            self.velocity = pull * self.current_friction;
            self.position = self.position + self.velocity;
            self.presented_position = self.position;
            self.current_friction = self.configured_friction * 0.965;
            self.project();
        } else if self.has_regular_physics() {
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
        //Entity+344 class0 (How to Jump in the air) has no pair collision.
        if player.entity_collision_off {
            return false;
        }
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
    pub fn try_collect(&mut self, inventory: &mut Inventory, player: &mut Player) -> bool {
        if !self.try_grant(inventory, player) {
            return false;
        }
        self.finish_collect();
        true
    }

    pub(crate) fn try_grant(&mut self, inventory: &mut Inventory, player: &mut Player) -> bool {
        if !self.can_collect || self.collected || self.removed || self.collection_delay != 0 {
            return false;
        }
        match (self.variant, self.subtype) {
            (20, 1..=4) => {
                let amount = [1, 5, 10, 2][self.subtype as usize - 1];
                inventory.coins = inventory.coins.saturating_add(amount).min(99);
            }
            (30, 2) => {
                // Collect504EEB calls508730: BD0 only, without AddKeys.
                player.golden_key = Some(true);
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
        if !self.try_grant_heart(player, red_multiplier, level_progress) {
            return false;
        }
        self.finish_collect();
        true
    }

    pub(crate) fn try_grant_heart(
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
        true
    }

    pub(crate) fn finish_collect(&mut self) {
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
    state.room.saved_regular_pickups = capture_regular_records(state)?;
    Ok(())
}

/// Capture once at Room.Save, before later PlayerManager.Save callbacks.
/// The caller owns where the immutable record list is retained.
pub(crate) fn capture_regular_records(
    state: &crate::State,
) -> Result<Vec<SavedRegularPickup>, String> {
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
        if (!pickup.has_regular_physics() && pickup.shop_price.is_none())
            || pickup.collected
            || pickup.removed
        {
            continue;
        }
        //Native Init does not initialize opaqueAB8 for350. Live contact
        //passes literal queuecharge0, but Save copiesAB8. Until its pooled
        //allocation history is owned, refuse rather than save spawn's0.
        if pickup.variant == 350 {
            return Err("saving a loose350 requires its raw pooled AB8 charge owner".into());
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
            save_byte_abd: pickup.save_byte_abd,
            shop_price: pickup
                .shop_price
                .as_ref()
                .map(|price| crate::shop::SavedPrice {
                    value: price.value,
                    slot: price.slot,
                }),
        });
    }
    Ok(records)
}

/// RoomLoad5cf585 calls5c5860 for Descriptor+70..78 saved records. Each
/// GameSpawn receives stored position+20/seed+2c and explicit zero velocity.
/// The distinct Descriptor+b8 additional-spawn vector is not this owner.
pub(crate) fn readmit_regular_records(state: &mut crate::State) -> Result<Vec<u32>, String> {
    let records = std::mem::take(&mut state.room.saved_regular_pickups);
    let mut admitted = Vec::with_capacity(records.len());
    let mut handles = BTreeSet::new();
    for record in records {
        if !(matches!(record.variant, 10 | 20 | 30 | 40 | 70 | 90 | 300)
            || (record.variant == 100 && record.shop_price.is_some()))
            || !handles.insert(record.handle)
        {
            return Err("invalid saved regular pickup record".into());
        }
        // Saved subtype is concrete. Fresh normal Isaac has no modifier that
        // rerolls it; initializing temporary RNG35 consumes no output here.
        let (variant, subtype) = if matches!(record.variant, 70 | 300) {
            let Some(selected) = source_pocket_selection(
                record.variant,
                record.subtype,
                record.spawn_seed,
                PocketSubtypeInputs::from_state(state),
            )?
            else {
                return Err("saved Pocket Init reached an unowned replacement input".into());
            };
            selected
        } else {
            (
                record.variant,
                regular_subtype(record.variant, record.subtype, record.spawn_seed)?,
            )
        };
        let mut pickup = Pickup::spawn(record.handle, variant, subtype, record.position, false)?;
        pickup.subtype = subtype;
        pickup.initialize_spawn_seed(record.spawn_seed)?;
        pickup.regular_rng = Some(crate::items::Rng::new(record.rng_seed, [4, 3, 17]));
        pickup.birth_game_update = state.game_update_count;
        pickup.charge = record.charge;
        pickup.touched = record.touched;
        if let Some(price) = record.shop_price {
            crate::shop::readmit_price(state, &mut pickup, price)?;
        }
        //5C5956 restores the saved byte after fresh Init. A legacy unknown
        //record must not inherit Init's known0 as if that byte were saved.
        pickup.save_byte_abd = record.save_byte_abd;
        admitted.push(pickup);
    }
    let ids = admitted.iter().map(|pickup| pickup.id).collect();
    let mut replacements: BTreeMap<_, _> = admitted
        .into_iter()
        .map(|pickup| (pickup.id, pickup))
        .collect();
    let old_pickups = std::mem::take(&mut state.room.pickups);
    for pickup in old_pickups {
        if !pickup.has_regular_physics() && pickup.shop_price.is_none() {
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

/// Pickup.Update 507940: once the sprite step lands on the clip's DropSound
/// trigger (IsEventTriggered 410790), 5042a0 plays the variant's drop sound
/// (delay 2), drawing its shared sample. Chests play theirs in chests.rs.
fn drop_sound(state: &mut crate::State, index: usize) {
    let pickup = &state.room.pickups[index];
    let Some(definition) = definition(
        pickup.variant,
        definition_subtype(pickup.variant, pickup.subtype),
    ) else {
        return;
    };
    let landed = definition
        .clips
        .iter()
        .find(|clip| clip.name == pickup.animation)
        .and_then(|clip| clip.drop_sound)
        .is_some_and(|frame| pickup.animation_frame == f64::from(frame));
    let id = match (pickup.variant, pickup.subtype) {
        (10, _) => 68,
        (20, 1) => 233,
        (20, 2) => 231,
        (20, 3) => 198,
        (30, _) => 59,
        (40, _) => 40,
        (70, _) => 38,
        (300, _) => 249,
        _ => return,
    };
    if landed {
        state.audio.admit(
            &mut state.boot_lifecycle.shared,
            state.system_clock.system_counter.wrapping_sub(1),
            crate::audio::SoundRequest {
                id,
                volume: 1.0,
                delay: 2,
                looping: false,
                pitch: 1.0,
            },
        );
    }
}

/// The 6128e0 resize: `v*(n/|v|)`, a zero vector unchanged.
fn resized(vector: WorldVector, length: f32) -> WorldVector {
    let current = (vector.x * vector.x + vector.y * vector.y).sqrt();
    if current > 0.0 {
        vector * (length / current)
    } else {
        vector
    }
}

/// The first room door slot (Room+E30, 561070 grid position) whose squared
/// distance to the Player (+448) is under 4900 (`66C1E4`).
pub(crate) fn door_near_player(state: &crate::State) -> Option<WorldVector> {
    let player = crate::bombs::player_world(state);
    let mut doors: Vec<_> = state.room.appearance.doors.iter().collect();
    doors.sort_by_key(|door| door.slot);
    doors
        .into_iter()
        .map(|door| door.world_position)
        .find(|door| {
            let (dx, dy) = (door.x - player.x, door.y - player.y);
            dx * dx + dy * dy < 4900.0
        })
}

/// 5071A9..50730A: for the first door near the Player, the pedestal is
/// pushed 5 away from the door plus 3 away from the Player; no door, no kick.
fn door_kick(state: &crate::State, position: WorldVector) -> WorldVector {
    let Some(door) = door_near_player(state) else {
        return WorldVector { x: 0.0, y: 0.0 };
    };
    let player = crate::bombs::player_world(state);
    let away_door = resized(
        WorldVector {
            x: position.x - door.x,
            y: position.y - door.y,
        },
        5.0,
    );
    let away_player = resized(
        WorldVector {
            x: position.x - player.x,
            y: position.y - player.y,
        },
        3.0,
    );
    away_player + away_door
}

/// Full Pickup Update owns motion/friction; the manager owns its later grid
/// pass. Fast4446a0 predicts and resolves its own ground preview directly.
pub(crate) fn advance_actor(state: &mut crate::State, phase: Phase, id: u32) -> Result<(), String> {
    let Some(index) = state.room.pickups.iter().position(|pickup| pickup.id == id) else {
        return Ok(());
    };
    //Manager454E8F skips actual inactive actors before their callbacks.
    //Contact-time Remove can leave the body listed until the next Full sweep.
    if state.room.pickups[index].removed {
        return Ok(());
    }
    if state.room.pickups[index].variant == 350 {
        return Err("loose350 callbacks are outside the admitted grant-only slice".into());
    }
    let door_kick = if state.room.pickups[index].door_kick_ad4 == 1 {
        door_kick(state, state.room.pickups[index].position)
    } else {
        WorldVector { x: 0.0, y: 0.0 }
    };
    state.room.pickups[index].advance(phase, state.game_update_count, door_kick);
    if phase == Phase::Update {
        drop_sound(state, index);
    }
    if phase == Phase::Update && matches!(state.room.pickups[index].variant, 70 | 300) {
        //5076DB is skipped by actual Room16 and Challenge5/6/11. The
        //ordinary branch decrements positiveAD0 only outside Appear.
        let ordinary = !matches!(state.challenge_id, 5 | 6 | 11)
            && state
                .room
                .descriptor
                .as_ref()
                .is_some_and(|room| room.template.room_type != 16);
        let pickup = &mut state.room.pickups[index];
        if ordinary && pickup.animation != "Appear" {
            if let Some(delay) = &mut pickup.pocket_contact_delay {
                if *delay > 0 {
                    *delay -= 1;
                }
            }
        }
    }
    let pickup = &mut state.room.pickups[index];
    if phase != Phase::Interpolate
        || !(pickup.has_regular_physics() || pickup.shop_price.is_some())
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

/// Full manager4551E0/4551FD stably compacts inactive actors after their
/// admission-fenced callbacks and before455292 entity contacts. A Remove
/// during contacts remains listed until the next Full sweep; Non-Full does not
/// erase it. Keep scheduler membership and actual Room bodies in one fence.
pub(crate) fn retire_full(state: &mut crate::State) {
    state.room.pickups.retain(|pickup| !pickup.removed);
    let pickups = &state.room.pickups;
    state.actor_scheduler.retain(|actor| {
        actor.kind != crate::actors::ActorKind::Pickup
            || pickups.iter().any(|pickup| pickup.id == actor.id)
    });
}

/// 4545d0 finishes all entity contacts, then454b10 invokes442b90 for every
/// listed nonplayer, without testing31E/31F. New and contact-removed bodies
/// get this grid pass without another full actor update.
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
    pickup
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
/// HP0 rejects ordinary Pickup damage. Closed51 alone owns the actual positive
/// flags4 Hurt callback after the velocity effect.
pub(crate) fn bomb_hit(
    state: &mut crate::State,
    center: WorldVector,
    blast_radius: f32,
    damage: f32,
    damage_flags: u32,
    line_clear: &dyn Fn(WorldVector) -> bool,
) -> Result<(), String> {
    if !center.is_finite() {
        return Err("pickup blast center must be finite".into());
    }
    //4536F0 freezes its receiver list before callbacks can append loot. This
    //retains the existing pickup order; global cross-family query order is a
    //separate frontier, not an invented second all-actor query.
    let candidate_count = state.room.pickups.len();
    let current_actors = state.actor_scheduler.full_phase();
    for index in 0..candidate_count {
        let id = state.room.pickups[index].id;
        let actor = crate::actors::ActorRef {
            kind: crate::actors::ActorKind::Pickup,
            id,
        };
        let current_owner =
            current_actors.contains(&actor) && state.actor_scheduler.sequence_id(actor).is_some();
        let pickup = &mut state.room.pickups[index];
        let bomb_chest = pickup.variant == 51 && pickup.chest.is_some();
        //Appear has source entity class0; Collect has dead31f. Admitted
        //ordinary families have ground class5 and positive authored mass≤20.
        if (!pickup.has_regular_physics() && !bomb_chest)
            || (!bomb_chest && !pickup.can_collect)
            || pickup.collected
            || pickup.removed
            || pickup.collision_mass == 0.0
        {
            continue;
        }
        if bomb_chest {
            let chest = pickup.chest.as_mut().expect("Bomb Chest");
            //554535/54A skip class0 and gridclass4 before either effect.
            if chest.entity_collision_class == 0 || chest.grid_collision_class == 4 {
                continue;
            }
            // Fresh51's current geometry is authored12/mass3/class5. Missing
            // current manager or other legacy geometry remains local unknown.
            if !current_owner
                || !pickup.position.is_finite()
                || !pickup.velocity.is_finite()
                || pickup.collision_radius != 12.0
                || pickup.collision_mass != 3.0
                || chest.grid_collision_class != 5
            {
                chest
                    .open_frontier
                    .get_or_insert(crate::chests::OpenFrontier::ExplosionGeometry);
                continue;
            }
        }
        let delta = WorldVector {
            x: pickup.position.x - center.x,
            y: pickup.position.y - center.y,
        };
        let squared = delta.x * delta.x + delta.y * delta.y;
        let radius = blast_radius + pickup.collision_radius;
        if !squared.is_finite() || !radius.is_finite() {
            if bomb_chest {
                pickup
                    .chest
                    .as_mut()
                    .expect("Bomb Chest")
                    .open_frontier
                    .get_or_insert(crate::chests::OpenFrontier::ExplosionGeometry);
                continue;
            }
            return Err("pickup blast geometry overflowed".into());
        }
        if squared >= radius * radius || !line_clear(pickup.position) {
            continue;
        }
        //554625 ->5547AA skips only impulse at exact zero distance; the
        //positive damage virtual still runs and can actually open51.
        if squared != 0.0 {
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
                if bomb_chest {
                    pickup
                        .chest
                        .as_mut()
                        .expect("Bomb Chest")
                        .open_frontier
                        .get_or_insert(crate::chests::OpenFrontier::ExplosionGeometry);
                    continue;
                }
                return Err("pickup blast velocity overflowed".into());
            }
        }
        if bomb_chest {
            let _taken = crate::chests::hurt_bomb_chest(state, index, damage, damage_flags)?;
        }
    }
    Ok(())
}

/// Game::Spawn(5, 100, position, zero velocity, no spawner, subtype, seed)
/// from a Room award: the caller already selected the concrete collectible.
pub(crate) fn admit_collectible(
    state: &mut crate::State,
    subtype: u32,
    position: WorldVector,
    spawn_seed: u32,
) -> Result<(), String> {
    let config =
        crate::items::definition(subtype).ok_or("awarded collectible subtype has no ItemConfig")?;
    let id = state.next_pickup;
    let mut pickup = Pickup::spawn(id, 100, subtype, position, true)?;
    pickup.spawn_seed = spawn_seed;
    pickup.charge = config.max_charges;
    pickup.birth_game_update = state.game_update_count;
    state.next_pickup = id.checked_add(1).ok_or("pickup identity exhausted")?;
    state.room.pickups.push(pickup);
    let actor = crate::actors::ActorRef {
        kind: crate::actors::ActorKind::Pickup,
        id,
    };
    state.room.authored_actor_order.push(actor);
    state.actor_scheduler.admit(actor)
}

/// Collect504480 family audio after either successful grant lifecycle.
pub(crate) fn collect_sound(state: &mut crate::State, index: usize) {
    // Collect504480 routes key ->504720 and bomb ->5046d0
    // after inventory mutation, before the System increment.
    let pickup = &state.room.pickups[index];
    let sound = match pickup.variant {
        10 => Some((
            match pickup.subtype {
                3 => 54,
                4 => 266,
                _ => 185,
            },
            1.0,
        )),
        //Collect504510: only successful free Coin20/1..4 reaches here.
        //The actual subtype owns the configured sound, not its amount.
        20 => match pickup.subtype {
            1 | 4 => Some((234, 1.0)),
            2 => Some((232, 1.0)),
            3 => Some((199, 1.0)),
            _ => None,
        },
        30 if pickup.subtype == 2 => Some((204, 1.0)),
        30 => Some((58, 0.85)),
        40 => Some((201, 1.0)),
        70 => Some((252, 1.0)),
        300 => Some((8, 1.0)),
        100 if state
            .room
            .descriptor
            .as_ref()
            .is_some_and(|room| room.template.room_type == 2) =>
        {
            Some((130, 0.5))
        }
        _ => None,
    };
    if let Some((id, volume)) = sound {
        state.audio.admit(
            &mut state.boot_lifecycle.shared,
            state.system_clock.system_counter.wrapping_sub(1),
            crate::audio::SoundRequest {
                id,
                volume,
                delay: 2,
                looping: false,
                pitch: 1.0,
            },
        );
    }
}

///5053BE..5402,50587A..58AE select509E10's absolute requested charge.
///Unknown canonical operands remain explicit before any paid grant/debit.
pub(crate) fn battery_target(state: &crate::State) -> Result<Option<i32>, &'static str> {
    use crate::active_charges::{owns, Word};
    let owner = state.player_items.as_ref().ok_or("batteryPlayerOwner")?;
    if !owns(owner, Word::ActiveId) {
        return Err("batteryActiveId");
    }
    if owner.active_id == 0 {
        return Ok(None);
    }
    let config = crate::items::definition(owner.active_id).ok_or("batteryActiveConfig")?;
    if !owns(owner, Word::Charge) {
        return Err("batteryCharge");
    }
    if config.max_charges > owner.active_charge {
        return Ok(Some(config.max_charges));
    }
    let have = crate::item_queries::AppearanceQueries::new(
        owner,
        state.weapon_cache.primary,
        state.player.entity_source,
        None,
    )
    .have(63)
    .ok_or("batteryHave63")?;
    if !have {
        return Ok(None);
    }
    if !owns(owner, Word::Reserve) {
        return Err("batteryReserve");
    }
    Ok((config.max_charges > owner.reserve_charge).then_some(config.max_charges.wrapping_mul(2)))
}

///Free Battery contact keeps the existing free Collect lifecycle. The paid
///owner calls the same target/setter before copy+debit+immediate retirement.
pub(crate) fn collect_battery(state: &mut crate::State, index: usize) -> Result<(), String> {
    //4453A0 dead Player veto precedes the Pickup callback and AD0.
    if state.player.dead {
        return Ok(());
    }
    let player_position = crate::bombs::player_world(state);
    let pickup_position = state.room.pickups[index].position;
    let pickup_radius = state.room.pickups[index].collision_radius;
    if collect_battery_callback(state, index)? != PlayerContactDecision::AllowPhysicalResponse {
        return Ok(());
    }
    player_physical_response(
        state,
        index,
        player_position,
        pickup_position,
        pickup_radius,
    )
}

fn collect_battery_callback(
    state: &mut crate::State,
    index: usize,
) -> Result<PlayerContactDecision, String> {
    let pickup = &state.room.pickups[index];
    if pickup.variant != 90
        || pickup.removed
        || pickup.collected
        || pickup
            .regular_collision
            .as_ref()
            .is_none_or(|collision| collision.entity_collision_class != 4)
        || !pickup.touches(&state.player)
    {
        return Ok(PlayerContactDecision::NotReached);
    }
    let delay = pickup
        .pocket_contact_delay
        .ok_or("Battery AD0 history unavailable")?;
    if delay > 0 {
        state.room.pickups[index].pocket_contact_delay = Some(10);
        return Ok(PlayerContactDecision::VetoPhysicalResponse);
    }
    if state
        .player
        .entity_source
        .is_none_or(|source| source.entity_type != 1 || source.variant != 0)
    {
        return Ok(PlayerContactDecision::Unowned);
    }
    let pickup = &state.room.pickups[index];
    if pickup.collection_delay > 0 || !player_can_collect(state) {
        return Ok(PlayerContactDecision::AllowPhysicalResponse);
    }
    if !pickup.can_collect {
        return Ok(PlayerContactDecision::Unowned);
    }
    let Some(target) = battery_target(state).map_err(str::to_owned)? else {
        return Ok(PlayerContactDecision::AllowPhysicalResponse);
    };
    crate::active_items::set_charge(state, target)?;
    //Collect504480 has no90 sound; charge setter owns its actual request.
    state.room.pickups[index].finish_collect();
    Ok(PlayerContactDecision::VetoPhysicalResponse)
}

///445710 uses pre-callback overlap and post-callback masses/velocities only
///after AL0. Shared unchanged Pocket/Battery response; paid callbacks veto it.
///No new solver, clamp, collision sample or physics constant is introduced.
pub(crate) fn player_physical_response(
    state: &mut crate::State,
    index: usize,
    player_position: WorldVector,
    pickup_position: WorldVector,
    pickup_radius: f32,
) -> Result<(), String> {
    //A missing legacy grid/flying/class owner cannot become ground geometry.
    if state.player.flying != Some(false) || state.player.grid_collision_class != Some(5) {
        return Ok(());
    }
    let Some(frame) = state.room.grid.clone() else {
        return Ok(());
    };
    let grid = crate::grid::Grid::new(frame)?;
    static PLAYER_COLLIDER: std::sync::OnceLock<crate::grid::Collider> = std::sync::OnceLock::new();
    let collider = PLAYER_COLLIDER.get_or_init(|| {
        crate::grid::Collider::new(crate::grid::CollisionGeometry {
            radius: crate::content::PLAYER_COLLISION_RADIUS_WORLD as f32,
            scale: crate::motion::WorldVector { x: 1.0, y: 1.0 },
            points: 40,
        })
        .expect("authored normal Isaac geometry")
    });
    let mut player = crate::combat::ContactBody {
        id: 0,
        entity_type: 1,
        position: player_position,
        velocity: crate::bombs::player_velocity(state),
        radius: crate::content::PLAYER_COLLISION_RADIUS_WORLD as f32,
        mass: crate::content::PLAYER_COLLISION_MASS_WORLD as f32,
    };
    let pickup = &state.room.pickups[index];
    let mut body = crate::combat::ContactBody {
        id: pickup.id,
        entity_type: 5,
        position: pickup_position,
        velocity: pickup.velocity,
        radius: pickup_radius,
        mass: pickup.collision_mass,
    };
    crate::combat::resolve_player_pickup_contact(&mut player, &mut body, |shift| {
        collider.blocks_player_shift(&grid, player_position, shift)
    })?;
    state.player.x = f64::from(player.position.x) * 0.65 + 32.0;
    state.player.y = f64::from(player.position.y) * 0.65 - 46.0;
    state.player.vx = f64::from(player.velocity.x) * 0.65;
    state.player.vy = f64::from(player.velocity.y) * 0.65;
    let pickup = &mut state.room.pickups[index];
    pickup.position = body.position;
    pickup.presented_position = body.position;
    pickup.velocity = body.velocity;
    pickup.project();
    Ok(())
}
