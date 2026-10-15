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

// Independent source-owned footprints. Equal current sets are deliberately
// separate: cache additions cannot silently admit a50D090 getter branch.
const TECHNOLOGY_CACHE_ITEMS: &[usize] = &[1, 4, 68];
const TECHNOLOGY_KIND2_ITEMS: &[usize] = &[1, 4, 68];
const TECHNOLOGY_KIND3_ITEMS: &[usize] = &[1, 4, 68];

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

    /// Exact per-callback cache32/64 proof: other modifier and trinket owners
    /// remain outside this scope. Future property operators extend their own
    /// cache reads instead of widening controller/contact admission.
    pub(crate) fn admits_selected_appearance(self) -> bool {
        if self.admits_technology_cache_context() {
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

    fn admits_scalar_context(self) -> bool {
        self.owner.trinket_slots == [0, 0]
            && self
                .owner
                .queued_id
                .is_none_or(|id| items::definition(id).is_some_and(|config| config.kind != 2))
            && self.owner.temporary_collectibles.is_empty()
    }

    /// Independent511F10 footprint for positive Technology with selected1/4.
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

    /// Each callback composes its implemented effects with its proven neutral
    /// read set. C08 remains an independent input to the ordered formulas.
    /// Config 118's request for all callbacks adds no implicit scalar effect.
    fn admits_scalar_stage(self, stage: CacheStage, effects: &[u32]) -> bool {
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
                                || (id == 68 && neutral_technology)))
                })
    }

    /// Auxiliary callbacks have their own narrower source proof. Scalar
    /// effects admitted above do not expand Flight, Familiar, Luck or history.
    fn admits_selected_auxiliary_callbacks(self) -> bool {
        if self.admits_technology_cache_context() {
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

/// One callback's source ownership, beside its literal pure operator.
/// Reads signed permanent/TEMP1/6, positive permanent222 and retained C08;
/// writes integer C78 only. Source-local bounds and truncation precede Libra.
/// The caller retains the separate TEMP6/negative-history guard. These neutral
/// items prove this callback only;304 remains a distinct coupled operator.
struct FireDelayStage;

impl FireDelayStage {
    const REQUESTED: CacheMask = CacheMask(2);
    const EFFECT_ITEMS: &'static [u32] = &[1, 6, 222];
    const NEUTRAL_ITEMS: &'static [u32] = &[4, 114, 118, 329];

    fn admits(inputs: ItemInputs<'_>, source_player_type: Option<u32>) -> bool {
        source_player_type == Some(0)
            && inputs.admits_scalar_stage(CacheStage::FireDelay, Self::EFFECT_ITEMS)
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
        //512610 reads retained C08; multiply BEFORE integer truncation.
        if primary == PrimaryWeaponC08::Brimstone {
            delay *= 3.0_f32;
        }
        if inputs.positive_owned_count(222) {
            delay -= 2.0_f32;
        }
        (delay as i32).max(1)
    }
}

/// One Damage callback's ownership beside its literal pure operator.
/// Reads signed permanent4 for the tier and positive presence for one multiplier;
/// writes Damage/C88. Its source-local bound precedes separate Libra coupling.
struct DamageStage;

impl DamageStage {
    const REQUESTED: CacheMask = CacheMask(1);
    const EFFECT_ITEMS: &'static [u32] = &[4];
    const NEUTRAL_ITEMS: &'static [u32] = &[1, 6, 114, 118, 329];

    fn admits(inputs: ItemInputs<'_>, source_player_type: Option<u32>) -> bool {
        source_player_type == Some(0)
            && inputs.admits_scalar_stage(CacheStage::Damage, Self::EFFECT_ITEMS)
    }

    fn resolve(inputs: ItemInputs<'_>) -> f32 {
        //513628..513644: f32 linear, f64 sqrt*3.5, then f32 store.
        let tier = inputs.owned_count(4) as f32 * 0.5_f32;
        let linear = (1.2_f32 * tier + 1.0_f32).max(0.0);
        let mut damage = (f64::from(linear).sqrt() * 3.5_f64) as f32;
        if inputs.positive_owned_count(4) {
            damage *= 1.5_f32;
        }
        damage.max(0.5_f32)
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
