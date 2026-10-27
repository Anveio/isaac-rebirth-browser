//! Source Player52a640 active input, the 522250 Use frame and the ported
//! leaves of its jump table, one table row per active (see [`USES`]).
//! Input edges belong to the released System poll; the Player callback runs
//! in both presentations, while its main sprite advances only on full updates.
use crate::{audio::SoundRequest, items, player_animation::Clip, player_items, State};

/// Canonical509E10 target setter. This replaces only the existing setter;
/// canonical D74/D78/D8C remain in PlayerItems. Source text, not built yet.
pub(crate) fn set_charge(state: &mut State, requested: i32) -> Result<(), String> {
    let Some(owner) = state.player_items.as_ref() else {
        return Ok(());
    };
    if owner.active_id == 0 {
        return Ok(());
    }
    let active_id = owner.active_id;
    let config = items::definition(active_id).ok_or("active ItemConfig absent")?;
    let maximum = config.max_charges;
    let charge = requested.min(maximum);
    let rising = charge > owner.active_charge;
    if rising {
        //573290 precedes even the164/289 no-sound exception. The source
        //HUD flash is a separate unresolved presentation, not a lost charge.
        crate::active_charges::note_effect(
            state,
            crate::active_charges::EffectFrontier::HudFlash573290,
        );
        state.charge_bar_flash = 8;
        if !matches!(active_id, 164 | 289) {
            state.audio.admit(
                &mut state.boot_lifecycle.shared,
                state.system_clock.system_counter.wrapping_sub(1),
                SoundRequest {
                    id: if requested >= maximum { 218 } else { 171 },
                    volume: 1.0,
                    delay: 2,
                    looping: false,
                    pitch: 1.0,
                },
            );
        }
    }
    state
        .player_items
        .as_mut()
        .expect("checked Player owner")
        .active_charge = charge;
    //Have63 is pure for this literal ID. If the computed overflow cannot
    //reach its body, no missing unrelated query is needed to complete D78.
    let overflow = requested.wrapping_sub(charge).max(0).min(maximum);
    if overflow <= 0 || matches!(active_id, 164 | 289 | 294 | 338) {
        return Ok(());
    }
    let owner = state.player_items.as_ref().expect("checked Player owner");
    let have_battery = crate::item_queries::AppearanceQueries::new(
        owner,
        state.weapon_cache.primary,
        state.player.entity_source,
        None,
    )
    .have(63);
    match have_battery {
        Some(false) => return Ok(()),
        None => {
            //The already completed D78 prefix survives. Only the reached
            //conditional reserve write and its source request/birth remain
            //unavailable; this is not global query or charge invalidation.
            crate::active_charges::invalidate(
                state,
                &[crate::active_charges::Word::Reserve],
                crate::active_charges::Frontier::Query {
                    source_pc: 0x509e8d,
                    field: "Have63 overflow branch".into(),
                },
            );
            crate::active_charges::note_effect(
                state,
                crate::active_charges::EffectFrontier::BatteryEffect48,
            );
            return Ok(());
        }
        Some(true) => {}
    }
    crate::active_charges::note_effect(
        state,
        crate::active_charges::EffectFrontier::HudFlash573290,
    );
    state.charge_bar_flash = 8;
    state.audio.admit(
        &mut state.boot_lifecycle.shared,
        state.system_clock.system_counter.wrapping_sub(1),
        SoundRequest {
            id: 170,
            volume: 1.0,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
    let owner = state.player_items.as_mut().expect("checked Player owner");
    //509F0D adds the bounded amount to the CURRENT reserve; it does not
    //clamp their sum to maximum. With max4/reserve1, amount4 produces5.
    owner.reserve_charge = owner.reserve_charge.wrapping_add(overflow);
    //509F14 obtains exactly one world seed after the sound and reserve
    //write, then spawns Effect1000/48 (not Pickup90) at the Player.
    crate::familiars::admit_battery(state)?;
    Ok(())
}

///5295C0's reserve transfer and source116 extra-charge branch. Trinket4's
///nested Use44 remains an explicit earlier effect frontier, not a fabricated
///charge value. This draft replaces the existing plain discharge only.
fn discharge(state: &mut State) -> Result<(), String> {
    let owner = state.player_items.as_ref().expect("checked owner");
    if !crate::active_charges::owns(owner, crate::active_charges::Word::ActiveId) {
        return Ok(());
    }
    if owner.active_id == 0 {
        return Ok(());
    }
    let config = items::definition(owner.active_id).ok_or("active ItemConfig absent")?;
    if !crate::active_charges::owns(owner, crate::active_charges::Word::Reserve) {
        crate::active_charges::invalidate(
            state,
            &[crate::active_charges::Word::Charge],
            crate::active_charges::Frontier::Query {
                source_pc: 0x52961b,
                field: "current reserve discharge target".into(),
            },
        );
        crate::active_charges::note_effect(
            state,
            crate::active_charges::EffectFrontier::BatteryEffect48,
        );
        return Ok(());
    }
    let reserve = owner.reserve_charge;
    let mut target = reserve;
    if reserve.wrapping_add(1) < config.max_charges {
        let nine_volt = crate::item_queries::AppearanceQueries::new(
            owner,
            state.weapon_cache.primary,
            state.player.entity_source,
            None,
        )
        .have(116);
        match nine_volt {
            Some(false) => {}
            Some(true) => {
                target = reserve.wrapping_add(1);
                state.audio.admit(
                    &mut state.boot_lifecycle.shared,
                    state.system_clock.system_counter.wrapping_sub(1),
                    SoundRequest {
                        id: 170,
                        volume: 1.0,
                        delay: 2,
                        looping: false,
                        pitch: 1.0,
                    },
                );
                crate::familiars::admit_battery(state)?;
            }
            None => {
                crate::active_charges::invalidate(
                    state,
                    &[crate::active_charges::Word::Charge],
                    crate::active_charges::Frontier::Query {
                        source_pc: 0x52963a,
                        field: "Have116 discharge target".into(),
                    },
                );
                //The unknown reached branch may request170 and initialize
                //Effect48 before509E10. Keep that producer/draw footprint
                //unavailable, then retain the later literal reserve clear.
                crate::active_charges::note_effect(
                    state,
                    crate::active_charges::EffectFrontier::BatteryEffect48,
                );
                let owner = state.player_items.as_mut().expect("checked owner");
                owner.reserve_charge = 0;
                owner.active_flag_3076 = false;
                return Ok(());
            }
        }
    }
    set_charge(state, target)?;
    let owner = state.player_items.as_mut().expect("checked owner");
    owner.reserve_charge = 0;
    owner.active_flag_3076 = false;
    Ok(())
}

/// One ported 522250 jump-table leaf. Every leaf shares the same Use frame:
/// 52A640's book request, 522349 costume admission, 5E4140's TEMP record,
/// the leaf itself, then 5223C0's D6C/Evaluate/3076 tail and 5295C0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Leaf {
    /// 524084->52598B: sound157, then 508230 heals two half-hearts.
    YumHeart,
    /// 523F76: sound34. TEMP34 is the room damage record.
    Belial,
    /// 523FBF: sound8, then any Mom45/Mom's Heart78 dies and Satan84 kills
    /// the Player. TEMP33 is the flight record.
    Bible,
    /// 523E5A: sound33 and 40 damage to every non-friendly NPC.
    Necronomicon,
    /// 5243AB->525C48: 5083B0 two soul half-hearts, 585820 and
    /// Game+10EAE0|=0x80.
    Revelations,
    /// 522A34: 508440 two black half-hearts and no item animation.
    SatanicBible,
    /// 522F61: the item animation only; TEMP58 (cooldown300) is the shield.
    BookOfShadows,
    /// 523481: the landing search, Entity+440, collision classes0 and the
    /// Jump clip. TEMP282 (cooldown14) moves the Player in 5322BA.
    HowToJump,
    /// 5235F5: 526DB0 UseCard on a held card, then the Pickup clip.
    BlankCard,
    /// 523789: max-2 (red clamped), sound157 and 5083B0 six soul halves.
    GuppysPaw,
    /// 523690->525850: 508640 one Eternal half, then the Pickup clip.
    PrayerCard,
    /// 5226DC: with more than three soul halves, each black half among the
    /// top four forces the Black-heart Necronomicon, then soul-4, 508540(2)
    /// and 508230(2).
    Converter,
    /// 524407: the item85 RNG draws a Card (5817A0 suits, no runes) for
    /// 50AF00, or a Pill (581950) for 50AE00 with Little Baggy252.
    DeckOfCards,
    /// 5241D4: the item102 RNG draws a Pill, or a Card with Starter Deck251.
    MomsBottleOfPills,
    /// 522A4E: every vulnerable non-boss room NPC except Poop245, Chub28
    /// and Mom's Hand213 is removed and replaced by a Poop245 with the
    /// shared seed and a brown colorize; then UseItem.
    Flush,
    /// 5253EE: 582340 clears the selected bytes, then every live
    /// collectible pedestal (455680 5/100) with an item is rerolled in place
    /// by Morph500A10(5,100,0,keepPrice1) and its ABC cleared.
    D6,
    /// 523AFA: Game554490 at the Player with no source, line check off,
    /// DamageFlags4, damage40 and radius60, then Spawn1000.1 with the shared
    /// seed and its immediate Update. It returns to 5223C0 with no UseItem.
    Kamikaze,
    /// 523B90: every 444810 enemy gets AddFreeze444970 for the ItemConfig39
    /// +48 duration; then UseItem.
    MomsBra,
    /// 5240D8 (variant0 only): every 444810 enemy gets AddFear4450D0 for the
    /// same duration; then UseItem.
    MomsPad,
    /// 525BBF (variant0 only): Random(9), FindFreePosition5C7F30 near the
    /// Player, the trinket22 check, then one null-spawner pickup from the
    /// 6705A8 table with the shared seed; then UseItem.
    BookOfSin,
    /// 5257C2 (variant0 only): map reveal585C60, then Random(2) picks a
    /// random card or (without trinket22) a soul heart at a free spot near
    /// the Player; then the Pickup clip.
    CrystalBall,
    /// 5238AB (variant0 only): outside shops, every live pickup (455680 5)
    /// other than a pedestal, trophy, bed or big chest is Morphed in place
    /// to a variant the shared Random draws; then UseItem.
    D20,
    /// 522F87 (variant0 only): every live non-boss room enemy (444800,
    /// not pending-dead 444950, not 17/33/28/68) is replaced by a random
    /// enemy from the 670AC0 table (670B80 for a flyer) at its position and
    /// velocity, with its health fraction; then UseItem. The Runtime owns
    /// the replacement loop because each new enemy's immediate Update draws
    /// from the shared Random between rolls (`d10_targets`, `d10_roll`).
    D10,
}

/// A ported active: its 52A640 GiantBook5805C0 id (requested with delay3
/// before Use) and its 522250 leaf. Ids absent here keep the unported
/// active-effect frontier. docs/active-items.md tracks every Rebirth active.
pub(crate) struct ActiveUse {
    pub id: u32,
    pub book: Option<u32>,
    pub leaf: Leaf,
}

pub(crate) const USES: &[ActiveUse] = &[
    ActiveUse {
        id: 33,
        book: Some(10),
        leaf: Leaf::Bible,
    },
    ActiveUse {
        id: 34,
        book: Some(34),
        leaf: Leaf::Belial,
    },
    ActiveUse {
        id: 35,
        book: Some(1),
        leaf: Leaf::Necronomicon,
    },
    ActiveUse {
        id: 45,
        book: None,
        leaf: Leaf::YumHeart,
    },
    ActiveUse {
        id: 58,
        book: None,
        leaf: Leaf::BookOfShadows,
    },
    ActiveUse {
        id: 78,
        book: Some(11),
        leaf: Leaf::Revelations,
    },
    ActiveUse {
        id: 282,
        book: None,
        leaf: Leaf::HowToJump,
    },
    ActiveUse {
        id: 286,
        book: None,
        leaf: Leaf::BlankCard,
    },
    ActiveUse {
        id: 292,
        book: Some(15),
        leaf: Leaf::SatanicBible,
    },
    ActiveUse {
        id: 133,
        book: None,
        leaf: Leaf::GuppysPaw,
    },
    ActiveUse {
        id: 146,
        book: None,
        leaf: Leaf::PrayerCard,
    },
    ActiveUse {
        id: 296,
        book: None,
        leaf: Leaf::Converter,
    },
    ActiveUse {
        id: 85,
        book: None,
        leaf: Leaf::DeckOfCards,
    },
    ActiveUse {
        id: 102,
        book: None,
        leaf: Leaf::MomsBottleOfPills,
    },
    ActiveUse {
        id: 291,
        book: None,
        leaf: Leaf::Flush,
    },
    ActiveUse {
        id: 105,
        book: None,
        leaf: Leaf::D6,
    },
    ActiveUse {
        id: 40,
        book: None,
        leaf: Leaf::Kamikaze,
    },
    ActiveUse {
        id: 39,
        book: Some(7),
        leaf: Leaf::MomsBra,
    },
    ActiveUse {
        id: 41,
        book: Some(8),
        leaf: Leaf::MomsPad,
    },
    ActiveUse {
        id: 97,
        book: None,
        leaf: Leaf::BookOfSin,
    },
    ActiveUse {
        id: 158,
        book: None,
        leaf: Leaf::CrystalBall,
    },
    ActiveUse {
        id: 166,
        book: None,
        leaf: Leaf::D20,
    },
    ActiveUse {
        id: 285,
        book: Some(18),
        leaf: Leaf::D10,
    },
];

pub(crate) fn ported(id: u32) -> Option<&'static ActiveUse> {
    USES.iter().find(|row| row.id == id)
}

/// Active0 and ported leaves own their whole Use footprint; any other held
/// active reaches an unported effect when its request is admitted.
fn owned_request(owner: &player_items::PlayerItems) -> bool {
    owner.active_id == 0 || ported(owner.active_id).is_some()
}

/// The unported effect of an admitted Use request: its writers may change
/// counts, CF8, held trinkets, retained C08, placements and stat inputs.
/// This is an explicit unknown frontier, not invocation credit.
fn forget_unported_effect(state: &mut State) {
    crate::ordinary_placement::invalidate(
        state,
        crate::ordinary_placement::Frontier::ActiveEffects,
    );
    if let Some(owner) = state.player_items.as_mut() {
        if let Some(history) = &mut owner.query_history {
            history.forget_unowned_effect();
        }
        crate::pill_effect_history::forget(owner);
        owner.damage_inputs = None;
        owner.range_inputs = None;
        owner.move_speed_inputs = None;
        owner.fire_delay_inputs = None;
        owner.shot_speed_inputs = None;
    }
}

fn sound(state: &mut State, id: u32) {
    state.audio.admit(
        &mut state.boot_lifecycle.shared,
        state.system_clock.system_counter.wrapping_sub(1),
        SoundRequest {
            id,
            volume: 1.0,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
}

/// 50AC90: copy the item Sprite+7E0 and play its PlayerPickupSparkle, then
/// 50A5C0 force-plays UseItem (or Pickup for Blank Card). 522385 clears the
/// show flag for Player variants other than0.
fn play_item_animation(state: &mut State, id: u32, variant0: bool, pickup_clip: bool) {
    if !variant0 {
        return;
    }
    if let Some(owner) = state.player_items.as_mut() {
        owner.pickup_presentation = Some(player_items::PickupPresentation {
            item_id: id,
            animation_frame: 0.0,
            animation_playing: true,
            pocket: None,
            regular: None,
        });
    }
    if pickup_clip {
        crate::player_animation::force_pickup(state);
    } else {
        crate::player_animation::force_play(state, Clip::UseItem);
    }
}

/// A Use whose leaf the Runtime finishes: the D10 replaces enemies and runs
/// each new enemy's immediate Update before `finish_runtime_leaf`.
pub(crate) struct RuntimeLeaf {
    pub id: u32,
    pub variant0: bool,
}

pub(crate) fn advance(
    state: &mut State,
    controls_enabled: bool,
) -> Result<Option<RuntimeLeaf>, String> {
    //Only empty active0 and the ported leaves exclude the D30 reset/random
    //and D38 writers in522250. A requested unported active loses this input
    //proof before dispatch; no claim of actual callback execution is made.
    if state.public_actions.active_triggered
        && controls_enabled
        && state
            .player_items
            .as_ref()
            .is_none_or(|owner| !owned_request(owner))
    {
        forget_unported_effect(state);
    }
    let Some(owner) = state.player_items.as_ref() else {
        return Ok(None);
    };
    //52a640 reads held/triggered action9 even on the Entity+318 word bit4000
    // fast path (the same little-endian byte+319 bit40 predicate). The
    // ported leaves have no held-use or frame-recharge leaf; an actual new
    // edge is required. Its guards are7d4/7d8, not the bf4 main-animation guard.
    let Some(row) = ported(owner.active_id) else {
        return Ok(None);
    };
    if !state.public_actions.active_triggered
        || !controls_enabled
        || state.player.input_cooldown > 0
        || owner.active_state_d80 != 0
        || owner.active_state_d84 != 0
    {
        return Ok(None);
    }
    let id = row.id;
    let config = items::definition(id).ok_or("ported active ItemConfig absent")?;
    if owner.active_charge != config.max_charges {
        return Ok(None);
    }
    //52AA00..1A: a live TEMP record of a zero-charge cooldown item refuses
    //the Use (How to Jump while it is still in the air).
    if owner.active_charge == 0
        && config.cooldown != 0
        && owner
            .temporary_collectibles
            .iter()
            .any(|record| record.item_id == id)
    {
        return Ok(None);
    }
    //522349 costume admission is empty for this actual config.5e4140 owns
    // a temporary record before the dispatch leaf; a zero authored cooldown
    // is persistent, not an immediately expired effect.
    //529600 can dispatch another active through positiveTrinket4.
    //The actual known-empty held/queue query closes that producer;
    //unknown/positive trinket history does not create a CF8 proof.
    let query_stores_preserved = state
        .player
        .entity_source
        .is_some_and(|source| source.entity_type == 1 && source.variant == 0)
        && crate::item_queries::AppearanceQueries::new(
            owner,
            state.weapon_cache.primary,
            state.player.entity_source,
            None,
        )
        .has_trinket(4)
            == Some(false);
    if let Some(book) = row.book {
        //52AA5D..: 5805c0(book, 3) precedes the Use call.
        if !state.information_animation.request_authored(book, 3) {
            return Err("ported active GiantBook definition absent".into());
        }
    }
    let owner = state.player_items.as_mut().expect("checked owner");
    if !query_stores_preserved {
        //The admitted529600 suffix can dispatch another active through
        //Trinket4. Its unclosed writer may change canonical Pill words,
        //independently of whether query history is still available.
        crate::pill_effect_history::forget(owner);
        crate::active_charges::invalidate_owner(
            owner,
            &[
                crate::active_charges::Word::ActiveId,
                crate::active_charges::Word::Charge,
                crate::active_charges::Word::Subordinate,
                crate::active_charges::Word::Reserve,
            ],
            crate::active_charges::Frontier::ActiveUse {
                active_id: id,
                source_pc: 0x529600,
            },
        );
        if let Some(history) = &mut owner.query_history {
            history.forget_unowned_effect();
        }
    }
    if matches!(id, 33 | 34 | 58) {
        //522349 registers the actual costume config (33/34/58); its layer
        //and pose owners are unported, so the registration list is unknown.
        owner.costume_registrations = None;
    }
    owner.add_temporary_collectible(id)?;
    //Entity+14 selects the variant0 leaf bodies (and 522385's show flag).
    //A legacy unowned source keeps the ordinary Isaac reading.
    let variant0 = state
        .player
        .entity_source
        .is_none_or(|source| source.variant == 0);
    if row.leaf == Leaf::D10 {
        return Ok(Some(RuntimeLeaf { id, variant0 }));
    }
    use_leaf(state, row.leaf, id, variant0)?;
    finish_use(state, id)?;
    Ok(None)
}

/// The D10's 524C48 UseItem and the shared 5223C0 tail, after the Runtime
/// replaced the enemies.
pub(crate) fn finish_runtime_leaf(state: &mut State, leaf: RuntimeLeaf) -> Result<(), String> {
    play_item_animation(state, leaf.id, leaf.variant0, false);
    finish_use(state, leaf.id)
}

/// 5223C0: D6C |= the config cache flags, Evaluate511F10, the 3076 flag
/// and 5295C0's discharge.
fn finish_use(state: &mut State, id: u32) -> Result<(), String> {
    let config = items::definition(id).ok_or("ported active ItemConfig absent")?;
    let owner = state.player_items.as_mut().expect("checked owner");
    owner.unevaluated_cache_flags |= config.cache_flags;
    let evaluate = owner.unevaluated_cache_flags != 0;
    owner.active_flag_3076 = true;
    if evaluate {
        //5223C8..DD: Evaluate511F10 runs for the nonzero D6C word.
        crate::player_stats::evaluate_cache(state)?;
    }
    let owner = state.player_items.as_ref().expect("checked owner");
    if matches!(owner.active_state_d80, 0 | 6) {
        discharge(state)?;
    }
    Ok(())
}

fn use_leaf(state: &mut State, leaf: Leaf, id: u32, variant0: bool) -> Result<(), String> {
    match leaf {
        Leaf::YumHeart => {
            //524084->52598b requests157 before508230 heals two native
            //half-hearts. This succeeds even when red health is already full.
            if variant0 {
                sound(state, 157);
                player_items::add_red_hearts(&mut state.player, 2);
            }
            play_item_animation(state, id, variant0, false);
        }
        Leaf::Belial => {
            sound(state, 34);
            play_item_animation(state, id, variant0, false);
        }
        Leaf::Bible => {
            sound(state, 8);
            //523FE2..: the first Mom45/78 is killed (and 5BD5E0(27) marks it)
            //or Satan84 kills the Player. Neither callback is ported here.
            if state
                .room
                .npcs
                .iter()
                .any(|npc| !npc.removed && matches!(npc.kind, 45 | 78 | 84))
            {
                forget_unported_effect(state);
                crate::active_charges::note_effect(
                    state,
                    crate::active_charges::EffectFrontier::UseLeaf {
                        active_id: id,
                        source_pc: 0x524055,
                    },
                );
            }
            play_item_animation(state, id, variant0, false);
        }
        Leaf::Necronomicon => {
            if variant0 {
                sound(state, 33);
                crate::health_damage::damage_room_npcs(state, 40.0)?;
            }
            play_item_animation(state, id, variant0, false);
        }
        Leaf::Revelations => {
            if variant0 {
                player_items::add_soul_hearts(&mut state.player, 2);
                //585820 walks the level rooms and 10EAE0 bit80 is a Level
                //flag; neither owner is ported.
                crate::active_charges::note_effect(
                    state,
                    crate::active_charges::EffectFrontier::UseLeaf {
                        active_id: id,
                        source_pc: 0x525c5c,
                    },
                );
            }
            play_item_animation(state, id, variant0, false);
        }
        Leaf::SatanicBible => {
            if variant0 {
                player_items::add_black_hearts(&mut state.player, 2);
            }
        }
        Leaf::BookOfShadows => play_item_animation(state, id, variant0, false),
        Leaf::HowToJump => {
            if variant0 {
                start_jump(state);
            }
        }
        Leaf::BlankCard => {
            let held_card = state
                .player_items
                .as_ref()
                .and_then(|owner| owner.held_pockets.as_ref())
                .map(|held| held.slots[0])
                .map(|slot| slot.card && slot.id != 0);
            let held_slot = state
                .player_items
                .as_ref()
                .and_then(|owner| owner.held_pockets.as_ref())
                .map(|held| held.slots[0]);
            match held_card {
                Some(false) => {}
                //523611 UseCard526DB0 with the held card; the card stays held.
                Some(true)
                    if variant0
                        && crate::pockets::use_card_effect(
                            state,
                            held_slot.expect("checked held slot"),
                        )? => {}
                //Other cards' effects are not ported, so the reached call
                //(or an unknown held slot) is a frontier.
                Some(true) | None if variant0 => {
                    forget_unported_effect(state);
                    crate::active_charges::invalidate(
                        state,
                        &[
                            crate::active_charges::Word::ActiveId,
                            crate::active_charges::Word::Charge,
                            crate::active_charges::Word::Subordinate,
                            crate::active_charges::Word::Reserve,
                        ],
                        crate::active_charges::Frontier::ActiveUse {
                            active_id: id,
                            source_pc: 0x523611,
                        },
                    );
                }
                _ => {}
            }
            play_item_animation(state, id, variant0, true);
        }
        Leaf::GuppysPaw => {
            if variant0 && state.player.max_hp > 1 {
                sound(state, 157);
                let player = &mut state.player;
                player.max_hp -= 2;
                player.hp = player.hp.min(player.max_hp);
                player_items::add_soul_hearts(player, 6);
                play_item_animation(state, id, variant0, false);
            }
        }
        Leaf::PrayerCard => {
            if variant0 {
                //508640 requests information13 only when a pair completes.
                if state.player.eternal_hearts.wrapping_add(1) > 1 {
                    state.information_animation.request_eternal();
                }
                player_items::add_eternal_hearts(&mut state.player, 1);
            }
            play_item_animation(state, id, variant0, true);
        }
        Leaf::DeckOfCards | Leaf::MomsBottleOfPills if variant0 => {
            let (modifier, card_by_default) = if leaf == Leaf::DeckOfCards {
                (252, true)
            } else {
                (251, false)
            };
            let owner = state.player_items.as_ref().expect("checked owner");
            let have = crate::item_queries::AppearanceQueries::new(
                owner,
                state.weapon_cache.primary,
                state.player.entity_source,
                None,
            )
            .have(modifier);
            let granted = match have {
                None => None,
                Some(have) => grant_pocket(state, id, card_by_default != have)?,
            };
            if granted.is_none() {
                unowned_leaf(
                    state,
                    id,
                    if leaf == Leaf::DeckOfCards {
                        0x524417
                    } else {
                        0x5241e4
                    },
                );
            }
        }
        Leaf::DeckOfCards | Leaf::MomsBottleOfPills => {}
        Leaf::Kamikaze => {
            if variant0 {
                let position = crate::bombs::player_world(state);
                crate::bombs::explode_unchecked(
                    state,
                    position,
                    60.0,
                    40.0,
                    crate::combat::DamageSource {
                        id: 0,
                        entity_type: 0,
                    },
                )?;
                //The blast only changes Player velocity; Spawn reads +448.
                let seed = state.boot_lifecycle.shared.next_u32();
                let effect = crate::explosion_effects::admit_explosion(
                    state,
                    seed,
                    position,
                    crate::attack_color::AttackColor::WHITE,
                )?;
                crate::explosion_effects::advance(state, crate::phase::Phase::Update, effect)?;
            }
        }
        Leaf::MomsBra | Leaf::MomsPad => {
            //Neither freeze444970 nor fear4450D0 is ported, so a room with
            //an eligible enemy records a frontier; otherwise only the
            //UseItem tail runs.
            let walks = variant0 || matches!(leaf, Leaf::MomsBra);
            if walks && vulnerable_npc(state, &[]) {
                let pc = if matches!(leaf, Leaf::MomsBra) {
                    0x523bfe
                } else {
                    0x52414e
                };
                unowned_leaf(state, id, pc);
            }
            play_item_animation(state, id, variant0, false);
        }
        Leaf::BookOfSin => {
            if variant0 {
                book_of_sin(state, id)?;
            }
            play_item_animation(state, id, variant0, false);
        }
        Leaf::CrystalBall => {
            if variant0 {
                crystal_ball(state, id)?;
            }
            play_item_animation(state, id, variant0, true);
        }
        Leaf::D6 => {
            if variant0 {
                reroll_pedestals(state, id)?;
            }
            play_item_animation(state, id, variant0, false);
        }
        Leaf::D20 => {
            if variant0 {
                reroll_pickups(state, id)?;
            }
            play_item_animation(state, id, variant0, false);
        }
        Leaf::D10 => return Err("the D10 leaf runs through the Runtime".into()),
        Leaf::Flush => {
            //522A80 walks Room+1638, skipping Poop245 and Chub28 before
            //444810 and Mom's Hand213 after it. Fire33 sets bit10 in its
            //initializer, so a room of fires spawns nothing. Any remaining
            //candidate would become Poop245, whose factory and AI48CFE0 are
            //not ported, so it stays a frontier.
            let candidate = vulnerable_npc(state, &[245, 28, 213]);
            if variant0 && candidate {
                unowned_leaf(state, id, 0x522b42);
            }
            play_item_animation(state, id, variant0, false);
        }
        Leaf::Converter => {
            if variant0 && state.player.soul_hp > 3 {
                let mut index = state.player.soul_hp;
                while state.player.soul_hp - 3 <= index {
                    //508290 IsBlackHeart then 508260 clears its mask bit.
                    let player = &state.player;
                    if index > 0
                        && index <= player.soul_hp
                        && index & 1 == 1
                        && player.black_heart_mask >> ((index - 1) >> 1) & 1 == 1
                    {
                        state.player.black_heart_mask &= !(1_u32 << ((index >> 1) & 31));
                        crate::health_damage::use_black_heart(state)?;
                    }
                    index -= 1;
                }
                state.player.soul_hp -= 4;
                player_items::add_max_hearts(&mut state.player, 2);
                player_items::add_red_hearts(&mut state.player, 2);
                play_item_animation(state, id, variant0, false);
            }
        }
    }
    Ok(())
}

/// The item RNG draw and AddCard/AddPill of a pocket-granting active. None
/// when the RNG history, achievements or held record is unknown.
fn grant_pocket(state: &mut State, id: u32, card: bool) -> Result<Option<()>, String> {
    let Ok(seed) = crate::pickup_world_grants::next_item_seed(state, id) else {
        return Ok(None);
    };
    let pocket = if card {
        let Some(profile) = state
            .profiles
            .get(usize::from(state.profile).wrapping_sub(1))
        else {
            return Ok(None);
        };
        crate::pockets::PocketSlot {
            id: crate::pickups::random_card(seed, &profile.achievements, true, false)?,
            card: true,
        }
    } else {
        //581950 is seed%9+1 and owns no RNG.
        crate::pockets::PocketSlot {
            id: seed % 9 + 1,
            card: false,
        }
    };
    crate::pockets::grant_from_active(state, pocket)
}

/// A reached leaf operator whose inputs are unknown: its writers are an
/// explicit frontier rather than an assumed no-op.
/// HasTrinket508FE0 through the Player's appearance queries; None when the
/// held-trinket history is not owned.
fn has_trinket(state: &State, trinket: u32) -> Option<bool> {
    let owner = state.player_items.as_ref()?;
    crate::item_queries::AppearanceQueries::new(
        owner,
        state.weapon_cache.primary,
        state.player.entity_source,
        None,
    )
    .has_trinket(trinket)
}

/// FindFreePosition5C7F30(Player+448, 40, 0). None (after recording a
/// frontier) when the room grid, path costs or occupied positions are not
/// owned.
fn free_position(
    state: &mut State,
    id: u32,
    source_pc: u32,
) -> Result<Option<crate::motion::WorldVector>, String> {
    let (Some(grid), Some(costs)) = (state.room.grid.clone(), state.room.path_costs.clone()) else {
        unowned_leaf(state, id, source_pc);
        return Ok(None);
    };
    let Ok(occupied) = crate::ordinary_placement::positions(state) else {
        unowned_leaf(state, id, source_pc);
        return Ok(None);
    };
    let grid = crate::grid::Grid::new(grid)?;
    let anchor = crate::bombs::player_world(state);
    crate::roundworm::find_free_position(
        anchor,
        40.0,
        &mut state.boot_lifecycle.shared,
        &grid,
        &costs,
        &occupied,
    )
    .map(Some)
}

/// Game.Spawn550A40(5, variant, position, zero velocity, null spawner,
/// subtype, seed) and the pickup's Init. Records a frontier instead when an
/// Init input is not owned.
fn spawn_free_pickup(
    state: &mut State,
    id: u32,
    source_pc: u32,
    variant: u32,
    subtype: u32,
    seed: u32,
    position: crate::motion::WorldVector,
) -> Result<(), String> {
    let handle = state.next_pickup;
    let pickup = if matches!(variant, 70 | 300) {
        crate::Pickup::from_pocket_parts(
            handle,
            variant,
            subtype,
            seed,
            position,
            true,
            crate::pickups::PocketSubtypeInputs::from_state(state),
        )?
    } else {
        let selector = crate::room_spawns::SpawnSelector {
            group_index: 0,
            grid_index: 0,
            category: crate::room_spawns::SpawnCategory::Entity,
            entity_type: 5,
            variant: variant as u16,
            subtype: subtype as u16,
            position,
            group_seed: 0,
            spawn_seed: seed,
        };
        crate::Pickup::from_source_selector(handle, &selector, true, state)?
    };
    let Some(mut pickup) = pickup else {
        unowned_leaf(state, id, source_pc);
        return Ok(());
    };
    pickup.birth_game_update = state.game_update_count;
    //Base441EE8 gives a null-spawner birth zero spawner words.
    pickup.source_spawner = Some(crate::pickups::SourceSpawner {
        entity_type: 0,
        variant: 0,
    });
    pickup.morph_links = Some(crate::pickups::MorphLinks {
        parent498: None,
        link4a0: None,
        link4a8: None,
    });
    state.next_pickup = handle
        .checked_add(1)
        .ok_or("active pickup identity exhausted")?;
    state.room.pickups.push(pickup);
    let actor = crate::actors::ActorRef {
        kind: crate::actors::ActorKind::Pickup,
        id: handle,
    };
    state.room.authored_actor_order.push(actor);
    state.actor_scheduler.admit(actor)
}

/// 525BBF..: Random(9) is drawn before the free spot; Daemon's Tail22 with
/// its 5C2DD0 roll (525CE5) is not ported. The table 6705A8 maps the roll
/// to red, soul and black hearts, a bomb, a key, a random pill, card, coin
/// and a Lil' Battery.
fn book_of_sin(state: &mut State, id: u32) -> Result<(), String> {
    let roll = state.boot_lifecycle.shared.bounded(9);
    let Some(position) = free_position(state, id, 0x525bea)? else {
        return Ok(());
    };
    if has_trinket(state, 22) != Some(false) {
        unowned_leaf(state, id, 0x525c00);
        return Ok(());
    }
    let (variant, subtype) = match roll {
        0 => (10, 1),
        1 => (10, 3),
        2 => (10, 6),
        3 => (40, 1),
        4 => (30, 1),
        5 => (70, 0),
        6 => (300, 0),
        7 => (20, 0),
        _ => (90, 0),
    };
    let seed = state.boot_lifecycle.shared.next_u32();
    spawn_free_pickup(state, id, 0x5264fd, variant, subtype, seed, position)
}

/// 5257C2: 585C60 reveals every room but the type8 super secret room
/// (display flags |= 5) and sets Level+81B0 bit 0x400; 5B0F40 redraws the
/// minimap. Random(2) == 0 then spawns a random card (seed before the free
/// spot); otherwise 525F90 checks trinket22 and spawns a soul heart.
fn crystal_ball(state: &mut State, id: u32) -> Result<(), String> {
    for index in 0..state.floor.descriptors.len() {
        if state.floor.descriptors[index].template.room_type != 8 {
            state.room_display_flags[index] |= 5;
        }
    }
    state.level_progress.flags |= 0x400;
    crate::rebuild_minimap_cells(state);
    let (variant, subtype, pc) = if state.boot_lifecycle.shared.bounded(2) == 0 {
        (300, 0, 0x525841)
    } else {
        if has_trinket(state, 22) != Some(false) {
            unowned_leaf(state, id, 0x525f98);
            return Ok(());
        }
        (10, 3, 0x526006)
    };
    let seed = state.boot_lifecycle.shared.next_u32();
    let Some(position) = free_position(state, id, pc)? else {
        return Ok(());
    };
    spawn_free_pickup(state, id, pc, variant, subtype, seed, position)
}

/// Whether Room+1638 holds an NPC that passes 444810 (types10..999,
/// collision class, 494, dead, Entity318 bit10), apart from `skip` types.
/// An NPC whose Entity318 owner is unknown counts, so the caller records a
/// frontier rather than assuming it was skipped.
fn vulnerable_npc(state: &State, skip: &[u32]) -> bool {
    state.room.npcs.iter().any(|npc| {
        !npc.removed
            && !skip.contains(&npc.kind)
            && crate::target_eligibility::eligible_target(
                crate::target_eligibility::TargetEligibility {
                    kind: npc.kind,
                    collision_class: npc.entity_collision_class,
                    raw_494: npc.raw_494,
                    dead: npc.dead,
                    entity_flags: crate::entities::family(npc.kind)
                        .map(|family| (family.entity_flags)(npc)),
                },
            )
            .unwrap_or(true)
    })
}

/// 525404..5254B1. Only a free pedestal (ABD 0, no price) is ported: the
/// ABD-set test reads AC8, and keepPrice with a nonzero AC0 redraws the
/// price (500C90); both stay a `UseLeaf` frontier, checked before any
/// pedestal changes.
fn reroll_pedestals(state: &mut State, id: u32) -> Result<(), String> {
    let Some(room_type) = state
        .room
        .descriptor
        .as_ref()
        .map(|descriptor| descriptor.template.room_type)
    else {
        unowned_leaf(state, id, 0x525404);
        return Ok(());
    };
    let Some(pools) = state.item_pools.as_mut() else {
        unowned_leaf(state, id, 0x525404);
        return Ok(());
    };
    pools.reset_selected();
    //31F (dead) pedestals are skipped; an empty pedestal (subtype 0) too.
    let targets: Vec<usize> = (0..state.room.pickups.len())
        .filter(|index| {
            let pickup = &state.room.pickups[*index];
            !pickup.removed && pickup.variant == 100 && pickup.subtype != 0
        })
        .collect();
    if targets.iter().any(|index| {
        let pickup = &state.room.pickups[*index];
        pickup.save_byte_abd != Some(0)
            || pickup.shop_price.is_some()
            || pickup.spawn_seed == 0
            || pickup
                .morph_links
                .as_ref()
                .is_some_and(|links| !links.known_empty())
    }) {
        unowned_leaf(state, id, 0x52549b);
        return Ok(());
    }
    //5C51E0: backdrop 0x17 selects pool14 by room type; 581310 then maps it.
    let pool_room = if state.room.appearance.backdrop_id == 0x17 {
        14
    } else {
        room_type
    };
    for index in targets {
        //500A2A..56 advances the old seed once; Init 4FF6B8 then draws the
        //new seed's local RNG35 for 5C51E0's 581590(pool, decrease).
        let seed = items::Rng::new(state.room.pickups[index].spawn_seed, [1, 21, 20]).raw();
        let pools = state.item_pools.as_mut().expect("checked pools");
        let pool = pools.room_pool(pool_room, &mut state.boot_lifecycle.shared);
        let subtype = pools.collectible(pool, true, crate::room_spawns::next35(seed))?;
        let pickup = &mut state.room.pickups[index];
        pickup.morph_rerolled(
            seed,
            subtype,
            state.game_update_count,
            &mut state.actor_scheduler,
        )?;
        //52545E clears ABC after the Morph restored it.
        pickup.touched = false;
    }
    Ok(())
}

/// 5238AB..523A0F, one draw sequence on the shared Random (42E8A0, MT %).
/// `None` is the trinket22 heart branch, which needs the Player's RNG at
/// +2B34.
fn d20_variant(shared: &mut crate::mt::Mt19937, trinket22: Option<bool>) -> Option<u32> {
    let mut variant = (shared.bounded(6) * 5 + 5) * 2;
    if variant == 60 {
        //5247D2: Random(2) == 0 turns a locked chest into a heart.
        if shared.bounded(2) == 0 {
            variant = 10;
        }
    } else if variant == 50 && shared.bounded(3) == 0 {
        //5247EB: Random(3) == 0 turns a chest into a coin.
        variant = 20;
    }
    if shared.bounded(10) == 0 {
        variant = if shared.bounded(2) == 0 { 30 } else { 40 };
    }
    if shared.bounded(15) == 0 {
        variant = 300;
    }
    if shared.bounded(15) == 0 {
        variant = 70;
    }
    if shared.bounded(20) == 0 {
        return Some(350);
    }
    //5247A3: with trinket22 a heart rerolls unless the Player RNG's
    //RandomInt(5) is 0. Without it the heart stands.
    if variant == 10 && trinket22 != Some(false) {
        return None;
    }
    Some(variant)
}

/// 670AC0: the D10's 45 ground replacements.
const D10_ENEMIES: [u32; 45] = [
    10, 11, 12, 13, 14, 15, 16, 18, 21, 22, 23, 24, 25, 26, 27, 29, 30, 31, 32, 34, 35, 38, 39, 40,
    41, 55, 56, 57, 58, 59, 60, 61, 77, 80, 85, 86, 87, 88, 91, 94, 205, 206, 207, 208, 217,
];
/// 670B80: the 15 replacements of an enemy that flies (444860).
const D10_FLYERS: [u32; 15] = [13, 14, 18, 25, 26, 29, 34, 38, 55, 56, 58, 59, 80, 86, 91];

/// One D10 replacement: Spawn550A40(type, 0, old position, old velocity,
/// no spawner, subtype0, seed), then its health fraction.
pub(crate) struct D10Roll {
    pub selector: crate::room_spawns::SpawnSelector,
    pub velocity: crate::motion::WorldVector,
    pub fraction: f32,
}

/// 522F8F..52309F, in Room+1638 order: 444800 (types10..999), not
/// 444950 (pending damage378 above HP484), not Poop17, Fire33, Chub28 or
/// 68, not dead (31F), and 49B890: not Gemini's cord 79.20, 228.10,
/// 216.10, a parented 260.10, or a boss (+103D). None (after recording a
/// frontier) when a candidate's damage owner is not finite.
pub(crate) fn d10_targets(state: &mut State, id: u32) -> Option<Vec<u32>> {
    let mut targets = Vec::new();
    for npc in &state.room.npcs {
        if npc.removed || !(10..=999).contains(&npc.kind) {
            continue;
        }
        let pending = npc.damage.pending_damage();
        if !pending.is_finite() || !npc.hp.is_finite() {
            unowned_leaf(state, id, 0x523003);
            return None;
        }
        if pending > npc.hp || matches!(npc.kind, 17 | 33 | 28 | 68) || npc.dead {
            continue;
        }
        let excluded = matches!((npc.kind, npc.variant), (79, 20) | (228, 10) | (216, 10))
            || (npc.kind, npc.variant) == (260, 10) && npc.parent.is_some();
        if excluded || npc.boss {
            continue;
        }
        targets.push(npc.id);
    }
    Some(targets)
}

/// 524DC0..524E76 for the enemy at `index`: the clamped health fraction
/// (1 without max health), Random(45) from 670AC0, Random(15) from 670B80
/// for a flyer (grid collision class 0..3), then the spawn seed. None
/// (after recording a frontier) when the drawn type has no ported family.
pub(crate) fn d10_roll(
    state: &mut State,
    id: u32,
    index: usize,
) -> Result<Option<D10Roll>, String> {
    let npc = &state.room.npcs[index];
    let fraction = if npc.max_hp > 0.0 {
        //42EFC0: maxss(0, f) then minss(1, ..), with SSE operand order.
        let value = npc.hp / npc.max_hp;
        let low = if 0.0 > value { 0.0 } else { value };
        if 1.0 < low {
            1.0
        } else {
            low
        }
    } else {
        1.0
    };
    let (position, velocity, flyer) = (npc.position, npc.velocity, npc.grid_collision_class <= 3);
    let shared = &mut state.boot_lifecycle.shared;
    let mut kind = D10_ENEMIES[shared.bounded(45) as usize];
    if flyer {
        kind = D10_FLYERS[shared.bounded(15) as usize];
    }
    let seed = shared.next_u32();
    let selector = crate::room_spawns::SpawnSelector {
        group_index: 0,
        grid_index: 0,
        category: crate::room_spawns::SpawnCategory::Entity,
        entity_type: kind as u16,
        variant: 0,
        subtype: 0,
        position,
        group_seed: seed,
        spawn_seed: seed,
    };
    let context = crate::entities::SpawnContext {
        stage: state.stage,
        game_update_count: state.game_update_count,
    };
    if crate::entities::spawn(0, &selector, &context)?.is_none() {
        unowned_leaf(state, id, 0x524e76);
        return Ok(None);
    }
    Ok(Some(D10Roll {
        selector,
        velocity,
        fraction,
    }))
}

/// The D20 (5238AB). Every outcome is drawn on a copy of the shared MT and
/// each new pickup built first, so an unported branch records a frontier
/// before anything changes.
fn reroll_pickups(state: &mut State, id: u32) -> Result<(), String> {
    //Room+10 type2 (a shop) first restocks through 5CDC70, not ported.
    if state
        .room
        .descriptor
        .as_ref()
        .is_none_or(|descriptor| descriptor.template.room_type == 2)
    {
        unowned_leaf(state, id, 0x5238c5);
        return Ok(());
    }
    let mut targets = Vec::new();
    for (index, pickup) in state.room.pickups.iter().enumerate() {
        if pickup.removed {
            continue;
        }
        //31F (dead) skips the entry; a pickup mid-collection is not modeled
        //by that word, so it stays unadmitted.
        if pickup.collected {
            unowned_leaf(state, id, 0x523949);
            return Ok(());
        }
        if pickup.variant == 100 {
            //52487C reprices a priced pedestal; a free one is skipped.
            if pickup.shop_price.is_some() {
                unowned_leaf(state, id, 0x52488c);
                return Ok(());
            }
            continue;
        }
        //5080C0 excludes the trophy, the bed and the big chest.
        if matches!(pickup.variant, 340 | 370 | 380) {
            continue;
        }
        //52484C Morphs a priced pickup into a shop item (variant150).
        if pickup.shop_price.is_some()
            || pickup.spawn_seed == 0
            || pickup
                .morph_links
                .as_ref()
                .is_some_and(|links| !links.known_empty())
        {
            unowned_leaf(state, id, 0x523984);
            return Ok(());
        }
        targets.push(index);
    }
    let trinket22 = has_trinket(state, 22);
    let mut shared = state.boot_lifecycle.shared.clone();
    let mut morphs = Vec::with_capacity(targets.len());
    for index in targets {
        let Some(variant) = d20_variant(&mut shared, trinket22) else {
            unowned_leaf(state, id, 0x5247b8);
            return Ok(());
        };
        let old = &state.room.pickups[index];
        //500A2A..56 advances the old seed once for the new Init.
        let seed = items::Rng::new(old.spawn_seed, [1, 21, 20]).raw();
        let next = match variant {
            70 | 300 => crate::Pickup::from_pocket_parts(
                old.id,
                variant,
                0,
                seed,
                old.position,
                false,
                crate::pickups::PocketSubtypeInputs::from_state(state),
            )?,
            //Trinket Init (350) is not ported.
            350 => None,
            _ => {
                let selector = crate::room_spawns::SpawnSelector {
                    group_index: 0,
                    grid_index: 0,
                    category: crate::room_spawns::SpawnCategory::Entity,
                    entity_type: 5,
                    variant: variant as u16,
                    subtype: 0,
                    position: old.position,
                    group_seed: 0,
                    spawn_seed: seed,
                };
                match variant {
                    50 => crate::chests::from_source_selector(old.id, &selector, false, state)?,
                    60 => crate::chests::from_locked_selector(old.id, &selector, false, state)?,
                    _ => crate::Pickup::from_source_selector(old.id, &selector, false, state)?,
                }
            }
        };
        let Some(next) = next else {
            unowned_leaf(state, id, 0x523929);
            return Ok(());
        };
        morphs.push((index, next));
    }
    state.boot_lifecycle.shared = shared;
    for (index, next) in morphs {
        state.room.pickups[index].morph_into(
            next,
            state.game_update_count,
            &mut state.actor_scheduler,
        )?;
    }
    Ok(())
}

fn unowned_leaf(state: &mut State, id: u32, source_pc: u32) {
    forget_unported_effect(state);
    crate::active_charges::note_effect(
        state,
        crate::active_charges::EffectFrontier::UseLeaf {
            active_id: id,
            source_pc,
        },
    );
}

/// 523489..5235F0. The landing target starts at the room-clamped
/// position+velocity*60 and steps back toward the Player by10 while a grid
/// cell is solid or an NPC's radius+Player radius+5 covers it.
fn start_jump(state: &mut State) {
    use crate::motion::WorldVector;
    let position = crate::bombs::player_world(state);
    let velocity = crate::bombs::player_velocity(state);
    let mut target = crate::critters::clamp_to_room(
        state,
        WorldVector {
            x: position.x + velocity.x * 60.0,
            y: position.y + velocity.y * 60.0,
        },
        20.0,
    );
    let mut step = WorldVector {
        x: position.x - target.x,
        y: position.y - target.y,
    };
    let mut remaining = step.x * step.x + step.y * step.y;
    let length = (step.x * step.x + step.y * step.y).sqrt();
    if length > 0.0 {
        let inverse = 1.0 / length;
        step.x *= inverse;
        step.y *= inverse;
    }
    step.x *= 10.0;
    step.y *= 10.0;
    let step_squared = step.x * step.x + step.y * step.y;
    let player_radius = crate::content::PLAYER_COLLISION_RADIUS_WORLD as f32;
    let mut landed = false;
    while remaining > 0.0 && !landed {
        if crate::critters::grid_class(state, target) != 0 {
            target.x += step.x;
            target.y += step.y;
            remaining -= step_squared;
            continue;
        }
        let mut clear = true;
        for npc in state.room.npcs.iter().filter(|npc| !npc.removed) {
            if !(10..1000).contains(&npc.kind) {
                continue;
            }
            let reach = npc.collision_radius + player_radius + 5.0;
            let dx = npc.position.x - target.x;
            let dy = npc.position.y - target.y;
            if reach * reach > dx * dx + dy * dy {
                target.x += step.x;
                target.y += step.y;
                clear = false;
                remaining -= step_squared;
            }
        }
        //524A2D clamps a copy into Entity+440; the search target stays.
        state.player.jump_target = Some(crate::critters::clamp_to_room(state, target, 20.0));
        state.player.vx = 0.0;
        state.player.vy = 0.0;
        state.player.grid_collision_class = Some(0);
        state.player.entity_collision_off = true;
        landed = clear;
    }
    if remaining <= 0.0 {
        state.player.jump_target = Some(position);
    }
    //5235D5: the Jump clip only when D68 (flight) is clear.
    if state.player.flying != Some(true) {
        crate::player_animation::force_play(state, Clip::Jump);
    }
}

/// 532247..53234C, after the full update's TEMP tick and on every
/// interpolation: while TEMP282 exists the Player moves by (target-P)*f^2,
/// f = 1 - (cooldown - (interpolation ? .5 : 0) - 1) / (cooldown282 - 2),
/// and its velocity is zeroed.
pub(crate) fn advance_jump(state: &mut State, phase: crate::phase::Phase) {
    if state
        .player
        .entity_source
        .is_none_or(|source| source.variant != 0)
    {
        return;
    }
    let Some(cooldown) = state.player_items.as_ref().and_then(|owner| {
        owner
            .temporary_collectibles
            .iter()
            .find(|record| record.item_id == 282)
            .map(|record| record.cooldown)
    }) else {
        return;
    };
    let Some(target) = state.player.jump_target else {
        return;
    };
    let Some(config) = items::definition(282) else {
        return;
    };
    let interpolation = if phase == crate::phase::Phase::Update {
        0.0
    } else {
        0.5
    };
    let progress = (cooldown as f32 - interpolation - 1.0) / (config.cooldown as f32 - 2.0);
    let factor = 1.0 - progress;
    let position = crate::bombs::player_world(state);
    let delta = crate::motion::WorldVector {
        x: target.x - position.x,
        y: target.y - position.y,
    };
    let scaled = crate::motion::WorldVector {
        x: delta.x * factor,
        y: delta.y * factor,
    };
    let next = crate::motion::WorldVector {
        x: position.x + scaled.x * factor,
        y: position.y + scaled.y * factor,
    };
    state.player.x = f64::from(next.x) * 0.65 + 32.0;
    state.player.y = f64::from(next.y) * 0.65 - 46.0;
    state.player.vx = 0.0;
    state.player.vy = 0.0;
}

/// 517798..5177BA: TEMP282's removal on Player variant0 sets D6C bit80,
/// Entity+344 back to class4 and evaluates the flight cache immediately.
pub(crate) fn land_jump(state: &mut State) -> Result<(), String> {
    if state
        .player
        .entity_source
        .is_none_or(|source| source.variant != 0)
    {
        return Ok(());
    }
    state.player.entity_collision_off = false;
    if let Some(owner) = state.player_items.as_mut() {
        owner.unevaluated_cache_flags |= 0x80;
    }
    crate::player_stats::evaluate_cache(state)
}
