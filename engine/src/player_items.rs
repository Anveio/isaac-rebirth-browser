//! Natural collectible contact and the Player-owned deferred grant queue.
//! Item-specific stat/cache/familiar callbacks are separate owners. Their
//! requested cache bits stay observable until those callbacks are implemented.
use crate::{items, phase::Phase, State};
use serde::{Deserialize, Serialize};

/// Source TemporaryEffects records own config references and counts separately
/// from the Player's permanent347 counts. Fresh normal Isaac starts empty.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TemporaryCollectible {
    pub item_id: u32,
    pub count: i32,
    pub cooldown: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PickupPresentation {
    pub item_id: u32,
    pub animation_frame: f32,
    pub animation_playing: bool,
}

/// Actual independent inputs of Luck at 513C80. The fresh 519350 constructor
/// stores zero; unsupported producers and restore histories remain absent.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LuckInputs {
    pub source_d3c: i32,
    pub source_d40: i32,
    pub source_d58: f32,
    pub source_13cc: i32,
    pub source_13d0: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayerItems {
    pub initializer_seed: u32,
    pub manager_rng: items::Rng,
    pub initializer_rng: items::Rng,
    pub item_rngs: Vec<items::Rng>,
    pub trinket_rngs: Vec<items::Rng>,
    pub pocket_rngs: Vec<items::Rng>,
    pub entity_spawn_seed: u32,
    #[serde(default)]
    pub costume118: Option<crate::player_costume118::Costume118>,
    ///Actual fresh68 registration owns LeftEye. Old restore/other wardrobe
    ///history never receives an inventory-derived positive null owner.
    #[serde(default)]
    pub technology_eye_registered: Option<bool>,
    pub owned_counts: Vec<i32>,
    pub temporary_collectibles: Vec<TemporaryCollectible>,
    #[serde(default)]
    pub luck_inputs: Option<LuckInputs>,
    ///Actual held trinket slots E14/E18; normal fresh constructor initializes0.
    pub trinket_slots: [u32; 2],
    pub queued_id: Option<u32>,
    pub queued_charge: i32,
    pub queued_touched: bool,
    pub pickup_animation_frame: f32,
    pub pickup_animation_playing: bool,
    pub pickup_animation_guard: bool,
    pub pickup_presentation: Option<PickupPresentation>,
    pub collection_delay: i32,
    pub active_id: u32,
    pub active_charge: i32,
    pub active_subordinate_charge: i32,
    /// Opaque saved Player+D88 word, distinct from subordinate charge D7C.
    /// Actual constructor5196c1 initializes0;511aaf restores this word.
    pub save_word_d88: i32,
    pub active_state_d80: i32,
    pub active_state_d84: i32,
    pub reserve_charge: i32,
    pub active_flag_3076: bool,
    /// Source AddCollectible requests these bits. Keeping them pending does
    /// not claim that native 511f10 stat/familiar evaluation has occurred.
    pub unevaluated_cache_flags: u32,
}

impl PlayerItems {
    /// PlayerManager5bf4a0 passes saved Seeds output13 unchanged to519350.
    /// Each child is initialized by its actual local RNG5 consumer; none of
    /// these 347+62+41 initializers consumes the shared MT stream.
    pub fn new(initializer_seed: u32) -> Result<Self, String> {
        if initializer_seed == 0 {
            return Err("Player initializer seed must be nonzero".into());
        }
        let mut initializer_rng = items::Rng::new(initializer_seed, [1, 11, 16]);
        let mut children = |count| {
            (0..count)
                .map(|_| items::Rng::new(initializer_rng.raw(), [5, 9, 7]))
                .collect()
        };
        let item_rngs = children(347);
        let trinket_rngs = children(62);
        let pocket_rngs = children(41);
        let entity_spawn_seed = initializer_rng.raw();
        Ok(Self {
            initializer_seed,
            manager_rng: items::Rng::new(initializer_seed, [5, 7, 22]),
            initializer_rng,
            item_rngs,
            trinket_rngs,
            pocket_rngs,
            entity_spawn_seed,
            costume118: None,
            technology_eye_registered: Some(false),
            owned_counts: vec![0; 347],
            temporary_collectibles: Vec::new(),
            // 519680/51968A/519524 and the 28-word loop at 519798.
            luck_inputs: Some(LuckInputs {
                source_d3c: 0,
                source_d40: 0,
                source_d58: 0.0,
                source_13cc: 0,
                source_13d0: 0,
            }),
            trinket_slots: [0; 2],
            queued_id: None,
            queued_charge: 0,
            queued_touched: false,
            pickup_animation_frame: 0.0,
            pickup_animation_playing: false,
            pickup_animation_guard: false,
            pickup_presentation: None,
            collection_delay: 0,
            active_id: 0,
            active_charge: 0,
            active_subordinate_charge: 0,
            save_word_d88: 0,
            active_state_d80: 0,
            active_state_d84: 0,
            reserve_charge: 0,
            active_flag_3076: false,
            unevaluated_cache_flags: 0,
        })
    }

    /// Actual getter5e3ed0 searches the temporary record owner; absence is zero.
    pub fn temporary_count(&self, item_id: u32) -> i32 {
        self.temporary_collectibles
            .iter()
            .find(|effect| effect.item_id == item_id)
            .map_or(0, |effect| effect.count)
    }

    /// 5e4140 creates or increments the separate temporary count/cooldown
    /// record. Costume admission and cache evaluation remain caller-owned.
    pub(crate) fn add_temporary_collectible(&mut self, item_id: u32) -> Result<(), String> {
        let config = items::definition(item_id).ok_or("temporary collectible ItemConfig absent")?;
        // TEMP producers and expiry do not prove the retained modifier inputs.
        // Removing a later record never restores this history.
        self.luck_inputs = None;
        if let Some(effect) = self
            .temporary_collectibles
            .iter_mut()
            .find(|effect| effect.item_id == item_id)
        {
            effect.count = effect.count.wrapping_add(1);
            effect.cooldown = effect.cooldown.wrapping_add(config.cooldown);
        } else {
            self.temporary_collectibles.push(TemporaryCollectible {
                item_id,
                count: 1,
                cooldown: config.cooldown,
            });
        }
        Ok(())
    }

    /// Save518120→5180d0→5e3f80 erases real temporary records before the
    /// Player stores. For35/45,517580 has only the costume-removal callback;
    /// both pinned configs are costume-free and cache0. Other record kinds
    /// retain their actual state until their removal effects are implemented.
    pub(crate) fn clear_save_temporary_effects(&mut self) -> Result<u32, String> {
        let mut cache_flags = 0;
        for entry in &self.temporary_collectibles {
            let config = items::definition(entry.item_id)
                .ok_or("saved temporary record requires its actual ItemConfig")?;
            if !matches!(entry.item_id, 35 | 45) || config.kind != 3 || config.cache_flags != 0 {
                return Err("Player save requires an unmapped temporary removal callback".into());
            }
            //5e3f80 ORs every removed record's config+24, independent of
            //its count/cooldown. No fitted use count controls this removal.
            cache_flags |= config.cache_flags;
        }
        self.temporary_collectibles.clear();
        self.unevaluated_cache_flags |= cache_flags;
        Ok(cache_flags)
    }
}

/// The base actor advances the authored main clip before Player.Update tests
/// the animation-finished guard. Interpolation does not flush queued grants.
pub(crate) fn advance_presentation(state: &mut State, phase: Phase) {
    if phase != Phase::Update {
        return;
    }
    let Some(items) = state.player_items.as_mut() else {
        return;
    };
    if items.pickup_animation_playing {
        items.pickup_animation_frame += 1.0;
        if items.pickup_animation_frame >= crate::content::PLAYER_PICKUP_FRAMES as f32 {
            items.pickup_animation_frame = crate::content::PLAYER_PICKUP_FRAMES as f32 - 1.0;
            items.pickup_animation_playing = false;
        }
    }
    if items.pickup_animation_guard && !items.pickup_animation_playing {
        items.pickup_animation_guard = false;
    }
}

/// Player531e6f advances the copied item sprite after active/pocket/shot
/// callbacks, so a full-phase use advances its newly reset sprite immediately.
/// The fast branch skips this work and rejoins at531e7f.
pub(crate) fn advance_item_sprite(state: &mut State, phase: Phase) {
    if phase != Phase::Update {
        return;
    }
    let Some(items) = state.player_items.as_mut() else {
        return;
    };
    if let Some(presentation) = &mut items.pickup_presentation {
        let clip = crate::pickups::definition(100, presentation.item_id)
            .expect("authored collectible actor")
            .clips
            .iter()
            .find(|clip| clip.name == "PlayerPickupSparkle")
            .expect("authored pickup presentation clip");
        presentation.animation_frame += 1.0;
        if presentation.animation_frame >= clip.frames as f32 {
            presentation.animation_frame = 0.0;
        }
    }
}

/// 531edb decrements collection delay on both Player callback paths. The
/// fast branch rejoins at531e7f; only531f01 gates the later queue flush.
/// Natural contact happens later, after the admission-fenced actors.
pub(crate) fn finish_player_update(state: &mut State, phase: Phase) -> Result<(), String> {
    let Some(items) = state.player_items.as_mut() else {
        return Ok(());
    };
    if items.collection_delay > 0 {
        items.collection_delay -= 1;
    }
    if phase != Phase::Update {
        return Ok(());
    }
    if !items.pickup_animation_guard && !state.player_animation.guard {
        flush_queued(state)?;
    }
    Ok(())
}

/// Source52f310 always flushes a prior config before replacing queue fields.
fn enqueue(state: &mut State, id: u32, charge: i32, touched: bool) -> Result<(), String> {
    flush_queued(state)?;
    let items = state
        .player_items
        .as_mut()
        .ok_or("Player item owner absent")?;
    items.queued_id = Some(id);
    items.queued_charge = charge;
    items.queued_touched = touched;
    Ok(())
}

/// First free natural collectible, normal Isaac, with no active swap. The
/// retained empty altar is an actor with stopped Empty main frame0.
pub(crate) fn collect(state: &mut State, index: usize) -> Result<bool, String> {
    if !crate::pickups::player_can_collect(state) {
        return Ok(false);
    }
    let pickup = &state.room.pickups[index];
    let Some(owner) = state.player_items.as_ref() else {
        return Ok(false);
    };
    if pickup.variant != 100
        || pickup.subtype == 0
        || pickup.removed
        || pickup.collection_delay != 0
        || pickup.age <= 20
        || owner.collection_delay > 0
        || owner.pickup_animation_guard
        //504820 ->5054ca uses the selected item-clip name, independently
        // of bf4. UseItem owns the same main Sprite after retiring Pickup.
        || state.player_animation.current.is_some_and(|clip| {
            matches!(clip.name(), "Pickup" | "LiftItem" | "HideItem" | "UseItem")
        })
        || !pickup.touches(&state.player)
    {
        return Ok(false);
    }
    let id = pickup.subtype;
    let config = items::definition(id).ok_or("natural collectible config absent")?;
    if config.kind == 3 && owner.active_id != 0 {
        // Active swapping also owns Morph/RemoveCollectible callbacks; this
        // first-acquisition leaf leaves that unsupported contact untouched.
        return Ok(false);
    }
    let (charge, touched) = (pickup.charge, pickup.touched);
    state
        .item_pools
        .as_mut()
        .ok_or("ItemPool owner absent")?
        .removed[id as usize] = true;
    crate::player_animation::force_pickup(state);
    let owner = state.player_items.as_mut().expect("checked Player owner");
    owner.pickup_presentation = Some(PickupPresentation {
        item_id: id,
        animation_frame: 0.0,
        animation_playing: true,
    });
    let pickup = &mut state.room.pickups[index];
    pickup.animation = "PlayerPickupSparkle".into();
    pickup.animation_frame = 0.0;
    pickup.animation_playing = true;
    enqueue(state, id, charge, touched)?;
    let pickup = &mut state.room.pickups[index];
    pickup.subtype = 0;
    // 40fef0 is named SetFrame, not PlayOverlay: it selects the main Empty
    // clip, sets frame0 and clears main playing. Alternates remains untouched.
    pickup.animation = "Empty".into();
    pickup.animation_frame = 0.0;
    pickup.animation_playing = false;
    pickup.collected = true;
    pickup.can_collect = false;
    let room_type = state
        .room
        .descriptor
        .as_ref()
        .map(|room| room.template.room_type);
    if matches!(room_type, Some(4 | 12)) {
        state.audio.admit(
            &mut state.boot_lifecycle.shared,
            state.system_clock.system_counter - 1,
            crate::audio::SoundRequest {
                id: 128,
                volume: 0.5,
                delay: 2,
                looping: false,
                pitch: 1.0,
            },
        );
    }
    Ok(true)
}

/// Successful Collect returns1 and vetoes445710 mass response. The subsequent
/// free subtype0 callback returns0, so the retained class4 altar separates from
/// the Player using the same native impulse/position kernel as other bodies.
pub(crate) fn empty_altar_contact(state: &mut State, index: usize) -> Result<(), String> {
    use crate::{bombs, combat, grid, motion::WorldVector};
    use std::sync::OnceLock;
    let pickup = &state.room.pickups[index];
    //4453e7..4453fe vetoes a dead body independently of the collection
    //callback. A living Player's collection/queue guard still permits the
    //retained free class4 altar's physical response after504820 returns0.
    if state.player.dead || pickup.variant != 100 || pickup.subtype != 0 || pickup.removed {
        return Ok(());
    }
    let lattice = grid::Grid::new(
        state
            .room
            .grid
            .clone()
            .ok_or("altar contact requires admitted room grid")?,
    )?;
    static PLAYER_COLLIDER: OnceLock<grid::Collider> = OnceLock::new();
    let collider = PLAYER_COLLIDER.get_or_init(|| {
        grid::Collider::new(grid::CollisionGeometry {
            radius: crate::content::PLAYER_COLLISION_RADIUS_WORLD as f32,
            scale: WorldVector { x: 1.0, y: 1.0 },
            points: 40,
        })
        .expect("authored normal Isaac geometry")
    });
    let mut player = combat::ContactBody {
        id: 0,
        entity_type: 1,
        position: bombs::player_world(state),
        velocity: bombs::player_velocity(state),
        radius: crate::content::PLAYER_COLLISION_RADIUS_WORLD as f32,
        mass: 5.0,
    };
    let mut altar = combat::ContactBody {
        id: pickup.id,
        entity_type: 5,
        position: pickup.position,
        velocity: pickup.velocity,
        radius: pickup.collision_radius,
        mass: pickup.collision_mass,
    };
    let original_position = player.position;
    combat::resolve_player_pickup_contact(&mut player, &mut altar, |shift| {
        collider.blocks_player_shift(&lattice, original_position, shift)
    })?;
    state.player.x = f64::from(player.position.x) * 0.65 + 32.0;
    state.player.y = f64::from(player.position.y) * 0.65 - 46.0;
    state.player.vx = f64::from(player.velocity.x) * 0.65;
    state.player.vy = f64::from(player.velocity.y) * 0.65;
    let pickup = &mut state.room.pickups[index];
    pickup.position = altar.position;
    pickup.presented_position = altar.position;
    pickup.velocity = altar.velocity;
    pickup.project();
    Ok(())
}

fn add_inventory(value: u32, delta: i32) -> u32 {
    (value as i32).wrapping_add(delta).clamp(0, 99) as u32
}

fn normalize_black_hearts(player: &mut crate::Player) {
    if player.soul_hp == 0 {
        player.black_heart_mask = 0;
        return;
    }
    let first_outside = (player.soul_hp + 1) / 2;
    let keep = (1_u32 << first_outside) - 1;
    let mut removed = (player.black_heart_mask & !keep).count_ones();
    player.black_heart_mask &= keep;
    for position in (1..=player.soul_hp)
        .rev()
        .filter(|position| position % 2 == 1)
    {
        let bit = 1_u32 << (position / 2);
        if removed != 0 && player.black_heart_mask & bit == 0 {
            player.black_heart_mask |= bit;
            removed -= 1;
        }
    }
}

/// Normal-Isaac branch of5081b0, in native half-heart units.
pub(crate) fn add_max_hearts(player: &mut crate::Player, delta: i32) {
    player.max_hp = (i32::from(player.max_hp).wrapping_add(delta).clamp(0, 24)) as u8;
    player.soul_hp = player.soul_hp.min(24 - i32::from(player.max_hp));
    player.hp = player.hp.min(player.max_hp);
    normalize_black_hearts(player);
}

/// Normal-Isaac branch of508230, in native half-heart units.
pub(crate) fn add_red_hearts(player: &mut crate::Player, delta: i32) {
    player.hp = i32::from(player.hp)
        .wrapping_add(delta)
        .clamp(0, i32::from(player.max_hp)) as u8;
}

/// Normal-Isaac Eternal508640 health fields. The information
/// queue5805c0(13) is a separate presentation owner, not an audio draw.
pub(crate) fn add_eternal_hearts(player: &mut crate::Player, delta: i32) {
    player.eternal_hearts = player.eternal_hearts.wrapping_add(delta);
    if player.eternal_hearts > 1 {
        let paired = player.eternal_hearts & !1;
        add_max_hearts(player, paired);
        add_red_hearts(player, paired);
        player.eternal_hearts %= 2;
    }
}

pub(crate) fn add_soul_hearts(player: &mut crate::Player, delta: i32) {
    player.soul_hp = player
        .soul_hp
        .wrapping_add(delta)
        .clamp(0, 24 - i32::from(player.max_hp));
    normalize_black_hearts(player);
}

pub(crate) fn add_black_hearts(player: &mut crate::Player, delta: i32) {
    // 508440 adjusts slots before5083b0 caps total health. The parity path
    // and full-cap recoloring path are distinct from ordinary soul grants.
    let odd_delta = delta & 1;
    let (rounded_soul, extra, start_adjustment) = if player.soul_hp & 1 == 0 {
        (player.soul_hp, odd_delta, 0)
    } else {
        (player.soul_hp + 1, 0, -odd_delta)
    };
    if i32::from(player.max_hp) + rounded_soul > 22 {
        let mut marked = 0;
        for position in (1..rounded_soul).rev().filter(|position| position % 2 == 1) {
            let bit = 1_u32 << (position / 2);
            if marked >= delta {
                break;
            }
            if player.black_heart_mask & bit == 0 {
                player.black_heart_mask |= bit;
                marked += 2;
            }
        }
    } else {
        let start = (rounded_soul + start_adjustment) / 2;
        let end = (rounded_soul + extra + delta) / 2;
        for slot in start..end {
            player.black_heart_mask |= 1_u32 << (slot as u32 & 31);
        }
    }
    add_soul_hearts(player, delta);
}

fn authored_grants(state: &mut State, config: &items::CollectibleDefinition) {
    let [max, red, soul, black, bombs, keys, coins] = config.inventory;
    state.inventory.bombs = add_inventory(state.inventory.bombs, bombs);
    state.inventory.keys = add_inventory(state.inventory.keys, keys);
    state.inventory.coins = add_inventory(state.inventory.coins, coins);
    let player = &mut state.player;
    add_max_hearts(player, max);
    add_red_hearts(player, red);
    add_soul_hearts(player, soul);
    add_black_hearts(player, black);
}

/// 52ee40 clears pointer/charge only after52cc90. The stored touched argument
/// remains; it is meaningful only while an actual queued config exists.
pub(crate) fn flush_queued(state: &mut State) -> Result<(), String> {
    let Some(owner) = state.player_items.as_ref() else {
        return Ok(());
    };
    let Some(id) = owner.queued_id else {
        return Ok(());
    };
    let (charge, touched) = (owner.queued_charge, owner.queued_touched);
    add_collectible(state, id, charge, touched)?;
    let owner = state
        .player_items
        .as_mut()
        .expect("checked Player item owner");
    owner.queued_id = None;
    owner.queued_charge = 0;
    Ok(())
}

///Actual52cc90 common grant callback, shared by queued pickup and Challenge
///Player52e1a0. Queue and pool-removal fields remain owned by the caller.
pub(crate) fn add_collectible(
    state: &mut State,
    id: u32,
    charge: i32,
    touched: bool,
) -> Result<(), String> {
    let config = items::definition(id).ok_or("granted collectible config absent")?;
    let normal_isaac = state.weapon_admission.source_player_type == Some(0);
    let owner = state
        .player_items
        .as_mut()
        .expect("checked Player item owner");
    owner.owned_counts[id as usize] = owner.owned_counts[id as usize].wrapping_add(1);
    //68 has the same closed zero-amount/common auxiliary producer. Preserve
    //only an existing owner in its bounded cache context; never create Some.
    let keep_technology_luck = id == 68
        && normal_isaac
        && crate::item_pipeline::ItemInputs::new(owner)
            .is_ok_and(|inputs| inputs.admits_technology_cache_context());
    let keep_fetus_luck = id == 52
        && normal_isaac
        && crate::item_pipeline::ItemInputs::new(owner)
            .is_ok_and(|inputs| inputs.admits_fetus_grant_context());
    let keep_epic_fetus_luck = id == 168
        && normal_isaac
        && crate::item_pipeline::ItemInputs::new(owner)
            .is_ok_and(|inputs| inputs.admits_epic_fetus_grant_context());
    if ![1, 4, 6, 114, 118, 329, 304].contains(&id)
        && !keep_technology_luck
        && !keep_fetus_luck
        && !keep_epic_fetus_luck
    {
        // These admitted grant paths preserve the fresh auxiliary words.
        // Nonzero setters and restore need their own producer binding.
        owner.luck_inputs = None;
    }
    if [81, 133, 134, 145, 187, 212].contains(&id) {
        // The 52CC90 producer needs its actual 576BB0 guard. Current item
        // counts cannot reconstruct these transformation history counters.
        state.player.transformation_counts = None;
    }
    crate::player_costume118::on_collectible_added(state, id);
    crate::technology::on_collectible_added(state, id);
    if config.kind != 3 || !touched {
        authored_grants(state, config);
    }
    let owner = state
        .player_items
        .as_mut()
        .expect("checked Player item owner");
    if config.kind == 3 {
        let old = owner.active_id;
        owner.active_id = id;
        owner.owned_counts[old as usize] = 0;
        let deficit = config.max_charges - charge;
        let transferred = if deficit > 0 && owner.reserve_charge > 0 {
            let transferred = deficit.min(owner.reserve_charge);
            owner.reserve_charge = (owner.reserve_charge - deficit).max(0);
            transferred
        } else {
            0
        };
        let requested = charge + transferred;
        owner.active_subordinate_charge = 0;
        owner.active_state_d80 = 0;
        owner.active_flag_3076 = false;
        owner.collection_delay = 30;
        if matches!(id, 130 | 181) {
            owner.unevaluated_cache_flags |= config.cache_flags;
        }
        crate::active_items::set_charge(state, requested)?;
    } else {
        owner.unevaluated_cache_flags |= config.cache_flags;
    }
    let owner = state
        .player_items
        .as_mut()
        .expect("checked Player item owner");
    if owner.owned_counts[109] != 0 {
        owner.unevaluated_cache_flags |= 1;
    }
    if id == 72 {
        for pool in &mut state
            .item_pools
            .as_mut()
            .ok_or("ItemPool owner absent")?
            .pools
        {
            pool.special_counter = pool.special_counter.wrapping_add(5);
        }
    }
    // AddCollectible52d173 calls cache evaluation before52ee40 clears queue
    // fields. The Familiar callback owns its actual local/shared consumers.
    crate::player_stats::evaluate_cache(state)?;
    Ok(())
}
