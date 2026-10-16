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
        let config = definition(1000, 22).expect("authored Creep (Red) config");
        let id = *self.next_effect;
        *self.next_effect = id.checked_add(1).ok_or("effect handle exhausted")?;
        let mut sprite = Sprite::new(config);
        sprite.scale = WorldVector { x: scale, y: scale };
        self.effects.push(SpriteEffect {
            id,
            variant: 22,
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
            time_to_live: 125,
            raw_7f4: 1,
            raw_7f8: 0.0,
            raw_7d8: true,
            sprite,
            full_updates: 0,
            poop_effect: None,
            blood_effect: None,
            epic_fetus: None,
            explosion_effect: None,
            creep_effect: Some(Controller {
                counter: 0,
                size: 1.0,
            }),
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
    let config = definition(1000, 22).ok_or("effect config absent")?;
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
    let game_counter = state.game_update_count;
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
