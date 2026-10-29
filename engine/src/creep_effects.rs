//! Effect 1000.22 Creep (Red): Entity_Effect 45aa70 for the enemy blood pool.
//! The spawner runs the first update at once (clip pick by incoming scale,
//! radius, frame by game counter). The pool then hurts a grounded player
//! standing on an opaque texel for 125 updates, fades for 40 and is removed.
use crate::{
    actors::{ActorKind, ActorRef},
    familiars::{definition, NpcEffectAdmission, Sprite, SpriteEffect},
    items,
    motion::WorldVector,
    mt::Mt19937,
    phase::Phase,
    State,
};
use serde::{Deserialize, Serialize};

include!(concat!(env!("OUT_DIR"), "/creep_masks.rs"));

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };
/// Lemon Mishap's creep variant.
pub(crate) const LEMON: u32 = 32;
/// Player Entity+474 collision radius.
const PLAYER_RADIUS: f32 = 10.0;
/// DAMAGE_ACID.
const ACID: u32 = 0x10;

/// One alpha>0.5 bitfield per creep clip (tools/foundation/extract-alpha-masks.py).
pub struct CreepMask {
    pub clip: &'static str,
    pub left: f32,
    pub top: f32,
    pub width: u32,
    pub rows: &'static [u128],
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Controller {
    /// +7d4: 0 before the first update, 1 while alive, then the fade count.
    pub counter: u32,
    /// +818 size multiplier.
    pub size: f32,
}

impl NpcEffectAdmission<'_> {
    /// Spawn550a40 of 1000.22 by an NPC at `scale` (+2c4 before the first
    /// update); Init458a70 draws nothing.
    pub(crate) fn admit_creep(
        &mut self,
        seed: u32,
        position: WorldVector,
        spawner: (u32, u32),
        scale: f32,
    ) -> Result<u32, String> {
        self.admit_pool(22, seed, position, spawner, (scale, 1.0), (125, 1))
    }

    /// A 1000.22 a spawner writes before its first update: sprite `scale`
    /// (+2c4), controller `size` (+818) and timeouts (+7f0, +7f4). Skinny's
    /// trail (4b858b) leaves the first update to the scheduler.
    pub(crate) fn admit_sized_creep(
        &mut self,
        seed: u32,
        position: WorldVector,
        scale: f32,
        size: f32,
        timeouts: (i32, i32),
    ) -> Result<u32, String> {
        self.admit_pool(22, seed, position, (0, 0), (scale, size), timeouts)
    }

    /// Peep's Spawn550a40 of 1000.24 (yellow; the same code as 22) at sprite
    /// `scale` (4c3f60), with 22's default timeouts.
    pub(crate) fn admit_yellow_creep(
        &mut self,
        seed: u32,
        position: WorldVector,
        scale: f32,
    ) -> Result<u32, String> {
        self.admit_pool(24, seed, position, (0, 0), (scale, 1.0), (125, 1))
    }

    /// A green champion's Spawn550a40 of 1000.23 (4fb02a, default scale),
    /// whose timeout +7f0 and +7f4 it sets to 30 before the first update.
    pub(crate) fn admit_green_creep(
        &mut self,
        seed: u32,
        position: WorldVector,
        spawner: (u32, u32),
    ) -> Result<u32, String> {
        self.admit_pool(23, seed, position, spawner, (1.0, 1.0), (30, 30))
    }

    /// Lemon Mishap 5242E7: Spawn550a40 of 1000.32 at the Player with zero
    /// velocity, no spawner, subtype 0 and the shared seed. Init458a70 gives
    /// it the creep words (+7D8 1, +460 2000, collision classes 0) with no
    /// time to live (+7F0 -1), damage +48C 22 and size +818 1.
    pub(crate) fn admit_lemon(&mut self, seed: u32, position: WorldVector) -> Result<u32, String> {
        self.admit_pool(LEMON, seed, position, (0, 0), (1.0, 1.0), (-1, 1))
    }

    fn admit_pool(
        &mut self,
        variant: u32,
        seed: u32,
        position: WorldVector,
        spawner: (u32, u32),
        (scale, size): (f32, f32),
        (time_to_live, raw_7f4): (i32, i32),
    ) -> Result<u32, String> {
        let config = definition(1000, variant).ok_or("authored Creep config absent")?;
        let id = *self.next_effect;
        *self.next_effect = id.checked_add(1).ok_or("effect handle exhausted")?;
        let mut sprite = Sprite::new(config);
        sprite.scale = WorldVector { x: scale, y: scale };
        self.effects.push(SpriteEffect {
            id,
            variant,
            subtype: 0,
            spawn_seed: seed,
            spawner_type: spawner.0,
            spawner_variant: spawner.1,
            parent: None,
            birth_game_update: self.game_update_count,
            actor_rng: items::Rng::new(seed, [4, 3, 17]),
            position,
            presented_position: position,
            render_offset: ZERO,
            velocity: ZERO,
            configured_friction: config.friction,
            current_friction: config.friction,
            collision_radius: config.collision_radius,
            collision_mass: config.collision_mass,
            collision_damage: 22.0,
            hp: config.base_hp,
            max_hp: config.base_hp,
            grid_collision_class: 0,
            entity_collision_class: 0,
            flags: 0,
            visible: true,
            active: true,
            removed: false,
            render_order_offset: 2000,
            lifetime_divisor: 0,
            time_to_live,
            raw_7f4,
            raw_7f8: 0.0,
            rotation_814: 0.0,
            raw_7d8: true,
            sprite,
            full_updates: 0,
            poop_effect: None,
            blood_effect: None,
            epic_fetus: None,
            explosion_effect: None,
            creep_effect: Some(Controller { counter: 0, size }),
            fly_effect: None,
            footprint: false,
            critter: None,
        });
        self.scheduler.admit(ActorRef {
            kind: ActorKind::Effect,
            id,
        })?;
        Ok(id)
    }

    /// The spawner's immediate vtable+0x18 update; returns whether the
    /// player contact landed (the spawner hands the hurt to the game).
    pub(crate) fn advance_creep_initial(
        &mut self,
        id: u32,
        random: &mut Mt19937,
        game_counter: u32,
        player: WorldVector,
    ) -> Result<bool, String> {
        let effect = self
            .effects
            .iter_mut()
            .find(|effect| effect.id == id)
            .ok_or("Creep first update requires its admitted actor")?;
        let touched = step(effect, random, game_counter, player)?;
        Ok(touched)
    }

    /// SetColor of the spawner's colour after the first update.
    pub(crate) fn set_creep_color(
        &mut self,
        id: u32,
        tint: [f32; 4],
        offsets: [f32; 3],
    ) -> Result<(), String> {
        let effect = self
            .effects
            .iter_mut()
            .find(|effect| effect.id == id)
            .ok_or("Creep color requires its admitted actor")?;
        effect.sprite.tint = tint;
        effect.sprite.offsets = offsets;
        Ok(())
    }
}

/// One full Effect update. Returns whether a live pool lies under the player.
fn step(
    effect: &mut SpriteEffect,
    random: &mut Mt19937,
    game_counter: u32,
    player: WorldVector,
) -> Result<bool, String> {
    let config = definition(1000, effect.variant).ok_or("effect config absent")?;
    effect.full_updates = effect
        .full_updates
        .checked_add(1)
        .ok_or("effect full count exhausted")?;
    effect.sprite.advance(config)?;
    effect.velocity = effect.velocity * effect.current_friction;
    effect.position = effect.position + effect.velocity;
    effect.presented_position = effect.position;
    effect.current_friction = effect.configured_friction;
    let controller = effect
        .creep_effect
        .as_mut()
        .ok_or("Creep controller absent")?;
    if controller.counter == 0 {
        // 45cbf8: pick the pool size by the incoming sprite scale.
        controller.counter = 1;
        let scale = effect.sprite.scale.x;
        let name = if scale < 0.99 {
            "SmallBlood"
        } else if scale < 1.3 {
            "Blood"
        } else if scale < 2.1 {
            "BigBlood"
        } else {
            "BiggestBlood"
        };
        let clip = format!("{name}{:02}", random.bounded(6) + 1);
        effect.collision_radius = (scale * 20.0) * controller.size;
        effect.sprite.scale = WorldVector {
            x: controller.size,
            y: controller.size,
        };
        let frames = config
            .clips
            .iter()
            .find(|candidate| candidate.name == clip)
            .ok_or("Creep clip absent from authored catalogue")?
            .frames;
        effect.sprite.animation = clip;
        effect.sprite.playing = true;
        effect.sprite.frame = (game_counter % frames).min(frames - 1) as f32;
    }
    let mut touched = false;
    if effect.removed {
        // 45cd8d: the dead pool shrinks and fades, then is removed.
        effect.sprite.scale = effect.sprite.scale * 0.98;
        effect.sprite.tint[3] *= 0.95;
        controller.counter += 1;
        if controller.counter > 40 {
            effect.active = false;
        }
    } else {
        touched = under_player(effect, player);
    }
    if effect.time_to_live > 0 {
        effect.time_to_live -= 1;
        if effect.time_to_live == 0 {
            effect.removed = true;
        }
    }
    Ok(touched)
}

/// 462b99: strict radius test, then the player's centre must land on a
/// texel with alpha > 0.5 (GetTexel at 0.65 screen pixels per unit).
fn under_player(effect: &SpriteEffect, player: WorldVector) -> bool {
    let reach = effect.collision_radius * effect.sprite.scale.x;
    let dx = player.x - effect.position.x;
    let dy = player.y - effect.position.y;
    if reach + PLAYER_RADIUS <= (dx * dx + dy * dy).sqrt() {
        return false;
    }
    let Some(mask) = CREEP_MASKS
        .iter()
        .find(|mask| mask.clip == effect.sprite.animation)
    else {
        return false;
    };
    let scale = effect.sprite.scale.x;
    let x = (dx * 0.65) / scale - mask.left;
    let y = (dy * 0.65) / scale - mask.top;
    if !(x > 0.0 && y > 0.0) {
        return false;
    }
    let (column, row) = (x as u32, y as usize);
    column < mask.width && row < mask.rows.len() && mask.rows[row] >> column & 1 == 1
}

/// 5516d0: hurt a grounded, colliding player without Callus.
pub(crate) fn hurt_player(state: &mut State) -> Result<(), String> {
    let flying = state.player.flying == Some(true);
    let callus = state
        .player_items
        .as_ref()
        .is_some_and(|items| items.trinket_slots.contains(&14));
    if state.player.dead || flying || callus {
        return Ok(());
    }
    crate::combat::try_damage_player(state, 1, 60, ACID)?;
    Ok(())
}

pub(crate) fn advance(state: &mut State, phase: Phase, id: u32) -> Result<(), String> {
    let player = crate::bombs::player_world(state);
    let game_counter = state.native_game_frame();
    let effect = state
        .sprite_effects
        .iter_mut()
        .find(|effect| effect.id == id)
        .ok_or("Creep update requires its admitted actor")?;
    if phase == Phase::Skipped || !effect.active {
        return Ok(());
    }
    if phase == Phase::Interpolate {
        effect.presented_position = effect.position;
        return Ok(());
    }
    if effect.variant == LEMON {
        return advance_lemon(state, id);
    }
    if step(
        effect,
        &mut state.boot_lifecycle.shared,
        game_counter,
        player,
    )? {
        hurt_player(state)?;
    }
    Ok(())
}

/// Effect::Update 45aa70 for the Lemon Mishap creep 1000.32: BaseUpdate,
/// then the 45CE80 case, then the shared time-to-live tail (+7F0 is -1).
/// The pool grows over 15 updates, holds until update 225, shrinks to
/// nothing by update 290 and is removed on the next one. Every update each
/// vulnerable ground enemy inside its ellipse takes +48C with
/// DAMAGE_COUNTDOWN and countdown 10.
pub(crate) fn advance_lemon(state: &mut State, id: u32) -> Result<(), String> {
    let index = state
        .sprite_effects
        .iter()
        .position(|effect| effect.id == id && effect.variant == LEMON)
        .ok_or("Lemon Mishap update requires its admitted actor")?;
    let effect = &mut state.sprite_effects[index];
    let config = definition(1000, LEMON).ok_or("Lemon Mishap config absent")?;
    effect.full_updates = effect
        .full_updates
        .checked_add(1)
        .ok_or("effect full count exhausted")?;
    effect.sprite.advance(config)?;
    effect.velocity = effect.velocity * effect.current_friction;
    effect.position = effect.position + effect.velocity;
    effect.presented_position = effect.position;
    effect.current_friction = effect.configured_friction;
    let controller = effect
        .creep_effect
        .as_mut()
        .ok_or("Lemon Mishap controller absent")?;
    let count = controller.counter;
    // 45CE80/461938/45E488: the scale for this update's counter.
    let scale = if count <= 14 {
        count as f32 / 15.0
    } else if count <= 225 {
        1.0
    } else if count <= 290 {
        1.0 - (count as f32 - 225.0) / 65.0
    } else {
        effect.removed = true;
        1.0
    };
    let size = controller.size;
    effect.sprite.scale = WorldVector {
        x: scale * size,
        y: scale * size,
    };
    let center = effect.position;
    let radius = effect.collision_radius;
    let damage = effect.collision_damage;
    let reach = radius * scale * size;
    let source = crate::combat::DamageSource {
        id: 0,
        entity_type: 0,
    };
    let targets = lemon_targets(state, center, reach)?;
    for npc_id in targets {
        let npc = state
            .room
            .npcs
            .iter_mut()
            .find(|npc| npc.id == npc_id)
            .ok_or("Lemon Mishap target left the room list")?;
        // 45CF97..45D027: the ellipse stretches y by 1.75; the reach adds
        // the target's own radius and the test is strict.
        let dx = npc.position.x - center.x;
        let dy = (npc.position.y - center.y) * 1.75;
        let bound = radius * scale * size + npc.collision_radius;
        if bound * bound <= dx * dx + dy * dy {
            continue;
        }
        let health = npc.health();
        npc.damage.enqueue(
            health,
            crate::combat::DamageEvent {
                amount: damage,
                kind: crate::combat::DamageKind::LemonCreep,
                source,
                copied_source: None,
            },
        )?;
    }
    let effect = &mut state.sprite_effects[index];
    let controller = effect
        .creep_effect
        .as_mut()
        .ok_or("Lemon Mishap controller absent")?;
    controller.counter += 1;
    if effect.removed {
        // vtable+0x48 Remove441bb0 clears active and marks dead.
        effect.active = false;
    }
    Ok(())
}

/// QueryRadius 4536f0 over the enemy partition (mask 8), then 444810 and
/// not 444860 (a grid collision class 0..3 flies). The partition is the
/// last 4545D0 rebuild; without one, every current member is a candidate.
fn lemon_targets(state: &State, center: WorldVector, reach: f32) -> Result<Vec<u32>, String> {
    let candidates: Vec<u32> = match state.knife_query_partition.as_ref() {
        Some(snapshot) => snapshot
            .query(center, reach)?
            .into_iter()
            .filter(|actor| actor.kind == ActorKind::Npc)
            .map(|actor| actor.id)
            .collect(),
        None => state
            .room
            .npcs
            .iter()
            .filter(|npc| npc.entity_collision_class != 0 && (10..=999).contains(&npc.kind))
            .map(|npc| npc.id)
            .collect(),
    };
    let mut targets = Vec::new();
    for id in candidates {
        let Some(npc) = state.room.npcs.iter().find(|npc| npc.id == id) else {
            continue;
        };
        let vulnerable = crate::target_eligibility::eligible_target(
            crate::target_eligibility::TargetEligibility {
                kind: npc.kind,
                collision_class: npc.entity_collision_class,
                raw_494: npc.raw_494,
                dead: npc.dead,
                entity_flags: crate::entities::family(npc.kind).map(|_| npc.entity_flags()),
            },
        )?;
        if vulnerable && npc.grid_collision_class > 3 {
            targets.push(id);
        }
    }
    Ok(targets)
}
