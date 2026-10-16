//! Pure resolution of the mapped Player cache callbacks.
//!
//! Item predicates retain their distinct source meanings. Actor removal,
//! admission, audio, and random draws belong to the caller/backend.
use crate::{
    items, player_items::PlayerItems, player_stats::PlayerStats, weapon_plan::PrimaryWeaponC08,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CacheMask(pub(crate) u32);

const LIBRA_SCALAR_FLAGS: u32 = 0x1b;

//Actual152 requestsWeapon/Damage; optional4 separately requestsShotSpeed.
//These callback/grant footprints do not admit a primary or secondary caller.
const TECHNOLOGY2_GRANT_ITEMS: &[usize] = &[4, 152];

// Source-owned footprints remain independent even when their admitted counts
// currently match. Cache completion is never a kind5 or grant-history proof.
const FETUS_CACHE_ITEMS: &[usize] = &[4, 52];
const FETUS_KIND5_ITEMS: &[usize] = &[4, 52];
const FETUS_GRANT_ITEMS: &[usize] = &[4, 52];
//Independent controlled52 cache, kind5 and caller footprints.
const CONTROLLED_FETUS_CACHE_ITEMS: &[usize] = &[1, 4, 52, 329];
const CONTROLLED_FETUS_KIND5_ITEMS: &[usize] = &[1, 4, 52, 329];
const CONTROLLED_FETUS_GRANT_ITEMS: &[usize] = &[1, 4, 52, 329];
const CONTROLLED_FETUS_CALLER_ITEMS: &[usize] = &[1, 4, 52, 329];
// Independent current caller and kind5 readsets do not consume the queued
// passive4 config. Pickup guard release precedes its late real Add callback.
const CONTROLLED_FETUS_KIND5_QUEUED_ITEMS: &[usize] = &[4];
const CONTROLLED_FETUS_CALLER_QUEUED_ITEMS: &[usize] = &[4];

// Independent Monstro229 callback, Familiar, grant and kind1 footprints.
// Equal sets do not transfer controller, getter or auxiliary ownership.
const MONSTRO_CACHE_ITEMS: &[usize] = &[1, 4, 229];
const MONSTRO_FAMILIAR_ITEMS: &[usize] = &[1, 4, 229];
const MONSTRO_GRANT_ITEMS: &[usize] = &[1, 4, 229];
const MONSTRO_KIND1_ITEMS: &[usize] = &[1, 4, 229];
const MONSTRO_CALLER_ITEMS: &[usize] = &[1, 4, 229];
// Independent current getter/caller histories admit actual queued passive4.
// Its late Add has not changed current stats or selected229 costume yet.
const MONSTRO_KIND1_QUEUED_ITEMS: &[usize] = &[4];
const MONSTRO_CALLER_QUEUED_ITEMS: &[usize] = &[4];

// Independently closed cache, Familiar and actual grant footprints. Equal
// sets do not transfer getter or controller admission.
const EPIC_FETUS_CACHE_ITEMS: &[usize] = &[1, 2, 4, 168];
const EPIC_FETUS_FAMILIAR_ITEMS: &[usize] = &[1, 2, 4, 168];
const EPIC_FETUS_GRANT_ITEMS: &[usize] = &[1, 2, 4, 168];
// Real2-before168 grants have independent callback and auxiliary histories.
// These prefixes admit no ordinary triple-tear emitter or Epic caller.
const INNER_EYE_PREFIX_CACHE_ITEMS: &[usize] = &[1, 2, 4];
const INNER_EYE_PREFIX_FAMILIAR_ITEMS: &[usize] = &[1, 2, 4];
const INNER_EYE_PREFIX_GRANT_ITEMS: &[usize] = &[1, 2, 4];

// Independently closed getter and live caller footprints. Cache/Familiar
// and grant ownership above never admits these consumers by itself.
const EPIC_FETUS_KIND6_ITEMS: &[usize] = &[1, 2, 4, 168];
const EPIC_FETUS_CALLER_ITEMS: &[usize] = &[1, 2, 4, 168];
// Actual queued passive2 supplies no early owned count or cache value.
const EPIC_FETUS_KIND6_QUEUED_ITEMS: &[usize] = &[2];
const EPIC_FETUS_CALLER_QUEUED_ITEMS: &[usize] = &[2];

//519FC0's ordinary Have2 emitter and50D090(kind1) have separate readsets.
//Neither follows from the existing cache/Familiar/grant prefix capabilities.
const INNER_EYE_CALLER_ITEMS: &[usize] = &[2, 4];
const INNER_EYE_KIND1_ITEMS: &[usize] = &[2, 4];
const INNER_EYE_CALLER_QUEUED_ITEMS: &[usize] = &[2, 4];
const INNER_EYE_KIND1_QUEUED_ITEMS: &[usize] = &[2, 4];

// Independent source-owned footprints. Equal current sets are deliberately
// separate: cache additions cannot silently admit a50D090 getter branch.
const TECHNOLOGY_CACHE_ITEMS: &[usize] = &[1, 4, 68, 329];
const TECHNOLOGY_KIND2_ITEMS: &[usize] = &[1, 4, 68];
const TECHNOLOGY_KIND3_ITEMS: &[usize] = &[1, 4, 68, 329];
const CONTROLLED_TECHNOLOGY_CALLER_ITEMS: &[usize] = &[1, 4, 68, 329];
const SOY_CACHE_ITEMS: &[usize] = &[1, 4, 118, 330];
const SOY_KIND1_ITEMS: &[usize] = &[1, 4, 118, 330];
const SOY_KIND2_ITEMS: &[usize] = &[1, 4, 118, 330];
const SOY_KIND3_ITEMS: &[usize] = &[1, 4, 118, 330];

// Independent169 callbacks, kind1 getter and current519FC0 caller footprints.
// Equal selected sets do not complete grant/history/contact or other stages.
const POLYPHEMUS_CALLER_ITEMS: &[usize] = &[4, 169];

/// Literal511f10 dispatch order. Weapon selection/removal precedes stats;
/// Libra304 has additional coupling before and after this sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CacheStage {
    Weapon,
    FireDelay,
    ShotSpeed,
    Damage,
    Range,
    MoveSpeed,
    TearFlags,
    Color,
    Flight,
    Familiar,
    Luck,
}

impl CacheStage {
    pub(crate) const ORDER: [Self; 11] = [
        Self::Weapon,
        Self::FireDelay,
        Self::ShotSpeed,
        Self::Damage,
        Self::Range,
        Self::MoveSpeed,
        Self::TearFlags,
        Self::Color,
        Self::Flight,
        Self::Familiar,
        Self::Luck,
    ];

    const fn flag(self) -> u32 {
        match self {
            Self::Weapon => 0x100,
            Self::FireDelay => FireDelayStage::REQUESTED.0,
            Self::ShotSpeed => 4,
            Self::Damage => DamageStage::REQUESTED.0,
            Self::Range => 8,
            Self::MoveSpeed => 16,
            Self::TearFlags => 32,
            Self::Color => 64,
            Self::Flight => 128,
            Self::Familiar => 512,
            Self::Luck => 1024,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ItemInputs<'a> {
    owner: &'a PlayerItems,
}

impl<'a> ItemInputs<'a> {
    pub(crate) fn new(owner: &'a PlayerItems) -> Result<Self, String> {
        if owner.owned_counts.len() != 347 {
            return Err("Player cache requires all347 actual owned counts".into());
        }
        Ok(Self { owner })
    }

    /// Actual D6C requests still uncompleted by their source callbacks.
    pub(crate) fn pending_cache(self) -> CacheMask {
        CacheMask(self.owner.unevaluated_cache_flags)
    }

    pub(crate) fn owned_count(self, id: usize) -> i32 {
        self.owner.owned_counts[id]
    }

    /// Positive raw permanent count. Actual5090c0 has additional327/328
    /// predicates, so this is not a universal HaveCollectible accessor.
    pub(crate) fn positive_owned_count(self, id: usize) -> bool {
        self.owned_count(id) > 0
    }

    /// Literal5090C0: only requested327/328 enter its trinket/age branch;
    /// every admitted C08 selector query below loads signed E1C+4*ID.
    /// Unknown query is rejected, never promoted to a generic Have accessor.
    pub(crate) fn primary_permanent_presence(self, id: usize) -> Result<bool, String> {
        match id {
            168 | 114 | 52 | 118 | 68 | 329 | 229 => Ok(self.owned_count(id) > 0),
            _ => Err("primary selector queried an unadmitted HaveCollectible domain".into()),
        }
    }

    pub(crate) fn temporary_count(self, id: u32) -> i32 {
        self.owner.temporary_count(id)
    }

    ///514FAF/5E3D70 tests Number One's collectible record, not its count.
    ///The actual config for ID6 is kind1; this is no universal TEMP getter.
    fn number_one_record_present(self) -> bool {
        self.owner
            .temporary_collectibles
            .iter()
            .any(|record| record.item_id == 6)
    }

    pub(crate) fn number_one_present(self) -> bool {
        self.positive_owned_count(6) || self.number_one_record_present()
    }

    ///50D090's kind1 result word reaches Tear+14 through517896/550A40.
    ///Only this closed neutral producer returns0; unknown modifiers retain
    ///an unknown owner, independently of regular-shot dispatch admission.
    pub(crate) fn neutral_regular_tear_variant(
        self,
        source_player_type: Option<u32>,
        primary: PrimaryWeaponC08,
        flags: crate::attack_flags::AttackFlags,
    ) -> Option<u32> {
        if self.admits_soy_milk_regular_properties(source_player_type, primary, flags) {
            return Some(0);
        }
        if source_player_type == Some(0)
            && primary == PrimaryWeaponC08::RegularTears
            && flags.0 == 0x200
            && self.admits_polyphemus_kind1_context()
        {
            return Some(0);
        }
        (source_player_type == Some(0)
            && primary == PrimaryWeaponC08::RegularTears
            && flags.0 == 0
            && self.admits_scalar_context()
            && self
                .owner
                .owned_counts
                .iter()
                .enumerate()
                .all(|(id, &count)| count == 0 || (count > 0 && [1, 4, 6].contains(&id))))
        .then_some(0)
    }

    ///Independent cache32/64 footprint for positive6 with selected1/4/114/118/329/304.
    ///This never widens a controller, contact or temporary-effect frontier.
    pub(crate) fn admits_number_one_appearance(self) -> bool {
        self.positive_owned_count(6)
            && self.owner.trinket_slots == [0, 0]
            && self
                .owner
                .queued_id
                .is_none_or(|id| items::definition(id).is_some_and(|c| c.kind != 2))
            && self.owner.temporary_collectibles.is_empty()
            && self
                .owner
                .owned_counts
                .iter()
                .enumerate()
                .all(|(id, &count)| {
                    count == 0 || (count > 0 && [1, 4, 6, 114, 118, 329, 304].contains(&id))
                })
    }

    ///Literal51316D's selected Flags32 callback. No color/grant permission.
    pub(crate) fn admits_polyphemus_flags_context(self) -> bool {
        TearFlagsStage::admits_polyphemus(self)
    }

    ///Separate50D090(kind1) readset;169 adds no modifier query or RNG here.
    pub(crate) fn admits_polyphemus_kind1_context(self) -> bool {
        self.positive_owned_count(169) && self.admits_kind1_neutral_fields()
    }

    ///Actual519FC0 Have169 changes the born origin after the eye draw. This
    ///capability says nothing about the later Tear's0x200 contact callback.
    pub(crate) fn admits_polyphemus_caller_context(self) -> bool {
        self.admits_permanent_readset(169, POLYPHEMUS_CALLER_ITEMS)
    }

    ///Independent positive330 callback footprint from full511F10 and the
    ///actual zero-amount Add330 path. No TEMP/trinkets or unknown grants.
    ///Kind-specific property consumers retain independent capabilities.
    pub(crate) fn admits_soy_milk_cache_context(self) -> bool {
        self.admits_permanent_readset(330, SOY_CACHE_ITEMS)
    }

    ///Independently closed50D090(kind1) outputs00=0 and3C*=.5.
    ///The cache footprint alone never grants a kind-specific property owner.
    pub(crate) fn admits_soy_milk_regular_properties(
        self,
        source_player_type: Option<u32>,
        primary: PrimaryWeaponC08,
        flags: crate::attack_flags::AttackFlags,
    ) -> bool {
        source_player_type == Some(0)
            && primary == PrimaryWeaponC08::RegularTears
            && flags.0 == 0
            && self.admits_permanent_readset(330, SOY_KIND1_ITEMS)
    }

    ///Selected positive330 changes only unused3C in kind2/3; actual38/30
    ///remain currentC88/C98. Factory50E4AF and476D70 consume those separately.
    pub(crate) fn admits_soy_milk_kind2_getter(self) -> bool {
        self.admits_permanent_readset(330, SOY_KIND2_ITEMS)
    }

    pub(crate) fn admits_soy_milk_kind3_getter(self) -> bool {
        self.admits_permanent_readset(330, SOY_KIND3_ITEMS)
    }

    /// Exact per-callback cache32/64 proof: other modifier and trinket owners
    /// remain outside this scope. Future property operators extend their own
    /// cache reads instead of widening controller/contact admission.
    pub(crate) fn admits_selected_appearance(self) -> bool {
        if self.admits_technology_cache_context() || self.admits_fetus_cache_context() {
            return true;
        }
        if self.admits_soy_milk_cache_context() || self.admits_monstro_cache_context() {
            return true;
        }
        if self.admits_epic_fetus_cache_context() {
            return true;
        }
        self.owner.trinket_slots == [0, 0]
            && self
                .owner
                .queued_id
                .is_none_or(|id| items::definition(id).is_some_and(|c| c.kind != 2))
            && self.owner.temporary_collectibles.is_empty()
            && self
                .owner
                .owned_counts
                .iter()
                .enumerate()
                .all(|(id, &count)| {
                    count == 0 || (count > 0 && [1, 4, 114, 118, 329, 257, 304].contains(&id))
                })
    }

    ///Actual519FC0 primary and52F370 secondary caller footprints remain distinct.
    pub(crate) fn admits_technology2_primary(self) -> bool {
        self.positive_owned_count(152)
            && self.admits_stage_owned_roles(|id| Technology2PrimaryReadRole::of(id).is_some())
    }
    pub(crate) fn admits_technology2_secondary(self) -> bool {
        self.positive_owned_count(152)
            && self.admits_stage_owned_roles(|id| Technology2SecondaryReadRole::of(id).is_some())
    }
    ///Selected50D090 kind1 and kind3 never inherit a factory/cache permission.
    pub(crate) fn admits_technology2_kind1_getter(self) -> bool {
        self.positive_owned_count(152) && self.admits_kind1_neutral_fields()
    }
    pub(crate) fn admits_technology2_kind3_getter(self) -> bool {
        self.positive_owned_count(152) && self.admits_kind3_neutral_fields()
    }

    ///Selected50D090(kind1): these items add no modifier query/RNG/variant.
    ///Current C98/all11CA0/C88/C8C remain actual copied inputs, not neutralized
    ///values. Consumers still own their supported property/body/caller flags.
    fn admits_kind1_neutral_fields(self) -> bool {
        self.admits_stage_owned_roles(|id| matches!(id, 4 | 152 | 169))
    }

    ///Independent50D090(kind3) proof for its current output fields. Equal
    ///neutral roles do not transfer kind1 or retained C40 ownership here.
    fn admits_kind3_neutral_fields(self) -> bool {
        self.admits_stage_owned_roles(|id| matches!(id, 4 | 152 | 169))
    }

    ///Mechanical context/count validation only; each stage classifies its own
    ///source-proven roles.508FE0 reads queued3058 only for configkind2, so a
    ///known passive config is inert here before lateAdd. Unknown configs stay
    ///unknown. No caller queue, grant or cache permission follows from this.
    fn admits_stage_owned_roles(self, owns_role: impl Fn(usize) -> bool) -> bool {
        self.admits_scalar_context()
            && self
                .owner
                .owned_counts
                .iter()
                .enumerate()
                .all(|(id, &count)| count == 0 || (count > 0 && owns_role(id)))
    }

    ///Actualpassive152 defaultAdd preserves only an existing auxiliary owner.
    pub(crate) fn admits_technology2_grant_context(self) -> bool {
        self.admits_permanent_readset(152, TECHNOLOGY2_GRANT_ITEMS)
    }

    fn admits_scalar_context(self) -> bool {
        self.owner.trinket_slots == [0, 0]
            && self
                .owner
                .queued_id
                .is_none_or(|id| items::definition(id).is_some_and(|config| config.kind != 2))
            && self.owner.temporary_collectibles.is_empty()
    }

    /// Independent511F10 footprint for positive Technology with selected1/4/329.
    /// Callers retain their normal-Isaac and typed auxiliary-owner checks.
    /// This is cache context only, never a property/controller/contact admission.
    pub(crate) fn admits_technology_cache_context(self) -> bool {
        self.admits_permanent_readset(68, TECHNOLOGY_CACHE_ITEMS)
    }

    ///Separate50D090 output38 footprint; cache coverage is not this owner.
    pub(crate) fn admits_technology_kind2_getter(self) -> bool {
        self.admits_permanent_readset(68, TECHNOLOGY_KIND2_ITEMS)
    }

    ///Separate50D090 output30 footprint consumed by476D70 status handling.
    pub(crate) fn admits_technology_kind3_getter(self) -> bool {
        self.admits_permanent_readset(68, TECHNOLOGY_KIND3_ITEMS)
    }

    ///519FC0 C08=3 + live Have329 reads current cache directly; it never
    ///calls the ordinary kind2 builder. Keep its caller footprint separate
    ///from cache completion and the independently closed kind3 status getter.
    pub(crate) fn admits_controlled_technology_context(self) -> bool {
        self.positive_owned_count(329)
            && self.admits_permanent_readset(68, CONTROLLED_TECHNOLOGY_CALLER_ITEMS)
    }

    ///Independent511F10 footprint.229 is neutral for the selected callbacks;
    ///FireDelay separately consumes the actual retained C08=7 multiplier.
    pub(crate) fn admits_monstro_cache_context(self) -> bool {
        self.admits_monstro_readset(MONSTRO_CACHE_ITEMS)
    }

    ///Independent strict511F10 path requests the same68 zero familiars.
    pub(crate) fn admits_monstro_zero_familiar_requests(self) -> bool {
        self.admits_monstro_readset(MONSTRO_FAMILIAR_ITEMS)
    }

    ///Only actualAdd229 may preserve an existing auxiliary owner. Unknown
    ///restore histories remain None regardless of the current item counts.
    pub(crate) fn admits_monstro_grant_context(self) -> bool {
        self.admits_monstro_readset(MONSTRO_GRANT_ITEMS)
    }

    ///Separate50D090(kind1) path: Have229 transforms the actual initial0
    ///variant to BloodTear1. This grants no controller or cache capability.
    pub(crate) fn admits_monstro_kind1_context(self) -> bool {
        self.admits_monstro_readset(MONSTRO_KIND1_ITEMS)
            && self
                .owner
                .queued_id
                .is_none_or(|id| MONSTRO_KIND1_QUEUED_ITEMS.contains(&(id as usize)))
    }

    ///Separate519FC0 C08=7 caller footprint. Has149/55/87/153/2/245
    ///are false; cache and kind1 admission do not grant this controller.
    pub(crate) fn admits_monstro_caller(self) -> bool {
        self.admits_monstro_readset(MONSTRO_CALLER_ITEMS)
            && self
                .owner
                .queued_id
                .is_none_or(|id| MONSTRO_CALLER_QUEUED_ITEMS.contains(&(id as usize)))
    }

    fn admits_monstro_readset(self, supported: &[usize]) -> bool {
        self.admits_permanent_readset(229, supported)
            && self
                .owner
                .queued_id
                .is_none_or(|id| supported.contains(&(id as usize)))
    }

    ///Independent511F10 footprint for positive168 with selected1/2/4,
    ///or the actual2-before168 prefix. C08=6 adds no FireDelay multiplier.
    ///Normal-Isaac, requested callbacks and field/history owners stay separate.
    pub(crate) fn admits_epic_fetus_cache_context(self) -> bool {
        self.admits_permanent_readset(168, EPIC_FETUS_CACHE_ITEMS)
            || self.admits_permanent_readset(2, INNER_EYE_PREFIX_CACHE_ITEMS)
    }

    ///Independent strict511F10 trace: all68 actual Familiar requests are0.
    ///Cache completion alone never admits a Familiar request or RNG owner.
    pub(crate) fn admits_epic_fetus_zero_familiar_requests(self) -> bool {
        self.admits_permanent_readset(168, EPIC_FETUS_FAMILIAR_ITEMS)
            || self.admits_permanent_readset(2, INNER_EYE_PREFIX_FAMILIAR_ITEMS)
    }

    ///Separate50D090(kind6) footprint used by Rocket31 at death. The consumer
    ///retains actualFIRSTPlayer/source0 and currentC88/C98/CA0 ownership;
    ///physicalParent and the Target's birth inventory are not this getter.
    pub(crate) fn admits_epic_fetus_kind6_context(self) -> bool {
        self.owner
            .queued_id
            .is_none_or(|id| EPIC_FETUS_KIND6_QUEUED_ITEMS.contains(&(id as usize)))
            && self.admits_permanent_readset(168, EPIC_FETUS_KIND6_ITEMS)
    }

    ///Only actualAdd2/168 may consume this independent zero-amount grant proof,
    ///with normal-Isaac and existingSomeLuck guards retained by its wrapper.
    ///No counts or cache result can create missing auxiliary history.
    pub(crate) fn admits_epic_fetus_grant_context(self) -> bool {
        self.admits_permanent_readset(168, EPIC_FETUS_GRANT_ITEMS)
            || self.admits_permanent_readset(2, INNER_EYE_PREFIX_GRANT_ITEMS)
    }

    ///Separate51A160/51E370/51B7C3/51B859/51DA6C caller footprint. Selected
    ///Has2 is current signed ownership;153/55/87 are absent. Source0, D80,
    ///charge, C18, RNG and canonicalTarget/Rocket remain caller obligations.
    pub(crate) fn admits_epic_fetus_caller(self) -> bool {
        self.owner
            .queued_id
            .is_none_or(|id| EPIC_FETUS_CALLER_QUEUED_ITEMS.contains(&(id as usize)))
            && self.admits_permanent_readset(168, EPIC_FETUS_CALLER_ITEMS)
    }

    ///51A420..51F3D7: normal C08=1, actual Have2 and no other emitter modifiers.
    pub(crate) fn admits_inner_eye_caller(self) -> bool {
        self.owner
            .queued_id
            .is_none_or(|id| INNER_EYE_CALLER_QUEUED_ITEMS.contains(&(id as usize)))
            && self.admits_permanent_readset(2, INNER_EYE_CALLER_ITEMS)
    }

    ///Independent selected50D090(kind1) trace; passive queued2/4 do not enter
    ///508FE0's queued-config-kind2 branch. Actual current cache stays separate.
    pub(crate) fn admits_inner_eye_kind1_context(self) -> bool {
        self.owner
            .queued_id
            .is_none_or(|id| INNER_EYE_KIND1_QUEUED_ITEMS.contains(&(id as usize)))
            && self.admits_permanent_readset(2, INNER_EYE_KIND1_ITEMS)
    }

    ///Mechanical signed-count/context check, with no item capability implied
    ///by the checker itself. Each caller supplies its own source-owned set.
    fn admits_permanent_readset(self, required: usize, supported: &[usize]) -> bool {
        self.positive_owned_count(required)
            && self.owner.trinket_slots == [0, 0]
            && self
                .owner
                .queued_id
                .is_none_or(|id| items::definition(id).is_some_and(|config| config.kind != 2))
            && self.owner.temporary_collectibles.is_empty()
            && self
                .owner
                .owned_counts
                .iter()
                .enumerate()
                .all(|(id, &count)| count == 0 || (count > 0 && supported.contains(&id)))
    }

    /// Independent511F10 footprint for one actual52 and selected4. The
    /// controlled path retains its separate source-owned cache footprint.
    /// Callback callers retain actual normal-Isaac and auxiliary owners.
    pub(crate) fn admits_fetus_cache_context(self) -> bool {
        if self.admits_controlled_fetus_readset(
            CONTROLLED_FETUS_CACHE_ITEMS,
            CONTROLLED_FETUS_CACHE_ITEMS,
        ) {
            return true;
        }
        self.owner.queued_id.is_none_or(|id| matches!(id, 4 | 52))
            && self.admits_fetus_readset(FETUS_CACHE_ITEMS, Some(1))
    }

    ///Separate50D090(kind5) footprints retain the125/140 tail proof and
    ///separate queue readsets. Controlled kind5 also admits actual passive4
    ///queued before its late Add; current C98/C88/CA0 remain independently owned.
    pub(crate) fn admits_fetus_kind5_context(self) -> bool {
        if self.admits_controlled_fetus_readset(
            CONTROLLED_FETUS_KIND5_ITEMS,
            CONTROLLED_FETUS_KIND5_QUEUED_ITEMS,
        ) {
            return true;
        }
        self.owner.queued_id.is_none() && self.admits_fetus_readset(FETUS_KIND5_ITEMS, Some(1))
    }

    ///Only the actual Add52 wrapper consumes this independent grant proof.
    ///It preserves existing SomeLuck without creating a missing history.
    pub(crate) fn admits_fetus_grant_context(self) -> bool {
        if self.admits_controlled_fetus_readset(
            CONTROLLED_FETUS_GRANT_ITEMS,
            CONTROLLED_FETUS_GRANT_ITEMS,
        ) {
            return true;
        }
        self.owner.queued_id.is_none_or(|id| matches!(id, 4 | 52))
            && self.admits_fetus_readset(FETUS_GRANT_ITEMS, Some(1))
    }

    ///519FC0's currentC7C motion branch is separate from cache and kind5.
    pub(crate) fn admits_controlled_fetus_caller(self) -> bool {
        self.admits_controlled_fetus_readset(
            CONTROLLED_FETUS_CALLER_ITEMS,
            CONTROLLED_FETUS_CALLER_QUEUED_ITEMS,
        )
    }

    fn admits_controlled_fetus_readset(self, supported: &[usize], queued: &[usize]) -> bool {
        self.positive_owned_count(329)
            && self.admits_fetus_readset(supported, None)
            && self
                .owner
                .queued_id
                .is_none_or(|id| queued.contains(&(id as usize)))
    }

    ///Mechanical signed-count/context validation, independent of the caller's
    ///semantic item set. Both source proofs require exactly one52; their
    ///separate count bounds and queue rules are explicit inputs, not inherited
    ///cache/getter/grant/controller permissions.
    fn admits_fetus_readset(self, supported: &[usize], maximum_count: Option<i32>) -> bool {
        self.owned_count(52) == 1
            && self.owner.trinket_slots == [0, 0]
            && self.owner.temporary_collectibles.is_empty()
            && self
                .owner
                .owned_counts
                .iter()
                .enumerate()
                .all(|(id, &count)| {
                    count == 0
                        || (count > 0
                            && supported.contains(&id)
                            && maximum_count.is_none_or(|maximum| count <= maximum))
                })
    }

    /// Each callback composes its implemented effects with its proven neutral
    /// read set. C08 remains an independent input to the ordered formulas.
    /// Config 118's request for all callbacks adds no implicit scalar effect.
    fn admits_scalar_stage(self, stage: CacheStage, effects: &[u32]) -> bool {
        if self.owned_count(152) != 0 {
            //New stage-local capabilities resolve their own reads. This is
            //only the preserved legacy scalar compatibility boundary.
            return false;
        }
        if self.owned_count(169) != 0 {
            //Its effect is owned by each stage, never legacy compatibility.
            return false;
        }
        //330's grant/readset context is separate from its two effect lists.
        if self.owned_count(330) != 0 && !self.admits_soy_milk_cache_context() {
            return false;
        }
        let neutral_soy_milk = matches!(
            stage,
            CacheStage::ShotSpeed | CacheStage::Range | CacheStage::MoveSpeed
        ) && self.admits_soy_milk_cache_context();
        let neutral: &[u32] = match stage {
            CacheStage::FireDelay => FireDelayStage::NEUTRAL_ITEMS,
            CacheStage::ShotSpeed => &[1, 6, 114, 118, 329, 304],
            CacheStage::Damage => DamageStage::NEUTRAL_ITEMS,
            CacheStage::Range => &[1, 4, 114, 118, 329],
            CacheStage::MoveSpeed => &[1, 4, 6, 114, 118, 329],
            _ => return false,
        };
        //304 owns a separate post-callback operator for these four fields.
        //It is neutral only for ShotSpeed; do not hide its dependency in the
        //other callback neutral lists or evaluate it as a loadout-only fold.
        let coupled_libra = stage.flag() & LIBRA_SCALAR_FLAGS != 0;
        let neutral_technology = self.admits_technology_cache_context();
        //52 is neutral for these scalar callbacks except the separately
        //implemented retained-C08 FireDelay multiplier below.
        let fetus_context = self.admits_fetus_cache_context();
        let neutral_monstro = self.admits_monstro_cache_context();
        let epic_fetus_context = self.admits_epic_fetus_cache_context();
        self.admits_scalar_context()
            && self
                .owner
                .owned_counts
                .iter()
                .enumerate()
                .all(|(id, &count)| {
                    count == 0
                        || (count > 0
                            && (effects.contains(&(id as u32))
                                || neutral.contains(&(id as u32))
                                || (id == 304 && coupled_libra)
                                || (id == 68 && neutral_technology)
                                || (id == 330 && neutral_soy_milk)
                                || (id == 52 && fetus_context)
                                || (id == 229 && neutral_monstro)
                                || (id == 168 && epic_fetus_context)
                                || (id == 2 && epic_fetus_context)))
                })
    }

    /// Auxiliary callbacks have their own narrower source proof. Scalar
    /// effects admitted above do not expand Flight, Familiar, Luck or history.
    fn admits_selected_auxiliary_callbacks(self) -> bool {
        if self.admits_technology_cache_context() || self.admits_fetus_cache_context() {
            return true;
        }
        if self.admits_soy_milk_cache_context() || self.admits_monstro_cache_context() {
            return true;
        }
        if self.admits_epic_fetus_cache_context() {
            return true;
        }
        self.admits_scalar_context()
            && self
                .owner
                .owned_counts
                .iter()
                .enumerate()
                .all(|(id, &count)| {
                    count == 0 || (count > 0 && [1, 4, 6, 114, 118, 329, 304].contains(&id))
                })
    }

    /// An independent strict 511F10 trace proves 68 ordered requests for zero
    /// familiars with these inputs. Actual manager queries remain required.
    pub(crate) fn admits_selected_zero_familiar_requests(self) -> bool {
        if self.owned_count(229) != 0 {
            return self.admits_monstro_zero_familiar_requests();
        }
        if self.owned_count(168) != 0 || self.owned_count(2) != 0 {
            return self.admits_epic_fetus_zero_familiar_requests();
        }
        self.admits_selected_auxiliary_callbacks()
    }

    pub(crate) fn admits_complete_selected_tail(self) -> bool {
        self.admits_selected_auxiliary_callbacks() && self.owner.luck_inputs.is_some()
    }

    ///Separate literal neutral kind4/contact modifier footprint. Extending
    ///cache32/64 support never expands this contact/controller domain.
    pub(crate) fn admits_neutral_kind2(self) -> bool {
        self.owner.trinket_slots == [0, 0]
            && self
                .owner
                .queued_id
                .is_none_or(|id| items::definition(id).is_some_and(|config| config.kind != 2))
            && self.owner.temporary_collectibles.is_empty()
            && self
                .owner
                .owned_counts
                .iter()
                .enumerate()
                .all(|(id, &count)| {
                    count == 0 || (count > 0 && [1, 4, 114, 118, 329].contains(&id))
                })
    }

    pub(crate) fn admits_neutral_kind3(self) -> bool {
        self.admits_neutral_kind2()
    }

    pub(crate) fn admits_neutral_kind4(self) -> bool {
        self.owner.trinket_slots == [0, 0]
            && self
                .owner
                .queued_id
                .is_none_or(|id| items::definition(id).is_some_and(|config| config.kind != 2))
            && self.owner.temporary_collectibles.is_empty()
            && self
                .owner
                .owned_counts
                .iter()
                .enumerate()
                .all(|(id, &count)| {
                    count == 0 || (count > 0 && [1, 4, 114, 118, 329].contains(&id))
                })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CachedScalars {
    pub(crate) stats: PlayerStats,
    pub(crate) max_fire_delay: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CacheResolution {
    pub(crate) values: CachedScalars,
    pub(crate) completed: CacheMask,
}

impl CacheResolution {
    pub(crate) fn pending(self, requested: CacheMask) -> CacheMask {
        CacheMask(requested.0 & !self.completed.0)
    }
}

///ShotSpeed4 owns one nonzero143 base operator plus its own neutral reads.
///The full511F68..51212E CFG never queries/loads4/152/169. Their presence
///therefore composes here without granting another cache, grant or caller.
struct ShotSpeedStage;

impl ShotSpeedStage {
    fn admits(inputs: ItemInputs<'_>, source_player_type: Option<u32>) -> bool {
        //The former scalar capability is retained only as a whole for old
        //neutral/coupled contexts. It supplies no newly combined neutral owner.
        let legacy_neutral = inputs.admits_scalar_stage(CacheStage::ShotSpeed, &[4, 143]);
        source_player_type == Some(0)
            && inputs.admits_scalar_context()
            && inputs
                .owner
                .owned_counts
                .iter()
                .enumerate()
                .all(|(id, &count)| {
                    count == 0
                        || (count > 0
                            && (id == 143 //Actual raw nonzero base operator.
                                || matches!(id, 4 | 152 | 169) //Source-neutral here only.
                                || legacy_neutral))
                })
    }
}

///519FC0's selected normal Tear caller:152 owns the alternating-eye
///branch;169 owns the later current-P override after the eye RNG draw.
///4 does not alter this caller. Getter/cache/body permissions stay separate.
enum Technology2PrimaryReadRole {
    AlternateEye,
    CurrentPlayerOrigin,
    Neutral,
}

impl Technology2PrimaryReadRole {
    fn of(id: usize) -> Option<Self> {
        match id {
            152 => Some(Self::AlternateEye),
            169 => Some(Self::CurrentPlayerOrigin),
            4 => Some(Self::Neutral),
            _ => None,
        }
    }
}

///52F370's distinct C40 caller:152 selects the real thin beam;4/169 do
///not add a caller branch. Birth reads actual current C88/C98/CCC while an
///existing C40 keeps its born properties. No primary permission follows.
enum Technology2SecondaryReadRole {
    ThinBeam,
    Neutral,
}

impl Technology2SecondaryReadRole {
    fn of(id: usize) -> Option<Self> {
        match id {
            152 => Some(Self::ThinBeam),
            4 | 169 => Some(Self::Neutral),
            _ => None,
        }
    }
}

///Source-local read roles for the literal FireDelay operator below. Full
///511F10 closure proves152 neutral; no caller/cache/grant permission follows.
///The shared multiplier owns permanent2/169, while153 and every TEMP record
///remain outside this selected plan. Retained C08 stays an independent input.
enum FireDelayReadRole {
    Tier,
    SharedMultiplier,
    PostBoundReduction,
    SoyMilk,
    Neutral,
}

impl FireDelayReadRole {
    fn of(id: usize) -> Option<Self> {
        match id {
            1 | 6 => Some(Self::Tier),
            2 | 169 => Some(Self::SharedMultiplier),
            222 => Some(Self::PostBoundReduction),
            330 => Some(Self::SoyMilk),
            4 | 152 => Some(Self::Neutral),
            _ => None,
        }
    }
}

///Only the Flags32 callback's implemented source roles.152 is neutral in
///this callback, while169 supplies the existing51316D OR0x200 operator.
///Neither role owns Color64, a returned getter field or a born actor body.
struct TearFlagsStage;

impl TearFlagsStage {
    fn admits_polyphemus(inputs: ItemInputs<'_>) -> bool {
        inputs.positive_owned_count(169)
            && inputs.admits_stage_owned_roles(|id| match id {
                4 | 152 => true, //Independently source-neutral Flags32 reads.
                169 => true,     //Existing one OR0x200 operator.
                _ => false,
            })
    }
}

/// One callback's source ownership, beside its literal pure operator.
/// Reads signed permanent/TEMP1/6, positive permanent2/222 and retained C08;
/// writes integer C78 only. Source-local bounds and truncation precede Libra.
/// The caller retains the separate TEMP6/negative-history guard. These neutral
/// items prove this callback only;304 remains a distinct coupled operator.
struct FireDelayStage;

impl FireDelayStage {
    const REQUESTED: CacheMask = CacheMask(2);
    const EFFECT_ITEMS: &'static [u32] = &[1, 6, 222, 330];
    const NEUTRAL_ITEMS: &'static [u32] = &[4, 114, 118, 329];

    fn admits(inputs: ItemInputs<'_>, source_player_type: Option<u32>) -> bool {
        //Legacy neutral/coupled histories remain one whole capability. They
        //cannot supply a new neutral role beside152/169 operators.
        let legacy = inputs.owned_count(152) == 0
            && inputs.owned_count(169) == 0
            && inputs.admits_scalar_stage(CacheStage::FireDelay, Self::EFFECT_ITEMS);
        let operator_reads = !inputs.positive_owned_count(153)
            && inputs.admits_stage_owned_roles(|id| FireDelayReadRole::of(id).is_some());
        source_player_type == Some(0) && (legacy || operator_reads)
    }

    fn resolve(inputs: ItemInputs<'_>, primary: PrimaryWeaponC08) -> i32 {
        //512320/512323 add signed Number One counts before CVTSI2SS.
        //512354/512373 round Sad Onion counts separately, then5123DB
        //adds that weighted contribution to the Number One term.
        let number_one = inputs
            .owned_count(6)
            .wrapping_add(inputs.temporary_count(6));
        let tier = number_one as f32 * 1.5_f32
            + (inputs.owned_count(1) as f32 + inputs.temporary_count(1) as f32) * 0.7_f32;
        let root = (1.3_f32 * tier + 1.0_f32).max(0.0).sqrt();
        let mut delay = 16.0_f32 - 6.0_f32 * root;
        if tier <= 0.0 {
            delay -= 6.0_f32 * tier;
        }
        //512562 bound precedes512666's one permanent222 modifier.
        delay = delay.max(5.0_f32);
        //5125B0/514D95 share one5125BD operator after the bound and before
        //retained C08. Separate multiply/add; simultaneous2/169 never stack.
        //153/TEMP2 remain independently unadmitted modifier owners.
        if inputs.positive_owned_count(2) || inputs.positive_owned_count(169) {
            delay *= 2.1_f32;
            delay += 3.0_f32;
        }
        //512610 reads retained C08; multiply BEFORE integer truncation.
        if primary == PrimaryWeaponC08::Brimstone {
            delay *= 3.0_f32;
        } else if primary == PrimaryWeaponC08::DrFetus {
            //51262F..51263D: retained C08=5, after max5 and before222.
            delay *= 2.5_f32;
        }
        if inputs.positive_owned_count(222) {
            delay -= 2.0_f32;
        }
        //512738/5151EE: Monstro uses the actual retained C08 after222,
        //before Soy and CVTT. The binary32 literal is 0x4089999A.
        if primary == PrimaryWeaponC08::MonstrosLung {
            delay *= 4.3_f32;
        }
        //512749 Have330: one permanent effect after retained-C08 modifiers;
        //51275B/512763 round subtraction and multiplication before C78.
        if inputs.positive_owned_count(330) {
            delay = (delay - 6.0_f32) * 0.25_f32;
        }
        (delay as i32).max(1)
    }
}

///Source514F40/515670 separates169 absence, positive2/153 add4-only,
///and no-presence add4-then-double. TEMP2 remains a distinct unknown query.
#[derive(Clone, Copy)]
enum PolyphemusDamageMode {
    Disabled,
    AddFour,
    AddFourThenDouble,
}

/// One source-ordered Damage operation and its permanent input ownership.
/// The variant order is chosen by DamagePlan, never by collectible order.
/// These reads grant no other cache/getter/caller/contact capability.
#[derive(Clone, Copy)]
enum DamageOperator {
    BaseTier {
        signed_four: i32,
        signed_twelve: i32,
    },
    Polyphemus {
        mode: PolyphemusDamageMode,
    },
    MinimumBound,
    Technology2 {
        enabled: bool,
    },
    SharedFourOrTwelve {
        enabled: bool,
    },
    SoyMilk {
        enabled: bool,
    },
}

impl DamageOperator {
    ///Only this callback's implemented producers, including their inactive
    ///presence reads. New effects extend their operator, not a pair table.
    fn reads_owned(self, id: usize) -> bool {
        match self {
            Self::BaseTier { .. } | Self::SharedFourOrTwelve { .. } => matches!(id, 4 | 12),
            Self::Polyphemus { mode } => {
                id == 169
                    || (!matches!(mode, PolyphemusDamageMode::Disabled) && matches!(id, 2 | 153))
            }
            Self::Technology2 { .. } => id == 152,
            Self::SoyMilk { .. } => id == 330,
            Self::MinimumBound => false,
        }
    }

    fn apply(self, mut damage: f32) -> f32 {
        match self {
            Self::BaseTier {
                signed_four,
                signed_twelve,
            } => {
                //513628..513880: signed contributions, then f32 linear,
                //f64 sqrt*3.5 and f32 store. TEMP12 stays unadmitted.
                let tier = signed_four as f32 * 0.5_f32 + signed_twelve as f32 * 0.3_f32;
                let linear = (1.2_f32 * tier + 1.0_f32).max(0.0);
                (f64::from(linear).sqrt() * 3.5_f64) as f32
            }
            Self::Polyphemus { mode } => match mode {
                PolyphemusDamageMode::Disabled => damage,
                PolyphemusDamageMode::AddFour => damage + 4.0_f32, //514F55.
                PolyphemusDamageMode::AddFourThenDouble => {
                    //515AF6: add4 then self-add, before the same later bound.
                    damage += 4.0_f32;
                    damage += damage;
                    damage
                }
            },
            Self::MinimumBound => damage.max(0.5_f32),
            Self::Technology2 { enabled } => {
                if enabled {
                    //5139B1→513E10, once; not the bornLaser *.2 operator.
                    damage *= f32::from_bits(0x3f26_6666);
                }
                damage
            }
            Self::SharedFourOrTwelve { enabled } => {
                if enabled {
                    //513A20→515026: one shared factor, including both items.
                    damage *= 1.5_f32;
                }
                damage
            }
            Self::SoyMilk { enabled } => {
                if enabled {
                    //513A78→513DF8, before the final bound and later Libra.
                    damage *= 0.2_f32;
                }
                damage
            }
        }
    }
}

///Immutable functional plan of one actual Damage callback. Partial numeric
///values are still evaluated for unknown contexts; only completion is withheld.
///No item pair or other callback's permission can supply a missing read owner.
struct DamagePlan {
    operators: [DamageOperator; 7],
    reads_admitted: bool,
}

impl DamagePlan {
    fn new(inputs: ItemInputs<'_>) -> Self {
        let polyphemus_mode = if !inputs.positive_owned_count(169) {
            PolyphemusDamageMode::Disabled
        } else if inputs.positive_owned_count(2) || inputs.positive_owned_count(153) {
            PolyphemusDamageMode::AddFour
        } else {
            PolyphemusDamageMode::AddFourThenDouble
        };
        let operators = [
            DamageOperator::BaseTier {
                signed_four: inputs.owned_count(4),
                signed_twelve: inputs.owned_count(12),
            },
            DamageOperator::Polyphemus {
                mode: polyphemus_mode,
            },
            DamageOperator::MinimumBound,
            DamageOperator::Technology2 {
                enabled: inputs.positive_owned_count(152),
            },
            DamageOperator::SharedFourOrTwelve {
                enabled: inputs.positive_owned_count(4) || inputs.positive_owned_count(12),
            },
            DamageOperator::SoyMilk {
                enabled: inputs.positive_owned_count(330),
            },
            DamageOperator::MinimumBound,
        ];
        //514F40/515670 owns the permanent2/153 presence diversion inside
        //this operator. TEMP2 record presence is a different source query;
        //the actual no-TEMP context below still excludes every such record.
        //Preserve existing neutral/coupled-owner capabilities exactly. This
        //compatibility path does not compose unclosed neutral histories with
        //new operators. Item1 is independently Damage-neutral in all selected
        //operator traces; other new neutrals still need their own source proof.
        let legacy_neutral = inputs.owned_count(12) == 0
            && inputs.owned_count(152) == 0
            && inputs.owned_count(169) == 0
            && inputs.admits_scalar_stage(CacheStage::Damage, DamageStage::EFFECT_ITEMS);
        let reads_admitted = inputs.admits_scalar_context()
            && inputs
                .owner
                .owned_counts
                .iter()
                .enumerate()
                .all(|(id, &count)| {
                    count == 0
                        || (count > 0
                            && (id == 1
                                || operators.iter().any(|operator| operator.reads_owned(id))
                                || legacy_neutral))
                });
        Self {
            operators,
            reads_admitted,
        }
    }

    fn resolve(self) -> f32 {
        self.operators
            .into_iter()
            .fold(0.0, |damage, operator| operator.apply(damage))
    }
}

///Damage-only stage capability. Its source-ordered plan composes implemented
///operators; primary selection, other callbacks, grants and contacts stay local.
struct DamageStage;

impl DamageStage {
    const REQUESTED: CacheMask = CacheMask(1);
    const EFFECT_ITEMS: &'static [u32] = &[4, 12, 152, 169, 330];
    const NEUTRAL_ITEMS: &'static [u32] = &[1, 6, 114, 118, 329];

    fn admits(inputs: ItemInputs<'_>, source_player_type: Option<u32>) -> bool {
        source_player_type == Some(0) && DamagePlan::new(inputs).reads_admitted
    }

    fn resolve(inputs: ItemInputs<'_>) -> f32 {
        DamagePlan::new(inputs).resolve()
    }
}

/// Recompute requested mapped callbacks from actual item inputs, retaining
/// partial values and pending bits for every unimplemented callback owner.
/// This extraction preserves Source110 arithmetic and predicates exactly.
pub(crate) fn resolve_scalars(
    inputs: ItemInputs<'_>,
    primary: PrimaryWeaponC08,
    source_player_type: Option<u32>,
    prior: CachedScalars,
    requested: CacheMask,
) -> Result<CacheResolution, String> {
    let mut values = prior;
    let mut completed = 0;
    for stage in CacheStage::ORDER {
        let flag = stage.flag();
        if requested.0 & flag == 0 {
            continue;
        }
        let supported: &[u32] = match stage {
            CacheStage::FireDelay => {
                values.max_fire_delay = FireDelayStage::resolve(inputs, primary);
                FireDelayStage::EFFECT_ITEMS
            }
            CacheStage::ShotSpeed => {
                // 511f68/513e48 test raw permanent and temporary143 counts
                // for nonzero. Presence and signed-positive getters are distinct.
                values.stats.shot_speed =
                    if inputs.owned_count(143) != 0 || inputs.temporary_count(143) != 0 {
                        1.2_f32
                    } else {
                        1.0_f32
                    };
                values.stats.shot_speed = values.stats.shot_speed.max(0.6_f32);
                &[4, 143]
            }
            CacheStage::Damage => {
                values.stats.damage = DamageStage::resolve(inputs);
                DamageStage::EFFECT_ITEMS
            }
            CacheStage::Range => {
                let roid = inputs.owned_count(14) as f32;
                //5127EE Have6, else514FAF record presence: one contribution
                //regardless of multiplicity or the record's signed count.
                let (height, falling) = if inputs.number_one_present() {
                    (-7.75_f32, 0.45_f32)
                } else {
                    (-23.75_f32, 0.0_f32)
                };
                values.stats.tear_falling_speed = falling + roid * 0.5_f32;
                values.stats.tear_height = ((height - roid * 5.0_f32)
                    - values.stats.tear_falling_speed * 0.5_f32)
                    .min(-7.0);
                values.stats.tear_falling_acceleration = 0.0;
                &[6, 14]
            }
            CacheStage::MoveSpeed => {
                let additive = inputs
                    .owned_count(27)
                    .wrapping_add(inputs.owned_count(28))
                    .wrapping_add(inputs.owned_count(143))
                    .wrapping_add(inputs.temporary_count(143));
                let roid = (inputs.owned_count(14) as f32).min(1.0);
                let speed = additive as f32 * 0.3_f32 + roid * 0.6_f32;
                values.stats.move_speed = (speed.min(1.0) + 1.0).clamp(0.1, 2.0);
                &[14, 27, 28, 143]
            }
            _ => continue,
        };
        //The literal TEMP6/negative6 values are distinct from their producer
        //history qualification. Keep these requested until that lifecycle is
        //admitted; only positive permanent6 expands this completion domain.
        let unadmitted_number_one = matches!(stage, CacheStage::FireDelay | CacheStage::Range)
            && (inputs.owned_count(6) < 0 || inputs.number_one_record_present());
        if !unadmitted_number_one
            && match stage {
                CacheStage::FireDelay => FireDelayStage::admits(inputs, source_player_type),
                CacheStage::Damage => DamageStage::admits(inputs, source_player_type),
                CacheStage::ShotSpeed => ShotSpeedStage::admits(inputs, source_player_type),
                _ => source_player_type == Some(0) && inputs.admits_scalar_stage(stage, supported),
            }
        {
            completed |= flag;
        }
    }
    values.stats.validate()?;
    Ok(CacheResolution {
        values,
        completed: CacheMask(completed),
    })
}

///511F39/5122C0 closes the four coupled requests after primary selection.
///Have304 tests positive permanent ownership; TEMP presence and multiplicity
///are not independent operators. ShotSpeed4 alone leaves the mask unchanged.
pub(crate) fn expand_libra_requests(inputs: ItemInputs<'_>, requested: CacheMask) -> CacheMask {
    CacheMask(
        if inputs.positive_owned_count(304) && requested.0 & LIBRA_SCALAR_FLAGS != 0 {
            requested.0 | LIBRA_SCALAR_FLAGS
        } else {
            requested.0
        },
    )
}

///A coupled output cannot claim completion while any one of its requested
///input callbacks is still unmapped. This is port qualification metadata;
///the source clears D6C only after every actual callback has run.
pub(crate) fn pending_after_libra(inputs: ItemInputs<'_>, pending: CacheMask) -> CacheMask {
    CacheMask(
        if inputs.positive_owned_count(304) && pending.0 & LIBRA_SCALAR_FLAGS != 0 {
            pending.0 | LIBRA_SCALAR_FLAGS
        } else {
            pending.0
        },
    )
}

///Actual513B20 coupling reads current cached fields after all callbacks,
///including retained fields when only an unrelated callback was requested.
///C78 is the already-converted integer, including currentC08's Brim multiplier.
///Value computation and the independently proved callback/history admission
///remain separate; pending inputs propagate through the coupled group.
pub(crate) fn finalize_scalars(
    inputs: ItemInputs<'_>,
    mut values: CachedScalars,
) -> Result<CachedScalars, String> {
    if inputs.positive_owned_count(304) {
        //42EF60 and42EF50 each round every literal f32 operation; no clamp
        //or reassociation is permitted inside either source helper.
        let normalize = |a: f32, b: f32, value: f32| (value - a) / (b - a);
        let interpolate = |a: f32, b: f32, average: f32| (b - a) * average + a;
        let move_high = f32::from_bits(0x3f93_3333); //Source66AD40.
        let delay_low = f32::from_bits(0x3dcc_cccd); //Source66ACF4.
        let delay_high = f32::from_bits(0x3de3_8e39); //Source670D70.
        let damage = normalize(3.5, 4.5, values.stats.damage);
        let movement = normalize(1.0, move_high, values.stats.move_speed);
        let range = normalize(-23.75, -26.25, values.stats.tear_height);
        let reciprocal_delay = normalize(
            delay_low,
            delay_high,
            1.0_f32 / values.max_fire_delay as f32,
        );
        //513BBB/513BC9 accumulate the first three;513BCF adds them to
        //the reciprocal term before513BDB's quarter multiplication.
        let average = (reciprocal_delay + ((damage + movement) + range)) * 0.25_f32;
        values.stats.damage = interpolate(3.5, 4.5, average);
        values.stats.move_speed = interpolate(1.0, move_high, average);
        values.stats.tear_height = interpolate(-23.75, -26.25, average);
        let delay = 1.0_f32 / interpolate(delay_low, delay_high, average);
        //513C6F CVTTSS2SI returns INT_MIN on NaN/overflow, unlike Rust's
        //saturating cast. The source final integer clamp then produces1.
        values.max_fire_delay =
            if delay.is_finite() && (-2_147_483_648.0_f32..2_147_483_648.0_f32).contains(&delay) {
                delay as i32
            } else {
                i32::MIN
            };
    }
    //51220A..512299 final clamps follow Libra and all auxiliary callbacks.
    values.stats.move_speed = values.stats.move_speed.clamp(0.1, 2.0);
    values.stats.shot_speed = values.stats.shot_speed.max(0.6);
    values.stats.tear_height = values.stats.tear_height.min(-7.0);
    values.stats.damage = values.stats.damage.max(0.5);
    values.max_fire_delay = values.max_fire_delay.max(1);
    values.stats.validate()?;
    Ok(values)
}

/// The negative Flight callback at 515328/51532F writes both D68 and class 340.
/// Selected inputs have no flight item or TEMP term; both transformation
/// predicates require their actual history. Flying class 3 remains unadmitted.
pub(crate) fn resolve_neutral_flight(
    inputs: ItemInputs<'_>,
    source_player_type: Option<u32>,
    transformation_counts: Option<[i32; 2]>,
) -> Option<(bool, u32)> {
    (source_player_type == Some(0)
        && inputs.admits_selected_auxiliary_callbacks()
        && transformation_counts == Some([0, 0]))
    .then_some((false, 5))
}

/// 513C80 computes a signed wrapping sum, then CVTSI2SS and ADDSS. Its
/// auxiliary words and Game 10EAD8 are distinct inputs; absent history is unknown.
pub(crate) fn resolve_luck(
    inputs: ItemInputs<'_>,
    source_player_type: Option<u32>,
    source_run_flags: Option<u64>,
) -> Result<Option<f32>, String> {
    if inputs
        .owner
        .luck_inputs
        .is_some_and(|auxiliary| !auxiliary.source_d58.is_finite())
    {
        return Err("Luck requires a finite actual D58 input".into());
    }
    let (Some(auxiliary), Some(run_flags)) = (inputs.owner.luck_inputs, source_run_flags) else {
        return Ok(None);
    };
    if source_player_type != Some(0) || !inputs.admits_selected_auxiliary_callbacks() {
        return Ok(None);
    }
    let sum = auxiliary
        .source_13d0
        .wrapping_sub(auxiliary.source_13cc)
        .wrapping_add(auxiliary.source_d3c)
        .wrapping_add(auxiliary.source_d40)
        .wrapping_add(inputs.owned_count(253))
        .wrapping_add(inputs.owned_count(46))
        .wrapping_add(inputs.owned_count(343));
    let mut luck = sum as f32 + auxiliary.source_d58;
    if run_flags & 0x100 != 0 {
        luck += 50.0_f32; // 513CFE/513D06, after the trinket and character branches.
    }
    if !luck.is_finite() {
        return Err("Luck callback overflowed its source f32 output".into());
    }
    Ok(Some(luck))
}
