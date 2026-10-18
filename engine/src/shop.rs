//! Descriptor-owned normal Shop stock and the distinct paid Pickup callback.
//! Catalogue/room selection and loot operators remain with their existing owners.
use crate::{items::Rng, pickups::Pickup, room_spawns::SpawnSelector, State};
use serde::{Deserialize, Serialize};

/// Descriptor+D0/E8/EC. An absent owner is a legacy/unknown history;
/// an owned negative cursor is the actual constructor's lazy-init state.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Stock {
    pub categories: Option<[u8; 6]>,
    pub cursor: i32,
    pub sale_slot: i32,
    /// Selected factories that remain explicitly outside this leaf. Their
    /// real category/seed/cursor effects survive; no free actor substitutes.
    pub unadmitted: Vec<UnadmittedSku>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UnadmittedSku {
    pub spawn_seed: u32,
    pub slot: u32,
    pub variant: u32,
    pub subtype: u32,
    pub reason: String,
}

impl Stock {
    pub(crate) fn fresh() -> Self {
        // Level constructors587976/80 write both signed words to-1.
        Self {
            categories: None,
            cursor: -1,
            sale_slot: -1,
            unadmitted: Vec::new(),
        }
    }

    fn select(&mut self, following_seed: &mut u32) -> Result<(u32, u8), String> {
        if self.cursor < 0 {
            if *following_seed == 0 {
                return Err("Shop requires actual descriptor RNG35 seed".into());
            }
            let mut rng = Rng::new(*following_seed, [5, 9, 7]);
            let mut categories = [0, 1, 2, 3, 4, 5];
            //5D4354 shuffles;5D4370 copies its last six in reverse order.
            for index in (1..categories.len()).rev() {
                let other = rng.bounded(index as u32 + 1) as usize;
                categories.swap(index, other);
            }
            categories.reverse();
            categories[rng.bounded(3) as usize] = 6;
            self.categories = Some(categories);
            self.cursor = 0;
            self.sale_slot = if rng.bounded(3) == 0 {
                -1
            } else {
                rng.bounded(6) as i32
            };
            *following_seed = rng.state;
        }
        let slot = usize::try_from(self.cursor)
            .ok()
            .filter(|&slot| slot < 6)
            .ok_or("invalid Shop cursor")?;
        let category = self.categories.ok_or("Shop table history is unknown")?[slot];
        if category > 6 {
            return Err("invalid Shop category".into());
        }
        self.cursor = ((slot + 1) % 6) as i32;
        Ok((slot as u32, category))
    }
}

/// AC0/AC4 survive save/load. Sprite7D8 is a separate, unloadable owner;
/// purchasing does not rewrite the signed price word to zero.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Price {
    pub value: i32,
    pub slot: u32,
    pub sprite: Option<PriceSprite>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purchase_frontier: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PriceSprite {
    pub actor: String,
    pub animation: String,
    pub animation_frame: f32,
    pub animation_playing: bool,
    /// Actual SetLayerFrame overrides; None means that layer is hidden.
    pub layer_frames: [Option<u32>; 3],
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedPrice {
    pub value: i32,
    pub slot: u32,
}

fn base_price(variant: u32, subtype: u32) -> i32 {
    //5CBCE0 normal Shop2. Devil14/22 negative-heart prices are separate.
    match (variant, subtype) {
        (100, _) => 15,
        (10, 1 | 2) => 3,
        _ => 5,
    }
}

fn sale(price: i32) -> i32 {
    if price <= 5 {
        (price + 1) >> 1
    } else {
        price >> 1
    }
}

fn queries(state: &State) -> Option<crate::item_queries::AppearanceQueries<'_>> {
    let source = state.player.entity_source?;
    if source.entity_type != 1 || source.variant != 0 {
        return None;
    }
    Some(crate::item_queries::AppearanceQueries::new(
        state.player_items.as_ref()?,
        state.weapon_cache.primary,
        state.player.entity_source,
        None,
    ))
}

fn normal_inputs(state: &State) -> bool {
    let Some(queries) = queries(state) else {
        return false;
    };
    //Reached positive/unknown modifier histories do not acquire fake neutral
    //prices. StoreCredit lifetime and SteamSale price updates are later leaves.
    queries.has_trinket(13) == Some(false) && queries.have(64) == Some(false)
}

/// SetPrice4FEC40 for an owned positive normal Shop price. Chest.price
/// retains its existing distinct owner; this function never rewrites loot.
pub(crate) fn set_price(pickup: &mut Pickup, saved: SavedPrice) -> Result<(), String> {
    if saved.value <= 0 || saved.value > 99 || saved.slot >= 6 || pickup.chest.is_some() {
        return Err("unadmitted normal Shop price or slot".into());
    }
    let animation = if saved.value < base_price(pickup.variant, pickup.subtype) {
        "NumbersRed"
    } else {
        "NumbersWhite"
    };
    let body = if pickup.variant == 100 {
        "ShopIdle"
    } else {
        "Idle"
    };
    if !crate::pickups::definition(
        pickup.variant,
        crate::pickups::definition_subtype(pickup.variant, pickup.subtype),
    )
    .is_some_and(|definition| definition.clips.iter().any(|clip| clip.name == body))
    {
        return Err("priced Pickup body clip is unavailable".into());
    }
    pickup.overlay = None;
    pickup.overlay_frame = 0.0;
    pickup.overlay_playing = false;
    pickup.animation = body.into();
    pickup.animation_frame = 0.0;
    pickup.animation_playing = false;
    //This port bool is derived body/contact eligibility, not50C880 or
    //ACC. SetPrice replaces Appear with Idle/ShopIdle and class1; the
    //native504820 callback has no remaining Appear predicate. Preserve
    //the actual ACC delay above; fresh/re-Init4FF21B independently owns0.
    pickup.can_collect = true;
    if pickup.variant == 100 {
        //BaseInit446B8C retains native class5 from authored24 samples.
        pickup.regular_collision = Some(crate::pickups::RegularCollision {
            grid_collision_class: 5,
            entity_collision_class: 1,
        });
    }
    if let Some(collision) = &mut pickup.regular_collision {
        collision.entity_collision_class = 1;
    }
    pickup.shop_price = Some(Price {
        value: saved.value,
        slot: saved.slot,
        purchase_frontier: None,
        sprite: Some(PriceSprite {
            actor: "gfx/005.150_shop item.anm2".into(),
            animation: animation.into(),
            animation_frame: -1.0,
            animation_playing: false,
            layer_frames: [
                (saved.value >= 10).then_some(saved.value as u32 / 10),
                Some(saved.value as u32 % 10),
                Some(10),
            ],
        }),
    });
    Ok(())
}

///RoomLoad5C592F calls SetPrice(savedAC0), then restores savedAC4;
///the cursor/table and ItemPool are not sampled again on this path.
pub(crate) fn readmit_price(
    state: &State,
    pickup: &mut Pickup,
    saved: SavedPrice,
) -> Result<(), String> {
    if !normal_inputs(state) {
        return Err("saved Shop price modifier history unavailable".into());
    }
    set_price(pickup, saved)
}

fn unadmitted(
    state: &mut State,
    selector: &SpawnSelector,
    slot: u32,
    variant: u32,
    subtype: u32,
    reason: &str,
) {
    let descriptor = &mut state.floor.descriptors[state.current_room];
    descriptor
        .shop
        .as_mut()
        .expect("checked Shop owner")
        .unadmitted
        .push(UnadmittedSku {
            spawn_seed: selector.spawn_seed,
            slot,
            variant,
            subtype,
            reason: reason.into(),
        });
    state
        .room
        .descriptor
        .as_mut()
        .expect("checked Room descriptor")
        .shop = descriptor.shop.clone();
}

/// Pickup.Init4FFF50 consumes exactly one local35 output, then5D44A0
/// selects an actual SKU from the descriptor. No new floor/catalogue sampler.
pub(crate) fn from_selector(
    state: &mut State,
    id: u32,
    selector: &SpawnSelector,
) -> Result<Option<Pickup>, String> {
    if selector.entity_type != 5
        || selector.variant != 150
        || state.challenge_id != 0
        || state
            .room
            .descriptor
            .as_ref()
            .is_none_or(|room| room.template.room_type != 2)
    {
        return Ok(None);
    }
    let Some(profile) = state
        .profiles
        .get(usize::from(state.profile).wrapping_sub(1))
    else {
        return Ok(None);
    };
    //RoomInit5CEEF5..5CF0F1 reads only StoreUpgrade151..154. This leaf
    //admits ShopLevel1; it never requires unrelated achievement bytes false.
    if profile.achievements.len() != 179
        || profile.achievements[151..155].iter().any(|&value| value)
    {
        return Ok(None);
    }
    let room_index = state.current_room;
    let descriptor = &mut state.floor.descriptors[room_index];
    if state.room.descriptor.as_ref().is_none_or(|room| {
        room.shop != descriptor.shop || room.following_seed != descriptor.following_seed
    }) {
        return Err("Shop descriptor owner is inconsistent".into());
    }
    let stock = descriptor
        .shop
        .as_mut()
        .ok_or("Shop descriptor history is unknown")?;
    let (slot, category) = stock.select(&mut descriptor.following_seed)?;
    let sale_slot = stock.sale_slot;
    let room = state
        .room
        .descriptor
        .as_mut()
        .expect("checked Room descriptor");
    room.shop = descriptor.shop.clone();
    room.following_seed = descriptor.following_seed;
    let raw = crate::room_spawns::next35(selector.spawn_seed);
    let (variant, subtype) = match category {
        0 => (10, 1),
        1 => (40, 1),
        2 => (70, raw % 9 + 1),
        3 => (30, 1),
        4 => (10, 3),
        5 => (90, u32::from(selector.subtype)),
        6 => {
            let pools = state
                .item_pools
                .as_mut()
                .ok_or("Shop ItemPool owner absent")?;
            let pool = pools.room_pool(2, &mut state.boot_lifecycle.shared);
            (100, pools.collectible(pool, true, raw)?)
        }
        _ => return Err("invalid selected Shop category".into()),
    };
    if !normal_inputs(state) {
        unadmitted(
            state,
            selector,
            slot,
            variant,
            subtype,
            "priceModifierHistory",
        );
        return Ok(None);
    }
    let (variant, subtype) = if category == 2 {
        //4FFF55 caches the selector's first local35 raw inR14D. The
        //StarterDeck branch50036F passes that same seed to the existing
        //GetCard5817A0 producer, suits=true/runes=false, with no new draw.
        match queries(state).and_then(|query| query.have(251)) {
            Some(false) => (variant, subtype),
            Some(true) => {
                let achievements = &state.profiles[usize::from(state.profile) - 1].achievements;
                (
                    300,
                    crate::pickups::random_card(raw, achievements, true, false)?,
                )
            }
            None => {
                unadmitted(
                    state,
                    selector,
                    slot,
                    variant,
                    subtype,
                    "shopStarterDeckInput",
                );
                return Ok(None);
            }
        }
    } else {
        (variant, subtype)
    };
    let mut pickup = Pickup::spawn(id, variant, subtype, selector.position, false)?;
    pickup.initialize_spawn_seed(selector.spawn_seed)?;
    //Entity.Init owns persistent RNG30 for every type5, including100.
    pickup.regular_rng = Some(Rng::new(selector.spawn_seed, [4, 3, 17]));
    if variant == 100 {
        pickup.charge = crate::items::definition(subtype)
            .ok_or("Shop ItemConfig absent")?
            .max_charges;
    }
    let base = base_price(variant, subtype);
    set_price(
        &mut pickup,
        SavedPrice {
            value: if slot as i32 == sale_slot {
                sale(base)
            } else {
                base
            },
            slot,
        },
    )?;
    Ok(Some(pickup))
}

/// Returns true exactly when this actor belongs to the paid callback owner.
/// Refusal neither enters the free callback nor applies altar mass response.
pub(crate) fn player_contact(state: &mut State, index: usize) -> Result<bool, String> {
    let Some(price) = state.room.pickups[index].shop_price.as_ref() else {
        return Ok(false);
    };
    let original_price = price.value;
    let pickup = &state.room.pickups[index];
    if state.player.dead || pickup.removed || pickup.collected || !pickup.touches(&state.player) {
        return Ok(true);
    }
    let delay = pickup
        .pocket_contact_delay
        .ok_or("paid pickup AD0 history unavailable")?;
    if delay > 0 {
        state.room.pickups[index].pocket_contact_delay = Some(10);
        return Ok(true);
    }
    if !crate::pickups::player_can_collect(state) || pickup.collection_delay > 0 {
        return Ok(true);
    }
    let item_clip = state
        .player_items
        .as_ref()
        .is_some_and(|owner| owner.pickup_animation_guard)
        || state.player_animation.current.is_some_and(|clip| {
            matches!(clip.name(), "Pickup" | "LiftItem" | "HideItem" | "UseItem")
        });
    if item_clip || original_price > state.inventory.coins as i32 {
        return Ok(true);
    }
    if original_price <= 0 || !normal_inputs(state) {
        return Err("paid pickup reached an unadmitted price modifier".into());
    }
    //The current paid Coin callback has not closed its held/Kind2 effect
    //readset. This is a port frontier, not a new native Has predicate or an
    //inference that41 is inert in every paid callback.
    if pickup.variant == 20
        && state
            .player_items
            .as_ref()
            .is_some_and(|owner| owner.trinket_slots != [0, 0] || !owner.queue_excludes_trinkets())
    {
        state.room.pickups[index]
            .shop_price
            .as_mut()
            .expect("checked paid Coin owner")
            .purchase_frontier = Some("paidCoinTrinketReadset".into());
        return Ok(true);
    }
    //All reached unowned debit branches stop before grant. Manual-seed
    //eligibility=false proves5BD5F5's achievement callback is a no-op.
    let debit_frontier = if queries(state).and_then(|query| query.have(109)).is_none() {
        Some("addCoinsHave109")
    } else if state.achievement_eligibility && state.inventory.coins - original_price as u32 > 54 {
        Some("addCoinsAchievement2")
    } else {
        None
    };
    if let Some(frontier) = debit_frontier {
        state.room.pickups[index]
            .shop_price
            .as_mut()
            .expect("checked price owner")
            .purchase_frontier = Some(frontier.into());
        return Ok(true);
    }
    if pickup.variant == 100 {
        if crate::player_items::collect(state, index)?
            != crate::player_items::CollectibleContact::Collected
        {
            return Ok(true);
        }
        //Collect504480 selects Shop100 sound130 after the grant. Existing
        //free collectible4/12 sounds remain in their current owner.
        crate::pickups::collect_sound(state, index);
    } else {
        let multiplier = if state
            .player_items
            .as_ref()
            .is_some_and(|items| items.owned_counts[312] > 0)
        {
            2
        } else {
            1
        };
        let granted = match state.room.pickups[index].variant {
            70 | 300 => crate::pockets::grant(state, index)?,
            90 => match crate::pickups::battery_target(state) {
                Err(frontier) => {
                    state.room.pickups[index]
                        .shop_price
                        .as_mut()
                        .expect("checked price owner")
                        .purchase_frontier = Some(frontier.into());
                    return Ok(true);
                }
                Ok(None) => false,
                Ok(Some(target)) => {
                    crate::active_items::set_charge(state, target)?;
                    true
                }
            },
            10 => state.room.pickups[index].try_grant_heart(
                &mut state.player,
                multiplier,
                &mut state.level_progress,
            ),
            _ => state.room.pickups[index].try_grant(&mut state.inventory, &mut state.player),
        };
        if !granted {
            return Ok(true);
        }
        crate::pickups::collect_sound(state, index);
        let copy =
            crate::player_items::RegularPickupPresentation::from_pickup(&state.room.pickups[index]);
        crate::player_animation::force_pickup(state);
        let owner = state
            .player_items
            .as_mut()
            .ok_or("paid pickup Player item owner absent")?;
        owner.pickup_presentation = Some(crate::player_items::PickupPresentation {
            item_id: 0,
            animation_frame: copy.animation_frame,
            animation_playing: copy.animation_playing,
            pocket: None,
            regular: Some(copy),
        });
    }
    state.room.pickups[index]
        .shop_price
        .as_mut()
        .expect("checked price owner")
        .sprite = None;
    crate::player_items::debit_shop_coins(state, original_price)?;
    if state.room.pickups[index].variant != 100 {
        //441BB0 is immediate active31E=false/dead31F=true. Actual manager
        //erasure remains at the ordinary cleanup, not inside this callback.
        state.room.pickups[index].removed = true;
        state.room.pickups[index].collected = true;
        state.room.pickups[index].can_collect = false;
    }
    Ok(true)
}
