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

//Independent actual Add48 footprint. Only this grant-history consumer is
//introduced here; cache/getter/caller permissions remain separate.
const CUPID_GRANT_ITEMS: &[usize] = &[1, 4, 48];

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

    ///Each selected511F10 callback owns its own input roles.2 contributes
    ///only the existing FireDelay multiplier here; the other callbacks have
    ///separate source-neutral proofs. Numeric operators and noninventory
    ///history guards stay with their actual stage and consumer.
    fn admits_knife_cache_stage(self, stage: CacheStage) -> bool {
        self.positive_owned_count(2)
            && self.admits_stage_owned_roles(|id| KnifeCacheReadRole::of(stage, id).is_some())
    }

    ///Independent50D090(kind4) trace: no2 query, RNG or modifier output.
    ///Actual C88/C98/all11CA0/C8C stay current copied fields. This readset
    ///does not admit an ordinary multi-Knife emitter or controlled satellites.
    pub(crate) fn admits_knife_kind4_getter(self) -> bool {
        self.admits_stage_owned_roles(|id| matches!(id, 1 | 2 | 4 | 114 | 118 | 329))
    }

    ///Only the selected live114+329 caller may consume the newly closed2
    ///satellite-count branch. The ordinary caller retains its old capability.
    pub(crate) fn admits_controlled_knife_caller(self) -> bool {
        self.positive_owned_count(114)
            && self.positive_owned_count(329)
            && self.admits_stage_owned_roles(|id| match id {
                2 => true,         //Source519FC0 owns the satellite-count branch.
                114 | 329 => true, //Actual Knife/control selection.
                118 => true,       //Ordinary Has118 child branch bypassed by control.
                1 | 4 => true,     //No additional live caller operator.
                _ => false,
            })
    }

    ///ActualAdd2's default tail writes none of the retained Luck inputs.
    ///Only this wrapper consumes the proof; None is never recreated here.
    pub(crate) fn admits_inner_eye_knife_grant(self) -> bool {
        self.positive_owned_count(2)
            && self.admits_stage_owned_roles(|id| matches!(id, 1 | 2 | 4 | 114 | 118 | 329))
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
    ///Actual zero-amount Add48 preserves an existing D38 owner only.
    ///No missing stat word or other callback capability is constructed.
    pub(crate) fn admits_cupid_grant_context(self) -> bool {
        self.admits_permanent_readset(48, CUPID_GRANT_ITEMS)
    }

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
        let knife_stage_reads = self.admits_knife_cache_stage(stage);
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
                                || (id == 2 && epic_fetus_context)
                                || knife_stage_reads))
                })
    }

    /// Auxiliary callbacks have their own narrower source proof. Scalar
    /// effects admitted above do not expand Flight, Familiar, Luck or history.
    fn admits_selected_auxiliary_callbacks(self) -> bool {
        if self.admits_knife_cache_stage(CacheStage::Flight)
            && self.admits_knife_cache_stage(CacheStage::Familiar)
            && self.admits_knife_cache_stage(CacheStage::Luck)
        {
            return true;
        }
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
    pub(crate) reached_completed: CacheMask,
}

impl CacheResolution {
    pub(crate) fn pending(self, requested: CacheMask) -> CacheMask {
        CacheMask(requested.0 & !self.completed.0)
    }
}

///Known inputs of one callback, classified by their literal source role.
///This finite footprint has no item-pair result or alternate numeric formula.
enum KnifeCacheReadRole {
    Neutral,
    DelayTier,
    InnerEyeDelay,
    DamageTier,
}
impl KnifeCacheReadRole {
    fn of(stage: CacheStage, id: usize) -> Option<Self> {
        use CacheStage as S;
        match (stage, id) {
            (S::FireDelay, 1) => Some(Self::DelayTier),
            (S::FireDelay, 2) => Some(Self::InnerEyeDelay),
            (S::FireDelay, 4 | 114 | 118 | 329) => Some(Self::Neutral),
            (S::Damage, 4) => Some(Self::DamageTier),
            (S::Damage, 1 | 2 | 114 | 118 | 329) => Some(Self::Neutral),
            (S::ShotSpeed | S::Range | S::MoveSpeed, 1 | 2 | 4 | 114 | 118 | 329) => {
                Some(Self::Neutral)
            }
            (S::TearFlags | S::Color, 1 | 2 | 4 | 114 | 118 | 329) => Some(Self::Neutral),
            (S::Flight | S::Familiar | S::Luck, 1 | 2 | 4 | 114 | 118 | 329) => Some(Self::Neutral),
            _ => None,
        }
    }
}

///511F68..512126: read actual operands lazily from canonical query history.
///The complete raw arithmetic is independent of grant, getter and controller.
#[derive(Clone, Copy)]
struct ShotSpeedReadPlan {
    base: f32,
    raw_contributions: [i32; 13],
    source_d38: i32,
    have_five: bool,
}

impl ShotSpeedReadPlan {
    fn read(
        inputs: ItemInputs<'_>,
        queries: crate::item_queries::AppearanceQueries<'_>,
        source_bfc: Option<u32>,
        source: CacheSourceInputs,
    ) -> Option<Self> {
        //The caller passes actual Player+BFC, independently of Entity+14.
        if source_bfc != Some(0)
            || !source
                .entity
                .is_some_and(|entity| entity.entity_type == 1 && entity.variant == 0)
        {
            return None;
        }
        //Raw143 nonzero skips the actual TEMP143 count query entirely.
        let base = if queries.raw_count(143)? != 0 || queries.temporary_count(143)? != 0 {
            1.2_f32
        } else {
            1.0_f32
        };
        //First-read source order; the wrapping sum is resolved below.
        let raw_contributions = [
            queries.raw_count(165)?,
            queries.raw_count(183)?,
            queries.raw_count(339)?,
            queries.raw_count(341)?,
            queries.raw_count(194)?,
            queries.raw_count(342)?,
            queries.raw_count(176)?,
            queries.raw_count(182)?,
            queries.raw_count(213)?,
            queries.raw_count(310)?,
            queries.raw_count(255)?,
            queries.raw_count(336)?,
            queries.raw_count(331)?,
        ];
        let source_d38 = inputs.owner.shot_speed_inputs?.source_d38;
        //The supported normal BFC0 path next queries actual trinket27.
        //Its positive effect remains pending; other held IDs are not scanned.
        if queries.has_trinket(27)? {
            return None;
        }
        //5090C0 positive skips514B9F's distinct TEMP record-presence query.
        let have_five = queries.have(5)? || queries.temporary_present(5)?;
        //Normal variant0 skips Config9C entirely for the actual Game key0.
        if source.challenge_id != 0 && source.shot_speed_minimum != Some(false) {
            return None;
        }
        Some(Self {
            base,
            raw_contributions,
            source_d38,
            have_five,
        })
    }

    fn resolve(self) -> f32 {
        let [count165, count183, count339, count341, count194, count342, count176, count182, count213, count310, count255, count336, count331] =
            self.raw_contributions;
        let mut value = count165 as f32 * 0.23_f32;
        let sum = count183
            .wrapping_add(count339)
            .wrapping_add(count341)
            .wrapping_add(count194)
            .wrapping_sub(count342);
        value += self.base;
        value += count176 as f32 * 0.16_f32;
        value += sum as f32 * 0.16_f32;
        value -= count182 as f32 * 0.25_f32;
        value -= count213 as f32 * 0.15_f32;
        value -= count310 as f32 * 0.5_f32;
        value += count255 as f32 * 0.2_f32;
        value -= count336 as f32 * 0.4_f32;
        value -= count331 as f32 * 0.3_f32;
        value += self.source_d38 as f32 * 0.2_f32;
        if self.have_five {
            value *= 1.6_f32;
        }
        value.max(0.6_f32)
    }
}

///Transient actual source context, never an owner synthesized from inventory.
#[derive(Clone, Copy)]
pub(crate) struct CacheSourceInputs {
    pub(crate) entity: Option<crate::PlayerEntitySource>,
    pub(crate) challenge_id: u32,
    pub(crate) damage_maximum: Option<bool>,
    pub(crate) shot_speed_minimum: Option<bool>,
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
        source_player_type == Some(0)
            && inputs.owner.fire_delay_inputs.is_some()
            && (legacy || operator_reads)
    }

    fn resolve(inputs: ItemInputs<'_>, primary: PrimaryWeaponC08) -> i32 {
        //512320/512323 add signed Number One counts before CVTSI2SS.
        //512354/512373 round Sad Onion counts separately, then5123DB
        //adds that weighted contribution to the Number One term.
        let number_one = inputs
            .owned_count(6)
            .wrapping_add(inputs.temporary_count(6));
        let mut tier = number_one as f32 * 1.5_f32
            + (inputs.owned_count(1) as f32 + inputs.temporary_count(1) as f32) * 0.7_f32;
        //512495/49D/4A3: signed conversion, separate *.5 then ADDSS into
        //the tier BEFORE the character/nonlinear transform and min5 bound.
        //An absent owner leaves only a partial value; completion stays pending.
        if let Some(input) = inputs.owner.fire_delay_inputs {
            tier += input.source_d2c as f32 * 0.5_f32;
        }
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
        //51279C CVTTSS2SI returns INT_MIN for NaN/overflow. Rust's
        //saturating cast differs for a large negative signed D2C history.
        let integer =
            if delay.is_finite() && (-2_147_483_648.0_f32..2_147_483_648.0_f32).contains(&delay) {
                delay as i32
            } else {
                i32::MIN
            };
        integer.max(1)
    }
}

///Source514F40/515670 separates169 absence, positive2/153 add5-only,
///and no-presence add4-then-double. TEMP2 remains a distinct unknown query.
#[derive(Clone, Copy)]
enum PolyphemusDamageMode {
    Disabled,
    AddFive,
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
    PersistentAdditive {
        source_d30: Option<i32>,
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
                //514F5B loads666108=5.0 for the Have2/153 diversion.
                PolyphemusDamageMode::AddFive => damage + 5.0_f32,
                PolyphemusDamageMode::AddFourThenDouble => {
                    //515AF6: add4 then self-add, before the same later bound.
                    damage += 4.0_f32;
                    damage += damage;
                    damage
                }
            },
            Self::PersistentAdditive { source_d30 } => {
                //5138EA/5138F2: signed CVTSI2SS then ADDSS after the
                //selected character/item terms, before the first bound.
                if let Some(value) = source_d30 {
                    damage += value as f32;
                }
                damage
            }
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
    operators: [DamageOperator; 8],
}

impl DamagePlan {
    fn new(inputs: ItemInputs<'_>) -> Self {
        let polyphemus_mode = if !inputs.positive_owned_count(169) {
            PolyphemusDamageMode::Disabled
        } else if inputs.positive_owned_count(2) || inputs.positive_owned_count(153) {
            PolyphemusDamageMode::AddFive
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
            DamageOperator::PersistentAdditive {
                source_d30: inputs.owner.damage_inputs.map(|input| input.source_d30),
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
        Self { operators }
    }

    // Add to the existing DamagePlan impl. Its original new()/partial formula
    // remains literal; requested1 completes exclusively through this read method.
    fn read_reached(
        inputs: ItemInputs<'_>,
        queries: crate::item_queries::AppearanceQueries<'_>,
        source_bfc: Option<u32>,
        source: CacheSourceInputs,
    ) -> Option<Self> {
        //The caller passes actual Player+BFC, not the Entity kind/variant.
        if source_bfc != Some(0)
            || !source
                .entity
                .is_some_and(|entity| entity.entity_type == 1 && entity.variant == 0)
        {
            return None;
        }
        //5134C9 starts the callback with this independent record-presence query.
        //Its positive pre-tier operator is not implemented by this small cut.
        if queries.temporary_present(34)? {
            return None;
        }
        for id in [7, 90, 216, 109] {
            if queries.have(id)? {
                return None;
            }
        }
        //Have109 false bypasses the coins operand. A present122 record has a
        //separate squared-count operator; its absence is queried, not assumed.
        if queries.temporary_present(122)? {
            return None;
        }

        //Only actually loaded E1C raw terms constrain this callback. Unknown
        //unqueried inventory and unrelated TEMP45 do not enter these guards.
        for id in [50, 51, 70, 345, 79, 259] {
            if queries.raw_count(id)? != 0 {
                return None;
            }
        }
        if queries.temporary_count(259)? != 0 {
            return None;
        }
        if queries.raw_count(80)? != 0 {
            return None;
        }
        let signed_four = queries.raw_count(4)?;
        if queries.raw_count(197)? != 0 {
            return None;
        }
        let signed_twelve = queries
            .raw_count(12)?
            .wrapping_add(queries.temporary_count(12)?);
        for id in [101, 121] {
            if queries.raw_count(id)? != 0 {
                return None;
            }
        }
        if queries.temporary_count(121)? != 0 {
            return None;
        }
        for id in [138, 193, 189, 201] {
            if queries.raw_count(id)? != 0 {
                return None;
            }
        }
        if queries.temporary_count(201)? != 0
            || queries.raw_count(237)? != 0
            || queries.temporary_count(237)? != 0
        {
            return None;
        }
        for id in [206, 230, 307, 208, 331, 165] {
            if queries.raw_count(id)? != 0 {
                return None;
            }
        }
        if queries.have(120)? || queries.temporary_present(120)? {
            return None;
        }
        if queries.temporary_present(83)? {
            return None;
        }
        if queries.temporary_trinket_present(58)? {
            return None;
        }
        if queries.temporary_present(126)? {
            return None;
        }
        if queries.temporary_trinket_present(40)? {
            return None;
        }

        //Positive169 alone reaches the2→153→TEMP2 short-circuit diversion.
        let mode = if !queries.have(169)? {
            PolyphemusDamageMode::Disabled
        } else if queries.have(2)? || queries.have(153)? || queries.temporary_present(2)? {
            PolyphemusDamageMode::AddFive
        } else {
            PolyphemusDamageMode::AddFourThenDouble
        };
        let source_d30 = inputs.owner.damage_inputs?.source_d30;
        if queries.has_trinket(35)? {
            return None;
        }
        if queries.have(69)? {
            return None;
        }
        if queries.have(261)? || queries.temporary_present(261)? {
            return None;
        }
        if queries.have(182)? {
            return None;
        }
        let technology2 = queries.have(152)?;
        //These false predicates skip their independent BEC and retainedC08 reads.
        if queries.have(157)? || queries.have(149)? {
            return None;
        }
        let shared_four_or_twelve =
            queries.have(4)? || queries.have(12)? || queries.temporary_present(12)?;
        //The earlier TEMP34 query was false and no records mutate in511F10.
        if queries.have(310)? {
            return None;
        }
        let soy_milk = queries.have(330)?;
        if source.challenge_id != 0 && source.damage_maximum != Some(false) {
            return None;
        }
        Some(Self {
            operators: [
                DamageOperator::BaseTier {
                    signed_four,
                    signed_twelve,
                },
                DamageOperator::Polyphemus { mode },
                DamageOperator::PersistentAdditive {
                    source_d30: Some(source_d30),
                },
                DamageOperator::MinimumBound,
                DamageOperator::Technology2 {
                    enabled: technology2,
                },
                DamageOperator::SharedFourOrTwelve {
                    enabled: shared_four_or_twelve,
                },
                DamageOperator::SoyMilk { enabled: soy_milk },
                DamageOperator::MinimumBound,
            ],
        })
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
    queries: crate::item_queries::AppearanceQueries<'_>,
    source: CacheSourceInputs,
) -> Result<CacheResolution, String> {
    let mut values = prior;
    let mut completed = 0;
    let mut reached_completed = 0;
    for stage in CacheStage::ORDER {
        let flag = stage.flag();
        if requested.0 & flag == 0 {
            continue;
        }
        let shot_speed_plan = if stage == CacheStage::ShotSpeed {
            ShotSpeedReadPlan::read(inputs, queries, source_player_type, source)
        } else {
            None
        };
        let damage_plan = if stage == CacheStage::Damage {
            DamagePlan::read_reached(inputs, queries, source_player_type, source)
        } else {
            None
        };
        let shot_speed_admitted = shot_speed_plan.is_some();
        let damage_admitted = damage_plan.is_some();
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
                //512063/06B/06F consumes the actual independent D38 word
                //after the selected item terms. An absent owner leaves a
                //partial value and cannot complete this callback below.
                if let Some(auxiliary) = inputs.owner.shot_speed_inputs {
                    values.stats.shot_speed += auxiliary.source_d38 as f32 * 0.2_f32;
                }
                values.stats.shot_speed = values.stats.shot_speed.max(0.6_f32);
                if let Some(plan) = shot_speed_plan {
                    values.stats.shot_speed = plan.resolve();
                }
                &[4, 143]
            }
            CacheStage::Damage => {
                values.stats.damage =
                    damage_plan.map_or_else(|| DamageStage::resolve(inputs), DamagePlan::resolve);
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
                let mut resolved_height = height - roid * 5.0_f32;
                //512AE9/AF1/B05: signed D34*.5 adds to C8C BEFORE the
                //512B0D subtraction of current C90*.5. C90/C94 are unchanged.
                if let Some(input) = inputs.owner.range_inputs {
                    resolved_height += input.source_d34 as f32 * 0.5_f32;
                }
                values.stats.tear_height =
                    (resolved_height - values.stats.tear_falling_speed * 0.5_f32).min(-7.0);
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
                let mut resolved_speed = speed.min(1.0) + 1.0;
                //512EC1/C9/D1: signed D28*.2 AFTER the base min1+1,
                //before late modifiers and the existing final .1..2 bound.
                if let Some(input) = inputs.owner.move_speed_inputs {
                    resolved_speed += input.source_d28 as f32 * 0.2_f32;
                }
                values.stats.move_speed = resolved_speed.clamp(0.1, 2.0);
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
                CacheStage::Damage => damage_admitted,
                CacheStage::Range => {
                    source_player_type == Some(0)
                        && inputs.owner.range_inputs.is_some()
                        && inputs.admits_scalar_stage(stage, supported)
                }
                CacheStage::MoveSpeed => {
                    source_player_type == Some(0)
                        && inputs.owner.move_speed_inputs.is_some_and(
                            crate::player_items::MoveSpeedInputs::supports_persistent_d28,
                        )
                        && inputs.admits_scalar_stage(stage, supported)
                }
                CacheStage::ShotSpeed => shot_speed_admitted,
                _ => source_player_type == Some(0) && inputs.admits_scalar_stage(stage, supported),
            }
        {
            completed |= flag;
            if matches!(stage, CacheStage::ShotSpeed | CacheStage::Damage) {
                reached_completed |= flag;
            }
        }
    }
    values.stats.validate()?;
    Ok(CacheResolution {
        values,
        completed: CacheMask(completed),
        reached_completed: CacheMask(reached_completed),
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

///All33 instructions of513C80..513D16 and their actual508FE0 helper
///own this callback's read footprint. Other inventory IDs do not add a Luck
///operator; they still require independently preserved input histories.
struct LuckStage;

impl LuckStage {
    fn admits(inputs: ItemInputs<'_>, source_player_type: Option<u32>) -> bool {
        //Empty actual slots and a non-trinket queue close helper42/21 for
        //either Have139 capacity. BFC0 excludes the separate BFC8 operator.
        source_player_type == Some(0)
            && inputs.admits_scalar_context()
            && inputs.owner.owned_counts.iter().all(|&count| count >= 0)
    }
}

///One immutable, source-ordered Luck callback. These integer-slot reads are
///operators, not an admission list. Input history owns all five auxiliary
///words and Game flags separately; no other cache/getter/caller is admitted.
struct LuckPlan {
    signed_sum: i32,
    float_additive: f32,
    run_flag_bonus: bool,
}

impl LuckPlan {
    fn from_owned(
        inputs: ItemInputs<'_>,
        auxiliary: crate::player_items::LuckInputs,
        run_flags: u64,
    ) -> Self {
        let signed_sum = auxiliary
            .source_13d0
            .wrapping_sub(auxiliary.source_13cc)
            .wrapping_add(auxiliary.source_d3c)
            .wrapping_add(auxiliary.source_d40)
            .wrapping_add(inputs.owned_count(253))
            .wrapping_add(inputs.owned_count(46))
            .wrapping_add(inputs.owned_count(343));
        Self {
            signed_sum,
            float_additive: auxiliary.source_d58,
            run_flag_bonus: run_flags & 0x100 != 0,
        }
    }

    fn evaluate(self) -> Result<f32, String> {
        let mut luck = self.signed_sum as f32 + self.float_additive;
        if self.run_flag_bonus {
            luck += 50.0_f32; //513CFE/513D06, after trinket and character.
        }
        if !luck.is_finite() {
            return Err("Luck callback overflowed its source f32 output".into());
        }
        Ok(luck)
    }
}

///513C80's auxiliary words and Game10EAD8 are independent actual inputs.
///Missing histories stay unknown; stage-neutral inventory cannot supply them.
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
    if !LuckStage::admits(inputs, source_player_type) {
        return Ok(None);
    }
    LuckPlan::from_owned(inputs, auxiliary, run_flags)
        .evaluate()
        .map(Some)
}
