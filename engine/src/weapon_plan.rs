//! Actual requested cache fields and immutable regular-Tear birth values.
//! Actor slots, charge, input and random draws retain their own owners.
use crate::{
    attack_color::AttackColor,
    attack_flags::AttackFlags,
    item_pipeline::{CacheMask, ItemInputs},
    item_queries::AppearanceQueries,
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
    ///Actual persistent Player-controller history, separate from a physical
    ///C18 pointer or an unrelated contact/grid diagnostic. Legacy is unknown.
    #[serde(default)]
    pub c18_controller_history_known: bool,
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
            c18_controller_history_known: true,
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

/// Complete 5130CA..513345 C98 callback. Only reached source predicates are
/// read; mutable body/controller/copy fields keep their independent owners.
/// AppearanceQueries reads the canonical stores through their own histories.
#[derive(Clone, Copy)]
struct FlagsReadPlan {
    flags: AttackFlags,
}

impl FlagsReadPlan {
    fn read(inputs: AppearanceQueries<'_>) -> Option<Self> {
        // 5130CA resets the entire qword. No prior cached C98 bit is retained.
        let mut bits = 0_u32;
        // 5130DD/514EF8/514F1C/515A3D/515A52/515A5F. Variant and 30E8
        // are read only after every preceding permanent/TEMP predicate fails.
        if inputs.have(182)?
            || inputs.have(3)?
            || inputs.temporary_present(192)?
            || inputs.temporary_present(3)?
            || inputs.have(331)?
            || inputs.homing_variant()?
        {
            bits |= 0x4;
        }
        if inputs.have(132)? {
            bits |= 0x80;
        }
        if inputs.have(261)? || inputs.temporary_present(261)? {
            bits |= 0x20_0000;
        }
        // 513133/514CA8..514CF8: earlier positive effects bypass C08 and 336.
        if inputs.have(185)?
            || inputs.have(115)?
            || inputs.have(159)?
            || inputs.temporary_present(115)?
            || inputs.primary()? == PrimaryWeaponC08::Ludovico
            || inputs.have(336)?
        {
            bits |= 0x1;
        }
        // 513150/514C10..514C60 is a separate ordered piercing chain.
        if inputs.have(48)?
            || inputs.have(237)?
            || inputs.temporary_present(237)?
            || inputs.have(306)?
            || inputs.primary()? == PrimaryWeaponC08::Ludovico
            || inputs.have(336)?
        {
            bits |= 0x2;
        }
        if inputs.have(169)? {
            bits |= 0x200;
        }
        if inputs.have(305)? {
            bits |= 0x10;
        }
        if inputs.have(104)? {
            bits |= 0x40;
        }
        if inputs.have(5)? || inputs.temporary_present(5)? {
            bits |= 0x100;
        }
        if inputs.has_trinket(10)? {
            bits |= 0x400;
        }
        if inputs.has_trinket(26)? {
            bits |= 0x4000_0000;
        }
        if inputs.have(149)? && inputs.primary()? != PrimaryWeaponC08::Ludovico {
            bits |= 0x1010;
        }
        if inputs.have(233)? {
            bits |= 0x1_0001;
        }
        // 513250..513272/515960..51598A. Have316/69 exist only for C08=1;
        // positive316 skips69, and weapon7/8 skip both queries and the mask.
        if inputs.have(222)? {
            let applies = match inputs.primary()? {
                PrimaryWeaponC08::Ludovico | PrimaryWeaponC08::MonstrosLung => false,
                PrimaryWeaponC08::RegularTears => !inputs.have(316)? && !inputs.have(69)?,
                _ => true,
            };
            if applies {
                bits |= 0x2_0000;
            }
        }
        if inputs.have(224)? {
            bits |= 0x4_0000;
        }
        if inputs.have(315)? {
            bits |= 0x80_0000;
        }
        if inputs.has_trinket(9)? {
            bits |= 0x200_0000;
        }
        if inputs.have(309)? {
            bits |= 0x100_0000;
        }
        if inputs.has_trinket(11)? {
            bits |= 0x400_0000;
        }
        if inputs.has_trinket(12)? {
            bits |= 0x800_0000;
        }
        // 513339 zero-extends EAX before OR into qword C98. Bit33 belongs
        // to later controlled actor birth, never to this cache callback.
        if inputs.have(331)? {
            bits |= 0x8000_0000;
        }
        Some(Self {
            flags: AttackFlags(u64::from(bits)),
        })
    }

    fn resolve(self) -> AttackFlags {
        self.flags
    }
}

///Complete513355..512160 Color64 callback. CCC and CA0 have independent
///source-ordered first-match chains; no later unread predicate is required.
#[derive(Clone, Copy)]
struct ColorReadPlan {
    color_ccc: AttackColor,
    color_ca0: AttackColor,
}

impl ColorReadPlan {
    fn read(appearance: AppearanceQueries<'_>) -> Option<Self> {
        Some(Self {
            color_ccc: Self::read_ccc(appearance)?,
            color_ca0: Self::read_ca0(appearance)?,
        })
    }

    fn read_ccc(appearance: AppearanceQueries<'_>) -> Option<AttackColor> {
        Some(
            if appearance.have(115)? || appearance.temporary_present(115)? {
                Self::tint([1.5, 2.0, 2.0, 0.5])
            } else if appearance.have(103)? || appearance.have(305)? {
                Self::byte_offsets([0.4, 0.97, 0.5, 1.0], [0, 150, 0])
            } else if appearance.have(104)? {
                Self::byte_offsets([1.0, 0.4, 0.13, 1.0], [30, 0, 0])
            } else if appearance.have(89)? {
                Self::byte_offsets([3.0, 3.0, 3.0, 1.0], [150, 150, 150])
            } else if appearance.have(69)? {
                Self::byte_offsets([0.33, 0.18, 0.18, 1.0], [66, 40, 40])
            } else if appearance.have(6)? || appearance.temporary_present(6)? {
                Self::byte_offsets([0.2, 1.0, 0.0, 1.0], [255, 255, 0])
            } else if appearance.have(3)?
                || appearance.temporary_present(3)?
                || appearance.temporary_present(192)?
                || appearance.homing_variant()?
            {
                Self::byte_offsets([0.4, 0.15, 0.38, 1.0], [90, 0, 180])
            } else if appearance.have(200)? {
                Self::byte_offsets([0.4, 0.0, 0.0, 1.0], [180, 0, 180])
            } else if appearance.have(132)? {
                Self::tint([0.1, 0.1, 0.1, 1.0])
            } else {
                AttackColor::WHITE
            },
        )
    }

    fn read_ca0(appearance: AppearanceQueries<'_>) -> Option<AttackColor> {
        Some(
            if appearance.have(149)? && appearance.primary()? != PrimaryWeaponC08::DrFetus {
                Self::tint([0.5, 0.9, 0.4, 1.0])
            } else if appearance.have(182)? {
                Self::tint([1.5, 2.0, 2.0, 1.0])
            } else if appearance.have(115)?
                || appearance.temporary_present(115)?
                || appearance.have(159)?
                || appearance.have(185)?
            {
                Self::tint([1.5, 2.0, 2.0, 0.5])
            } else if appearance.have(132)? {
                Self::tint([0.2, 0.09, 0.065, 1.0])
            } else if appearance.have(104)? {
                Self::tint([0.9, 0.3, 0.08, 1.0])
            } else if appearance.have(69)? {
                Self::byte_offsets([0.33, 0.18, 0.18, 1.0], [66, 40, 40])
            } else if appearance.have(6)? || appearance.temporary_present(6)? {
                Self::byte_offsets([1.0, 1.0, 0.0, 1.0], [45, 15, 0])
            } else if appearance.have(3)?
                || appearance.temporary_present(3)?
                || appearance.temporary_present(192)?
                || appearance.homing_variant()?
            {
                Self::byte_offsets([0.4, 0.15, 0.38, 1.0], [71, 0, 116])
            } else if appearance.have(310)? {
                Self::tint([0.2, 0.2, 0.2, 1.0])
            } else if appearance.have(221)? {
                //42F500 copies these f32 offsets directly;42F510 is not reached.
                AttackColor {
                    tint: [1.0, 1.0, 0.8, 1.0],
                    source_middle: [0.0; 4],
                    offsets: [0.1, 0.1, 0.1],
                }
            } else if appearance.have(336)? {
                Self::tint([0.5, 0.25, 0.1, 0.9])
            } else if appearance.have(330)? {
                Self::tint([1.5, 2.0, 2.0, 1.0])
            } else {
                AttackColor::WHITE
            },
        )
    }

    ///42F450 establishes the middle/offset zeroes;42F480 changes tint only.
    fn tint(tint: [f32; 4]) -> AttackColor {
        AttackColor {
            tint,
            ..AttackColor::WHITE
        }
    }

    ///42F510 converts each signed integer then multiplies by the pinned
    ///binary32 constant668980. Preserve the three separate f32 products.
    fn byte_offsets(tint: [f32; 4], offsets: [i32; 3]) -> AttackColor {
        let byte_scale = 1.0_f32 / 255.0_f32;
        AttackColor {
            tint,
            source_middle: [0.0; 4],
            offsets: [
                offsets[0] as f32 * byte_scale,
                offsets[1] as f32 * byte_scale,
                offsets[2] as f32 * byte_scale,
            ],
        }
    }
}

///Each appearance callback follows only its reached source queries.
///Unavailable facts leave its request pending and cached values intact.
///These writes do not admit controller, getter or contact effects.
pub(crate) fn resolve_appearance(
    mut prior: WeaponCache,
    requested: CacheMask,
    appearance: AppearanceQueries<'_>,
) -> (WeaponCache, CacheMask) {
    let mut completed = 0;
    if requested.0 & 32 != 0 {
        if let Some(plan) = FlagsReadPlan::read(appearance) {
            prior.flags_c98 = plan.resolve();
            completed |= 32;
        }
    }
    if requested.0 & 64 != 0 {
        if let Some(plan) = ColorReadPlan::read(appearance) {
            prior.color_ccc = plan.color_ccc;
            prior.color_ca0 = plan.color_ca0;
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
    MonstroLung,
    ControlledFetus,
    EpicFetus,
    ControlledKnife,
    ChargedBrimstone,
    OrdinaryTechnology,
    ControlledTechnology,
    ControlledBrimstone,
    ControlledTear,
}

impl ControllerChannel {
    ///Source callback footprint at an actual skipped Player invocation.
    ///Ordinary68/52 and regular/burst emitters do not bind C18.
    pub(crate) fn writes_c18(self) -> bool {
        matches!(
            self,
            Self::OrdinaryKnife
                | Self::ControlledKnife
                | Self::ChargedBrimstone
                | Self::ControlledBrimstone
                | Self::ControlledTechnology
                | Self::ControlledTear
                | Self::ControlledFetus
                | Self::EpicFetus
        )
    }
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
        PrimaryWeaponC08::DrFetus if items.primary_permanent_presence(329)? => {
            ControllerChannel::ControlledFetus
        }
        PrimaryWeaponC08::DrFetus => ControllerChannel::OrdinaryFetus,
        PrimaryWeaponC08::MonstrosLung => ControllerChannel::MonstroLung,
        PrimaryWeaponC08::EpicFetus => ControllerChannel::EpicFetus,
        PrimaryWeaponC08::Ludovico => ControllerChannel::ControlledTear,
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
            let regular = RegularTearPlan::resolve(stats, max_fire_delay, cache)?;
            Some(
                if items.admits_soy_milk_regular_properties(
                    admission.source_player_type,
                    cache.primary,
                    cache.flags_c98,
                ) {
                    regular.with_soy_milk_scale()?
                } else {
                    regular
                },
            )
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

///Selected controlled114+329 caller admission is independent of the ordinary
///Knife emitter and current kind4 damage. Actor/satellite owners stay concrete.
pub(crate) fn admits_controlled_knife_caller(
    items: ItemInputs<'_>,
    source_player_type: Option<u32>,
) -> bool {
    source_player_type == Some(0) && items.admits_controlled_knife_caller()
}

///Pure preflight at eligible overlap before hit-registry/queue mutation.
///ParentKnife uses only admit_knife_transformation_history; ParentPlayer
///also demands its real kind4 builder domain.
pub(crate) fn admit_neutral_knife_contact(
    items: ItemInputs<'_>,
    source_player_type: Option<u32>,
    counts: Option<[i32; 2]>,
) -> KnifeContactAdmission {
    if source_player_type != Some(0) || !items.admits_knife_kind4_getter() {
        return KnifeContactAdmission::UnmappedBuilderModifiers;
    }
    admit_knife_transformation_history(counts)
}

///Actual neutral50D090(kind4) after registry insertion reads CURRENT C88,
///which can differ from the earlier Knife48C copied by Player reapply.
///This stage requests no RNG; proc/contact callback ownership is separate.
/// Pure 50D090(kind4) read schedule. The caller supplies independently owned
/// current fields/query histories and the actual Player receiver. No mutable
/// state, cached-output completion, actor capability or RNG ownership is made
/// here. C70 and actor/caller history remain the live caller's separate inputs:
/// this source getter itself never reads C70.
pub(crate) trait Kind4Source {
    fn have(&self, id: usize) -> Option<bool>;
    fn temporary_present(&self, id: u32) -> Option<bool>;
    fn has_trinket(&self, id: u32) -> Option<bool>;
    fn height_c8c(&self) -> Option<f32>;
    fn damage_c88(&self) -> Option<f32>;
    fn flags_c98(&self) -> Option<AttackFlags>;
    fn color_ca0(&self) -> Option<AttackColor>;
    fn luck_d64(&self) -> Option<f32>;
    fn entity_variant(&self) -> Option<u32>;
    fn variant_subtype_30e8(&self) -> Option<u32>;
    fn turn_state_d70(&self) -> Option<i8>;
    fn source_player_type_bfc(&self) -> Option<u32>;
    fn primary_c08(&self) -> Option<PrimaryWeaponC08>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind4Frontier {
    MissingField {
        source_pc: u32,
        field: &'static str,
    },
    MissingHave {
        source_pc: u32,
        id: usize,
    },
    MissingTemporary {
        source_pc: u32,
        id: u32,
    },
    MissingTrinket {
        source_pc: u32,
        id: u32,
    },
    InvalidField {
        source_pc: u32,
        field: &'static str,
    },
    /// An actual reached SharedBound invocation; no draw is fabricated or
    /// consumed by this pure planner. Earlier reads do not complete Get4.
    SharedDraw {
        source_pc: u32,
        bound: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Kind4ReadPlan {
    pub variant: u32,
    pub color: AttackColor,
    pub flags: AttackFlags,
    pub damage: f32,
    pub scale: f32,
    pub height: f32,
}

impl Kind4ReadPlan {
    pub(crate) fn read(source: &impl Kind4Source) -> Result<Self, Kind4Frontier> {
        // 50D0B1..128: source order is C8C, C88, C98, then all eleven CA0
        // words. The separate result color cannot alias the Player field.
        let height = Self::finite(source.height_c8c(), 0x50d0b1, "C8C")?;
        let damage = Self::finite(source.damage_c88(), 0x50d0b7, "C88")?;
        let flags = Self::field(source.flags_c98(), 0x50d0ce, "C98")?;
        let color = Self::field(source.color_ca0(), 0x50d0e5, "CA0")?;
        if damage < 0.0 || !color.is_finite() {
            return Err(Kind4Frontier::InvalidField {
                source_pc: 0x50d12b,
                field: "nonnegative C88 / finite CA0",
            });
        }
        // The linear pair is binary32; sqrt and its .15f32 coefficient
        // execute in binary64 before the final binary32 conversion.
        let linear = damage * 0.04_f32 + 0.6_f32;
        let scale = (f64::from(linear) + f64::from(damage).sqrt() * f64::from(0.15_f32)) as f32;
        let mut plan = Self {
            variant: 0,
            color,
            flags,
            damage,
            scale,
            height,
        };

        if Self::trinket(source, 30, 0x50d176)? {
            return Err(Self::luck_draw(source, 0x50d18c, 0.5, 10, 1, 0x50d1ad)?);
        }
        if Self::trinket(source, 31, 0x50d1c7)? {
            return Err(Self::luck_draw(source, 0x50d1dd, 0.5, 10, 1, 0x50d1fe)?);
        }
        if Self::have(source, 110, 0x50d218)? || Self::temporary(source, 110, 0x50db6c)? {
            return Err(Self::luck_draw(source, 0x50d232, 0.15, 5, 2, 0x50d254)?);
        }
        if Self::have(source, 89, 0x50d26e)? {
            return Err(Self::luck_draw(source, 0x50d27f, 0.2, 4, 1, 0x50d29c)?);
        }
        if Self::have(source, 231, 0x50d2b1)? {
            return Err(Self::luck_draw(source, 0x50d2c7, 0.5, 10, 1, 0x50d2df)?);
        }
        if Self::have(source, 103, 0x50d2f4)? {
            return Err(Self::luck_draw(source, 0x50d30a, 0.25, 4, 1, 0x50d322)?);
        }
        if Self::have(source, 200, 0x50d337)? {
            return Err(Self::luck_draw(source, 0x50d34d, 0.334, 10, 1, 0x50d365)?);
        }
        if Self::have(source, 201, 0x50d37a)? || Self::temporary(source, 201, 0x50db8c)? {
            return Err(Self::luck_draw(source, 0x50d394, 0.334, 10, 1, 0x50d3ac)?);
        }
        if Self::have(source, 221, 0x50d3c1)? {
            plan.flags.0 |= 0x80000;
        }
        if Self::have(source, 330, 0x50d3da)? {
            plan.scale *= 0.5;
        }
        if Self::have(source, 336, 0x50d3fd)? {
            plan.scale *= 1.5;
        }
        // This second Have330 is an actual reached source query. A positive
        // result bypasses the entire conversion chain below.
        if !Self::have(source, 330, 0x50d420)? {
            let converts = Self::have(source, 230, 0x50db38)?
                || Self::have(source, 229, 0x50dd18)?
                || Self::have(source, 7, 0x50dd2d)?
                || Self::have(source, 138, 0x50dd42)?
                || Self::have(source, 80, 0x50dd57)?
                || Self::have(source, 183, 0x50dd6c)?
                || (Self::field(source.entity_variant(), 0x50dd79, "variant")? == 1
                    && Self::field(source.variant_subtype_30e8(), 0x50dd7f, "30E8")? == 13)
                || Self::temporary(source, 34, 0x50dd9b)?
                || Self::have(source, 90, 0x50ddb0)?
                || Self::temporary(source, 122, 0x50ddc5)?
                || Self::have(source, 126, 0x50ddda)?;
            if converts {
                plan.variant = if plan.variant == 11 { 12 } else { 1 };
            }
        }
        if Self::have(source, 228, 0x50d435)? || Self::have(source, 230, 0x50dba8)? {
            return Err(Self::luck_draw(source, 0x50d442, 1.0, 100, 1, 0x50d45f)?);
        }
        if Self::field(source.entity_variant(), 0x50d4b1, "variant")? == 1
            && Self::field(source.variant_subtype_30e8(), 0x50dbc0, "30E8")? == 21
        {
            return Err(Kind4Frontier::SharedDraw {
                source_pc: 0x50dbd2,
                bound: 30,
            });
        }
        if Self::have(source, 154, 0x50d4c3)? {
            if Self::field(source.turn_state_d70(), 0x50d4cc, "D70")? == -1 {
                plan.variant = 1;
                plan.damage += 2.0;
            } else if Self::field(source.source_player_type_bfc(), 0x50d4d9, "BFC")? == 2
                || Self::field(source.primary_c08(), 0x50d4e2, "C08")?
                    != PrimaryWeaponC08::RegularTears
            {
                return Err(Kind4Frontier::SharedDraw {
                    source_pc: 0x50d4f0,
                    bound: 2,
                });
            }
        }
        if Self::have(source, 254, 0x50d508)? {
            if Self::field(source.turn_state_d70(), 0x50d511, "D70")? < 0 {
                plan.color.tint = [1.0, 0.0, 0.0, 1.0];
                plan.color.offsets = [0.0; 3];
                plan.damage += 1.0;
                plan.scale *= 1.25;
                plan.height -= 10.0;
            } else if Self::field(source.source_player_type_bfc(), 0x50d51e, "BFC")? == 2
                || Self::field(source.primary_c08(), 0x50d527, "C08")?
                    != PrimaryWeaponC08::RegularTears
            {
                return Err(Kind4Frontier::SharedDraw {
                    source_pc: 0x50d535,
                    bound: 2,
                });
            }
        }
        if Self::have(source, 261, 0x50d550)? {
            plan.scale += plan.scale;
        }
        if Self::have(source, 309, 0x50d56f)? {
            plan.scale *= 1.25;
        }
        if Self::have(source, 48, 0x50d592)? || Self::have(source, 306, 0x50db18)? {
            match plan.variant {
                0 => plan.variant = 11,
                1 => plan.variant = 12,
                _ => {}
            }
        }
        if Self::have(source, 257, 0x50d5b9)? || Self::temporary(source, 257, 0x50d9c4)? {
            plan.flags.0 |= 0x400000;
            plan.variant = 5;
        }
        if Self::have(source, 259, 0x50d5dd)? || Self::temporary(source, 259, 0x50d9ec)? {
            return Err(Self::luck_draw(source, 0x50d5f7, 0.1, 3, 1, 0x50d616)?);
        }
        if Self::have(source, 317, 0x50d630)? || Self::temporary(source, 317, 0x50dafc)? {
            plan.variant = 7;
        }
        if Self::have(source, 237, 0x50d64c)? || Self::temporary(source, 237, 0x50dadc)? {
            plan.variant = 8;
        }
        if Self::have(source, 315, 0x50d668)? || Self::temporary(source, 315, 0x50dabc)? {
            plan.variant = 3;
        }
        if Self::have(source, 213, 0x50d684)? || Self::temporary(source, 213, 0x50da9c)? {
            plan.variant = 10;
        }
        if Self::have(source, 305, 0x50d6a0)? {
            let byte = 1.0_f32 / 255.0_f32;
            plan.color.tint = [50.0 * byte, 255.0 * byte, 50.0 * byte, 255.0 * byte];
            plan.color.offsets = [0.0; 3];
        }
        if Self::have(source, 150, 0x50d6b5)? || Self::temporary(source, 150, 0x50da0c)? {
            let request = Self::luck_draw(source, 0x50d6c2, 1.0, 10, 0, 0x50d6dd)?;
            if matches!(request, Kind4Frontier::SharedDraw { bound: 0, .. }) {
                // Actual42E8A3..AA returns zero before5E4FE0 when bound0.
                // Therefore50D6E6..FD is deterministic and consumes no MT.
                plan.variant = 2;
                plan.damage = 3.2_f32 * Self::finite(source.damage_c88(), 0x50d6ee, "C88")?;
            } else {
                return Err(request);
            }
        }
        // kind4 bypasses both kind5's Have125/140 and kind1's flag scaling.
        if !plan.scale.is_finite() || !plan.damage.is_finite() || !plan.height.is_finite() {
            return Err(Kind4Frontier::InvalidField {
                source_pc: 0x50d716,
                field: "Get4 result",
            });
        }
        Ok(plan)
    }

    fn field<T>(value: Option<T>, source_pc: u32, field: &'static str) -> Result<T, Kind4Frontier> {
        value.ok_or(Kind4Frontier::MissingField { source_pc, field })
    }
    fn finite(
        value: Option<f32>,
        source_pc: u32,
        field: &'static str,
    ) -> Result<f32, Kind4Frontier> {
        let value = Self::field(value, source_pc, field)?;
        if value.is_finite() {
            Ok(value)
        } else {
            Err(Kind4Frontier::InvalidField { source_pc, field })
        }
    }
    fn have(source: &impl Kind4Source, id: usize, source_pc: u32) -> Result<bool, Kind4Frontier> {
        source
            .have(id)
            .ok_or(Kind4Frontier::MissingHave { source_pc, id })
    }
    fn temporary(
        source: &impl Kind4Source,
        id: u32,
        source_pc: u32,
    ) -> Result<bool, Kind4Frontier> {
        source
            .temporary_present(id)
            .ok_or(Kind4Frontier::MissingTemporary { source_pc, id })
    }
    fn trinket(source: &impl Kind4Source, id: u32, source_pc: u32) -> Result<bool, Kind4Frontier> {
        source
            .has_trinket(id)
            .ok_or(Kind4Frontier::MissingTrinket { source_pc, id })
    }
    fn luck_draw(
        source: &impl Kind4Source,
        field_pc: u32,
        coefficient: f32,
        base: i32,
        minimum: i32,
        source_pc: u32,
    ) -> Result<Kind4Frontier, Kind4Frontier> {
        let product = Self::finite(source.luck_d64(), field_pc, "D64")? * coefficient;
        // CVTTSS2SI: out-of-range/unordered converts to the x86 indefinite
        // i32::MIN; SUB wraps before the signed CMOV bound/minimum operation.
        let truncated = if (-2147483648.0_f32..2147483648.0_f32).contains(&product) {
            product.trunc() as i32
        } else {
            i32::MIN
        };
        let bound = base.wrapping_sub(truncated).max(minimum) as u32;
        Ok(Kind4Frontier::SharedDraw { source_pc, bound })
    }
}

/// Read-only current Player inputs for the pure50D090 kind4 schedule. This
/// adapter grants no getter/caller/actor permission and writes no cache value.
struct Kind4SharedInputs<'a>(&'a crate::State);

impl Kind4SharedInputs<'_> {
    fn queries(&self) -> Option<crate::item_queries::AppearanceQueries<'_>> {
        Some(crate::item_queries::AppearanceQueries::new(
            self.0.player_items.as_ref()?,
            self.0.weapon_cache.primary,
            self.0.player.entity_source,
            None,
        ))
    }

    fn cached(&self, mask: u32) -> bool {
        self.0
            .player_items
            .as_ref()
            .and_then(|owner| owner.query_history.as_ref())
            .is_some_and(|history| history.cached_callbacks & mask == mask)
    }
}

impl Kind4Source for Kind4SharedInputs<'_> {
    fn have(&self, id: usize) -> Option<bool> {
        self.queries()?.have(id)
    }
    fn temporary_present(&self, id: u32) -> Option<bool> {
        self.queries()?.temporary_present(id)
    }
    fn has_trinket(&self, id: u32) -> Option<bool> {
        self.queries()?.has_trinket(id)
    }
    fn height_c8c(&self) -> Option<f32> {
        self.cached(8).then_some(self.0.player_stats.tear_height)
    }
    fn damage_c88(&self) -> Option<f32> {
        self.cached(1).then_some(self.0.player_stats.damage)
    }
    fn flags_c98(&self) -> Option<AttackFlags> {
        self.cached(32).then_some(self.0.weapon_cache.flags_c98)
    }
    fn color_ca0(&self) -> Option<AttackColor> {
        self.cached(64).then_some(self.0.weapon_cache.color_ca0)
    }
    fn luck_d64(&self) -> Option<f32> {
        if self.cached(1024) {
            self.0.player_stats.luck
        } else {
            None
        }
    }
    fn entity_variant(&self) -> Option<u32> {
        let source = self.0.player.entity_source?;
        (source.entity_type == 1).then_some(source.variant)
    }
    fn variant_subtype_30e8(&self) -> Option<u32> {
        None
    }
    fn turn_state_d70(&self) -> Option<i8> {
        None
    }
    fn source_player_type_bfc(&self) -> Option<u32> {
        self.0.weapon_admission.source_player_type
    }
    fn primary_c08(&self) -> Option<PrimaryWeaponC08> {
        self.queries()?.primary()
    }
}

///The getter consumes its actual current fields and query owners. It has no
///inventory roster, mutable RNG, actor permission or C70 input.
pub(crate) fn read_current_kind4(state: &crate::State) -> Result<Kind4ReadPlan, Kind4Frontier> {
    if state.weapon_admission.source_player_type != Some(0)
        || !matches!(state.player.entity_source,
            Some(source) if source.entity_type == 1 && source.variant == 0)
    {
        return Err(Kind4Frontier::MissingField {
            source_pc: 0x50d090,
            field: "actual normal Player1/0 receiver",
        });
    }
    Kind4ReadPlan::read(&Kind4SharedInputs(state))
}

///Keep every original getter alternative literal. The new branch reads only
///its reached source operands and never promotes an unknown transformation.
pub(crate) fn admit_current_knife_contact(
    state: &crate::State,
    knife: &crate::knives::Knife,
) -> Result<KnifeContactAdmission, String> {
    let owner = state
        .player_items
        .as_ref()
        .ok_or("Knife contact requires actual first Player items")?;
    let inputs = ItemInputs::new(owner)?;
    let old = admit_neutral_knife_contact(
        inputs,
        state.weapon_admission.source_player_type,
        state.player.transformation_counts,
    );
    if old != KnifeContactAdmission::UnmappedBuilderModifiers {
        state.player_stats.validate()?;
        if !state.weapon_cache.color_ca0.is_finite() {
            return Err("Knife contact requires finite actual Player color".into());
        }
        return Ok(old);
    }
    //The new getter domain cannot certify geometry produced by an earlier
    //unperformed Player controller or parent homing callback. These facts
    //are independent of current flags, C18 physical equality and the getter.
    if !state.weapon_admission.c18_controller_history_known
        || !knife.source_motion_known
        || read_current_kind4(state).is_err()
    {
        return Ok(KnifeContactAdmission::UnmappedBuilderModifiers);
    }
    Ok(admit_knife_transformation_history(
        state.player.transformation_counts,
    ))
}

///50D090 executes after actual registry insertion and returns current38.
///Other Get4 outputs are resolved in source order but this contact copies only
///damage; Knife flags/color/48C still belong to their independent real caller.
pub(crate) fn resolve_current_kind4_damage(state: &crate::State) -> Result<f32, String> {
    let inputs = ItemInputs::new(
        state
            .player_items
            .as_ref()
            .ok_or("Knife contact requires actual first Player items")?,
    )?;
    if state.weapon_admission.source_player_type == Some(0) && inputs.admits_knife_kind4_getter() {
        return resolve_neutral_kind4_damage(
            inputs,
            state.weapon_admission.source_player_type,
            state.player_stats,
        );
    }
    read_current_kind4(state)
        .map(|plan| plan.damage)
        .map_err(|frontier| format!("kind4 reached source input is unowned: {frontier:?}"))
}

///Transient51B060 caller read plan, not an inventory/actor capability.
pub(crate) struct ControlledKnifeCallerPlan {
    pub(crate) requested_satellites: usize,
}

impl ControlledKnifeCallerPlan {
    pub(crate) fn read(state: &crate::State, held_input: bool) -> Option<Self> {
        if state.weapon_admission.source_player_type != Some(0)
            || !matches!(state.player.entity_source,
                Some(source) if source.entity_type == 1 && source.variant == 0)
            || state.attack_runtime.aim_rotation_c70.is_none()
            || state.weapon_admission.pending_primary_cleanup.is_some()
            || !state.weapon_admission.c18_controller_history_known
            || (state.attack_runtime.persistent.is_none()
                && !state.weapon_admission.c18_is_known_empty)
        {
            return None;
        }
        let owner = state.player_items.as_ref()?;
        let history = owner.query_history.as_ref()?;
        //50BDD0/51B16B copy C88,C98,CA0. C8C belongs independently to
        //the later Get4 contact. Held steering additionally reads currentC7C.
        let copied = 1 | 32 | 64 | if held_input { 4 } else { 0 };
        if history.cached_callbacks & copied != copied {
            return None;
        }
        let queries = AppearanceQueries::new(
            owner,
            state.weapon_cache.primary,
            state.player.entity_source,
            None,
        );
        if queries.primary()? != PrimaryWeaponC08::MomsKnife || !queries.have(329)? {
            return None;
        }
        //519FC0's preceding217 luck proc runs only with actual held input.
        //The active323 charging prefix is likewise separate from this caller.
        if held_input && (queries.have(217)? || crate::active_charges::active_id(owner)? == 323) {
            return None;
        }
        let requested_satellites = if queries.have(153)? {
            3
        } else if queries.have(2)? {
            2
        } else if queries.have(245)? || queries.temporary_present(245)? {
            1
        } else {
            0
        };
        Some(Self {
            requested_satellites,
        })
    }
}

//The live caller separately checks requested_satellites==0 OR current copied
//flags==0 until the actual nonneutral4A0 formation operator is implemented.
//That actor restriction does not change this source query plan or its count.

pub(crate) fn resolve_neutral_kind4_damage(
    items: ItemInputs<'_>,
    source_player_type: Option<u32>,
    stats: PlayerStats,
) -> Result<f32, String> {
    stats.validate()?;
    if source_player_type != Some(0) || !items.admits_knife_kind4_getter() {
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
    source_player_type == Some(0)
        && (items.admits_neutral_kind2() || items.admits_soy_milk_kind2_getter())
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

    ///50D3E3/3EB applies the Soy size operator before541FF0's .01 bound.
    ///Selected finite nonnegative damage makes the earlier scale() bound a
    ///no-op (raw scale>=.6); preserve the actual post-multiply bound too.
    fn with_soy_milk_scale(mut self) -> Result<Self, String> {
        self.scale = (self.scale * 0.5_f32).max(0.01_f32);
        self.animation_tier = projectiles::regular_animation_tier(self.scale)?;
        self.collision_radius = projectiles::collision_radius(self.scale, 8.0)?;
        Ok(self)
    }

    /// FireTear's 50D090(kind1) result: damage (vtable+60), flags (541810),
    /// size under 541FF0's .01 bound, color (vtable+58) and height (541790).
    pub(crate) fn with_getter(
        mut self,
        properties: &crate::tear_getter::TearProperties,
    ) -> Result<Self, String> {
        self.damage = properties.damage;
        self.flags = properties.flags;
        self.color = properties.color;
        self.scale = properties.scale.max(0.01_f32);
        if !self.scale.is_finite() || !self.damage.is_finite() || !properties.height.is_finite() {
            return Err("50D090 produced a nonfinite tear property".into());
        }
        self.animation_tier = projectiles::regular_animation_tier(self.scale)?;
        self.collision_radius = projectiles::collision_radius(self.scale, 8.0)?;
        self.parameters.height = properties.height;
        Ok(self)
    }

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

///Independent selected50D090(kind1) result for ordinary2. Layout, eye,
///clock, RNG and actor operations stay with its caller and canonical factory.
pub(crate) fn resolve_inner_eye_kind1(state: &State) -> Result<RegularTearPlan, String> {
    let inputs = ItemInputs::new(
        state
            .player_items
            .as_ref()
            .ok_or("Inner Eye items absent")?,
    )?;
    if state.weapon_admission.source_player_type != Some(0)
        || state.weapon_cache.primary != PrimaryWeaponC08::RegularTears
        || state.weapon_cache.flags_c98.0 != 0
        || !inputs.admits_inner_eye_kind1_context()
        || inputs.pending_cache().0 & RegularTearPlan::INPUT_CACHE_MASK.0 != 0
    {
        return Err("Inner Eye kind1 requires its actual current cache/context".into());
    }
    RegularTearPlan::resolve(state.player_stats, state.max_fire_delay, state.weapon_cache)
}

///Independent selected50D090(kind1) result for the229 burst. The common
///scale/range/color/damage arithmetic matches the selected regular builder;
///50DD18/50DB45 produces BloodTear1 instead of ordinary variant0.
pub(crate) fn resolve_monstro_kind1(state: &State) -> Result<RegularTearPlan, String> {
    let inputs = ItemInputs::new(
        state
            .player_items
            .as_ref()
            .ok_or("Monstro kind1 item owner absent")?,
    )?;
    if state.weapon_admission.source_player_type != Some(0)
        || state.weapon_cache.primary != PrimaryWeaponC08::MonstrosLung
        || !inputs.admits_monstro_kind1_context()
        || inputs.pending_cache().0 & RegularTearPlan::INPUT_CACHE_MASK.0 != 0
        || state.weapon_cache.flags_c98.0 != 0
    {
        return Err("Monstro kind1 requires its independent current cache/context".into());
    }
    RegularTearPlan::resolve(state.player_stats, state.max_fire_delay, state.weapon_cache)
}

///Actual selected50D090(kind1) for positive152: source variant0 and current
///C88/C98/CA0/kinematics. The secondary mode1 factory does not use this getter.
///Current bit9 is copied by kind1 without changing its source variant0.
///Independent caller, cache and born-body permissions remain separate.
pub(crate) fn resolve_technology2_kind1(state: &State) -> Result<RegularTearPlan, String> {
    let inputs = ItemInputs::new(
        state
            .player_items
            .as_ref()
            .ok_or("Technology2 item owner absent")?,
    )?;
    if state.weapon_admission.source_player_type != Some(0)
        || state.weapon_cache.primary != PrimaryWeaponC08::RegularTears
        || !inputs.admits_technology2_kind1_getter()
        || inputs.pending_cache().0 & RegularTearPlan::INPUT_CACHE_MASK.0 != 0
        || !matches!(state.weapon_cache.flags_c98.0, 0 | 0x200)
    {
        return Err("Technology2 kind1 requires its current source-owned context".into());
    }
    RegularTearPlan::resolve(state.player_stats, state.max_fire_delay, state.weapon_cache)
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
    if !inputs.admits_neutral_kind3()
        && !inputs.admits_technology_kind3_getter()
        && !inputs.admits_soy_milk_kind3_getter()
        && !inputs.admits_technology2_kind3_getter()
    {
        return Ok(LaserStatusAdmission::UnmappedBuilderModifiers);
    }
    //The current kind3 status predicate is independent of born Laser flags.
    //0x200 is source-inert here, even when a retained C40 was born with0.
    if state.weapon_cache.flags_c98.has_laser_status_effect() {
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
