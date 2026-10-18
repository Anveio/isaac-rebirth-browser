//! Actual240 shuffle and the independently owned portion of its effects.
//! Pending cache callbacks do not become admitted by owning this event.
use crate::{items::Rng, State};
use serde::{Deserialize, Serialize};

///Unresolved health-helper effects only, not all common Add callbacks,
///completed achievements, or invented counters. Coin0/Have109 stays separate.
///Requests remain ordered, including repeated calls with an amount of zero.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Frontiers {
    pub health_source_unresolved: bool,
    pub achievement_unlocks: Vec<AchievementUnlockRequest>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AchievementUnlockRequest {
    pub achievement_id: u32,
}

#[derive(Clone, Copy)]
struct HealthSource {
    player_type: u32,
    variant: u32,
}

impl HealthSource {
    fn read(state: &State) -> Option<Self> {
        let entity = state.player.entity_source?;
        let player_type = state.weapon_admission.source_player_type?;
        if entity.entity_type != 1 {
            return None;
        }
        //Nonzero variants skip both this common inventory branch and the
        //selected health arm. Their health values are not read by this event.
        if entity.variant == 0 && player_type != 10 {
            let player = &state.player;
            if player.max_hp > 24
                || player.hp > player.max_hp
                || player.soul_hp < 0
                || player.soul_hp > 24 - i32::from(player.max_hp)
            {
                return None;
            }
        }
        Some(Self {
            player_type,
            variant: entity.variant,
        })
    }
}

///Transient proof of this actual normal240 preflight; no serialized inputs.
///Only this module constructs it after all existing source-domain guards.
#[derive(Clone, Copy)]
pub(crate) struct Admission {
    luck_history_preserved: bool,
    move_speed_history_preserved: bool,
    c70_history_preserved: bool,
    normal_inputs_preserved: bool,
    health_source: Option<HealthSource>,
}

impl Admission {
    pub(crate) fn preserves_normal_inputs(self) -> bool {
        self.normal_inputs_preserved
    }

    pub(crate) fn preserves_c70_history(self) -> bool {
        self.c70_history_preserved
    }

    pub(crate) fn preserves_move_speed_history(self) -> bool {
        self.move_speed_history_preserved
    }

    pub(crate) fn preserves_luck_history(self) -> bool {
        self.luck_history_preserved
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stat {
    MaxHealth,
    MoveSpeed,
    FireDelay,
    Damage,
    Range,
    ShotSpeed,
    Luck,
}

struct EventPlan {
    order: [Stat; 7],
}

impl EventPlan {
    fn draw(rng: &mut Rng) -> Self {
        let mut order = [
            Stat::MaxHealth,
            Stat::MoveSpeed,
            Stat::FireDelay,
            Stat::Damage,
            Stat::Range,
            Stat::ShotSpeed,
            Stat::Luck,
        ];
        //52D340..52D364: six draws7,6,5,4,3,2 from this actual item's RNG.
        for end in (1..7).rev() {
            let selected = rng.bounded((end + 1) as u32) as usize;
            order.swap(end, selected);
        }
        Self { order }
    }

    fn effects(&self) -> impl Iterator<Item = (Stat, i32)> + '_ {
        //52D369..52D528 applies the first6 in their actual shuffled order.
        self.order[..6]
            .iter()
            .copied()
            .enumerate()
            .map(|(position, stat)| (stat, if position < 4 { 1 } else { -1 }))
    }
}

///Only malformed actual RNG ownership is an error. Ordinary character and
///health contexts never refuse the grant or strand the public pickup queue.
pub(crate) fn preflight(state: &State) -> Result<Admission, String> {
    let health_source = HealthSource::read(state);
    //The old independently proved input footprint excludes reached unlock
    //helpers. This restricts preservation, never activation or RNG selection.
    let normal_inputs_preserved = health_source
        .is_some_and(|source| source.player_type == 0 && source.variant == 0)
        && state.player.max_hp <= 11
        && state.player.soul_hp <= 7;
    let owner = state
        .player_items
        .as_ref()
        .ok_or("Experimental Treatment Player item owner absent")?;
    let rng = owner
        .item_rngs
        .get(240)
        .ok_or("Experimental Treatment actual per-item RNG absent")?;
    if owner.owned_counts.len() != 347
        || owner.item_rngs.len() != 347
        || rng.state == 0
        || [rng.right_a, rng.left, rng.right_b] != [5, 9, 7]
    {
        return Err("Experimental Treatment requires its initialized RNG35/count owner".into());
    }
    //518AB0(0) reaches request2 above54 coins, or early511F10 when
    //actual5090C0(109) is true.109 is the ordinary permanent-count query;
    //its source path reads no TEMP/trinket/queue. Keep both unclosed calls
    //outside preservation only; neither condition can refuse this grant.
    let normal_inputs_preserved =
        normal_inputs_preserved && state.inventory.coins <= 54 && owner.owned_counts[109] == 0;
    //Only this closed no-foreign-item footprint owns all five Luck words.
    //Other stat owners keep their independent event-preservation policies.
    let luck_history_preserved = normal_inputs_preserved
        && owner.trinket_slots == [0, 0]
        && owner.temporary_collectibles.is_empty()
        && owner.queued_collectible_is_none_or(|queued| queued == 240)
        && owner.owned_counts[240] >= 0
        && owner
            .owned_counts
            .iter()
            .enumerate()
            .all(|(id, &count)| id == 240 || count == 0);
    //Common passive240 and its selected branch do not write D44/13BC/13C0.
    //This separate physical-input footprint makes no inventory-stage claim.
    let move_speed_history_preserved = normal_inputs_preserved
        && owner.trinket_slots == [0, 0]
        && owner.temporary_collectibles.is_empty()
        && owner.queued_collectible_is_none_or(|queued| queued == 240);
    //Independent sourceC70 footprint:240 touches no timer and cannot enter
    //constructor/restore/pocket/Full. Keep the initial no-foreign-item scope.
    //This separate proof preserves Some(value), never infers zero from counts.
    let c70_history_preserved = normal_inputs_preserved
        && owner.trinket_slots == [0, 0]
        && owner.temporary_collectibles.is_empty()
        && owner.queued_collectible_is_none_or(|queued| queued == 240)
        && owner.owned_counts[240] >= 0
        && owner
            .owned_counts
            .iter()
            .enumerate()
            .all(|(id, &count)| id == 240 || count == 0);
    Ok(Admission {
        luck_history_preserved,
        move_speed_history_preserved,
        c70_history_preserved,
        normal_inputs_preserved,
        health_source,
    })
}

fn mark_health_unresolved(state: &mut State) {
    state
        .player_items
        .as_mut()
        .expect("preflight item owner")
        .treatment240_frontiers
        .get_or_insert_with(Frontiers::default)
        .health_source_unresolved = true;
}

fn request_achievement(state: &mut State, achievement_id: u32) {
    state
        .player_items
        .as_mut()
        .expect("preflight item owner")
        .treatment240_frontiers
        .get_or_insert_with(Frontiers::default)
        .achievement_unlocks
        .push(AchievementUnlockRequest { achievement_id });
}

///5083B0: BFC10 has its positive-only one-soul branch. Other source types
///cap soul health, request80 when reached, then normalize the black mask.
fn add_soul_health(state: &mut State, source: HealthSource, delta: i32) {
    if source.player_type == 10 {
        if delta > 0 {
            state.player.soul_hp = 1;
        }
        return;
    }
    let limit = if source.variant == 1 { 12 } else { 24 };
    state.player.soul_hp = state.player.soul_hp.wrapping_add(delta).max(0);
    if i32::from(state.player.max_hp) + state.player.soul_hp > limit {
        state.player.soul_hp = limit - i32::from(state.player.max_hp);
    }
    if state.player.soul_hp > 7 {
        request_achievement(state, 80);
    }
    crate::player_items::normalize_black_hearts(&mut state.player);
}

///508540 dispatch is shared by common amount0 and the selected health arm.
///The unresolved5BD5E0 request is recorded before source5082D0 normalization.
fn add_max_health(state: &mut State, source: HealthSource, delta: i32) {
    if source.player_type == 10 {
        return;
    }
    if source.player_type == 4 {
        add_soul_health(state, source, delta);
        return;
    }
    let limit = if source.variant == 1 { 12 } else { 24 };
    state.player.max_hp = i32::from(state.player.max_hp)
        .wrapping_add(delta)
        .clamp(0, limit) as u8;
    state.player.soul_hp = state
        .player
        .soul_hp
        .min(limit - i32::from(state.player.max_hp));
    state.player.hp = state.player.hp.min(state.player.max_hp);
    if state.player.max_hp > 13 {
        request_achievement(state, 1);
    }
    crate::player_items::normalize_black_hearts(&mut state.player);
}

///Actual240 config has seven zero inventory amounts. Variant!=0 skips the
///entire common branch. For variant0 the four health calls still occur in
///order: max0, red0, soul0, black0 (whose tail is another soul0 call).
pub(crate) fn authored_grants(state: &mut State, admission: Admission) {
    let Some(source) = admission.health_source else {
        mark_health_unresolved(state);
        return;
    };
    if source.variant != 0 {
        return;
    }
    add_max_health(state, source, 0);
    if source.player_type != 10 {
        crate::player_items::add_red_hearts(&mut state.player, 0);
    }
    add_soul_health(state, source, 0);
    //508440 with amount0 writes no mask before its5083B0 tailcall.
    if source.player_type != 10 {
        add_soul_health(state, source, 0);
    }
}

///Preflight is performed at Add entry; this is the source's selected late event.
pub(crate) fn apply(state: &mut State, admission: Admission) {
    let plan = {
        let owner = state.player_items.as_mut().expect("preflight item owner");
        EventPlan::draw(&mut owner.item_rngs[240])
    };
    //Preserve the actual source's shuffled effect order, including health.
    for (stat, delta) in plan.effects() {
        match stat {
            Stat::MaxHealth => {
                if let Some(source) = admission.health_source {
                    //52D570 skips nonzero variants.52D574..52D58C protects
                    //the sole red container; BFC dispatch belongs to508540.
                    if source.variant == 0
                        && !(delta == -1 && state.player.max_hp == 2 && state.player.soul_hp == 0)
                    {
                        add_max_health(state, source, delta * 2);
                    }
                } else {
                    mark_health_unresolved(state);
                }
            }
            Stat::Damage => {
                let owner = state.player_items.as_mut().expect("preflight item owner");
                if let Some(input) = owner.damage_inputs.as_mut() {
                    input.apply_signed_delta(delta);
                } else {
                    //An unowned current word remains unknown and requested.
                    owner.unevaluated_cache_flags |= 1;
                }
            }
            Stat::FireDelay => {
                let owner = state.player_items.as_mut().expect("preflight item owner");
                if let Some(input) = owner.fire_delay_inputs.as_mut() {
                    input.apply_signed_delta(delta);
                } else {
                    owner.unevaluated_cache_flags |= 2;
                }
            }
            Stat::Range => {
                let owner = state.player_items.as_mut().expect("preflight item owner");
                if let Some(input) = owner.range_inputs.as_mut() {
                    input.apply_signed_delta(delta);
                } else {
                    owner.unevaluated_cache_flags |= 8;
                }
            }
            Stat::ShotSpeed => {
                if let Some(input) = state
                    .player_items
                    .as_mut()
                    .expect("preflight item owner")
                    .shot_speed_inputs
                    .as_mut()
                {
                    input.source_d38 = input.source_d38.wrapping_add(delta);
                }
            }
            Stat::Luck => {
                let owner = state.player_items.as_mut().expect("preflight item owner");
                if let Some(input) = owner.luck_inputs.as_mut() {
                    input.source_d3c = input.source_d3c.wrapping_add(delta);
                } else {
                    owner.unevaluated_cache_flags |= 1024;
                }
            }
            Stat::MoveSpeed => {
                let owner = state.player_items.as_mut().expect("preflight item owner");
                if let Some(input) = owner.move_speed_inputs.as_mut() {
                    input.apply_signed_delta(delta);
                } else {
                    owner.unevaluated_cache_flags |= 16;
                }
            }
        }
    }
    //Actual cache-all remains requested. Owning D38 and this health endpoint
    //grants no other scalar/cache, costume, getter, controller or contact proof.
}
