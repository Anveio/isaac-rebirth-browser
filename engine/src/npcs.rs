//! Supported NPC actors from authored room selectors.
//! Full NPC Update owns AI, native BaseUpdate motion and FIFO damage drain.
//! Intermediate presentation predicts half velocity without advancing AI.
use crate::entities::{Birth, Family, NpcWorld, SpawnContext};
use crate::plain_npc::{ShooterTimers, ShotMode, ShotParameters};
use crate::{
    bombs, combat, content,
    grid::Grid,
    motion::WorldVector,
    mt::Mt19937,
    phase::Phase,
    room_spawns::{SpawnCategory, SpawnSelector},
    State,
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Npc {
    pub id: u32,
    pub kind: u32,
    pub variant: u32,
    pub subtype: u32,
    pub spawn_seed: u32,
    pub position: WorldVector,
    pub presented_position: WorldVector,
    pub velocity: WorldVector,
    pub target: WorldVector,
    pub wander_velocity: WorldVector,
    pub hp: f32,
    pub max_hp: f32,
    pub collision_radius: f32,
    pub collision_mass: f32,
    pub collision_damage: f32,
    pub base_friction: f32,
    /// Entity+470 as this actor's last full update left it.
    #[serde(default)]
    pub friction: f32,
    pub shadow_size: f32,
    pub grid_collision_class: u32,
    pub entity_collision_class: u32,
    pub can_shut_doors: bool,
    pub boss: bool,
    pub state: u32,
    pub frame_age: u32,
    pub animation_frame: u32,
    pub animation: String,
    pub animation_clock: f32,
    pub animation_playing: bool,
    pub visible: bool,
    pub saved_initialized_state: u32,
    pub appearance_effect_seed: Option<u32>,
    pub death_effect_seed: Option<u32>,
    pub dead: bool,
    pub removed: bool,
    /// Entity Init 446A90 clears the independent eligibility byte +494.
    pub raw_494: bool,
    pub damage: combat::DamageQueue,
    pub keeper: Option<crate::keepers::KeeperState>,
    pub fireplace: Option<crate::fireplaces::FireplaceState>,
    pub monstro: Option<crate::monstro::MonstroState>,
    pub level2_fly: Option<crate::level2_flies::Level2FlyState>,
    pub dip: Option<crate::dips::DipState>,
    /// Authored actor for families drawn from the entity catalogue.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub actor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shooter: Option<crate::plain_npc::ShooterTimers>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub champion: Option<crate::champions::Champion>,
    /// Sprite colour (+2d0) and its timed stack; the renderer draws it.
    #[serde(default)]
    pub sprite_color: crate::sprite_color::SpriteColor,
    /// Entity::Init441c80 clears status timers and flags on a fresh actor.
    #[serde(default)]
    pub status: crate::npc_status::NpcStatus,
    /// SplatColor +3cc: Entity::Init 441c80 resets it to white. Base-kill
    /// branches colour their effects with it (the fly family's 1000.3 at
    /// 4fb11e). Dingle's 4ada20 tint is still held in its own state.
    #[serde(default = "white", skip_serializing_if = "is_white")]
    pub splat_color: crate::sprite_color::Color,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gluttony: Option<crate::gluttony::GluttonyState>,
    /// NPCAI_Pathfinder (+7d8) for families that walk the grid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pathfinder: Option<crate::pathfinder::Pathfinder>,
    /// Narrow embedded Pathfinder +8/+10 owner where the complete Port
    /// pathfinder is unavailable. Never duplicates a full pathfinder owner.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pathfinder_vectors: Option<crate::pathfinder::PathVectors>,
    /// Sprite FlipX (+2fc) for families that face their target.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub flip_x: bool,
    /// Sprite layer event masks (+118 this step, +11c since the last restart),
    /// one bit per trigger of the current clip.
    #[serde(default, skip_serializing_if = "SpriteEvents::is_empty")]
    pub sprite_events: SpriteEvents,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dingle: Option<crate::dingle::DingleState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peep: Option<crate::peep::PeepState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boil: Option<crate::boil::BoilState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chub: Option<crate::chub::ChubState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gurdy_jr: Option<crate::gurdy_jr::GurdyJrState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mega_maw: Option<crate::mega_maw::MegaMawState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mom: Option<crate::mom::MomState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_creep: Option<crate::wall_creep::WallCreepState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub squirt: Option<crate::squirt::SquirtState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_hugger: Option<crate::wall_hugger::WallHuggerState>,
    /// Entity+494: BaseUpdate discards every queued hit.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub invincible: bool,
    /// Dying boss state16 frame (Entity_NPC+f58), see boss_death.rs.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub death_frame: u32,
    /// 4f2d10's boss callback 5c8430 (boss count, rewards) ran at removal; its
    /// reward effects are not admitted yet.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub boss_callback_pending: bool,
    /// Entity+498/+4a0 parent and child handles (SetParent447fc0,
    /// SetChild447f40). Removal448040 nulls every handle naming the purged
    /// actor, see `unlink_removed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub larry: Option<crate::larry::LarryState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maggot: Option<crate::maggots::MaggotState>,
    /// Entity+34c: the last non-preview grid pass hit a solid sample.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub grid_collided: bool,
    /// +318 APPEAR (4) cleared by a mid-room spawner: no hide/Poof/Appear.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub no_appear: bool,
    /// Entity+1c spawner type (0 when authored or spawned without one).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub spawner_type: u32,
    /// +320: spawned by a boss.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub boss_spawned: bool,
    /// Sprite scale +2c4 set by a spawner (None keeps the champion/1 scale).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sprite_scale: Option<f32>,
    /// Sprite overlay (ANM2 overlay slot): Gemini's head.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overlay: Option<Overlay>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gemini: Option<crate::gemini::GeminiState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hopper: Option<crate::hoppers::HopperState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fatty: Option<crate::fatty::FattyState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub greed: Option<crate::greed::GreedState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gurdy: Option<crate::gurdy::GurdyState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mega_fatty: Option<crate::mega_fatty::MegaFattyState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ring_fly: Option<crate::ring_flies::RingFlyState>,
}

/// One overlay clip of an NPC sprite, advanced with the main clip.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Overlay {
    pub animation: String,
    pub frame: u32,
    pub clock: f32,
    pub playing: bool,
}

fn white() -> crate::sprite_color::Color {
    crate::sprite_color::Color::WHITE
}

fn is_white(color: &crate::sprite_color::Color) -> bool {
    *color == crate::sprite_color::Color::WHITE
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpriteEvents {
    pub now: u32,
    pub seen: u32,
}

impl SpriteEvents {
    fn is_empty(&self) -> bool {
        self.now == 0 && self.seen == 0
    }
}

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };

/// Ordinary actor explosion collection is strict (75 + collision radius).
/// Velocity changes at admission; HP waits for the next full BaseUpdate.
pub fn bomb_hit(
    state: &mut State,
    center: WorldVector,
    blast_radius: f32,
    damage: f32,
    source: combat::DamageSource,
    line_clear: &dyn Fn(WorldVector) -> bool,
) -> Result<(), String> {
    bomb_hit_with_copied_source(
        state,
        center,
        blast_radius,
        damage,
        source,
        None,
        line_clear,
    )
}

///The same admitted blast path, with a separately owned pre-callback EntityRef.
///Unclosed ordinary callers retain None through the unchanged public wrapper.
pub(crate) fn bomb_hit_with_copied_source(
    state: &mut State,
    center: WorldVector,
    blast_radius: f32,
    damage: f32,
    source: combat::DamageSource,
    copied_source: Option<combat::CopiedDamageSource>,
    line_clear: &dyn Fn(WorldVector) -> bool,
) -> Result<(), String> {
    blast_hit_with_kind(
        state,
        center,
        blast_radius,
        damage,
        source,
        copied_source,
        combat::DamageKind::Bomb,
        line_clear,
    )
}

/// The same circle/impulse/queue with the caller's actual incoming flags.
#[expect(
    clippy::too_many_arguments,
    reason = "each argument is a separate input of the 554490 blast circle: center, radius, damage, source, copied EntityRef, the caller's incoming damage flags and its line-clear test"
)]
pub(crate) fn blast_hit_with_kind(
    state: &mut State,
    center: WorldVector,
    blast_radius: f32,
    damage: f32,
    source: combat::DamageSource,
    copied_source: Option<combat::CopiedDamageSource>,
    kind: combat::DamageKind,
    line_clear: &dyn Fn(WorldVector) -> bool,
) -> Result<(), String> {
    for npc in &mut state.room.npcs {
        // 554490 skips the blast's own source entity (a Mulligan's EXPLODE).
        if npc.id == source.id && npc.kind == source.entity_type {
            continue;
        }
        if npc.dead
            || npc.removed
            || npc.entity_collision_class == 0
            || npc.grid_collision_class == 4
        {
            continue;
        }
        let delta = WorldVector {
            x: npc.position.x - center.x,
            y: npc.position.y - center.y,
        };
        let squared = delta.x * delta.x + delta.y * delta.y;
        let radius = blast_radius + npc.collision_radius;
        if squared >= radius * radius || !line_clear(npc.position) {
            continue;
        }
        if squared > 0.0 {
            let distance = squared.sqrt();
            let impulse = (radius - distance).min(15.0);
            npc.velocity = npc.velocity * 0.5
                + WorldVector {
                    x: delta.x / distance,
                    y: delta.y / distance,
                } * impulse;
        }
        let health = npc.health();
        npc.damage.enqueue(
            health,
            combat::DamageEvent {
                amount: damage,
                kind,
                source,
                copied_source,
            },
        )?;
    }
    Ok(())
}

/// Called by the actual typed TearMotion collision pass. Projected sprite
/// coordinates never substitute for authoritative ground geometry.
pub fn tear_hit(
    state: &mut State,
    motion: &mut crate::projectiles::TearMotion,
    radius: f32,
    tear_id: u32,
    damage: f32,
) -> Result<Option<u32>, String> {
    let ground = motion.position;
    if !ground.is_finite() || !radius.is_finite() || radius < 0.0 || !damage.is_finite() {
        return Err("tear contact requires bounded ground geometry and damage".into());
    }
    for npc in &mut state.room.npcs {
        if tear_hit_receiver(npc, motion, radius, tear_id, damage)? {
            return Ok(Some(npc.id));
        }
    }
    Ok(None)
}

///One actual candidate from the canonical source contact partition. The old
///entry point above retains its previous receiver order when Poly is absent.
pub(crate) fn tear_hit_target(
    state: &mut State,
    motion: &mut crate::projectiles::TearMotion,
    radius: f32,
    tear_id: u32,
    damage: f32,
    target_id: u32,
) -> Result<Option<u32>, String> {
    let ground = motion.position;
    if !ground.is_finite() || !radius.is_finite() || radius < 0.0 || !damage.is_finite() {
        return Err("tear contact requires bounded ground geometry and damage".into());
    }
    let Some(npc) = state.room.npcs.iter_mut().find(|npc| npc.id == target_id) else {
        return Ok(None);
    };
    Ok(tear_hit_receiver(npc, motion, radius, tear_id, damage)?.then_some(npc.id))
}

fn tear_hit_receiver(
    npc: &mut Npc,
    motion: &mut crate::projectiles::TearMotion,
    radius: f32,
    tear_id: u32,
    damage: f32,
) -> Result<bool, String> {
    let ground = motion.position;
    if npc.dead || npc.removed || npc.entity_collision_class != 4 {
        return Ok(false);
    }
    let dx = ground.x - npc.position.x;
    let dy = ground.y - npc.position.y;
    let sum = radius + npc.collision_radius;
    if dx.abs() > sum || dy.abs() > sum || (dx * dx + dy * dy).sqrt() >= sum {
        return Ok(false);
    }
    let health = npc.health();
    if npc.damage.enqueue(
        health,
        combat::DamageEvent {
            amount: damage,
            kind: combat::DamageKind::Tear,
            source: combat::DamageSource {
                id: tear_id,
                entity_type: 2,
            },
            copied_source: None,
        },
    )? {
        let mut tear_body = combat::ContactBody {
            id: tear_id,
            entity_type: 2,
            position: ground,
            velocity: motion.velocity,
            radius,
            mass: 8.0,
        };
        let mut npc_body = combat::ContactBody {
            id: npc.id,
            entity_type: npc.kind,
            position: npc.position,
            velocity: npc.velocity,
            radius: npc.collision_radius,
            mass: npc.collision_mass,
        };
        combat::resolve_tear_npc_contact(&mut tear_body, &mut npc_body)?;
        motion.position = tear_body.position;
        motion.presented_position = tear_body.position;
        motion.velocity = tear_body.velocity;
        npc.position = npc_body.position;
        npc.presented_position = npc_body.position;
        npc.velocity = npc_body.velocity;
        return Ok(true);
    }
    Ok(false)
}

pub fn player_contacts(state: &mut State) -> Result<(), String> {
    //Entity+344 class0 (How to Jump in the air) has no pair collision.
    if state.room.npcs.is_empty() || state.player.entity_collision_off {
        return Ok(());
    }
    let grid = Grid::new(
        state
            .room
            .grid
            .clone()
            .ok_or("NPC contact requires admitted room grid")?,
    )?;
    static COLLIDER: OnceLock<crate::grid::Collider> = OnceLock::new();
    let collider = COLLIDER.get_or_init(|| {
        crate::grid::Collider::new(crate::grid::CollisionGeometry {
            radius: 10.0,
            scale: WorldVector { x: 1.0, y: 1.0 },
            points: 40,
        })
        .expect("authored Isaac geometry is valid")
    });
    for index in 0..state.room.npcs.len() {
        //4453a0 rejects either actual dead31f before the next callback.
        //The contact entered alive still completes its physical response.
        if state.player.dead {
            break;
        }
        let npc = &state.room.npcs[index];
        if npc.dead || npc.removed || npc.entity_collision_class == 0 {
            continue;
        }
        let mut player_body = combat::ContactBody {
            id: 0,
            entity_type: 1,
            position: bombs::player_world(state),
            velocity: bombs::player_velocity(state),
            radius: 10.0,
            mass: 5.0,
        };
        let mut npc_body = combat::ContactBody {
            id: npc.id,
            entity_type: npc.kind,
            position: npc.position,
            velocity: npc.velocity,
            radius: npc.collision_radius,
            mass: npc.collision_mass,
        };
        if !combat::player_npc_overlap(&player_body, &npc_body)? {
            continue;
        }
        // Callback49c2ec: a Wall Hugger still showing "No-Spikes" pushes the
        // player without hurting it.
        let contact_damage = if npc.kind == 218 && npc.animation == "No-Spikes" {
            0.0
        } else {
            npc.collision_damage
        };
        if contact_damage > 0.0 {
            //49C060 supplies collision damage to52B080. Its normal branch
            //stores BF8 as rounded incoming damage *60 (52BED6). Owned NPC
            //config/champion amounts are integers0/1/2, so the existing
            //health-unit conversion is also that rounded incoming amount.
            let health_units = contact_damage as u8;
            let immunity_presentations = u32::from(health_units) * 60;
            combat::try_damage_player_from(
                state,
                health_units,
                immunity_presentations,
                0,
                Some(npc.kind),
            )?;
        }
        let original_position = player_body.position;
        combat::resolve_player_npc_contact(&mut player_body, &mut npc_body, |shift| {
            collider.blocks_player_shift(&grid, original_position, shift)
        })?;
        state.player.x = f64::from(player_body.position.x) * 0.65 + 32.0;
        state.player.y = f64::from(player_body.position.y) * 0.65 - 46.0;
        state.player.vx = f64::from(player_body.velocity.x) * 0.65;
        state.player.vy = f64::from(player_body.velocity.y) * 0.65;
        let npc = &mut state.room.npcs[index];
        npc.position = npc_body.position;
        npc.presented_position = npc_body.position;
        npc.velocity = npc_body.velocity;
    }
    Ok(())
}

/// Full manager contacts also separate ordinary NPC pairs and planted Bombs.
/// Callback49b8f0 returns false for these uncharmed13/18 pairs; regular Bomb
/// damps both entering velocities before the source mass response. Spatial candidate ordering remains diagnostic.
const WORM_SEGMENTS: [u32; 4] = [19, 28, 89, 239];

/// 448040: purging an actor nulls every parent and child handle naming it,
/// so a severed chain's follower becomes a head and its leader a tail.
pub(crate) fn unlink_removed(npcs: &mut [Npc]) {
    let removed: Vec<u32> = npcs
        .iter()
        .filter(|npc| npc.removed)
        .map(|npc| npc.id)
        .collect();
    if removed.is_empty() {
        return;
    }
    for npc in npcs.iter_mut() {
        if npc.parent.is_some_and(|id| removed.contains(&id)) {
            npc.parent = None;
        }
        if npc.child.is_some_and(|id| removed.contains(&id)) {
            npc.child = None;
        }
    }
}

pub fn nonplayer_contacts(state: &mut State) -> Result<(), String> {
    let body = |npc: &Npc| combat::ContactBody {
        id: npc.id,
        entity_type: npc.kind,
        position: npc.position,
        velocity: npc.velocity,
        radius: npc.collision_radius,
        mass: npc.collision_mass,
    };
    for first in 0..state.room.npcs.len() {
        for second in first + 1..state.room.npcs.len() {
            let (left, right) = state.room.npcs.split_at_mut(second);
            let first = &mut left[first];
            let second = &mut right[0];
            // 49b980: two Wall Huggers never collide; 49bbb4 vetoes every
            // pair of worm segments (Larry Jr., 28, 89, 239) without charm,
            // and 49c4c0 every pair of Ring Flies (222).
            if first.dead
                || second.dead
                || first.entity_collision_class != 4
                || second.entity_collision_class != 4
                || (first.kind == 218 && second.kind == 218)
                || (first.kind == 222 && second.kind == 222)
                || (WORM_SEGMENTS.contains(&first.kind) && WORM_SEGMENTS.contains(&second.kind))
            {
                continue;
            }
            let mut a = body(first);
            let mut b = body(second);
            if combat::resolve_npc_or_bomb_pair(&mut a, &mut b)?.colliding {
                // 49bdf8: a charging Chub head bites a Charger or Spider.
                for flipped in [false, true] {
                    let (chub, other) = if flipped {
                        (&*second, &mut *first)
                    } else {
                        (&*first, &mut *second)
                    };
                    if !crate::chub::bites(chub, other) {
                        continue;
                    }
                    let source = combat::DamageSource {
                        id: chub.id,
                        entity_type: 0,
                    };
                    let health = other.health();
                    other.damage.enqueue(
                        health,
                        combat::DamageEvent {
                            amount: 20.0,
                            kind: combat::DamageKind::RoomEffect,
                            source,
                            copied_source: None,
                        },
                    )?;
                    state.audio.admit(
                        &mut state.boot_lifecycle.shared,
                        state.system_clock.system_counter.wrapping_sub(1),
                        crate::audio::SoundRequest {
                            id: 142,
                            volume: 1.0,
                            delay: 2,
                            looping: false,
                            pitch: 1.0,
                        },
                    );
                }
                // Fire49c249 first checks OTHER with the numeric NPC predicate
                // 444800. Player, Tear and Bomb are excluded from this callback.
                if first.kind == 33 {
                    let health = second.health();
                    second.damage.enqueue(
                        health,
                        combat::DamageEvent {
                            amount: 16.0,
                            kind: combat::DamageKind::FireContact,
                            source: combat::DamageSource {
                                id: first.id,
                                entity_type: 33,
                            },
                            copied_source: None,
                        },
                    )?;
                }
                if second.kind == 33 {
                    let health = first.health();
                    first.damage.enqueue(
                        health,
                        combat::DamageEvent {
                            amount: 16.0,
                            kind: combat::DamageKind::FireContact,
                            source: combat::DamageSource {
                                id: second.id,
                                entity_type: 33,
                            },
                            copied_source: None,
                        },
                    )?;
                }
                first.position = a.position;
                first.presented_position = a.position;
                first.velocity = a.velocity;
                second.position = b.position;
                second.presented_position = b.position;
                second.velocity = b.velocity;
            }
        }
    }
    for bomb in &mut state.bombs {
        if bomb.motion.lifetime != bombs::BombLifetime::Alive {
            continue;
        }
        for npc in &mut state.room.npcs {
            if npc.dead || npc.entity_collision_class != 4 {
                continue;
            }
            let mut a = combat::ContactBody {
                id: bomb.id,
                entity_type: 4,
                position: bomb.motion.position,
                velocity: bomb.motion.velocity,
                radius: 16.0,
                mass: 6.0,
            };
            let mut b = body(npc);
            if combat::resolve_npc_or_bomb_pair(&mut a, &mut b)?.colliding {
                // 49bd60: a charging Chub head eats the bomb; no response.
                // PORT GAP: bombs carry no tear flags yet, so none pass on.
                let damage = bomb
                    .shot
                    .as_ref()
                    .map_or(60.0, |shot| shot.explosion_damage);
                if crate::chub::swallow(npc, damage, 0) {
                    bomb.motion.lifetime = bombs::BombLifetime::Removed;
                    break;
                }
                bomb.motion.position = a.position;
                bomb.motion.presented_position = a.position;
                bomb.motion.velocity = a.velocity;
                bomb.x = f64::from(a.position.x) * 0.65 + 32.0;
                bomb.y = f64::from(a.position.y) * 0.65 - 46.0;
                npc.position = b.position;
                npc.presented_position = b.position;
                npc.velocity = b.velocity;
            }
        }
    }
    Ok(())
}

/// Source4545d0 finishes entity contacts, then454b10 visits the current
/// manager list with preview=false, including actors appended during Update.
/// Removed cached NPC records are no longer manager members. Dead active
/// bosses still receive the grid query. Keeper class1 remains its own scope.
pub(crate) fn post_contact_grid(state: &mut State) -> Result<(), String> {
    let grid = Grid::new(
        state
            .room
            .grid
            .clone()
            .ok_or("NPC grid pass requires admitted room grid")?,
    )?;
    for npc in &mut state.room.npcs {
        if npc.removed || npc.grid_collision_class == 0 {
            continue;
        }
        if let Some(family) = crate::entities::family(npc.kind) {
            (family.post_contact_grid)(npc, &grid)?;
        }
    }
    Ok(())
}

/// Source Room::Update 5d149b resets countdown10 while countable actors
/// remain. After retirement it decrements once per full pass; only a pass
/// whose entry countdown is zero marks the descriptor clear and opens doors.
pub fn admit_room_combat(state: &mut State) {
    if state.room.npcs.iter().any(|npc| npc.can_shut_doors) {
        state.room.combat_clear = false;
        state.room.combat_clear_countdown = 10;
        change_doors(state, false);
    }
}

/// Fresh5cecc0 reads signed current16ac+pending16b0 after factories. A
/// nonpositive total sets descriptor3d; a positive total preserves its flag.
/// This is separate from the doorway's immediate update/countdown10 tail.
/// It shuts no door. In the original the start room's doors Close on the
/// first Room.Update (Game+ce9f4 = 1), so their SFX35 and its shared Random
/// come two updates later (#862: two Boils kept the port 30 draws ahead).
/// Which Room.Update call issues that Close is not identified, so only this
/// start-room case is modelled.
pub(crate) fn initialize_startup_clear(state: &mut State) {
    state.room.combat_clear_countdown = 0;
    if state.room.manager_counts.countable_total() <= 0 {
        state.room.combat_clear = true;
    }
    state.room.startup_close_pending = !state.room.combat_clear;
}

pub fn advance_room_clear(state: &mut State) -> Result<(), String> {
    // Room5d1487 checks signed dc+e0 after the full manager/contact pass.
    // It is independent of HP, visible actor count and current boss_count2490.
    let startup_close = std::mem::take(&mut state.room.startup_close_pending);
    if state.room.manager_counts.countable_total() > 0 {
        state.room.combat_clear_countdown = 10;
        if startup_close && !state.room.combat_clear {
            change_doors(state, false);
        }
    } else if state.room.combat_clear_countdown > 0 {
        state.room.combat_clear_countdown -= 1;
    } else {
        //5d1988 calls5659f0 before the descriptor-clear predicate. An
        //already-clear empty start room still opens on this actual Full.
        change_doors(state, true);
        if !state.room.combat_clear {
            state.room.combat_clear = true;
            // 5d19cb..5d19e9 PlaySound(36 door heavy open, volume1, delay2) before
            // SpawnClearAward5c9240.
            state.audio.admit(
                &mut state.boot_lifecycle.shared,
                state.system_clock.system_counter - 1,
                crate::audio::SoundRequest {
                    id: 36,
                    volume: 1.0,
                    delay: 2,
                    looping: false,
                    pitch: 1.0,
                },
            );
            crate::clear_award::spawn(state)?;
            crate::active_charges::room_clear(state)?;
        }
    }
    Ok(())
}

fn change_doors(state: &mut State, open: bool) {
    let mut changed = Vec::new();
    let mut opened = Vec::new();
    let source_type = state.floor.descriptors[state.current_room]
        .template
        .room_type;
    for door in &mut state.room.appearance.doors {
        if !open && door.state == "Opened" {
            door.state = "Closed".into();
            if !state.room.combat_shut_door_slots.contains(&door.slot) {
                state.room.combat_shut_door_slots.push(door.slot);
            }
            changed.push(door.world_position);
        } else if open {
            //565b00 binds44/48 to the real current/neighbor room types.
            //5659f0 refuses either zero, hidden config6 and already-state2.
            let destination_type = state.floor.descriptors[door.target_room as usize]
                .template
                .room_type;
            //Type11 takes separate586300/5cc800/TEMP175 callbacks before
            //the normal config branch; retain its current state unadmitted.
            if source_type == 0
                || destination_type == 0
                || source_type == 11
                || destination_type == 11
            {
                continue;
            }
            let Some(grid) = &mut state.room.grid else {
                continue;
            };
            let column = ((door.world_position.x - 40.0) / 40.0 + 0.5).floor() as usize;
            let row = ((door.world_position.y - 120.0) / 40.0 + 0.5).floor() as usize;
            let Some(entity) = grid
                .cells
                .get_mut(column + row * usize::from(grid.width))
                .and_then(|cell| cell.entity.as_mut())
                .filter(|entity| entity.kind == 16)
            else {
                continue;
            };
            if entity.state == 2 || !matches!(entity.variant, 7 | 4) {
                continue;
            }
            //Admitted normal configs7/4 store FSM2. Graphics refresh and
            //other locked configs remain separate; no collision-class store.
            entity.state = 2;
            if door.state == "Closed" && state.room.combat_shut_door_slots.contains(&door.slot) {
                door.state = "Opened".into();
                opened.push(door.slot);
            }
        }
    }
    if open {
        state.room.combat_shut_door_slots.clear();
        state.room.combat_shut_update = None;
        //563b70 plays state2's "Open" from frame0, except that a door whose
        //grid Init (+28) is within 29 frames of the Game frame (+ce9f4) skips
        //to its last frame: a room cleared that soon after loading shows the
        //doors already open.
        let settled = state
            .room
            .initialization_game_update
            .is_none_or(|init| state.game_update_count.wrapping_sub(init) > 29);
        if !opened.is_empty() && settled {
            state.room.combat_open_door_slots = opened;
            state.room.combat_open_update = Some(state.game_update_count);
        }
    } else if !changed.is_empty() {
        state.room.combat_shut_update = Some(state.game_update_count);
        state.room.combat_open_door_slots.clear();
        state.room.combat_open_update = None;
    }
    if let Some(grid) = &mut state.room.grid {
        for position in changed {
            let column = ((position.x - 40.0) / 40.0 + 0.5).floor() as usize;
            let row = ((position.y - 120.0) / 40.0 + 0.5).floor() as usize;
            if let Some(entity) = grid
                .cells
                .get_mut(column + row * usize::from(grid.width))
                .and_then(|cell| cell.entity.as_mut())
            {
                entity.state = 1;
                entity.collision_class = 4;
            }
        }
    }
}

pub(crate) fn normalized(vector: WorldVector) -> WorldVector {
    let length = (vector.x * vector.x + vector.y * vector.y).sqrt();
    if length > 0.0 {
        vector * (1.0 / length)
    } else {
        vector
    }
}

pub(crate) fn direction(random: &mut Mt19937) -> WorldVector {
    // SharedDirection 0x42e8d0 rounds the sampled f32 angle before sinf/cosf.
    let radians = (f64::from(random.uniform()) * std::f64::consts::TAU) as f32;
    WorldVector {
        x: crate::fmath::cos(radians),
        y: crate::fmath::sin(radians),
    }
}

impl Npc {
    /// The authored entities2.xml row this NPC was born from. The entity
    /// keeps the config its spawn resolved, and Die 445c60, the champion
    /// roll and the grid colliders read that row; a family whose runtime
    /// variants are not authored rows (Envy's split heads) maps them back.
    pub(crate) fn config(&self) -> &'static crate::content::EntityConfig {
        let variant = match self.kind {
            51 => crate::envy::config_variant(self.variant),
            _ => self.variant,
        };
        crate::content::entity(self.kind, variant)
    }

    /// Morph480750: rewrite type/variant/subtype, then Init47f500 reloads the
    /// config row and anm2 (overlay cleared, default clip stopped at frame 0)
    /// and re-applies a kept champion color. HP is clamped to the new max;
    /// state, counters, motion, age and visibility are kept. The pathfinder
    /// is Reset5b4010. Only live-room morphs exist, where the Room+0 gate at
    /// 47f750 closes the champion roll.
    pub(crate) fn morph(
        &mut self,
        kind: u32,
        variant: u32,
        default_animation: &str,
        shared: &mut Mt19937,
    ) -> Result<(), String> {
        if self.status.base_update_admitted {
            return Err(crate::npc_status::refuse(
                self,
                crate::npc_status::Frontier::UnsupportedActor,
            ));
        }
        let config = content::entity(kind, variant);
        if config.stage_hp != 0.0 {
            return Err(format!(
                "morph target {kind}.{variant} needs the stage HP formula"
            ));
        }
        // Morph47f500 has its own friction/status reload rules. A previous
        // Full's current470 proof cannot cross that unclosed boundary.
        self.status.current_friction = None;
        self.kind = kind;
        self.variant = variant;
        self.subtype = 0;
        self.actor = config.actor.into();
        self.max_hp = config.base_hp;
        self.collision_radius = config.collision_radius;
        self.collision_mass = config.collision_mass;
        self.collision_damage = config.collision_damage;
        self.base_friction = config.friction;
        self.shadow_size = config.shadow_size;
        self.grid_collision_class = config.grid_class;
        self.can_shut_doors = config.can_shut_doors;
        self.boss = config.boss;
        self.overlay = None;
        self.animation = default_animation.into();
        self.animation_frame = 0;
        self.animation_clock = 0.0;
        self.animation_playing = false;
        self.sprite_events = SpriteEvents::default();
        if let Some(color) = self.champion.take().map(|champion| champion.color) {
            crate::champions::apply(self, color, shared)?;
        }
        self.hp = self.hp.min(self.max_hp);
        if let Some(path) = self.pathfinder.as_mut() {
            path.reset();
        }
        Ok(())
    }

    /// ResetColor 47f430: a champion's table colour, otherwise plain white.
    pub(crate) fn base_color(&self) -> crate::sprite_color::Color {
        self.champion
            .as_ref()
            .map_or(crate::sprite_color::Color::WHITE, |champion| {
                crate::champions::color(champion.color)
            })
    }
    /// BaseUpdate after the damage drain: colour countdown, then hit flashes.
    pub(crate) fn advance_sprite_color(
        &mut self,
        phase: crate::phase::Phase,
        applied: &[crate::combat::AppliedDamage],
    ) {
        let base = self.base_color();
        self.sprite_color.advance(phase, applied, base);
    }

    /// Actual Entity+318 view for the supported typed NPC owners. Normal
    /// Fly/AttackFly/RoundWorm Init441c80 clears flags then47ed20 ORs4;
    /// concrete subclasses preserve their own later flag writes.
    pub(crate) fn entity_flags(&self) -> u32 {
        crate::entities::family(self.kind).map_or(4, |family| (family.entity_flags)(self))
            | self.status.flags()
    }

    pub fn from_selector(
        id: u32,
        selector: &SpawnSelector,
        stage: u32,
    ) -> Result<Option<Self>, String> {
        crate::entities::spawn(
            id,
            selector,
            &crate::entities::SpawnContext {
                stage,
                game_update_count: 0,
            },
        )
    }

    /// Generic actor fields from the authored entities2.xml row, before the
    /// family constructor applies what its source config sets differently.
    pub(crate) fn authored(
        id: u32,
        config: &content::EntityConfig,
        selector: &SpawnSelector,
    ) -> Self {
        Self {
            id,
            kind: config.id,
            variant: config.variant,
            subtype: u32::from(selector.subtype),
            spawn_seed: selector.spawn_seed,
            position: selector.position,
            presented_position: selector.position,
            velocity: ZERO,
            target: ZERO,
            wander_velocity: ZERO,
            hp: config.base_hp,
            max_hp: config.base_hp,
            collision_radius: config.collision_radius,
            collision_mass: config.collision_mass,
            collision_damage: config.collision_damage,
            base_friction: config.friction,
            friction: config.friction,
            shadow_size: config.shadow_size,
            grid_collision_class: config.grid_class,
            entity_collision_class: 4,
            can_shut_doors: config.can_shut_doors,
            boss: config.boss,
            state: 1,
            frame_age: 0,
            animation_frame: 0,
            animation: String::new(),
            animation_clock: 0.0,
            animation_playing: false,
            visible: true,
            saved_initialized_state: 0,
            appearance_effect_seed: None,
            death_effect_seed: None,
            dead: false,
            removed: false,
            raw_494: false,
            damage: combat::DamageQueue::default(),
            keeper: None,
            fireplace: None,
            monstro: None,
            level2_fly: None,
            dip: None,
            actor: String::new(),
            shooter: None,
            champion: None,
            sprite_color: crate::sprite_color::SpriteColor::default(),
            status: crate::npc_status::NpcStatus::default(),
            splat_color: crate::sprite_color::Color::WHITE,
            flip_x: false,
            pathfinder: None,
            pathfinder_vectors: None,
            gluttony: None,
            sprite_events: SpriteEvents::default(),
            dingle: None,
            peep: None,
            boil: None,
            chub: None,
            gurdy_jr: None,
            mega_maw: None,
            mom: None,
            wall_creep: None,
            squirt: None,
            wall_hugger: None,
            invincible: false,
            death_frame: 0,
            boss_callback_pending: false,
            parent: None,
            child: None,
            larry: None,
            maggot: None,
            grid_collided: false,
            no_appear: false,
            spawner_type: 0,
            boss_spawned: false,
            sprite_scale: None,
            overlay: None,
            gemini: None,
            hopper: None,
            fatty: None,
            greed: None,
            gurdy: None,
            mega_fatty: None,
            ring_fly: None,
        }
    }

    pub fn health(&self) -> combat::ActorHealth {
        combat::ActorHealth {
            hp: self.hp,
            max_hp: self.max_hp,
        }
    }

    fn fly_movement(
        &mut self,
        counter: u32,
        random: &mut Mt19937,
        player: WorldVector,
        grid: &Grid,
        initial_friction: f32,
    ) -> f32 {
        let mut friction = initial_friction;
        if self.kind == 13 {
            // 4f0238 executes bounded1000 on each passive Fly AI update.
            // Conditional poop transformation remains unadmitted.
            let _poop_chance = random.bounded(1000);
        }
        if counter.is_multiple_of(3) {
            if self.kind == 18 {
                self.target = player;
            } else {
                let radius = (random.uniform() * random.uniform()) * 240.0;
                let candidate = self.position + direction(random) * radius;
                if grid.index(candidate).is_some_and(|index| {
                    grid.descriptor().cells[index]
                        .entity
                        .as_ref()
                        .is_some_and(|entity| entity.kind == 14)
                }) {
                    self.target = candidate;
                }
                // 4f0828 executes this chance; its type33 target callback is
                // unadmitted. Neither discarded result establishes RNG parity.
                let _other_target_chance = random.bounded(100);
            }
        }
        if matches!(self.kind, 13 | 14) {
            // 4efa10/4f06e0: Fly and Pooter clear the chosen target every update
            // unless a spawner102 owns them, so no target impulse follows.
            self.target = ZERO;
        }
        if self.target != ZERO && counter.is_multiple_of(2) {
            let delta = WorldVector {
                x: self.target.x - self.position.x,
                y: (self.target.y - self.position.y) * 0.5,
            };
            let length = (delta.x * delta.x + delta.y * delta.y).sqrt();
            if self.kind == 13 {
                self.wander_velocity = self.wander_velocity * 0.7;
            }
            if self.boss_spawned {
                // 4efccd: boss-spawned flies chase with extra friction.
                friction *= 0.9;
            }
            let range = if self.kind == 18 { 25.0 } else { 30.0 };
            if length > range {
                let impulse = ((length - range) * 0.08).min(1.2);
                self.velocity = self.velocity
                    + WorldVector {
                        x: delta.x / length,
                        y: delta.y / length,
                    } * (impulse / friction);
            }
        }
        if self.target == ZERO && random.bounded(10) == 0 {
            self.velocity = self.velocity
                + normalized(WorldVector {
                    x: player.x - self.position.x,
                    y: player.y - self.position.y,
                }) * 0.5;
        }
        if counter % 4 == 1 {
            self.wander_velocity = (self.wander_velocity + direction(random) * 0.6) * 0.4;
        }
        // 4f0600: a Duke's orbiting fly skips the room-edge nudges.
        if counter % 3 == 2 && self.spawner_type != 67 {
            if self.position.y < 230.0 {
                self.wander_velocity.y += 0.06;
            } else if self.position.y > 420.0 {
                self.wander_velocity.y -= 0.06;
            }
            if self.position.x > 540.0 {
                self.wander_velocity.x -= 0.06;
            } else if self.position.x < 100.0 {
                self.wander_velocity.x += 0.06;
            }
        }
        if counter % 3 == 2 {
            self.velocity = self.velocity + self.wander_velocity * (2.0 / friction);
            friction *= 0.6;
        }
        friction
    }
}

pub(crate) const FLY_FAMILY: Family = Family {
    kinds: &[13, 14, 18],
    spawn: spawn_fly,
    advance: advance_fly,
    post_contact_grid: crate::entities::clamp_walls,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

fn spawn_fly(id: u32, selector: &SpawnSelector, _: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || !matches!(
            (selector.entity_type, selector.variant),
            (13 | 18, 0) | (14, 0 | 1)
        )
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err(
            "NPC admission requires bounded authored position and nonzero spawn seed".into(),
        );
    }
    let config = content::entity(u32::from(selector.entity_type), u32::from(selector.variant));
    Ok(Some(Npc {
        // Config47f573 SetFrame(default,0) leaves the default clip stopped.
        animation: "Fly".into(),
        actor: config.actor.into(),
        shooter: (selector.entity_type == 14).then(ShooterTimers::default),
        ..Npc::authored(id, config, selector)
    }))
}

fn advance_fly(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    crate::plain_npc::advance(npc, phase, random, world, births, fly_ai)
}

/// ai_fly4ef880 for Fly13, Pooter14 and AttackFly18.
fn fly_ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.state == 0 {
        // 4f0510: grid class3, cleared target store, state3.
        npc.grid_collision_class = 3;
        npc.state = 3;
    }
    if npc.kind == 14 {
        pooter_ai(npc, random, world, births, friction)?;
    } else {
        // 4ef941 plays the looping Fly clip without forcing a restart.
        crate::plain_npc::play(npc, "Fly", false);
    }
    if npc.kind == 18 && npc.spawner_type == 67 && world.game_counter.is_multiple_of(3) {
        duke_orbit(npc, world, *friction);
    }
    *friction = npc.fly_movement(
        world.game_counter,
        random,
        world.player,
        world.grid,
        *friction,
    );
    Ok(())
}

/// 4f101a: a Duke-spawned Attack Fly (spawner type 67) orbits the last
/// type-67 actor in the room list within 260 on a breathing 60..70 ring;
/// with none left it is released for good.
fn duke_orbit(npc: &mut Npc, world: &NpcWorld, friction: f32) {
    let Some(duke) = world
        .peers
        .iter()
        .filter(|other| other.kind == 67 && !other.removed)
        .last()
    else {
        npc.spawner_type = 0;
        return;
    };
    let delta = WorldVector {
        x: duke.position.x - npc.position.x,
        y: duke.position.y - npc.position.y,
    };
    let length = (delta.x * delta.x + delta.y * delta.y).sqrt();
    if length >= 260.0 {
        return;
    }
    let ring = crate::fmath::sin(world.game_counter as f32 * 0.02) * 5.0 + 65.0;
    let error = length - ring;
    let pull = if error > -5.0 { error.min(5.0) } else { -5.0 };
    let radial = (delta * pull) * 0.5;
    npc.velocity = npc.velocity
        + WorldVector {
            x: (radial.x / length) / friction,
            y: (radial.y / length) / friction,
        };
    let spin = pull.abs() + 2.0;
    npc.velocity.y += (((0.2 * delta.x) / spin) / friction) / friction;
    npc.velocity.x -= (((0.2 * delta.y) / spin) / friction) / friction;
}

/// Pooter14.0/Super Pooter14.1: 4f0bd0 clip state, then 4f029a aimed shots
/// (4f10d0 mode0, 4f13a8 mode1).
fn pooter_ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.state <= 4 {
        crate::plain_npc::play(npc, "Fly", false);
    } else if npc.state == 8 && crate::plain_npc::finished(npc) {
        npc.state = 4;
    }
    if npc.shooter.as_ref().is_some_and(|timers| timers.delay == 0) {
        // 4f1250: the shot frame's sound precedes FireProjectiles.
        world.audio.admit(
            random,
            world.system_counter,
            crate::audio::SoundRequest {
                id: 178,
                volume: 1.0,
                delay: 2,
                looping: false,
                pitch: 1.0,
            },
        );
    }
    // 4f10d0: default ProjectileParams draw their angle before the shot.
    let parameter_angle = (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
    let shot = ShotParameters {
        mode: if npc.variant == 1 {
            ShotMode::Pair
        } else {
            ShotMode::Single
        },
        range: 200.0,
        cooldown: 4,
        delay: 7,
        speed: 7.0,
        parameter_angle,
        lead: 0.0,
        adjust: None,
    };
    if crate::plain_npc::try_shoot_player(npc, random, world, births, shot)? {
        crate::plain_npc::play(npc, "Attack", false);
        npc.state = 8;
        npc.flip_x = world.player.x - npc.position.x < 0.0;
    }
    if npc
        .shooter
        .as_ref()
        .is_some_and(|timers| timers.cooldown > 0)
    {
        // 4f02d3: cooling Pooters brake and damp their wander.
        *friction *= 0.8;
        npc.wander_velocity = npc.wander_velocity * 0.7;
    }
    Ok(())
}
