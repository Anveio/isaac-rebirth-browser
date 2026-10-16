//! Provenance for the canonical Player query stores, not another inventory.
//! Reached queries return Unknown after unimplemented writes; a later cache
//! request, empty-looking store or current loadout cannot recover history.
use crate::{items, phase::Phase, player_items::PlayerItems, weapon_plan::PrimaryWeaponC08, PlayerEntitySource, State};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemQueryHistory {
    pub permanent_counts: bool,
    pub temporary_records: bool,
    ///Only these actual CF8 IDs may differ after a bounded unowned writer.
    ///Other queries still read the canonical vector. This is uncertainty
    ///metadata, never another temporary inventory or a generated effect.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub temporary_unknown_inputs: BTreeSet<TemporaryQueryKey>,
    pub trinket_slots: bool,
    pub primary_c08: bool,
    ///Only source-owned damage health operands can prune later damage
    ///queries. Foreign healing/refusal does not invalidate cached outputs.
    #[serde(default)]
    pub damage_health_known: bool,
    ///Known cached callback outputs other than C08, whose independent owner
    ///is primary_c08. Source writes, not current requests, own these facts.
    ///Missing legacy metadata establishes no output.
    #[serde(default)]
    pub cached_callbacks: u32,
    ///The source CF8 writer defers evaluation until531E44. The mask is a
    ///bounded possible footprint, not a fabricated actual D6C request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temporary_evaluation: Option<TemporaryEvaluationFrontier>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub unresolved_effects: BTreeMap<u32, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TemporaryEvaluationFrontier {
    pub possible_cache_mask: Option<u32>,
    ///Actual writer PCs whose unmodeled effects make the outcome unknown.
    pub writer_pcs: BTreeSet<u32>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TemporaryQueryKey {
    pub config_kind: u32,
    pub item_id: u32,
}

impl TemporaryQueryKey {
    fn collectible(id: u32) -> Option<Self> {
        Some(Self { config_kind: u32::from(items::definition(id)?.kind), item_id: id })
    }
}

impl ItemQueryHistory {
    pub(crate) const UNKNOWN: Self = Self {
        permanent_counts: false,
        temporary_records: false,
        temporary_unknown_inputs: BTreeSet::new(),
        trinket_slots: false,
        primary_c08: false,
        damage_health_known: false,
        cached_callbacks: 0,
        temporary_evaluation: None,
        unresolved_effects: BTreeMap::new(),
    };

    ///519350 clears CF8, initializes C08=1, then clears E1C/E14/E18.
    ///Only the actual fresh Player owner establishes all four domains.
    pub(crate) const FRESH: Self = Self {
        permanent_counts: true,
        temporary_records: true,
        temporary_unknown_inputs: BTreeSet::new(),
        trinket_slots: true,
        primary_c08: true,
        damage_health_known: true,
        cached_callbacks: 0x6ff,
        temporary_evaluation: None,
        unresolved_effects: BTreeMap::new(),
    };

    ///The current SavedPlayerItems copies permanent counts. It does not
    ///copy C08 or the held trinkets, or serialize a complete CF8 vector.
    ///Copying this narrowed provenance never transfers those other facts.
    pub(crate) fn saved_counts(&self) -> Self {
        Self {
            permanent_counts: self.permanent_counts,
            ..Self::UNKNOWN
        }
    }

    pub(crate) fn forget_unowned_effect(&mut self) {
        *self = Self::UNKNOWN;
    }

    ///A reached, bounded unimplemented CF8 writer affects only its actual
    ///possible IDs. Existing permanent/trinket/C08 and cached output facts
    ///survive here; no evaluation has happened at the room callback.
    pub(crate) fn note_temporary_writer(
        &mut self,
        keys: &[TemporaryQueryKey],
        possible_cache_mask: Option<u32>,
        writer_pc: u32,
    ) {
        self.temporary_unknown_inputs.extend(keys.iter().copied());
        let frontier = self.temporary_evaluation.get_or_insert_with(|| {
            TemporaryEvaluationFrontier {
                possible_cache_mask: Some(0),
                writer_pcs: BTreeSet::new(),
            }
        });
        frontier.possible_cache_mask = match (frontier.possible_cache_mask, possible_cache_mask) {
            (Some(old), Some(new)) => Some(old | new),
            _ => None,
        };
        frontier.writer_pcs.insert(writer_pc);
    }

    pub(crate) fn temporary_key_known(&self, config_kind: u32, item_id: u32) -> bool {
        self.temporary_records && !self.temporary_unknown_inputs.contains(&TemporaryQueryKey {
            config_kind, item_id,
        })
    }

    pub(crate) fn temporary_collectible_known(&self, id: u32) -> bool {
        items::definition(id).is_some_and(|config| self.temporary_key_known(u32::from(config.kind), id))
    }

    ///Called only at the full Player531E44 D18/Evaluate boundary. A possible
    ///unmodeled source request makes those outputs unknown at this point;
    ///the values and persistent D30/D38 inputs remain untouched. This does
    ///not claim a random branch or callback was observed to execute.
    pub(crate) fn reach_temporary_evaluation(&mut self) {
        let Some(frontier) = self.temporary_evaluation.take() else {
            return;
        };
        if let Some(mask) = frontier.possible_cache_mask {
            self.invalidate_cached_footprint(mask);
        } else {
            self.cached_callbacks = 0;
            self.primary_c08 = false;
        }
    }

    fn invalidate_cached_footprint(&mut self, possible_cache_mask: u32) {
        self.cached_callbacks &= !possible_cache_mask;
        if possible_cache_mask & 0x100 != 0 {
            self.primary_c08 = false;
        }
    }

}

///An actual350 factory attempt has no body/contact owner yet. Its possible
///later kind2 queue/held write makes only trinket-query completeness unknown.
///No contact, held value, cache request or effect is fabricated here.
pub(crate) fn note_unrepresented_trinket_spawn(state: &mut State) {
    if let Some(history) = state.player_items.as_mut()
        .and_then(|owner| owner.query_history.as_mut()) {
        history.trinket_slots = false;
        history.unresolved_effects.insert(0x4ff1e0,
            "Unsupported350 Init/body may reach canonical trinket queue/held collection".into());
    }
}

///Manager5BF220 invokes510550 after the new or restored Room.Init. This
///records only actual reached write uncertainty; grants do not call it.
pub(crate) fn note_room_placement(state: &mut State) {
    let Some(owner) = state.player_items.as_ref() else {
        return;
    };
    let initialized = owner.query_history.as_ref().map(|_| state.player.initialized_3074);
    let plan = RoomTemporaryWritePlan::read(
        AppearanceQueries::new(owner, state.weapon_cache.primary, state.player.entity_source, None),
        initialized,
        i32::from(state.player.hp),
        state.player.soul_hp,
        None,
        None,
        state.weapon_admission.source_player_type,
    );
    let Some(history) = state.player_items.as_mut().and_then(|owner| owner.query_history.as_mut()) else {
        return;
    };
    for writer in plan.writes {
        let Some(keys) = writer.ids.iter().copied().map(TemporaryQueryKey::collectible)
            .collect::<Option<Vec<_>>>() else {
                history.forget_unowned_effect();
                continue;
            };
        history.note_temporary_writer(&keys, Some(writer.possible_cache_mask), writer.writer_pc);
    }
    for effect in plan.unresolved {
        history.unresolved_effects.insert(effect.pc, effect.law.to_owned());
    }
}

fn scalar_evaluation_footprint(queries: AppearanceQueries<'_>, mask: u32) -> u32 {
    //511F39 expands only scalar requests. OrdinaryHave304 is a separate
    //reached permanent query; it never adds appearance or primary bits.
    if mask & 0x1b != 0 && queries.have(304) != Some(false) {
        mask | 0x1b
    } else {
        mask
    }
}

///Actual late full Player path:531E44 deferred Evaluate precedes the
///515F40 status callback. Interpolation bypasses both. These uncertainty
///hooks neither invent requests nor execute the unimplemented effects.
pub(crate) fn reach_late_temporary_evaluation(state: &mut State, phase: Phase) {
    if phase != Phase::Update {
        return;
    }
    let Some(owner) = state.player_items.as_ref() else {
        return;
    };
    let queries = AppearanceQueries::new(owner, state.weapon_cache.primary, state.player.entity_source, None);
    let deferred = owner.query_history.as_ref().and_then(|history| history.temporary_evaluation.as_ref())
        .map(|frontier| frontier.possible_cache_mask.map(|mask| scalar_evaluation_footprint(queries, mask)));
    let Some(history) = state.player_items.as_mut().and_then(|owner| owner.query_history.as_mut()) else {
        return;
    };
    if let Some(mask) = deferred {
        //Expansion belongs to this reached Evaluate boundary, not the room
        //writer. Existing numeric outputs survive as unavailable values.
        if let Some(frontier) = &mut history.temporary_evaluation {
            frontier.possible_cache_mask = mask;
        }
        history.reach_temporary_evaluation();
    }
    //531E5F clears CF8's pending word, then owns positive-TTL decrements
    //and transition-to-zero erasure before the515F40 source queries.
    advance_known_temporary_expiry(state);
    let Some(owner) = state.player_items.as_ref() else {
        return;
    };
    let queries = AppearanceQueries::new(owner, state.weapon_cache.primary, state.player.entity_source, None);
    //These are source515F40's only direct CF8-mutated IDs. Other TEMP
    //records (including45) do not imply either of these reached branches.
    let possible_taurus = queries.temporary_present(299) != Some(false);
    let possible_taurus_cache = possible_taurus || queries.have(299) != Some(false);
    let possible_breath = match queries.temporary_cooldown(326) {
        Some(Some(1)) | None => true,
        Some(_) => false,
    };
    let taurus_cache_mask = scalar_evaluation_footprint(queries, 0x10);
    let Some(history) = state.player_items.as_mut().and_then(|owner| owner.query_history.as_mut()) else {
        return;
    };
    if possible_taurus_cache {
        //516175/51663B can Evaluate actual cache16 within515F40 itself.
        //Missing D5C/room-clear inputs do not establish which branch ran.
        history.invalidate_cached_footprint(taurus_cache_mask);
    }
    if possible_taurus {
        //5164CD/516EC6 removal and516F11 replacement mutate only TEMP299.
        //Their deferred CF8 effects are considered at the next full pass.
        if let Some(key) = TemporaryQueryKey::collectible(299) {
            history.note_temporary_writer(&[key], Some(0x10), 0x516f11);
        }
    }
    if possible_breath {
        //517268 removes TEMP326 before the unowned virtual damage callback
        //and5172F7 re-add. That reached foreign effect has no closed write
        //footprint yet. Do not infer its history from the retained vector.
        history.forget_unowned_effect();
    }
}

fn advance_known_temporary_expiry(state: &mut State) {
    let Some(owner) = state.player_items.as_mut() else {
        return;
    };
    let Some(history) = &mut owner.query_history else {
        return;
    };
    if !history.temporary_records {
        return;
    }
    let mut expired = Vec::new();
    for record in &mut owner.temporary_collectibles {
        if !history.temporary_collectible_known(record.item_id) || record.cooldown <= 0 {
            continue;
        }
        record.cooldown -= 1;
        if record.cooldown == 0 {
            expired.push(record.item_id);
        }
    }
    for id in expired {
        //5E3EF0 erases the actual record before its517580 removal effect.
        owner.temporary_collectibles.retain(|record| record.item_id != id);
        let Some(config) = items::definition(id) else {
            history.note_temporary_writer(&[], None, 0x5e3ef0);
            continue;
        };
        history.note_temporary_writer(&[], Some(config.cache_flags), 0x5e3ef0);
        if !matches!(id, 35 | 45) || config.kind != 3 || config.cache_flags != 0 {
            //The actual CF8 erasure is independent of unimplemented
            //costume/health/audio effects.517580 has no direct E1C/CF8/C08
            //store. Keep its frontier without erasing unrelated output
            //facts; config-mask evaluation remains deferred to531E44.
            history.unresolved_effects.insert(0x517580,
                format!("Unimplemented removal effect for config kind{} id{}", config.kind, id));
            if id == 282 {
                //5177A3..7BA is the sole direct immediateEvaluate here.
                history.invalidate_cached_footprint(0x80);
            }
        }
    }
}

///The ordinary damage caller has already passed its real zero/dead/BF8
///refusal. Source313 can still consume a record and returnfalse before HP.
///Return whether the normal later source path remains possible. Unknown313
///must union both paths; this does not change the existing health implementation.
pub(crate) fn before_damage_queries(state: &mut State, health_units: u8, flags: u32) -> bool {
    let normal_source_context = state.weapon_admission.source_player_type == Some(0)
        && state.player.entity_source.is_some_and(|source| {
            source.entity_type == 1 && source.variant == 0
        });
    if !normal_source_context {
        if let Some(history) = state.player_items.as_mut()
            .and_then(|owner| owner.query_history.as_mut()) {
            history.forget_unowned_effect();
            history.unresolved_effects.insert(0x52b080,
                "Unowned Player entity/character damage callback".into());
        }
        return false;
    }
    let Some(owner) = state.player_items.as_ref() else {
        return false;
    };
    let queries = AppearanceQueries::new(owner, state.weapon_cache.primary, state.player.entity_source, None);
    //Read certainty before recording313 uncertainty. A possible refusal
    //alone cannot prune the possible ordinary path and its later writers.
    let certain_shield = flags & 0x20 == 0
        && queries.temporary_present(313) == Some(true);
    let shield = DamageQueryWritePlan::after_immunity_before_shield(queries, flags);
    let possible_shield = shield.possible_shield_refusal;
    apply_damage_query_plan(state, shield);
    if possible_shield && certain_shield {
        return false;
    }
    let Some(owner) = state.player_items.as_ref() else {
        return false;
    };
    let health_known = owner.query_history.as_ref()
        .is_some_and(|history| history.damage_health_known);
    let plan = DamageQueryWritePlan::before_health_store(
        AppearanceQueries::new(owner, state.weapon_cache.primary, state.player.entity_source, None),
        health_known.then_some(i32::from(state.player.hp)),
        health_known.then_some(state.player.soul_hp), Some(i32::from(health_units)),
    );
    !apply_damage_query_plan(state, plan)
}

pub(crate) fn after_damage_queries(state: &mut State, normal_source_path: bool) {
    if !normal_source_path {
        return;
    }
    let Some(owner) = state.player_items.as_ref() else {
        return;
    };
    let health_known = owner.query_history.as_ref()
        .is_some_and(|history| history.damage_health_known);
    let plan = DamageQueryWritePlan::after_health_store(
        AppearanceQueries::new(owner, state.weapon_cache.primary, state.player.entity_source, None),
        health_known.then_some(i32::from(state.player.hp)),
        health_known.then_some(state.player.soul_hp),
        state.weapon_admission.source_player_type,
        health_known.then_some(state.player.dead), None,
    );
    apply_damage_query_plan(state, plan);
}

///Apply only query/cache uncertainty at these actual source boundaries.
///Random-choice and cosmetic frontiers do not erase unrelated inputs.
fn apply_damage_query_plan(state: &mut State, plan: DamageQueryWritePlan) -> bool {
    let Some(history) = state.player_items.as_mut().and_then(|owner| owner.query_history.as_mut()) else {
        return true;
    };
    let mut unowned_use = false;
    if plan.possible_shield_refusal {
        //Source313 can refuse the HP write that the port still applies.
        //Keep both future health paths possible; cached fields stay intact.
        history.damage_health_known = false;
    }
    for write in plan.temporary {
        if write.writer_pc == 0x52cc0e {
            //52CC01 heals soul HP by2 before this actual142 removal. The
            //port has not implemented that heal; later HP reads are unknown.
            history.damage_health_known = false;
        }
        let key = TemporaryQueryKey {
            config_kind: u32::from(write.config_kind), item_id: write.item_id,
        };
        history.note_temporary_writer(&[key], write.possible_cache_mask, write.writer_pc);
        history.unresolved_effects.insert(write.writer_pc,
            format!("{:?}: unimplemented CF8 writer kind{} id{}", write.boundary, write.config_kind, write.item_id));
    }
    if plan.possible_trinket20_removal {
        history.trinket_slots = false;
    }
    for effect in plan.foreign_effects {
        if effect.kind == DamageForeignEffectKind::NestedUse {
            //The unknown nested active effect can mutate every query store.
            //This is a reached effect frontier, never grant-time taint.
            history.forget_unowned_effect();
            unowned_use = true;
        }
        history.unresolved_effects.insert(effect.pc,
            format!("{:?}/{:?}: {}", effect.boundary, effect.kind, effect.law));
    }
    unowned_use
}

///One immutable view of the existing stores and retained source fields.
///Queries remain lazy so an earlier source match bypasses later owners.
#[derive(Clone, Copy)]
pub(crate) struct AppearanceQueries<'a> {
    owner: &'a PlayerItems,
    primary: PrimaryWeaponC08,
    entity: Option<PlayerEntitySource>,
    source_30e8: Option<i32>,
}

impl<'a> AppearanceQueries<'a> {
    pub(crate) fn new(
        owner: &'a PlayerItems,
        primary: PrimaryWeaponC08,
        entity: Option<PlayerEntitySource>,
        source_30e8: Option<i32>,
    ) -> Self {
        Self {
            owner,
            primary,
            entity,
            source_30e8,
        }
    }

    fn history(self) -> Option<&'a ItemQueryHistory> {
        //The persistent actual pocket frontier independently excludes a
        //completed query read, including imported/contradictory metadata.
        //This is not a D30/D38 or pending-cache surrogate.
        if self.owner.held_pockets.as_ref()?.frontier.is_some() {
            return None;
        }
        self.owner.query_history.as_ref()
    }

    ///5090C0 queries signed E1C+4*ID.327/328 first call actualHaveTrinket21;
    ///known absence bypasses its source-clock branch. Positive/unknown
    ///Paper ownership still leaves that reached query unavailable.
    pub(crate) fn have(self, id: usize) -> Option<bool> {
        if id == 0 || !self.history()?.permanent_counts {
            return None;
        }
        if matches!(id, 327 | 328) && self.has_trinket(21) != Some(false) {
            return None;
        }
        self.owner.owned_counts.get(id).map(|count| *count > 0)
    }

    ///5E3D70 searches actual CF8 config kinds1/3/4 for ID, not count>0.
    ///Known unrelated records therefore do not turn an absent query unknown.
    pub(crate) fn temporary_present(self, id: u32) -> Option<bool> {
        let history = self.history()?;
        if !history.temporary_collectible_known(id) {
            return None;
        }
        let config = items::definition(id)?;
        Some(
            matches!(config.kind, 1 | 3 | 4)
                && self.owner.temporary_collectibles.iter().any(|record| record.item_id == id),
        )
    }

    ///5E3E00 returns a matching record regardless of its signed count.
    ///A known absent record is distinct from an unknown record/TTL owner.
    fn temporary_cooldown(self, id: u32) -> Option<Option<i32>> {
        if !self.temporary_present(id)? {
            return Some(None);
        }
        self.owner.temporary_collectibles.iter()
            .find(|record| record.item_id == id)
            .map(|record| Some(record.cooldown))
    }

    ///Both actual E14/E18 empty and a known empty/nontrinket3058 close
    ///Have21, capacity and the ordinary scan independently. Positive held,
    ///queued-kind2 and Paper emulation remain the canonical350 owner's work.
    pub(crate) fn has_trinket(self, id: u32) -> Option<bool> {
        if id == 0
            || !self.history()?.trinket_slots
            || self.owner.trinket_slots != [0, 0]
        {
            return None;
        }
        //508FC0 reaches ordinaryHave139 even though both empty slots prove
        //the same result for either capacity. Retain that query provenance.
        let _capacity = if self.have(139)? { 2 } else { 1 };
        if let Some(id) = self.owner.queued_id {
            if items::definition(id)?.kind == 2 {
                return None;
            }
        }
        Some(false)
    }

    ///Direct ownership of retained C08; request completion is a separate
    ///C18 cleanup contract and pendingMask0 is not a field-history proof.
    pub(crate) fn primary(self) -> Option<PrimaryWeaponC08> {
        self.history()?.primary_c08.then_some(self.primary)
    }

    ///Flags/Color read subtype30E8 only on actual variant1. Normal variant0
    ///does not require it; no source subtype is guessed from actor metadata.
    pub(crate) fn homing_variant(self) -> Option<bool> {
        if self.entity?.variant == 1 {
            Some(self.source_30e8? == 4)
        } else {
            Some(false)
        }
    }
}

// Source-only pure footprint draft; no records, RNG draws or callback completion.
// Place inside item_queries.rs, reusing its AppearanceQueries type.

#[derive(Clone, Debug)]
pub(crate) struct TemporaryWriteFrontier {
    pub ids: Vec<u32>,
    pub possible_cache_mask: u32,
    pub writer_pc: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct UnresolvedRoomWriter {
    pub pc: u32,
    pub law: &'static str,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RoomTemporaryWritePlan {
    pub writes: Vec<TemporaryWriteFrontier>,
    pub unresolved: Vec<UnresolvedRoomWriter>,
}

impl RoomTemporaryWritePlan {
    ///The union of CF8 IDs that MAY be written by this reached room callback.
    ///None3074 is not known-false. HP is an actual current source operand;
    ///unknown low-health/costume branches do not become fake health updates.
    ///This plan neither executes nor claims the effects of any writer.
    pub(crate) fn read(
        queries: AppearanceQueries<'_>,
        initialized: Option<bool>,
        red_hp: i32,
        soul_hp: i32,
        source_bf5: Option<bool>,
        room_byte_1: Option<u8>,
        source_bfc: Option<u32>,
    ) -> Self {
        let mut plan = Self::default();
        if initialized == Some(false) {
            return plan; //510562..569; no query is reached.
        }
        if initialized.is_none() {
            plan.unresolved.push(UnresolvedRoomWriter {
                pc: 0x510562,
                law: "3074 unknown: retain the possible initialized path",
            });
        }

        //510622→510C1D Bound4. A successful branch then draws Bound4
        //from these four IDs. No branch or selected draw is fabricated.
        if queries.has_trinket(32) != Some(false) {
            plan.write(&[21, 71, 121, 120], 0x1b, 0x510caf);
            plan.unresolved.push(UnresolvedRoomWriter {
                pc: 0x510c1d,
                law: "trinket32 random admission and chosen CF8 record are unimplemented",
            });
        }

        //Source short circuits permanent313→TEMP313→BF5. An unknown
        //owner retains only the possible313 writer, not unrelated IDs.
        if queries.have(313) != Some(false)
            && queries.temporary_present(313) != Some(true)
            && source_bf5 != Some(true)
        {
            plan.write(&[313], 0, 0x510d96);
        }

        //5106E7→510E0D Bound18→literal670BC0. All18 are possible;
        //the source-config union is0x47F, so it does not include weapon100.
        if queries.have(191) != Some(false) {
            plan.write(
                &[1, 2, 3, 5, 6, 46, 53, 110, 115, 143, 150, 201, 261, 257, 259, 317, 237, 245],
                0x47f,
                0x510e24,
            );
            plan.unresolved.push(UnresolvedRoomWriter {
                pc: 0x510e0d,
                law: "191 selects one actual Bound18 result; no draw or record executed",
            });
        }

        //51074E's low-health branch rejoins510754 AFTER its optional
        //blood effect. It NEVER bypasses the Have299 query at51075C.
        if red_hp.wrapping_add(soul_hp) <= 1 && source_bfc != Some(10) {
            plan.unresolved.push(UnresolvedRoomWriter {
                pc: 0x5109d7,
                law: "possible variant0 low-health blood1000/7 branch before Have299; no CF8 footprint assigned",
            });
        }
        if queries.have(299) != Some(false)
            && room_byte_1 != Some(0)
        {
            plan.write(&[299], 0x10, 0x510e3f);
        }

        if queries.have(122) != Some(false)
            && queries.temporary_present(122) != Some(true)
        {
            //510420→5102F0 is the costume/config path, not a health
            //grant. Its full transitive costume footprint is separate.
            //Until that footprint is independently closed, the source
            //post-helper red-HP predicate may still admit122. Do not
            //substitute an initial HP read to skip the possible writer.
            plan.unresolved.push(UnresolvedRoomWriter {
                pc: 0x510dc6,
                law: "reached510420 costume helper; post-helper redHP/BFC guard remains a separate owner obligation",
            });
            plan.write(&[122], 0x11, 0x510df9);
        }
        if queries.have(142) != Some(false) {
            plan.write(&[142], 0, 0x510b79);
        }
        plan
    }

    fn write(&mut self, ids: &[u32], possible_cache_mask: u32, writer_pc: u32) {
        self.writes.push(TemporaryWriteFrontier {
            ids: ids.to_vec(),
            possible_cache_mask,
            writer_pc,
        });
    }
}


// Source-only pure footprint for Player52B080. Include beside the canonical
// AppearanceQueries; this does not execute damage, random draws, effects or
// CF8 writes. Each method requires the caller to reach its named boundary.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DamageQueryBoundary {
    AfterImmunityBeforeShield,
    BeforeHealthStore,
    AfterHealthStore,
}

#[derive(Clone, Debug)]
pub(crate) struct DamageTemporaryWriteFrontier {
    pub boundary: DamageQueryBoundary,
    pub config_kind: u8,
    pub item_id: u32,
    // Missing catalogue metadata is unavailable, never a zero-mask claim.
    pub possible_cache_mask: Option<u32>,
    pub writer_pc: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct DamageForeignEffectFrontier {
    pub boundary: DamageQueryBoundary,
    pub kind: DamageForeignEffectKind,
    pub pc: u32,
    pub law: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DamageForeignEffectKind {
    //The possible typed record already bounds the uncertainty. Do not erase
    //unrelated query stores merely because the selected draw is unavailable.
    RandomAdmission,
    ConfigCostume,
    PersistentBecInput,
    NestedUse,
    ShieldRefusal,
    SpecialPresence,
    TrinketRemoval,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct DamageQueryWritePlan {
    pub temporary: Vec<DamageTemporaryWriteFrontier>,
    pub foreign_effects: Vec<DamageForeignEffectFrontier>,
    // This is a possible source return, not the port's damage decision.
    pub possible_shield_refusal: bool,
    pub possible_trinket20_removal: bool,
}

impl DamageQueryWritePlan {
    //52B23B..52B260 already checked normal signedBF8, positive amount and
    //the earlier immunity predicates. No pickup/main-animation guard is
    //among those checks. The313 branch may mutate CF8 and returnfalse.
    pub(crate) fn after_immunity_before_shield(
        queries: AppearanceQueries<'_>,
        flags: u32,
    ) -> Self {
        let mut plan = Self::default();
        if flags & 0x20 == 0 && queries.temporary_present(313) != Some(false) {
            plan.write(DamageQueryBoundary::AfterImmunityBeforeShield, 1, 313, 0x52b298);
            plan.possible_shield_refusal = true;
            plan.foreign(DamageQueryBoundary::AfterImmunityBeforeShield, DamageForeignEffectKind::ShieldRefusal, 0x52b2bf,
                "313 removal precedes costume5102F0/BF8 store and false return; no admitted health callback is reached on that branch");
        }
        plan
    }

    //The source has passed the shield branch and reached52B82E. These
    //operands are the actual pre-health values, not values after damage.
    //Unknown random results retain a possible write; no RNG is consumed.
    pub(crate) fn before_health_store(
        queries: AppearanceQueries<'_>,
        red_hp: Option<i32>,
        soul_hp: Option<i32>,
        damage_units: Option<i32>,
    ) -> Self {
        use DamageQueryBoundary::BeforeHealthStore as B;
        let mut plan = Self::default();
        if queries.has_trinket(40) != Some(false) {
            //57BEC0 is the TRINKET getter. This is not collectible40.
            plan.temporary.push(DamageTemporaryWriteFrontier {
                boundary: B, config_kind: 2, item_id: 40,
                possible_cache_mask: Some(1), writer_pc: 0x52b8b2,
            });
            plan.foreign(B, DamageForeignEffectKind::RandomAdmission, 0x52b864, "trinket40 shared random admission is unimplemented; configKind2/cooldown from actual trinket40");
        }
        if queries.has_trinket(48) != Some(false) {
            plan.use_effect(B, 35, 0x52b94a);
            plan.foreign(B, DamageForeignEffectKind::RandomAdmission, 0x52b8f2, "trinket48 shared random admission; Info30 and nested Use35 only after result0");
        }
        if queries.have(162) != Some(false) {
            //Actual162 is Celtic Cross. The record is58, with cooldown
            //from config162+48; it is not TEMP162 or a Scapular branch.
            plan.write(B, 3, 58, 0x52ba23);
            plan.foreign(B, DamageForeignEffectKind::RandomAdmission, 0x52b9b3, "Have162 shared random admission; config58 record uses config162 cooldown, no selected draw is fabricated");
        }
        for (id, pc) in [(117, 0x52c2b6), (274, 0x52c29f), (214, 0x52c382)] {
            if queries.have(id) != Some(false) {
                plan.write(B, if id == 214 { 1 } else { 4 }, id as u32, pc);
            }
        }
        if queries.have(117) != Some(false) {
            plan.foreign(B, DamageForeignEffectKind::ConfigCostume, 0x52c2d9, "117 TEMP writer precedes costume5102F0; transitive costume effects remain separate");
        }
        if queries.has_trinket(8) != Some(false)
            && queries.temporary_present(93) != Some(true)
        {
            plan.use_effect(B, 93, 0x52bc38);
            plan.foreign(B, DamageForeignEffectKind::RandomAdmission, 0x52bc03, "trinket8 own2A54 random admission precedes TEMP93 absence and Use93");
        }
        if queries.have(262) != Some(false) {
            let crosses_two = match (red_hp, soul_hp, damage_units) {
                (Some(red), Some(soul), Some(damage)) => {
                    let total = red.wrapping_add(soul);
                    Some(total > 2 && total.wrapping_sub(damage) <= 2)
                }
                _ => None,
            };
            if crosses_two != Some(false) {
                plan.use_effect(B, 35, 0x52bcd2);
                plan.foreign(B, DamageForeignEffectKind::NestedUse, 0x52bc68, "262 checks actual pre-health signed HP sum crossing2 before Info31 and Use35");
            }
        }
        if queries.have(301) != Some(false) {
            plan.write(B, 1, 108, 0x52c1a9);
        }
        if queries.have(303) != Some(false) {
            plan.write(B, 3, 58, 0x52bd50);
            plan.foreign(B, DamageForeignEffectKind::RandomAdmission, 0x52bd3a, "Have303 own2714 random admission; result0 alone adds TEMP58");
        }
        if queries.have(157) != Some(false) {
            plan.write(B, 1, 157, 0x52c245);
            plan.foreign(B, DamageForeignEffectKind::PersistentBecInput, 0x52c24a, "157 writer is followed by actual BEC increment; config-kind-dependent effect/cached-input ownership is separate");
        }
        plan
    }

    //The caller supplies actual post-health values at52BE72. Low-health
    //branches have different guards; there is no single all-items test.
    pub(crate) fn after_health_store(
        queries: AppearanceQueries<'_>,
        red_hp: Option<i32>,
        soul_hp: Option<i32>,
        source_bfc: Option<u32>,
        dead: Option<bool>,
        source_bf0: Option<i32>,
    ) -> Self {
        use DamageQueryBoundary::AfterHealthStore as B;
        let mut plan = Self::default();
        let whore_gate = match (red_hp, source_bfc) {
            (Some(red), _) if red <= 1 => Some(true),
            (Some(red), Some(bfc)) => Some(red == 2 && bfc == 5),
            (Some(red), None) if red != 2 => Some(false),
            _ => None,
        };
        if dead != Some(true) && whore_gate != Some(false)
            && queries.have(122) != Some(false)
            && queries.temporary_present(122) != Some(true)
        {
            plan.write(B, 1, 122, 0x52c909);
        }
        if red_hp.is_some_and(|hp| hp > 1) {
            return plan;
        }
        //CanonicalHave327/328 first queries actual Paper21. Known absent
        //Paper permits ordinary signed E1C counts; positive/unavailable Paper
        //retainsNone until its actualGameCE9F4 emulation is source-owned.
        if queries.have(327) != Some(false) {
            plan.write(B, 3, 58, 0x52c775);
            plan.foreign(B, DamageForeignEffectKind::SpecialPresence, 0x52c705, "Have327 includes Paper21/actualGameCE9F4; config58 uses config327 cooldown and no count reconstruction");
        }
        if queries.have(328) != Some(false) {
            plan.use_effect(B, 35, 0x52ca2a);
            plan.foreign(B, DamageForeignEffectKind::SpecialPresence, 0x52c782, "Have328 includes Paper21/actualGameCE9F4; reaches nested Use35 before the later health reads");
        }
        let low_health_cluster = match (red_hp, soul_hp) {
            (Some(red), Some(soul)) => Some(soul <= 0 || (red <= 0 && soul == 1)),
            _ => None,
        };
        //If an unimplemented nested Use35 is possible, its transitive
        //effect cannot justify pruning later branches with stale HP reads.
        if low_health_cluster == Some(false) && queries.have(328) == Some(false) {
            return plan;
        }
        if queries.has_trinket(33) != Some(false)
            && queries.temporary_present(100) != Some(true)
        {
            plan.write(B, 4, 100, 0x52cc49);
        }
        if dead != Some(true) && queries.have(142) != Some(false)
            && queries.temporary_present(142) != Some(false)
        {
            plan.write(B, 1, 142, 0x52cc0e);
            plan.foreign(B, DamageForeignEffectKind::ConfigCostume, 0x52cc1d, "Scapular142 TEMP removal then510420 costume path; source presence is not positive record count");
        }
        if queries.has_trinket(20) != Some(false)
            && source_bf0.is_none_or(|counter| counter.wrapping_sub(1) <= 0)
        {
            plan.possible_trinket20_removal = true;
            plan.foreign(B, DamageForeignEffectKind::TrinketRemoval, 0x52c8a5, "actual trinket20 low-health BF0 decrement reaches50BBF0 removal; no permanent collectible20 write inferred");
        }
        plan
    }

    fn write(&mut self, boundary: DamageQueryBoundary, config_kind: u8, item_id: u32, writer_pc: u32) {
        self.temporary.push(DamageTemporaryWriteFrontier {
            boundary, config_kind, item_id,
            possible_cache_mask: items::definition(item_id)
                .filter(|config| config.kind == config_kind)
                .map(|config| config.cache_flags),
            writer_pc,
        });
    }

    fn use_effect(&mut self, boundary: DamageQueryBoundary, item_id: u32, caller_pc: u32) {
        //522349 costume511060 precedes the literal mandatory CF8 insertion
        //at522380. Report this known ID separately from unclosed Use effects.
        self.write(boundary, 3, item_id, caller_pc);
        self.foreign(boundary, DamageForeignEffectKind::NestedUse, caller_pc,
            "nested Use522250 has mandatory config record insertion at522380; costume511060/effect/child callbacks are separate, not completed by this plan");
    }

    fn foreign(&mut self, boundary: DamageQueryBoundary, kind: DamageForeignEffectKind, pc: u32, law: &'static str) {
        self.foreign_effects.push(DamageForeignEffectFrontier { boundary, kind, pc, law });
    }
}
