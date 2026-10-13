//! Source player/NPC mass contact and ordinary nonplayer damage admission.
//!
//! Actors must pass their collision-class/removed checks and damage callbacks
//! before physical separation. Damage queues drain at that actor's next full
//! update, not when a later projectile or bomb admits damage. These kernels do
//! not instantiate enemies, supply AI, or replace subclass death callbacks.
use crate::{motion::WorldVector, phase::Phase};
use serde::{Deserialize, Serialize};

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
    if !player_npc_overlap(player, npc)? {
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
    if distance < 0.0001 {
        return Ok(ContactResolution {
            colliding: true,
            player_displacement_blocked: false,
        });
    }
    let relative = WorldVector {
        x: player.velocity.x - npc.velocity.x,
        y: player.velocity.y - npc.velocity.y,
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
    let mut next_player = *player;
    let mut next_npc = *npc;
    next_player.position.x -= (delta.x * player_share) * overlap_ratio;
    next_player.position.y -= (delta.y * player_share) * overlap_ratio;
    next_npc.position = next_npc.position + (delta * npc_share) * overlap_ratio;
    player_share *= 0.5;
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

/// Only observed ordinary tear and regular blast flags are admitted here.
/// Contact/periodic damage flags with other cooldown rules require their own
/// receiver contract; player red-heart damage uses its separate source path.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DamageKind {
    Tear,
    Bomb,
}

impl DamageKind {
    pub fn flags(self) -> u32 {
        match self {
            Self::Tear => 0,
            Self::Bomb => 4,
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

#[derive(Clone, Debug, Default, Serialize, PartialEq)]
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
                lethal: hp <= 0.0,
            });
        }
        health.hp = hp;
        self.events.clear();
        self.pending_damage = 0.0;
        self.pending_flags = 0;
        Ok(applied)
    }
}
