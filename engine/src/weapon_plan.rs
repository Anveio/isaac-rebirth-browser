//! Actual requested cache fields and immutable regular-Tear birth values.
//! Actor slots, charge, input and random draws retain their own owners.
use crate::{
    attack_color::AttackColor,
    attack_flags::AttackFlags,
    item_pipeline::{CacheMask, ItemInputs},
    player_stats::PlayerStats,
    projectiles, State,
};
use serde::{Deserialize, Serialize};

/// Player+C08 is a source weapon selector, independent of the spawned entity type.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[repr(u32)]
#[serde(into = "u32", try_from = "u32")]
pub enum PrimaryWeaponC08 {
    RegularTears = 1,
    Brimstone = 2,
    Technology = 3,
    MomsKnife = 4,
    DrFetus = 5,
    EpicFetus = 6,
    MonstrosLung = 7,
    Ludovico = 8,
}

impl PrimaryWeaponC08 {
    pub const fn numeric(self) -> u32 {
        self as u32
    }
}

impl From<PrimaryWeaponC08> for u32 {
    fn from(primary: PrimaryWeaponC08) -> Self {
        primary.numeric()
    }
}

impl TryFrom<u32> for PrimaryWeaponC08 {
    type Error = String;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::RegularTears),
            2 => Ok(Self::Brimstone),
            3 => Ok(Self::Technology),
            4 => Ok(Self::MomsKnife),
            5 => Ok(Self::DrFetus),
            6 => Ok(Self::EpicFetus),
            7 => Ok(Self::MonstrosLung),
            8 => Ok(Self::Ludovico),
            _ => Err("invalid source primary weapon C08".into()),
        }
    }
}

/// The actual cache fields. Controller choice and mutable charge/actor slots
/// belong to their callbacks; this owner never caches permanent Has329.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WeaponCache {
    pub primary: PrimaryWeaponC08,
    pub flags_c98: AttackFlags,
    pub color_ca0: AttackColor,
    pub color_ccc: AttackColor,
}

impl WeaponCache {
    pub(crate) const fn fresh_player() -> Self {
        Self {
            primary: PrimaryWeaponC08::RegularTears,
            flags_c98: AttackFlags(0),
            color_ca0: AttackColor::WHITE,
            color_ccc: AttackColor::WHITE,
        }
    }
}

impl Default for WeaponCache {
    fn default() -> Self {
        Self::fresh_player()
    }
}

/// Source context admission, separate from the registered actor reference.
/// Fresh NULL C18 and each admitted actor transition have explicit owners.
/// A real unsupported invocation invalidates that proof until the next run.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WeaponAdmission {
    pub source_player_type: Option<u32>,
    pub c18_is_known_empty: bool,
    ///Port effect history: an earlier requested primary change reached an
    ///unadmitted prior C18. Equality on a later request cannot erase it.
    #[serde(default)]
    pub pending_primary_cleanup: Option<PrimaryCleanupIntent>,
    pub unadmitted_controller: Option<UnadmittedController>,
}
/// First unperformed source virtual48 intent, not an invented actor pointer.
/// Real actor cleanup or fresh Player construction must own its completion.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PrimaryCleanupIntent {
    pub from_primary: PrimaryWeaponC08,
    pub to_primary: PrimaryWeaponC08,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UnadmittedController {
    pub primary: u32,
    pub channel: String,
}
impl WeaponAdmission {
    /// Actual supported Isaac constructor BFC=0 and5196xx NULL C18.
    pub(crate) fn fresh_isaac() -> Self {
        Self {
            source_player_type: Some(0),
            c18_is_known_empty: true,
            pending_primary_cleanup: None,
            unadmitted_controller: None,
        }
    }
    pub(crate) fn record_unadmitted(&mut self, primary: PrimaryWeaponC08, channel: &str) {
        self.c18_is_known_empty = false;
        self.unadmitted_controller
            .get_or_insert(UnadmittedController {
                primary: primary.numeric(),
                channel: channel.into(),
            });
    }
}

/// Requested100 only: C08 store precedes every scalar callback. The caller
/// completes this stage only when its actual prior C18 effect is admitted.
pub(crate) fn resolve_primary(
    items: ItemInputs<'_>,
    source_player_type: u32,
    prior: PrimaryWeaponC08,
    requested: CacheMask,
) -> Result<PrimaryWeaponC08, String> {
    if requested.0 & 0x100 == 0 {
        return Ok(prior);
    }
    Ok(if items.primary_permanent_presence(168)? {
        PrimaryWeaponC08::EpicFetus
    } else if items.primary_permanent_presence(114)? {
        PrimaryWeaponC08::MomsKnife
    } else if items.primary_permanent_presence(52)? {
        PrimaryWeaponC08::DrFetus
    } else if items.primary_permanent_presence(118)? {
        PrimaryWeaponC08::Brimstone
    } else if items.primary_permanent_presence(68)? {
        PrimaryWeaponC08::Technology
    } else if source_player_type == 7 {
        PrimaryWeaponC08::Brimstone
    } else if items.primary_permanent_presence(329)? {
        PrimaryWeaponC08::Ludovico
    } else if items.primary_permanent_presence(229)? {
        PrimaryWeaponC08::MonstrosLung
    } else {
        PrimaryWeaponC08::RegularTears
    })
}

/// Actual32/64 writes within the source-closed fresh selected cache paths.
/// This is cache support, not controller/contact admission.
pub(crate) fn resolve_appearance(
    items: ItemInputs<'_>,
    source_player_type: Option<u32>,
    mut prior: WeaponCache,
    requested: CacheMask,
) -> (WeaponCache, CacheMask) {
    let mut completed = 0;
    let number_one = source_player_type == Some(0) && items.admits_number_one_appearance();
    if source_player_type == Some(0) && (items.admits_selected_appearance() || number_one) {
        if requested.0 & 32 != 0 {
            prior.flags_c98 = AttackFlags(if prior.primary == PrimaryWeaponC08::Ludovico {
                3
            } else {
                0
            });
            completed |= 32;
        }
        if requested.0 & 64 != 0 {
            //515997/515AB1 Have6, else5E3D70 record-presence. This leaf
            //admits permanent6 only; the two literal ColorMods are distinct.
            if number_one && items.number_one_present() {
                let byte_scale = 1.0_f32 / 255.0_f32;
                prior.color_ca0 = AttackColor {
                    tint: [1.0, 1.0, 0.0, 1.0],
                    source_middle: [0.0; 4],
                    offsets: [45.0 * byte_scale, 15.0 * byte_scale, 0.0],
                };
                prior.color_ccc = AttackColor {
                    tint: [0.2, 1.0, 0.0, 1.0],
                    source_middle: [0.0; 4],
                    offsets: [255.0 * byte_scale, 255.0 * byte_scale, 0.0],
                };
            } else {
                prior.color_ca0 = AttackColor::WHITE;
                prior.color_ccc = AttackColor::WHITE;
            }
            completed |= 64;
        }
    }
    (prior, CacheMask(completed))
}

///Source519fc0 resolves permanent329 on each admitted invocation. These
///are caller channels, separate from actor types and secondary birth callbacks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ControllerChannel {
    RegularShots,
    OrdinaryKnife,
    OrdinaryFetus,
    EpicFetus,
    ControlledKnife,
    ChargedBrimstone,
    OrdinaryTechnology,
    ControlledTechnology,
    ControlledBrimstone,
    ControlledTear,
    Unimplemented(PrimaryWeaponC08),
}

pub(crate) fn resolve_controller(
    items: ItemInputs<'_>,
    primary: PrimaryWeaponC08,
) -> Result<ControllerChannel, String> {
    Ok(match primary {
        PrimaryWeaponC08::RegularTears => ControllerChannel::RegularShots,
        PrimaryWeaponC08::MomsKnife if items.primary_permanent_presence(329)? => {
            ControllerChannel::ControlledKnife
        }
        PrimaryWeaponC08::MomsKnife => ControllerChannel::OrdinaryKnife,
        PrimaryWeaponC08::Brimstone if items.primary_permanent_presence(329)? => {
            ControllerChannel::ControlledBrimstone
        }
        PrimaryWeaponC08::Brimstone => ControllerChannel::ChargedBrimstone,
        PrimaryWeaponC08::Technology if items.primary_permanent_presence(329)? => {
            ControllerChannel::ControlledTechnology
        }
        PrimaryWeaponC08::Technology => ControllerChannel::OrdinaryTechnology,
        PrimaryWeaponC08::DrFetus if !items.primary_permanent_presence(329)? => {
            ControllerChannel::OrdinaryFetus
        }
        PrimaryWeaponC08::EpicFetus => ControllerChannel::EpicFetus,
        PrimaryWeaponC08::Ludovico => ControllerChannel::ControlledTear,
        other => ControllerChannel::Unimplemented(other),
    })
}

///Current invocation's values. Cache reevaluation never broadcasts these
///copies to already admitted actors; the actual Player callback owns reapply.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AttackProperties {
    pub(crate) flags: AttackFlags,
    pub(crate) color: AttackColor,
    pub(crate) collision_damage: f32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ResolvedWeaponPlan {
    pub(crate) primary: PrimaryWeaponC08,
    pub(crate) controller: ControllerChannel,
    pub(crate) properties: AttackProperties,
    /// Current cached C7C, read by the controlled caller each invocation.
    /// This is separate from the actor's born velocity and current friction.
    pub(crate) source_shot_speed_c7c: f32,
    pub(crate) secondary_color_ccc: AttackColor,
    pub(crate) source_range_c8c: f32,
    pub(crate) max_fire_delay: i32,
    pub(crate) regular: Option<RegularTearPlan>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum LiveWeaponResolution {
    Resolved(Box<ResolvedWeaponPlan>),
    Unadmitted(&'static str),
}

///Pure current read/resolve stage. Mutable input, C0C/C18, actual phase,
///RNG, registration, contact and requested-primary cleanup stay concrete.
///No builder is evaluated eagerly for a nonregular controller.
pub(crate) fn resolve_live_weapon(
    items: ItemInputs<'_>,
    cache: WeaponCache,
    stats: PlayerStats,
    max_fire_delay: i32,
    admission: &WeaponAdmission,
) -> Result<LiveWeaponResolution, String> {
    stats.validate()?;
    if max_fire_delay <= 0 {
        return Err("Player C78 violates its source positive clamp".into());
    }
    if !cache.color_ca0.is_finite() || !cache.color_ccc.is_finite() {
        return Err("weapon cache contains nonfinite source color owners".into());
    }
    if admission.pending_primary_cleanup.is_some() {
        return Ok(LiveWeaponResolution::Unadmitted("pendingPrimaryCleanup"));
    }
    if admission.source_player_type != Some(0) {
        return Ok(LiveWeaponResolution::Unadmitted("weaponContext"));
    }
    let controller = resolve_controller(items, cache.primary)?;
    if controller == ControllerChannel::RegularShots
        && items.pending_cache().0 & RegularTearPlan::INPUT_CACHE_MASK.0 != 0
    {
        return Ok(LiveWeaponResolution::Unadmitted("regularTearCache"));
    }
    let plan = ResolvedWeaponPlan {
        primary: cache.primary,
        controller,
        properties: AttackProperties {
            flags: cache.flags_c98,
            color: cache.color_ca0,
            collision_damage: stats.damage,
        },
        source_shot_speed_c7c: stats.shot_speed,
        secondary_color_ccc: cache.color_ccc,
        source_range_c8c: stats.tear_height,
        max_fire_delay,
        regular: if controller == ControllerChannel::RegularShots {
            Some(RegularTearPlan::resolve(stats, max_fire_delay, cache)?)
        } else {
            None
        },
    };
    Ok(LiveWeaponResolution::Resolved(Box::new(plan)))
}

///Concrete owner's read-only adapter. Live dispatch handles typed frontier
///results before actor writes; requested cache evaluation remains separate.
pub(crate) fn resolve_current_weapon(state: &State) -> Result<LiveWeaponResolution, String> {
    resolve_live_weapon(
        ItemInputs::new(
            state
                .player_items
                .as_ref()
                .ok_or("Player item owner absent at firing")?,
        )?,
        state.weapon_cache,
        state.player_stats,
        state.max_fire_delay,
        &state.weapon_admission,
    )
}

///Actual511D90 signed3068/index0 history, distinct from kind4 modifiers.
///It is never reconstructed from current inventory or reset on removal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum KnifeContactAdmission {
    Mapped,
    UnmappedBuilderModifiers,
    UnknownTransformationHistory,
    TransformationProc,
}
pub(crate) fn admit_knife_transformation_history(
    counts: Option<[i32; 2]>,
) -> KnifeContactAdmission {
    match counts {
        None => KnifeContactAdmission::UnknownTransformationHistory,
        Some(counts) if counts[0] >= 3 => KnifeContactAdmission::TransformationProc,
        Some(_) => KnifeContactAdmission::Mapped,
    }
}

///Only the literal selected neutral kind4 path.257/6, TEMP, trinket and
///unknown fresh context remain separate frontiers. No general Has wrapper.
pub(crate) fn admits_neutral_knife_properties(
    items: ItemInputs<'_>,
    source_player_type: Option<u32>,
) -> bool {
    source_player_type == Some(0) && items.admits_neutral_kind4()
}

///Pure preflight at eligible overlap before hit-registry/queue mutation.
///ParentKnife uses only admit_knife_transformation_history; ParentPlayer
///also demands its real kind4 builder domain.
pub(crate) fn admit_neutral_knife_contact(
    items: ItemInputs<'_>,
    source_player_type: Option<u32>,
    counts: Option<[i32; 2]>,
) -> KnifeContactAdmission {
    if !admits_neutral_knife_properties(items, source_player_type) {
        return KnifeContactAdmission::UnmappedBuilderModifiers;
    }
    admit_knife_transformation_history(counts)
}

///Actual neutral50D090(kind4) after registry insertion reads CURRENT C88,
///which can differ from the earlier Knife48C copied by Player reapply.
///This stage requests no RNG; proc/contact callback ownership is separate.
pub(crate) fn resolve_neutral_kind4_damage(
    items: ItemInputs<'_>,
    source_player_type: Option<u32>,
    stats: PlayerStats,
) -> Result<f32, String> {
    stats.validate()?;
    if !admits_neutral_knife_properties(items, source_player_type) {
        return Err("kind4 requires an unadmitted source modifier/context branch".into());
    }
    Ok(stats.damage)
}

///OrdinaryBrimstone caller and its kind2 damage read have their own
///source footprint. This is independent of Knife contact/transformation.
pub(crate) fn admits_neutral_brimstone_properties(
    items: ItemInputs<'_>,
    source_player_type: Option<u32>,
) -> bool {
    source_player_type == Some(0) && items.admits_neutral_kind2()
}

///Independent neutral50D090 kinds2/3 footprint for Technology. Its cached
///context predicate alone does not establish a getter/controller capability.
pub(crate) fn admits_neutral_technology_properties(
    items: ItemInputs<'_>,
    source_player_type: Option<u32>,
) -> bool {
    source_player_type == Some(0) && items.admits_technology_kind2_getter()
}

///50E4A7/50E2A8 execute this current38 getter only AFTER the real Laser Spawn.
///Neutral kind2 requests no RNG; the generic3C scale is not a Laser consumer.
pub(crate) fn resolve_neutral_kind2_damage(
    items: ItemInputs<'_>,
    source_player_type: Option<u32>,
    stats: PlayerStats,
) -> Result<f32, String> {
    stats.validate()?;
    if !admits_neutral_brimstone_properties(items, source_player_type)
        && !admits_neutral_technology_properties(items, source_player_type)
    {
        return Err("kind2 requires an unadmitted source modifier/context branch".into());
    }
    Ok(stats.damage)
}

///50DE83 consumes actual50D090(kind5) outputs30/38/04 for the Bomb.
///The factory owns its later damage multiplier, variant, RNG and Parent setter;
///unused generic size/variant outputs are not promoted to Bomb properties.
pub(crate) fn resolve_fetus_kind5(
    items: ItemInputs<'_>,
    source_player_type: Option<u32>,
    cache: WeaponCache,
    stats: PlayerStats,
) -> Result<AttackProperties, String> {
    stats.validate()?;
    if source_player_type != Some(0)
        || cache.primary != PrimaryWeaponC08::DrFetus
        || !items.admits_fetus_kind5_context()
    {
        return Err("kind5 requires an unadmitted source modifier/context branch".into());
    }
    if !cache.color_ca0.is_finite() {
        return Err("kind5 current CA0 owner is nonfinite".into());
    }
    Ok(AttackProperties {
        flags: cache.flags_c98,
        color: cache.color_ca0,
        collision_damage: stats.damage,
    })
}

///45F3F0 calls FIRSTPlayer kind6 on the actual dead Rocket Full.
///No target/rocket birth owns these current cache values.
pub(crate) fn resolve_epic_fetus_kind6(
    items: ItemInputs<'_>,
    source_player_type: Option<u32>,
    cache: WeaponCache,
    stats: PlayerStats,
) -> Result<AttackProperties, String> {
    stats.validate()?;
    if source_player_type != Some(0)
        || cache.primary != PrimaryWeaponC08::EpicFetus
        || !items.admits_epic_fetus_kind6_context()
    {
        return Err("kind6 requires an unadmitted FIRSTPlayer context".into());
    }
    if !cache.color_ca0.is_finite() {
        return Err("kind6 actual current CA0 is nonfinite".into());
    }
    Ok(AttackProperties {
        flags: cache.flags_c98,
        color: cache.color_ca0,
        collision_damage: stats.damage,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RegularTearPlan {
    pub(crate) flags: AttackFlags,
    pub(crate) color: AttackColor,
    pub(crate) shot_speed: f32,
    pub(crate) damage: f32,
    pub(crate) parameters: projectiles::Parameters,
    pub(crate) scale: f32,
    pub(crate) animation_tier: u8,
    pub(crate) collision_radius: f32,
    pub(crate) fire_delay: i32,
    pub(crate) head_timer: i32,
}

impl RegularTearPlan {
    /// C88, C78, C7C, C8C/C90/C94, C98 and CA0 are birth inputs.
    /// Move/Flight/Familiar/Luck requests do not own these copied fields.
    const INPUT_CACHE_MASK: CacheMask = CacheMask(1 | 2 | 4 | 8 | 32 | 64);

    pub(crate) fn resolve(
        stats: PlayerStats,
        max_fire_delay: i32,
        cache: WeaponCache,
    ) -> Result<Self, String> {
        stats.validate()?;
        if !cache.color_ca0.is_finite() {
            return Err("regular Tear color owner is nonfinite".into());
        }
        let scale = projectiles::scale(stats.damage)?;
        Ok(Self {
            flags: cache.flags_c98,
            color: cache.color_ca0,
            shot_speed: stats.shot_speed,
            damage: stats.damage,
            parameters: stats.parameters(),
            scale,
            animation_tier: projectiles::regular_animation_tier(scale)?,
            collision_radius: projectiles::collision_radius(scale, 8.0)?,
            fire_delay: max_fire_delay,
            head_timer: ((max_fire_delay as f32 * 0.45) as i32 + 7).max(11),
        })
    }
}

///Actual50D090(kind3) builder/status frontier, independent of actor511D90
///history and ordinary/controlled Laser geometry and enqueue ownership.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LaserStatusAdmission {
    Mapped,
    UnknownPlayerContext,
    UnmappedBuilderModifiers,
    UnmappedStatusFlags,
}

pub(crate) fn admit_current_laser_contact_flags(
    state: &State,
) -> Result<LaserStatusAdmission, String> {
    state.player_stats.validate()?;
    if !state.weapon_cache.color_ca0.is_finite() || !state.weapon_cache.color_ccc.is_finite() {
        return Err("Laser kind3 requires finite actual Player color owners".into());
    }
    if state.weapon_admission.source_player_type != Some(0) {
        return Ok(LaserStatusAdmission::UnknownPlayerContext);
    }
    let inputs = ItemInputs::new(
        state
            .player_items
            .as_ref()
            .ok_or("Laser kind3 actual Player items absent")?,
    )?;
    if !inputs.admits_neutral_kind3() && !inputs.admits_technology_kind3_getter() {
        return Ok(LaserStatusAdmission::UnmappedBuilderModifiers);
    }
    //Actual476869..8A1 status masks start at8; currentlow3 is inert.
    if state.weapon_cache.flags_c98.0 & !3 != 0 {
        return Ok(LaserStatusAdmission::UnmappedStatusFlags);
    }
    Ok(LaserStatusAdmission::Mapped)
}

///This actual output30 read is called AFTER each admitted NPC enqueue.
///Selected neutral kind3 has no RNG/proc demand; it does not resolve kind2
///birth fields, kind4 damage, or the actor's separate transformation query.
pub(crate) fn resolve_current_laser_contact_flags(state: &State) -> Result<AttackFlags, String> {
    if admit_current_laser_contact_flags(state)? != LaserStatusAdmission::Mapped {
        return Err("Laser kind3 reached an unadmitted actual source path".into());
    }
    Ok(state.weapon_cache.flags_c98)
}
