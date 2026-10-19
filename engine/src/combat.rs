//! Source player/NPC mass contact and ordinary nonplayer damage admission.
//!
//! Actors must pass their collision-class/removed checks and damage callbacks
//! before physical separation. Damage queues drain at that actor's next full
//! update, not when a later projectile or bomb admits damage. These kernels do
//! not instantiate enemies, supply AI, or replace subclass death callbacks.
use crate::{motion::WorldVector, phase::Phase};
use serde::{Deserialize, Serialize};

/// Apply an admitted fresh-Isaac damage callback. Each caller owns its source
/// flags, contact/impulse eligibility and observed presentation immunity;
/// health units do not imply a universal cooldown formula.
pub fn try_damage_player(
    state: &mut crate::State,
    health_units: u8,
    immunity_presentations: u32,
    flags: u32,
) -> Result<bool, String> {
    try_damage_player_from(state, health_units, immunity_presentations, flags, None)
}

/// `try_damage_player` with the source entity type, or for a projectile its
/// spawner's (EntityRef +0 / +8): a Greed, Keeper or Hanger hit steals coins.
pub fn try_damage_player_from(
    state: &mut crate::State,
    health_units: u8,
    immunity_presentations: u32,
    flags: u32,
    source_type: Option<u32>,
) -> Result<bool, String> {
    if health_units == 0 || state.player.dead || state.damage_cooldown != 0 {
        return Ok(false);
    }
    let normal_source_query_path =
        crate::item_queries::before_damage_queries(state, health_units, flags);
    //Reached Player damage can have unclosed item/effect producers.
    crate::ordinary_placement::invalidate(
        state,
        crate::ordinary_placement::Frontier::PlayerDamageEffects,
    );
    if matches!(source_type, Some(50 | 86 | 90)) {
        steal_coins(state)?;
    }
    crate::health_damage::apply_admitted(state, health_units, flags)?;
    state.damage_cooldown = immunity_presentations;
    crate::player_death::after_damage(state, flags)?;
    //52C668 resolves NO_KILL/dead before52BE72 low-health query branches.
    crate::item_queries::after_damage_queries(state, normal_source_query_path);
    crate::health_damage::finish_admitted(state);
    Ok(true)
}

/// 52bfa0..52c0b1, ahead of the hurt sound: with coins, Isaac loses one, then
/// drops `min(bounded(3) + 1, coins)` more as pennies 5.20.1 he can pick back
/// up, each with a seed, a speed `float·5 + 2` and a direction.
fn steal_coins(state: &mut crate::State) -> Result<(), String> {
    if state.inventory.coins == 0 {
        return Ok(());
    }
    state.inventory.coins -= 1;
    let shared = &mut state.boot_lifecycle.shared;
    let count = (shared.bounded(3) + 1).min(state.inventory.coins);
    let position = crate::bombs::player_world(state);
    for _ in 0..count {
        let shared = &mut state.boot_lifecycle.shared;
        let seed = shared.next_u32();
        let speed = shared.uniform() * 5.0 + 2.0;
        let angle = (f64::from(shared.uniform()) * std::f64::consts::TAU) as f32;
        let direction = WorldVector {
            x: crate::fmath::cos(angle),
            y: crate::fmath::sin(angle),
        };
        let id = state.next_pickup;
        state.next_pickup = id.checked_add(1).ok_or("stolen coin identity exhausted")?;
        let mut pickup = crate::pickups::Pickup::spawn(id, 20, 1, position, true)?;
        pickup.initialize_spawn_seed(seed)?;
        pickup.velocity = direction * speed;
        pickup.birth_game_update = state.game_update_count;
        state.room.pickups.push(pickup);
        let actor = crate::actors::ActorRef {
            kind: crate::actors::ActorKind::Pickup,
            id,
        };
        state.room.authored_actor_order.push(actor);
        state.actor_scheduler.admit(actor)?;
    }
    state.inventory.coins -= count;
    Ok(())
}

/// Player Update and its intermediate counterpart each advance admitted hurt
/// immunity once. Transition freeze does not dispatch either callback.
pub(crate) fn advance_player_immunity(state: &mut crate::State) {
    state.damage_cooldown = state.damage_cooldown.saturating_sub(1);
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContactBody {
    pub id: u32,
    pub entity_type: u32,
    pub position: WorldVector,
    pub velocity: WorldVector,
    pub radius: f32,
    pub mass: f32,
}

impl ContactBody {
    fn validate(self) -> Result<(), String> {
        if !self.position.is_finite()
            || !self.velocity.is_finite()
            || !self.radius.is_finite()
            || self.radius < 0.0
            || !self.mass.is_finite()
            || self.mass <= 0.0
        {
            return Err("contact requires finite world geometry and positive mass".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContactResolution {
    pub colliding: bool,
    pub player_displacement_blocked: bool,
}

/// Source physical overlap tests a strict radius sum after an axial broadphase.
/// This predicate is distinct from the explosion's squared-distance collector.
pub fn player_npc_overlap(player: &ContactBody, npc: &ContactBody) -> Result<bool, String> {
    player.validate()?;
    npc.validate()?;
    if player.entity_type != 1 || !(10..1000).contains(&npc.entity_type) {
        return Err("ordinary contact requires a player and an NPC descriptor".into());
    }
    contact_overlap(player, npc)
}

fn contact_overlap(player: &ContactBody, npc: &ContactBody) -> Result<bool, String> {
    player.validate()?;
    npc.validate()?;
    let delta = WorldVector {
        x: npc.position.x - player.position.x,
        y: npc.position.y - player.position.y,
    };
    let radius = player.radius + npc.radius;
    if !radius.is_finite() {
        return Err("contact radius sum overflowed".into());
    }
    if delta.x.abs() > radius || delta.y.abs() > radius {
        return Ok(false);
    }
    let distance_squared = delta.x * delta.x + delta.y * delta.y;
    if !distance_squared.is_finite() {
        return Err("contact distance overflowed".into());
    }
    Ok(distance_squared.sqrt() < radius)
}

/// Mass separation for an admitted ordinary player/NPC pair. The callback
/// evaluates the proposed player displacement against its own source collider.
/// Both bodies remain unchanged if validation or arithmetic fails.
pub fn resolve_player_npc_contact(
    player: &mut ContactBody,
    npc: &mut ContactBody,
    blocks_player_shift: impl FnOnce(WorldVector) -> bool,
) -> Result<ContactResolution, String> {
    if player.entity_type != 1 || !(10..1000).contains(&npc.entity_type) {
        return Err("ordinary player contact requires source actor types".into());
    }
    resolve_contact(player, npc, blocks_player_shift, |_, _| {})
}

/// A free empty collectible's callback returnsfalse. Its class4 type5 body
/// then uses445710's ordinary mass split and Player-half-share impulse.
pub fn resolve_player_pickup_contact(
    player: &mut ContactBody,
    pickup: &mut ContactBody,
    blocks_player_shift: impl FnOnce(WorldVector) -> bool,
) -> Result<ContactResolution, String> {
    if player.entity_type != 1 || pickup.entity_type != 5 {
        return Err("pickup contact requires Player1/Pickup5 bodies".into());
    }
    resolve_contact(player, pickup, blocks_player_shift, |_, _| {})
}

/// Free regular Pickup5 pairs with unit collision scale use445710. Their
///504820 callbacks return false in both modes, preserving the circle response.
pub(crate) fn resolve_pickup_pair(
    first: &mut ContactBody,
    second: &mut ContactBody,
) -> Result<ContactResolution, String> {
    if first.entity_type != 5 || second.entity_type != 5 {
        return Err("pickup pair requires two admitted Pickup5 bodies".into());
    }
    resolve_contact(first, second, |_| false, |_, _| {})
}

/// Fresh regular Tear callback5462a9 adds half incoming velocity to AttackFly,
/// then54638a advances the Tear by0.8 of the NPC velocity. Generic445710
/// retains its pre-callback overlap vector for mass separation afterward.
pub fn resolve_tear_npc_contact(
    tear: &mut ContactBody,
    npc: &mut ContactBody,
) -> Result<ContactResolution, String> {
    if tear.entity_type != 2 || crate::entities::family(npc.entity_type).is_none() {
        return Err("regular tear response requires an NPC with a registered family".into());
    }
    resolve_contact(
        tear,
        npc,
        |_| false,
        |tear, npc| {
            if npc.entity_type == 18 {
                npc.velocity = npc.velocity + tear.velocity * 0.5;
            }
            tear.position = tear.position + npc.velocity * 0.8;
        },
    )
}

/// Pairs of NPCs with a registered family have no callback veto.
/// The caller admits Fireplace's NPC-only queued damage alongside this response.
/// The first body may instead be a regular Bomb; its callback
/// 456c20 damps NPC velocity0.2 and its own0.5 before the mass response.
/// Source445710 sorts by entity type and retains its pre-callback overlap.
pub fn resolve_npc_or_bomb_pair(
    first: &mut ContactBody,
    second: &mut ContactBody,
) -> Result<ContactResolution, String> {
    let registered = |kind| crate::entities::family(kind).is_some();
    if !(first.entity_type == 4 || registered(first.entity_type)) || !registered(second.entity_type)
    {
        return Err(
            "NPC pairs require registered NPC families, with an optional first Bomb".into(),
        );
    }
    if first.entity_type > second.entity_type {
        resolve_contact(second, first, |_| false, bomb_pair_callback)
    } else {
        resolve_contact(first, second, |_| false, bomb_pair_callback)
    }
}

/// Fresh flags-zero hostile projectile538dd0 returns false after its Player
/// damage callback, so it retains the ordinary Player mass response445710.
pub fn resolve_player_projectile_contact(
    player: &mut ContactBody,
    projectile: &mut ContactBody,
    blocks_player_shift: impl FnOnce(WorldVector) -> bool,
) -> Result<ContactResolution, String> {
    if player.entity_type != 1 || projectile.entity_type != 9 {
        return Err("hostile projectile contact requires source types1/9".into());
    }
    resolve_contact(player, projectile, blocks_player_shift, |_, _| {})
}

///Player509160 returnsfalse for other type4 before Bomb456C20(flagfalse).
///A positive7DC resets4 and vetoes physics; expiry adds PlayerV*.4 first.
pub(crate) fn resolve_player_bomb_contact(
    player: &mut ContactBody,
    bomb: &mut ContactBody,
    countdown: &mut u8,
    blocks_player_shift: impl FnOnce(WorldVector) -> bool,
) -> Result<ContactResolution, String> {
    if player.entity_type != 1 || bomb.entity_type != 4 || *countdown > 4 {
        return Err("ordinary Player/Bomb contact requires actual types/countdown".into());
    }
    let mut next_countdown = *countdown;
    let result = resolve_contact_with_veto(player, bomb, blocks_player_shift, |player, bomb| {
        if next_countdown > 0 {
            next_countdown = 4;
            true
        } else {
            bomb.velocity = bomb.velocity + player.velocity * 0.4;
            false
        }
    })?;
    *countdown = next_countdown;
    Ok(result)
}

///Neutral456C20 first(flagtrue), then second(flagfalse); the latter is a
///no-op for otherBomb. First-markertrue vetoes only after both callbacks.
pub(crate) fn resolve_bomb_pair(
    first: &mut ContactBody,
    second: &mut ContactBody,
    first_shot_marker: bool,
) -> Result<ContactResolution, String> {
    if first.entity_type != 4 || second.entity_type != 4 {
        return Err("Bomb pair requires source actor type4".into());
    }
    resolve_contact_with_veto(
        first,
        second,
        |_| false,
        |first, second| {
            if first_shot_marker {
                true
            } else {
                bomb_pair_callback(first, second);
                false
            }
        },
    )
}

/// Beggars and Key Master (SizeMulti (1,1)) take 445710 like a free
/// pickup body: Slot539690 and Player509160 both return false here.
pub(crate) fn resolve_player_beggar_contact(
    player: &mut ContactBody,
    slot: &mut ContactBody,
    blocks_player_shift: impl FnOnce(WorldVector) -> bool,
) -> Result<ContactResolution, String> {
    if player.entity_type != 1 || slot.entity_type != 6 {
        return Err("beggar contact requires Player1/Slot6 bodies".into());
    }
    resolve_contact(player, slot, blocks_player_shift, |_, _| {})
}

/// 445be0's shape test for a pair with a Slot: either SizeMulti other than
/// (1,1) (612c20 operator!=) selects the capsule test, else the circle.
pub(crate) fn slot_overlap(
    other: &ContactBody,
    slot: &ContactBody,
    slot_size_multi: WorldVector,
    rotation_degrees: f32,
) -> Result<bool, String> {
    if slot_size_multi.x != 1.0 || slot_size_multi.y != 1.0 {
        other.validate()?;
        slot.validate()?;
        let (a, a_size, b, b_size) =
            lower_type_first(other, WorldVector { x: 1.0, y: 1.0 }, slot, slot_size_multi);
        let first = capsule(a, a_size, rotation_degrees)?;
        let second = capsule(b, b_size, rotation_degrees)?;
        Ok(capsule_penetration(&first, &second).is_some())
    } else {
        contact_overlap(other, slot)
    }
}

fn lower_type_first<'a>(
    first: &'a ContactBody,
    first_size: WorldVector,
    second: &'a ContactBody,
    second_size: WorldVector,
) -> (&'a ContactBody, WorldVector, &'a ContactBody, WorldVector) {
    if first.entity_type > second.entity_type {
        (second, second_size, first, first_size)
    } else {
        (first, first_size, second, second_size)
    }
}

/// 42d9d0 capsule: a segment p1..p2 with a radius. Only +448 position,
/// +474 radius, +478 SizeMulti and sprite rotation +2cc are read.
#[derive(Clone, Copy)]
struct Capsule {
    p1: WorldVector,
    p2: WorldVector,
    radius: f32,
}

fn capsule(
    body: &ContactBody,
    size_multi: WorldVector,
    rotation_degrees: f32,
) -> Result<Capsule, String> {
    // 6124a0 feeds glibc sinf/cosf/sincosf; only angle0 is exactly (0,1)
    // without them, and no admitted body writes another rotation.
    if rotation_degrees != 0.0 {
        return Err("capsule contact with sprite rotation +2cc is unadmitted".into());
    }
    let sx = size_multi.x * body.radius;
    let sy = body.radius * size_multi.y;
    // ucomiss/ja: NaN takes the vertical branch. Direction is (cos,sin)
    // horizontally and sincosf's (sin,cos) otherwise.
    let (radius, half_length, direction) = if sx > sy {
        (sy, sx - sy, WorldVector { x: 1.0, y: 0.0 })
    } else {
        (sx, sy - sx, WorldVector { x: 0.0, y: 1.0 })
    };
    let reach = WorldVector {
        x: direction.x * half_length,
        y: direction.y * half_length,
    };
    Ok(Capsule {
        p1: WorldVector {
            x: body.position.x - reach.x,
            y: body.position.y - reach.y,
        },
        p2: WorldVector {
            x: body.position.x + reach.x,
            y: body.position.y + reach.y,
        },
        radius,
    })
}

fn difference(a: WorldVector, b: WorldVector) -> WorldVector {
    WorldVector {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}

fn length_squared(v: WorldVector) -> f32 {
    v.x * v.x + v.y * v.y
}

/// 42dbc0: `point` minus its closest point on segment s1..s2.
fn from_segment(point: WorldVector, s1: WorldVector, s2: WorldVector) -> WorldVector {
    let span = difference(s2, s1);
    let span_squared = length_squared(span);
    // 666160: a segment shorter than 1e-4 squared is its first endpoint.
    if 1e-4_f32 > span_squared.abs() {
        return difference(point, s1);
    }
    let along = difference(point, s1);
    let t = (along.x * span.x + along.y * span.y) / span_squared;
    if 0.0 > t {
        return difference(point, s1);
    }
    if t >= 1.0 {
        return difference(point, s2);
    }
    let closest = WorldVector {
        x: s1.x + span.x * t,
        y: s1.y + span.y * t,
    };
    difference(point, closest)
}

/// 42dd10. Parallel or degenerate segments (denominator ±0) never cross;
/// the crossing point applies B's parameter to A's segment, as the source does.
fn segments_cross(
    a1: WorldVector,
    a2: WorldVector,
    b1: WorldVector,
    b2: WorldVector,
) -> Option<WorldVector> {
    let d1 = difference(a2, a1);
    let d2 = difference(b2, b1);
    let denominator = d1.y * d2.x - d1.x * d2.y;
    if denominator == 0.0 {
        return None;
    }
    let u = ((b1.y - a1.y) * d1.x + (a1.x - b1.x) * d1.y) / denominator;
    // jb also rejects an unordered NaN parameter.
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let s = ((a1.y - b1.y) * d2.x + (b1.x - a1.x) * d2.y) / -denominator;
    if !(0.0..=1.0).contains(&s) {
        return None;
    }
    Some(WorldVector {
        x: a1.x + d1.x * u,
        y: a1.y + d1.y * u,
    })
}

/// 42dec0 capsule penetration, pointing from A toward B, or None when the
/// strict `(rA+rB)^2 > dist2` test fails.
fn capsule_penetration(a: &Capsule, b: &Capsule) -> Option<WorldVector> {
    let mut offset = WorldVector { x: 0.0, y: 0.0 };
    if let Some(crossing) = segments_cross(a.p1, a.p2, b.p1, b.p2) {
        let mut best_vector = difference(crossing, a.p1);
        let mut best = length_squared(best_vector);
        let mut radius = a.radius;
        for (end, end_radius) in [(a.p2, a.radius), (b.p1, b.radius), (b.p2, b.radius)] {
            let vector = difference(crossing, end);
            let squared = length_squared(vector);
            if best > squared {
                best_vector = vector;
                best = squared;
                radius = end_radius;
            }
        }
        let depth = best.sqrt() + radius;
        // 612860 Normalize multiplies by a rounded reciprocal.
        let length = length_squared(best_vector).sqrt();
        if length > 0.0 {
            let inverse = 1.0 / length;
            best_vector = WorldVector {
                x: best_vector.x * inverse,
                y: best_vector.y * inverse,
            };
        }
        offset = WorldVector {
            x: best_vector.x * depth,
            y: best_vector.y * depth,
        };
    }
    // 42e1d0 shifts all four endpoints: no relative change, but it rounds.
    let shift = |point: WorldVector| {
        if offset.x != 0.0 || offset.y != 0.0 {
            WorldVector {
                x: point.x + offset.x,
                y: point.y + offset.y,
            }
        } else {
            point
        }
    };
    let (a1, a2, b1, b2) = (shift(a.p1), shift(a.p2), shift(b.p1), shift(b.p2));
    let negate = |v: WorldVector| WorldVector { x: -v.x, y: -v.y };
    let candidates = [
        negate(from_segment(a1, b1, b2)),
        negate(from_segment(a2, b1, b2)),
        from_segment(b1, a1, a2),
        from_segment(b2, a1, a2),
    ];
    let squared = candidates.map(length_squared);
    // Ties keep the second candidate of each side, then the B side.
    let (min_a, vector_a) = if squared[1] > squared[0] {
        (squared[0], candidates[0])
    } else {
        (squared[1], candidates[1])
    };
    let (min_b, vector_b) = if squared[3] > squared[2] {
        (squared[2], candidates[2])
    } else {
        (squared[3], candidates[3])
    };
    let (distance_squared, selected) = if min_b > min_a {
        (min_a, vector_a)
    } else {
        (min_b, vector_b)
    };
    let radius = a.radius + b.radius;
    let overlapping = radius * radius > distance_squared;
    if !overlapping {
        return None;
    }
    // 666164: coincident closest points yield the (usually zero) offset.
    if distance_squared == 0.0 {
        return Some(offset);
    }
    let distance = distance_squared.sqrt();
    let depth = radius - distance;
    let normal = WorldVector {
        x: selected.x / distance,
        y: selected.y / distance,
    };
    Some(WorldVector {
        x: normal.x * depth + offset.x,
        y: normal.y * depth + offset.y,
    })
}

/// 4454a0, the pair for a SizeMulti other than (1,1). A is the lower entity
/// type. Unlike 445710 there is no high-speed reflection, Player half share,
/// grid-shift veto or 1e-4 distance exit: mass shares move both bodies and
/// an impulse of |B.vel - A.vel| pushes them apart along the penetration.
/// The caller has already run both collision callbacks (each false here).
pub(crate) fn resolve_capsule_contact(
    first: &mut ContactBody,
    first_size_multi: WorldVector,
    second: &mut ContactBody,
    second_size_multi: WorldVector,
    rotation_degrees: f32,
) -> Result<ContactResolution, String> {
    first.validate()?;
    second.validate()?;
    let swap = first.entity_type > second.entity_type;
    let (a, a_size, b, b_size) = if swap {
        (second, second_size_multi, first, first_size_multi)
    } else {
        (first, first_size_multi, second, second_size_multi)
    };
    let Some(penetration) = capsule_penetration(
        &capsule(a, a_size, rotation_degrees)?,
        &capsule(b, b_size, rotation_degrees)?,
    ) else {
        return Ok(ContactResolution {
            colliding: false,
            player_displacement_blocked: false,
        });
    };
    let mut next_a = *a;
    let mut next_b = *b;
    let a_share = a.mass / (b.mass + a.mass);
    let b_share = 1.0 - a_share;
    next_a.position.x -= penetration.x * b_share;
    next_a.position.y -= penetration.y * b_share;
    next_b.position.x += penetration.x * a_share;
    next_b.position.y += penetration.y * a_share;
    if penetration.x != 0.0 || penetration.y != 0.0 {
        let relative = difference(b.velocity, a.velocity);
        let speed = length_squared(relative).sqrt();
        let length = length_squared(penetration).sqrt();
        let impulse = WorldVector {
            x: penetration.x / length * speed,
            y: penetration.y / length * speed,
        };
        next_a.velocity.x -= impulse.x * b_share;
        next_a.velocity.y -= impulse.y * b_share;
        next_b.velocity.x += impulse.x * a_share;
        next_b.velocity.y += impulse.y * a_share;
    }
    next_a.validate()?;
    next_b.validate()?;
    *a = next_a;
    *b = next_b;
    Ok(ContactResolution {
        colliding: true,
        player_displacement_blocked: false,
    })
}

fn bomb_pair_callback(first: &mut ContactBody, second: &mut ContactBody) {
    if first.entity_type == 4 {
        second.velocity = second.velocity * 0.2;
        first.velocity = first.velocity * 0.5;
    }
}

fn resolve_contact(
    player: &mut ContactBody,
    npc: &mut ContactBody,
    blocks_player_shift: impl FnOnce(WorldVector) -> bool,
    callback: impl FnOnce(&mut ContactBody, &mut ContactBody),
) -> Result<ContactResolution, String> {
    resolve_contact_with_veto(player, npc, blocks_player_shift, |first, second| {
        callback(first, second);
        false
    })
}

fn resolve_contact_with_veto(
    player: &mut ContactBody,
    npc: &mut ContactBody,
    blocks_player_shift: impl FnOnce(WorldVector) -> bool,
    callback: impl FnOnce(&mut ContactBody, &mut ContactBody) -> bool,
) -> Result<ContactResolution, String> {
    if !contact_overlap(player, npc)? {
        return Ok(ContactResolution {
            colliding: false,
            player_displacement_blocked: false,
        });
    }
    let mut delta = WorldVector {
        x: npc.position.x - player.position.x,
        y: npc.position.y - player.position.y,
    };
    let mut distance = (delta.x * delta.x + delta.y * delta.y).sqrt();
    let mut next_player = *player;
    let mut next_npc = *npc;
    let veto = callback(&mut next_player, &mut next_npc);
    next_player.validate()?;
    next_npc.validate()?;
    if veto {
        //445429 ORs both callback returns;445814 skips generic mass response.
        *player = next_player;
        *npc = next_npc;
        return Ok(ContactResolution {
            colliding: false,
            player_displacement_blocked: false,
        });
    }
    if distance < 0.0001 {
        *player = next_player;
        *npc = next_npc;
        return Ok(ContactResolution {
            colliding: true,
            player_displacement_blocked: false,
        });
    }
    let relative = WorldVector {
        x: next_player.velocity.x - next_npc.velocity.x,
        y: next_player.velocity.y - next_npc.velocity.y,
    };
    let relative_speed = (relative.x * relative.x + relative.y * relative.y).sqrt();
    let radius = player.radius + npc.radius;
    let combined_mass = player.mass + npc.mass;
    if !relative_speed.is_finite() || !combined_mass.is_finite() {
        return Err("contact speed or mass overflowed".into());
    }
    if relative_speed > radius * 0.5 && relative.x * delta.x + relative.y * delta.y < 0.0 {
        // Source high-speed separating branch adjusts the overlap direction
        // before the ordinary mass split. Independent slow-contact acceptance
        // is recorded separately from this statically recovered branch.
        // Vector::Normalize (612860 -> 612820) rounds the reciprocal first.
        // The later impulse's Vector::operator/ uses direct division instead.
        let normalized_delta = delta * (1.0 / distance);
        let normalized_relative = relative * (1.0 / relative_speed);
        let dot =
            normalized_relative.x * normalized_delta.x + normalized_relative.y * normalized_delta.y;
        delta = delta + (delta * dot) * 2.0;
        distance = (delta.x * delta.x + delta.y * delta.y).sqrt();
        if distance <= 0.0 || !distance.is_finite() {
            return Err("separating contact direction is degenerate".into());
        }
    }
    let overlap_ratio = (radius - distance) / distance;
    let mut npc_share = player.mass / combined_mass;
    let mut player_share = 1.0 - npc_share;
    let shift = (delta * -player_share) * overlap_ratio;
    let blocked = blocks_player_shift(shift);
    if blocked {
        player_share = 0.0;
        npc_share = 1.0;
    }
    next_player.position.x -= (delta.x * player_share) * overlap_ratio;
    next_player.position.y -= (delta.y * player_share) * overlap_ratio;
    next_npc.position = next_npc.position + (delta * npc_share) * overlap_ratio;
    if player.entity_type == 1 {
        player_share *= 0.5;
    }
    let impulse = WorldVector {
        x: delta.x / distance * relative_speed,
        y: delta.y / distance * relative_speed,
    };
    next_player.velocity.x -= impulse.x * player_share;
    next_player.velocity.y -= impulse.y * player_share;
    next_npc.velocity = next_npc.velocity + impulse * npc_share;
    next_player.validate()?;
    next_npc.validate()?;
    *player = next_player;
    *npc = next_npc;
    Ok(ContactResolution {
        colliding: true,
        player_displacement_blocked: blocked,
    })
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActorHealth {
    pub hp: f32,
    pub max_hp: f32,
}

impl ActorHealth {
    pub fn validate(self) -> Result<(), String> {
        if !self.hp.is_finite() || !self.max_hp.is_finite() || self.max_hp < 0.0 {
            return Err("actor health requires finite HP and nonnegative maximum HP".into());
        }
        Ok(())
    }
}

/// Ordinary tear, regular blast and Fireplace NPC-contact flags are admitted here.
/// Contact/periodic damage flags with other cooldown rules require their own
/// receiver contract; player red-heart damage uses its separate source path.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DamageKind {
    Tear,
    Knife,
    Bomb,
    /// TNT grid blast forwards DAMAGE_EXPLOSION | DAMAGE_TNT.
    Tnt,
    FireContact,
    RoomEffect,
    /// Entity_Laser do_damage476760 passes DAMAGE_LASER.
    Laser,
    /// Grub head HP sync 4f43d9: TakeDamage with flags 0.
    HealthSync,
}

impl DamageKind {
    pub fn flags(self) -> u32 {
        match self {
            Self::Tear | Self::Knife => 0,
            Self::Bomb => 4,
            Self::Tnt => 0x1004,
            Self::FireContact => 2,
            Self::Laser => 8,
            Self::RoomEffect => 0,
            Self::HealthSync => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DamageSource {
    pub id: u32,
    pub entity_type: u32,
}

///The actual EntityRef words copied before a damage callback. This snapshot
///never resolves a moving/retired actor to fill historical position or spawner.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CopiedDamageSource {
    pub entity_type: u32,
    pub variant: u32,
    pub spawner_type: u32,
    pub spawner_variant: u32,
    pub position: WorldVector,
    pub velocity: WorldVector,
    ///Actual actor reference captured with the copy. This is not a claim that
    ///the actor is still registered/live or that Uninit clears this raw word.
    pub actor_at_copy: Option<crate::actors::ActorRef>,
}
impl CopiedDamageSource {
    fn validate(self, source: DamageSource) -> Result<(), String> {
        if !self.position.is_finite()
            || !self.velocity.is_finite()
            || self.entity_type != source.entity_type
        {
            return Err("copied damage source requires actual finite matching fields".into());
        }
        if let Some(actor) = self.actor_at_copy {
            use crate::actors::ActorKind;
            let matches_kind = match actor.kind {
                ActorKind::Player => self.entity_type == 1,
                ActorKind::Familiar => self.entity_type == 3,
                ActorKind::Tear => self.entity_type == 2,
                ActorKind::Bomb => self.entity_type == 4,
                ActorKind::Pickup => self.entity_type == 5,
                ActorKind::Slot => self.entity_type == 6,
                ActorKind::Laser => self.entity_type == 7,
                ActorKind::Knife => self.entity_type == 8,
                ActorKind::Projectile => self.entity_type == 9,
                ActorKind::Npc => (10..1000).contains(&self.entity_type),
                ActorKind::Effect => self.entity_type == 1000,
            };
            if !matches_kind || actor.id != source.id {
                return Err("copied damage source actor differs from its actual source".into());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DamageEvent {
    pub amount: f32,
    pub kind: DamageKind,
    pub source: DamageSource,
    ///None is an unowned caller/legacy history, never a zero payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub copied_source: Option<CopiedDamageSource>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DamageQueue {
    events: Vec<DamageEvent>,
    pending_damage: f32,
    pending_flags: u32,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppliedDamage {
    pub event: DamageEvent,
    pub hp_after: f32,
    pub lethal: bool,
}

impl DamageQueue {
    pub fn pending_damage(&self) -> f32 {
        self.pending_damage
    }
    pub fn pending_flags(&self) -> u32 {
        self.pending_flags
    }
    pub fn events(&self) -> &[DamageEvent] {
        &self.events
    }

    /// Subclass AI may consume the ordered queue before common BaseUpdate.
    /// Ordinary Fireplace49a722 clears these fields after its own HP law.
    pub(crate) fn clear(&mut self) {
        self.events.clear();
        self.pending_damage = 0.0;
        self.pending_flags = 0;
    }

    /// A subclass AI zeroing queued entries in place (Boil 490ef0): the drain
    /// subtracts nothing for them; the pending total and flags stay.
    pub(crate) fn zero_events(&mut self, mut zero: impl FnMut(&DamageEvent) -> bool) {
        for event in &mut self.events {
            if zero(event) {
                event.amount = 0.0;
            }
        }
    }

    /// A subclass AI erasing queued entries (4fd320 vector erase) leaves the
    /// pending total and flags as they were.
    pub(crate) fn erase_events(&mut self, erase: impl Fn(&DamageEvent) -> bool) {
        self.events.retain(|event| !erase(event));
    }

    /// The source admits positive damage when maximum HP is positive; damage
    /// does not immediately change current HP. Boss/co-op scaling belongs to
    /// admission before this ordinary single-player queue contract.
    pub fn enqueue(&mut self, health: ActorHealth, event: DamageEvent) -> Result<bool, String> {
        health.validate()?;
        if let Some(copied) = event.copied_source {
            copied.validate(event.source)?;
        }
        if !event.amount.is_finite() {
            return Err("damage amount must be finite".into());
        }
        if event.amount <= 0.0 || health.max_hp <= 0.0 {
            return Ok(false);
        }
        let pending = self.pending_damage + event.amount;
        if !pending.is_finite() {
            return Err("pending damage overflowed".into());
        }
        self.events.push(event);
        self.pending_damage = pending;
        self.pending_flags |= event.kind.flags();
        Ok(true)
    }

    /// Invoke at this actor's update boundary before later actor/collision
    /// admissions. A nonplayer's intermediate phase preserves the queue.
    /// Negative HP is retained because subclass overkill/death rules inspect it.
    pub fn advance(
        &mut self,
        health: &mut ActorHealth,
        phase: Phase,
    ) -> Result<Vec<AppliedDamage>, String> {
        health.validate()?;
        if !phase.updates_nonplayers() {
            return Ok(vec![]);
        }
        let mut hp = health.hp;
        let mut applied = Vec::with_capacity(self.events.len());
        for &event in &self.events {
            hp -= event.amount;
            if !hp.is_finite() {
                return Err("actor damage overflowed HP".into());
            }
            applied.push(AppliedDamage {
                event,
                hp_after: hp,
                // Source4478b4 enters Kill only below zero; equality continues Hit.
                lethal: hp < 0.0,
            });
        }
        health.hp = hp;
        self.events.clear();
        self.pending_damage = 0.0;
        self.pending_flags = 0;
        Ok(applied)
    }
}
