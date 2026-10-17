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
    ///The same copied Sprite7E0 owns collectible and Card/Pill presentation.
    ///Absent here preserves the existing collectible representation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pocket: Option<crate::pockets::PocketPresentation>,
}

/// Actual independent inputs of Luck at 513C80. The fresh 519350 constructor
/// stores zero; unsupported producers and restore histories remain absent.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LuckInputs {
    pub source_d3c: i32,
    pub source_d40: i32,
    pub source_d58: f32,
}

/// Actual Player+D38, independent of inventory and the evaluated C7C output.
/// Fresh519676 owns zero; an untracked writer never regains a guessed zero.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShotSpeedInputs {
    pub source_d38: i32,
}

impl ShotSpeedInputs {
    pub(crate) const NEUTRAL: Self = Self { source_d38: 0 };
}

///Independent mutable MoveSpeed16 inputs, not a Luck or inventory owner.
///D28 is a signed persistent term; the other three terms remain neutral-only.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MoveSpeedInputs {
    pub source_d28: i32,
    pub source_d44: i32,
}
impl MoveSpeedInputs {
    pub(crate) const NEUTRAL: Self = Self {
        source_d28: 0,
        source_d44: 0,
    };

    pub fn apply_signed_delta(&mut self, delta: i32) {
        self.source_d28 = self.source_d28.wrapping_add(delta);
    }
}

/// Actual Player+D30, independent of collectible counts and cached C88.
/// Fresh519662 owns zero; an untracked writer leaves this input unknown.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DamageInputs {
    pub source_d30: i32,
}

impl DamageInputs {
    /// Actual signed ADD/LEA producer law. Call only for an already owned
    /// input after its source event/RNG plan supplies the delta; never heal None.
    pub fn apply_signed_delta(&mut self, delta: i32) {
        self.source_d30 = self.source_d30.wrapping_add(delta);
    }
}

/// Actual Player+D2C signed tier input, independent of cached C78.
/// Constructor519658 owns zero; unknown producers never reconstruct it.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FireDelayInputs {
    pub source_d2c: i32,
}

impl FireDelayInputs {
    /// Actual signed32 ADD producer law on an already owned input.
    pub fn apply_signed_delta(&mut self, delta: i32) {
        self.source_d2c = self.source_d2c.wrapping_add(delta);
    }
}

/// Actual Player+D34 signed height input, independent of cached C8C.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RangeInputs {
    pub source_d34: i32,
}

impl RangeInputs {
    pub fn apply_signed_delta(&mut self, delta: i32) {
        self.source_d34 = self.source_d34.wrapping_add(delta);
    }
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
    ///Actual two eight-byte held records, independent of reserve chargeD8C.
    ///An old restore does not inherit fresh empty pocket ownership.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub held_pockets: Option<crate::pockets::HeldPockets>,
    ///Retained unknown producer footprints survive an otherwise empty Room.
    #[serde(default)]
    pub ordinary_producer_history: Option<crate::ordinary_placement::Coverage>,
    ///Validity of completed world grants and conditional writes to the
    ///existing item RNGs. Omitted legacy history remains unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pickup_grant_history: Option<crate::pickup_world_grants::History>,
    pub entity_spawn_seed: u32,
    #[serde(default)]
    pub costume118: Option<crate::player_costume118::Costume118>,
    ///Only an actual229 registration admits this pose history. Old restored
    ///counts stay None, and absent ownership adds no field to old game states.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub costume229: Option<crate::player_costume229::Costume229>,
    ///Actual collectible costume-config registrations, separate from each
    ///selected pose/null/caller readset. Old restores have unknown history.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub costume_registrations:
        Option<crate::player_costume_registration::CostumeRegistrationHistory>,
    ///Actual fresh68 registration owns LeftEye. Old restore/other wardrobe
    ///history never receives an inventory-derived positive null owner.
    #[serde(default)]
    pub technology_eye_registered: Option<bool>,
    pub owned_counts: Vec<i32>,
    pub temporary_collectibles: Vec<TemporaryCollectible>,
    ///Provenance of the existing permanent/TEMP/trinket/C08 stores.
    ///Legacy omission is unknown, never a fresh empty inventory proof.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_history: Option<crate::item_queries::ItemQueryHistory>,
    #[serde(default, rename = "luckTerms")]
    pub luck_inputs: Option<LuckInputs>,
    #[serde(
        default,
        rename = "moveSpeedTerms",
        skip_serializing_if = "Option::is_none"
    )]
    pub move_speed_inputs: Option<MoveSpeedInputs>,
    ///One canonical Player1388..13F4 array; State owns its versioned wire view.
    #[serde(skip)]
    pub pill_effect_history: Option<crate::pill_effect_history::PillEffectHistory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_inputs: Option<DamageInputs>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range_inputs: Option<RangeInputs>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fire_delay_inputs: Option<FireDelayInputs>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shot_speed_inputs: Option<ShotSpeedInputs>,
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
    ///Typed unresolved effects from an actual240 grant. These do not mark
    ///profile achievements complete or admit any missing cache/weapon role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub treatment240_frontiers: Option<crate::experimental_treatment::Frontiers>,
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
            held_pockets: Some(crate::pockets::HeldPockets::fresh()),
            ordinary_producer_history: Some(crate::ordinary_placement::Coverage::fresh_player()),
            pickup_grant_history: Some(crate::pickup_world_grants::History::fresh()),
            entity_spawn_seed,
            costume118: None,
            costume229: None,
            costume_registrations: Some(
                crate::player_costume_registration::CostumeRegistrationHistory::fresh(),
            ),
            technology_eye_registered: Some(false),
            owned_counts: vec![0; 347],
            temporary_collectibles: Vec::new(),
            query_history: Some(crate::item_queries::ItemQueryHistory::FRESH),
            // 519680/51968A/519524 and the 28-word loop at 519798.
            luck_inputs: Some(LuckInputs {
                source_d3c: 0,
                source_d40: 0,
                source_d58: 0.0,
            }),
            shot_speed_inputs: Some(ShotSpeedInputs::NEUTRAL),
            // Actual519662, not a default reconstructed from inventory.
            damage_inputs: Some(DamageInputs { source_d30: 0 }),
            //Actual51966C owns this initial word; no inventory inference.
            range_inputs: Some(RangeInputs { source_d34: 0 }),
            //51964E/51954C and519798 own these four original zeros.
            move_speed_inputs: Some(MoveSpeedInputs::NEUTRAL),
            pill_effect_history: Some(crate::pill_effect_history::PillEffectHistory::FRESH),
            //Actual519658, not a cache/count-derived neutral default.
            fire_delay_inputs: Some(FireDelayInputs { source_d2c: 0 }),
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
            treatment240_frontiers: None,
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
        //The existing Use wrapper retains its independent input policy.
        self.luck_inputs = None;
        self.move_speed_inputs = None;
        self.insert_source_temporary_collectible(item_id)
    }

    ///5E4140's canonical record/count/cooldown prefix. The caller separately
    ///owns costume registration and effect dispatch. This single vector is
    ///also the source for lazy record queries; unknown prior counts stay
    ///unknown in query_history after this owned increment.
    pub(crate) fn insert_source_temporary_collectible(
        &mut self,
        item_id: u32,
    ) -> Result<(), String> {
        let config = items::definition(item_id).ok_or("temporary collectible ItemConfig absent")?;
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
        if let Some(history) = &mut self.query_history {
            history.record_temporary_cache_write(config.cache_flags);
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
        if presentation.pocket.is_some() {
            if presentation.animation_playing {
                presentation.animation_frame += 1.0;
                if presentation.animation_frame >= crate::pickups::POCKET_SPARKLE_FRAMES as f32 {
                    presentation.animation_frame =
                        crate::pickups::POCKET_SPARKLE_FRAMES as f32 - 1.0;
                    presentation.animation_playing = false;
                }
            }
            return;
        }
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

/// What 504820 returns for a Player touching a free collectible pedestal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CollectibleContact {
    /// The collect path returns 1, vetoing 445710's mass response.
    Collected,
    /// Every refusal returns 0 through 5048b8/504910 for normal Isaac, so the
    /// class4 altar separates from the Player: the empty altar (5054ba), a
    /// FrameCount of at most 20 (505d3a), a held item (50c780), the Player
    /// delay (5054d3) or the pickup wait (504a30).
    Refused,
    /// A removed actor, or the unadmitted active swap: left untouched.
    Untouched,
}

/// First free natural collectible, normal Isaac, with no active swap. The
/// retained empty altar is an actor with stopped Empty main frame0.
pub(crate) fn collect(state: &mut State, index: usize) -> Result<CollectibleContact, String> {
    if state.room.pickups[index].variant != 100 || state.room.pickups[index].removed {
        return Ok(CollectibleContact::Untouched);
    }
    if !crate::pickups::player_can_collect(state) {
        return Ok(CollectibleContact::Refused);
    }
    let pickup = &state.room.pickups[index];
    let Some(owner) = state.player_items.as_ref() else {
        return Ok(CollectibleContact::Untouched);
    };
    if pickup.subtype == 0
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
        return Ok(CollectibleContact::Refused);
    }
    let id = pickup.subtype;
    let config = items::definition(id).ok_or("natural collectible config absent")?;
    if config.kind == 3 && owner.active_id != 0 {
        // Active swapping also owns Morph/RemoveCollectible callbacks; this
        // first-acquisition leaf leaves that unsupported contact untouched.
        return Ok(CollectibleContact::Untouched);
    }
    let (charge, touched) = (pickup.charge, pickup.touched);
    state
        .item_pools
        .as_mut()
        .ok_or("ItemPool owner absent")?
        .removed[id as usize] = true;
    //505DE9 starts the authored title/description streak before505E5E's
    //Pickup pose and505ED6's queue. Rejected contact never requests it.
    state.run_introduction = crate::RunIntroduction::collectible(config);
    crate::player_animation::force_pickup(state);
    let owner = state.player_items.as_mut().expect("checked Player owner");
    owner.pickup_presentation = Some(PickupPresentation {
        item_id: id,
        animation_frame: 0.0,
        animation_playing: true,
        pocket: None,
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
    Ok(CollectibleContact::Collected)
}

/// Successful Collect returns1 and vetoes445710 mass response. Every refused
/// contact returns0, so the class4 altar, full or empty, separates from the
/// Player using the same native impulse/position kernel as other bodies.
pub(crate) fn altar_contact(state: &mut State, index: usize) -> Result<(), String> {
    use crate::{bombs, combat, grid, motion::WorldVector};
    use std::sync::OnceLock;
    let pickup = &state.room.pickups[index];
    //4453e7..4453fe vetoes a dead body independently of the collection
    //callback. A living Player's collection/queue guard still permits the
    //retained free class4 altar's physical response after504820 returns0.
    if state.player.dead || pickup.variant != 100 || pickup.removed {
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

pub(crate) fn normalize_black_hearts(player: &mut crate::Player) {
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

///Actual normal-player52CF87 dispatch and52D0D0 transformation tail.
///These are source effect branches, not an inventory/cache/getter allowlist.
///The caller separately requires actual passive config and Player1/0+BFC0.
fn has_default_passive_grant_path(id: u32) -> bool {
    //Selected dispatch effects: world/pool, spawn, body/sprite scale, health,
    //active charge and actual240/258 producers stay outside this common path.
    if matches!(
        id,
        10 | 12
            | 21
            | 54
            | 71
            | 72
            | 75
            | 76
            | 81
            | 91
            | 116
            | 120
            | 121
            | 141
            | 155
            | 179
            | 194
            | 195
            | 196
            | 198
            | 230
            | 232
            | 240
            | 246
            | 251
            | 252
            | 258
            | 260
            | 313
            | 333
            | 340
            | 343
            | 344
    ) {
        return false;
    }
    //Either source transformation counter/helper path remains independent.
    !matches!(
        id,
        9 | 10
            | 57
            | 81
            | 128
            | 133
            | 134
            | 145
            | 148
            | 151
            | 187
            | 212
            | 248
            | 264
            | 272
            | 274
            | 279
            | 320
    )
}

///Reached normal passive dispatch effect, separate from its completion.
///Only closed write domains may retain an existing persistent input owner.
#[derive(Clone, Copy)]
enum PassiveGrantSourceEffect {
    DefaultTail,
    ItemRngCardSpawn,
}

impl PassiveGrantSourceEffect {
    fn from_dispatch(id: u32) -> Option<Self> {
        //52CF87/52DF20 selects this effect; it is not an inventory role.
        if id == 194 {
            Some(Self::ItemRngCardSpawn)
        } else if has_default_passive_grant_path(id) {
            Some(Self::DefaultTail)
        } else {
            None
        }
    }

    ///Closed E1C/CF8/E14/E18 write footprint, independent of scalar
    ///input retention and the actual world producer's completion.
    fn preserves_query_stores(self) -> bool {
        match self {
            Self::DefaultTail => true,
            Self::ItemRngCardSpawn => true,
        }
    }

    fn preserves_default_inputs(self) -> bool {
        matches!(self, Self::DefaultTail)
    }

    fn preserves_shot_speed_input(self) -> bool {
        match self {
            Self::DefaultTail => true,
            //52DF20..52DFAE: free-position search, two local item RNG
            //draws, GetCard, explicit Card Init/admission and ordinary
            //cache do not write D38. Other words and the actual world
            //effect's completion retain their independent frontiers.
            Self::ItemRngCardSpawn => true,
        }
    }
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
    //Validate the actual240 RNG owner before count or draws. Gameplay
    //contexts retain independent partial ownership and never refuse pickup.
    let treatment = if id == 240 {
        Some(crate::experimental_treatment::preflight(state)?)
    } else {
        None
    };
    let normal_isaac = state.weapon_admission.source_player_type == Some(0);
    //The common authored inventory/costume domains independently exclude
    //D2C/D30/D34/D38, the five Luck inputs, and D28/D44/13BC/13C0.
    //Each input has its own write-domain proof; cache flags grant none.
    //This event only retains existing owners; every None remains None.
    let normal_passive_source_context = normal_isaac
        && state
            .player
            .entity_source
            .is_some_and(|source| source.entity_type == 1 && source.variant == 0)
        && config.kind == 1;
    let source_effect = if normal_passive_source_context {
        PassiveGrantSourceEffect::from_dispatch(id)
    } else {
        None
    };
    let keep_default_passive_inputs =
        source_effect.is_some_and(PassiveGrantSourceEffect::preserves_default_inputs);
    let keep_shot_speed_inputs =
        source_effect.is_some_and(PassiveGrantSourceEffect::preserves_shot_speed_input);
    let owner = state
        .player_items
        .as_mut()
        .expect("checked Player item owner");
    //Query provenance follows the reached source dispatch, independently
    //of persistent scalar inputs or world-effect completion. A default
    //active grant from actual empty/D80=0 has the same plain dispatch;
    //52D1B0 also clears the old active count, modeled below. No Use is
    //inferred from granting an active.194 only spawns a world pickup.
    let normal_entity = state
        .player
        .entity_source
        .is_some_and(|source| source.entity_type == 1 && source.variant == 0);
    let plain_active_grant = normal_isaac
        && normal_entity
        && config.kind == 3
        && owner.active_id == 0
        && owner.active_state_d80 == 0
        && has_default_passive_grant_path(id);
    let query_stores_preserved = source_effect
        .is_some_and(PassiveGrantSourceEffect::preserves_query_stores)
        || plain_active_grant
        || (normal_isaac && normal_entity && treatment.is_some());
    if !query_stores_preserved {
        //An actual unclosed Add dispatch may reach another counter writer.
        //Closed default/194/treatment/plain-active footprints preserve the array.
        crate::pill_effect_history::forget(owner);
        if let Some(history) = &mut owner.query_history {
            history.forget_unowned_effect();
        }
    }
    owner.owned_counts[id as usize] = owner.owned_counts[id as usize].wrapping_add(1);
    crate::pockets::note_modifier_grant(owner, id);
    //52CEA2..52CEFD: the actual variant0 Add inserts into Game8BA0,
    //after permanent increment/source unlock checks and before effects.
    //The membership result is captured now, never reconstructed from counts
    //or reread after later effects. Raw first metadata is independently absent.
    let acquisition_variant = state.player.entity_source.map(|source| source.variant);
    let previously_acquired = match acquisition_variant {
        Some(0) => state
            .acquisition_history
            .as_mut()
            .map(|history| history.seen_or_insert(id as i32, None)),
        Some(_) => None,
        None => {
            //An unknown branch may have inserted. Retaining an exact set
            //would falsely claim absence for this and future membership reads.
            state.acquisition_history = None;
            None
        }
    };
    //Actual48 independently preserves D28/D44/13BC/13C0 on its
    //closed common path. This input history gives no stage or caller role.
    let keep_cupid_move_speed = id == 48
        && normal_isaac
        && crate::item_pipeline::ItemInputs::new(owner)
            .is_ok_and(|inputs| inputs.admits_cupid_grant_context());
    //Actual14/27/28/143 have cached speed effects but preserve these four
    //persistent inputs. Each selected source Add retains existing Some only;
    //untracked258/reset routes invalidate, never reconstruct history.
    if !(normal_isaac
        && matches!(
            id,
            1 | 2
                | 3
                | 4
                | 6
                | 14
                | 27
                | 28
                | 52
                | 68
                | 114
                | 118
                | 143
                | 152
                | 168
                | 169
                | 229
                | 304
                | 329
                | 330
        ))
        && !keep_cupid_move_speed
        && !keep_default_passive_inputs
        && !treatment.is_some_and(|proof| proof.preserves_move_speed_history())
    {
        owner.move_speed_inputs = None;
    }
    //Actual48's selected common path independently preserves D34.
    //This input proof grants no Range callback/getter/caller permission.
    let keep_cupid_range = id == 48
        && normal_isaac
        && crate::item_pipeline::ItemInputs::new(owner)
            .is_ok_and(|inputs| inputs.admits_cupid_grant_context());
    //Actual14's default Add route preserves D34 despite its cached Range
    //effect. Untracked258/reset routes invalidate this existing owner;
    //None never inherits constructor zero through counts or cache evaluation.
    if !matches!(
        id,
        1 | 2
            | 3
            | 4
            | 6
            | 14
            | 52
            | 68
            | 114
            | 118
            | 143
            | 152
            | 168
            | 169
            | 229
            | 304
            | 329
            | 330
    ) && !keep_cupid_range
        && !keep_default_passive_inputs
        && !treatment.is_some_and(|proof| proof.preserves_normal_inputs())
    {
        owner.range_inputs = None;
    }
    //Actual Add48 preserves D2C on its independently closed common path.
    //Neither D30/D38 preservation nor a cache role supplies this word's proof.
    let keep_cupid_fire_delay = id == 48
        && normal_isaac
        && crate::item_pipeline::ItemInputs::new(owner)
            .is_ok_and(|inputs| inputs.admits_cupid_grant_context());
    //These selected Add routes preserve an existing D2C owner. Actual222's
    //default tail is separately closed; untracked240/258/reset routes lose
    //proof. None stays None and is never reconstructed from counts/cache.
    if !matches!(
        id,
        1 | 2
            | 3
            | 4
            | 6
            | 52
            | 68
            | 114
            | 118
            | 143
            | 152
            | 168
            | 169
            | 222
            | 229
            | 304
            | 329
            | 330
    ) && !keep_cupid_fire_delay
        && !keep_default_passive_inputs
        && !treatment.is_some_and(|proof| proof.preserves_normal_inputs())
    {
        owner.fire_delay_inputs = None;
    }
    //Actual Add48's selected default/common path preserves D30.
    //This word's proof is independent of D38 and every cache role.
    let keep_cupid_damage = id == 48
        && normal_isaac
        && crate::item_pipeline::ItemInputs::new(owner)
            .is_ok_and(|inputs| inputs.admits_cupid_grant_context());
    // These actual Add routes preserve D30, independently of their cache,
    // getter and caller permissions. Unmapped258 and other routes lose
    // the existing owner; later counts/cache work cannot recreate it.
    if !matches!(
        id,
        1 | 2
            | 3
            | 4
            | 6
            | 12
            | 52
            | 68
            | 114
            | 118
            | 143
            | 152
            | 168
            | 169
            | 229
            | 304
            | 329
            | 330
    ) && !keep_cupid_damage
        && !keep_default_passive_inputs
        && !treatment.is_some_and(|proof| proof.preserves_normal_inputs())
    {
        owner.damage_inputs = None;
    }
    //Actual Add48's selected default/common path preserves D38.
    //This word's proof is independent of Luck and every cache role.
    let keep_cupid_shot_speed = id == 48
        && normal_isaac
        && crate::item_pipeline::ItemInputs::new(owner)
            .is_ok_and(|inputs| inputs.admits_cupid_grant_context());
    // These actual Add paths preserve D38. This event proof grants no stat,
    // getter or caller permission; unknown Add258 and other producers
    // invalidate an existing owner rather than deriving one from counts.
    if !matches!(
        id,
        1 | 2 | 3 | 4 | 6 | 52 | 68 | 114 | 118 | 143 | 152 | 168 | 169 | 229 | 304 | 329 | 330
    ) && !keep_cupid_shot_speed
        && !keep_shot_speed_inputs
        && !treatment.is_some_and(|proof| proof.preserves_normal_inputs())
    {
        owner.shot_speed_inputs = None;
    }
    //Only actual2/4/152/169 Add paths have a closed neutral C70 lifetime here.
    //Keep an existing owner for those grants; other producers invalidate it.
    //Neither current counts nor a cache evaluation can recreate this word.
    //169's default52D0D0..52D170 path preserves this existing owner too.
    if !matches!(id, 2 | 4 | 152 | 169)
        && !keep_default_passive_inputs
        && !treatment.is_some_and(|proof| proof.preserves_c70_history())
    {
        state.attack_runtime.aim_rotation_c70 = None;
    }
    //68 has the same closed zero-amount/common auxiliary producer. Preserve
    //only an existing owner in its bounded cache context; never create Some.
    let keep_technology_luck = id == 68
        && normal_isaac
        && crate::item_pipeline::ItemInputs::new(owner)
            .is_ok_and(|inputs| inputs.admits_technology_cache_context());
    //Use actual post-increment counts; preserve an existing Some, never
    //construct unknown Luck words from current inventory.
    let keep_soy_luck = id == 330
        && normal_isaac
        && crate::item_pipeline::ItemInputs::new(owner)
            .is_ok_and(|inputs| inputs.admits_soy_milk_cache_context());
    let keep_technology2_luck = id == 152
        && normal_isaac
        && crate::item_pipeline::ItemInputs::new(owner)
            .is_ok_and(|inputs| inputs.admits_technology2_grant_context());
    let keep_fetus_luck = id == 52
        && normal_isaac
        && crate::item_pipeline::ItemInputs::new(owner)
            .is_ok_and(|inputs| inputs.admits_fetus_grant_context());
    let keep_monstro_luck = id == 229
        && normal_isaac
        && crate::item_pipeline::ItemInputs::new(owner)
            .is_ok_and(|inputs| inputs.admits_monstro_grant_context());
    let keep_inner_eye_knife_luck = id == 2
        && normal_isaac
        && crate::item_pipeline::ItemInputs::new(owner)
            .is_ok_and(|inputs| inputs.admits_inner_eye_knife_grant());
    let keep_epic_fetus_luck = matches!(id, 2 | 168)
        && normal_isaac
        && crate::item_pipeline::ItemInputs::new(owner)
            .is_ok_and(|inputs| inputs.admits_epic_fetus_grant_context());
    if ![1, 4, 6, 114, 118, 329, 304].contains(&id)
        && !keep_technology_luck
        && !keep_technology2_luck
        && !keep_soy_luck
        && !keep_fetus_luck
        && !keep_monstro_luck
        && !keep_epic_fetus_luck
        && !keep_default_passive_inputs
        && !keep_inner_eye_knife_luck
        && !treatment.is_some_and(|proof| proof.preserves_luck_history())
    {
        // These admitted grant paths preserve the fresh auxiliary words.
        // Nonzero setters and restore need their own producer binding.
        owner.luck_inputs = None;
    }
    let transformation_tail_skipped = acquisition_variant.is_some_and(|variant| variant != 0)
        || previously_acquired == Some(true);
    if [81, 133, 134, 145, 187, 212].contains(&id) && !transformation_tail_skipped {
        //52D0D3..DE skips credit for nonzero variants or already-seen IDs.
        //The first/unknown-acquisition counter/helper path remains unowned;
        //a duplicate never restores an absent counter or repeats credit.
        state.player.transformation_counts = None;
    }
    crate::player_costume118::on_collectible_added(state, id);
    crate::player_costume_registration::on_collectible_added(state, id);
    crate::player_costume229::on_collectible_added(state, id);
    crate::technology::on_collectible_added(state, id);
    if config.kind != 3 || !touched {
        if let Some(proof) = treatment {
            crate::experimental_treatment::authored_grants(state, proof);
        } else {
            authored_grants(state, config);
        }
    }
    //This is the actual reached default CFG effect, not an ID/loadout
    //whitelist. It never heals a prior missing producer history.
    let ordinary_no_world_effect =
        matches!(source_effect, Some(PassiveGrantSourceEffect::DefaultTail))
            && config.inventory.iter().all(|amount| *amount == 0)
            && config.cache_flags & (256 | 512) == 0;
    if id == 194 {
        crate::pickup_world_grants::on_card194_added(state);
    } else if !ordinary_no_world_effect {
        crate::ordinary_placement::invalidate(
            state,
            crate::ordinary_placement::Frontier::CollectibleEffects { collectible_id: id },
        );
    }
    //52D2FF..52D5A6: one source-ordered event after common authored grants.
    //The existing cache-all request/evaluation below retains its own owners.
    if let Some(proof) = treatment {
        crate::experimental_treatment::apply(state, proof);
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
