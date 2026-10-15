//! Actual requested cache fields and immutable regular-Tear birth values.
//! Actor slots, charge, input and random draws retain their own owners.
use crate::{
    attack_color::AttackColor,
    attack_flags::AttackFlags,
    item_pipeline::{CacheMask, ItemInputs},
    player_stats::PlayerStats,
    projectiles,
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

/// Source context admission, not a replacement native pointer. Main has no
/// nonregular actor producer yet, so only actual fresh NULL C18 is admitted.
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
    if source_player_type == Some(0) && items.admits_selected_appearance() {
        if requested.0 & 32 != 0 {
            prior.flags_c98 = AttackFlags(if prior.primary == PrimaryWeaponC08::Ludovico {
                3
            } else {
                0
            });
            completed |= 32;
        }
        if requested.0 & 64 != 0 {
            prior.color_ca0 = AttackColor::WHITE;
            prior.color_ccc = AttackColor::WHITE;
            completed |= 64;
        }
    }
    (prior, CacheMask(completed))
}

/// Live invocation support is distinct from requested selector/cache writes.
/// An unsupported family never consumes regular birth RNG or synthesizes a
/// Tear. Its persistent diagnostic is independent of ordinary body/tail work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DispatchAdmission {
    Regular,
    /// Ordered concrete adapter effect: retain diagnostic and invalidate the
    /// fresh NULL-slot proof. No actor/RNG/property effects are invented.
    Unadmitted(&'static str),
}

pub(crate) fn resolve_dispatch_admission(
    cache: WeaponCache,
    source_player_type: Option<u32>,
    stats: PlayerStats,
    pending_primary_cleanup: Option<PrimaryCleanupIntent>,
) -> Result<DispatchAdmission, String> {
    stats.validate()?;
    if !cache.color_ca0.is_finite() || !cache.color_ccc.is_finite() {
        return Err("weapon cache contains nonfinite source color owners".into());
    }
    if pending_primary_cleanup.is_some() {
        return Ok(DispatchAdmission::Unadmitted("pendingPrimaryCleanup"));
    }
    if source_player_type != Some(0) {
        return Ok(DispatchAdmission::Unadmitted("weaponContext"));
    }
    if cache.primary != PrimaryWeaponC08::RegularTears {
        return Ok(DispatchAdmission::Unadmitted("weaponActorFamily"));
    }
    Ok(DispatchAdmission::Regular)
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
