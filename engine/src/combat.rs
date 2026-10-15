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
    if health_units == 0 || state.player.dead || state.damage_cooldown != 0 {
        return Ok(false);
    }
    crate::health_damage::apply_admitted(state, health_units, flags)?;
    state.damage_cooldown = immunity_presentations;
    crate::player_death::after_damage(state, flags)?;
    crate::health_damage::finish_admitted(state);
    Ok(true)
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

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DamageEvent {
    pub amount: f32,
    pub kind: DamageKind,
    pub source: DamageSource,
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
