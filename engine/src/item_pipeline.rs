//! Pure resolution of the mapped Player cache callbacks.
//!
//! Item predicates retain their distinct source meanings. Actor removal,
//! admission, audio, and random draws belong to the caller/backend.
use crate::{
    items, player_items::PlayerItems, player_stats::PlayerStats, weapon_plan::PrimaryWeaponC08,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CacheMask(pub(crate) u32);

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
            Self::FireDelay => 2,
            Self::ShotSpeed => 4,
            Self::Damage => 1,
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

    /// Exact per-callback cache32/64 proof: other modifier and trinket owners
    /// remain outside this scope. Future property operators extend their own
    /// cache reads instead of widening controller/contact admission.
    pub(crate) fn admits_selected_appearance(self) -> bool {
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
                    count == 0 || (count > 0 && [1, 4, 114, 118, 329, 257].contains(&id))
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

    /// Each callback composes its implemented effects with its proven neutral
    /// read set. C08 remains an independent input to the ordered formulas.
    /// Config 118's request for all callbacks adds no implicit scalar effect.
    fn admits_scalar_stage(self, stage: CacheStage, effects: &[u32]) -> bool {
        let neutral: &[u32] = match stage {
            CacheStage::FireDelay => &[4, 114, 118, 329],
            CacheStage::ShotSpeed | CacheStage::Damage => &[1, 114, 118, 329],
            CacheStage::Range | CacheStage::MoveSpeed => &[1, 4, 114, 118, 329],
            _ => return false,
        };
        self.admits_scalar_context()
            && self
                .owner
                .owned_counts
                .iter()
                .enumerate()
                .all(|(id, &count)| {
                    count == 0
                        || (count > 0
                            && (effects.contains(&(id as u32)) || neutral.contains(&(id as u32))))
                })
    }

    /// Auxiliary callbacks have their own narrower source proof. Scalar
    /// effects admitted above do not expand Flight, Familiar, Luck or history.
    fn admits_selected_auxiliary_callbacks(self) -> bool {
        self.admits_scalar_context()
            && self
                .owner
                .owned_counts
                .iter()
                .enumerate()
                .all(|(id, &count)| {
                    count == 0 || (count > 0 && [1, 4, 114, 118, 329].contains(&id))
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

    fn has_unmapped_owner(self, flag: u32, supported: &[u32]) -> bool {
        self.owner
            .owned_counts
            .iter()
            .enumerate()
            .any(|(id, &count)| {
                count != 0
                    && !supported.contains(&(id as u32))
                    && items::definition(id as u32)
                        .is_some_and(|config| config.cache_flags & flag != 0)
            })
            || self.owner.temporary_collectibles.iter().any(|entry| {
                !supported.contains(&entry.item_id)
                    && items::definition(entry.item_id)
                        .is_some_and(|config| config.cache_flags & flag != 0)
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
                //512354/512373 round permanent and temporary counts separately.
                let tier =
                    (inputs.owned_count(1) as f32 + inputs.temporary_count(1) as f32) * 0.7_f32;
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
                values.max_fire_delay = (delay as i32).max(1);
                &[1, 222]
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
                //513628..513644: f32 linear, f64 sqrt*3.5, then f32 store.
                let tier = inputs.owned_count(4) as f32 * 0.5_f32;
                let linear = (1.2_f32 * tier + 1.0_f32).max(0.0);
                let mut damage = (f64::from(linear).sqrt() * 3.5_f64) as f32;
                if inputs.positive_owned_count(4) {
                    damage *= 1.5_f32;
                }
                values.stats.damage = damage.max(0.5_f32);
                &[4]
            }
            CacheStage::Range => {
                let roid = inputs.owned_count(14) as f32;
                values.stats.tear_falling_speed = roid * 0.5_f32;
                values.stats.tear_height = ((-23.75_f32 - roid * 5.0_f32)
                    - values.stats.tear_falling_speed * 0.5_f32)
                    .min(-7.0);
                values.stats.tear_falling_acceleration = 0.0;
                &[14]
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
        if (source_player_type == Some(0) && inputs.admits_scalar_stage(stage, supported))
            || !inputs.has_unmapped_owner(flag, supported)
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
