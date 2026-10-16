//! Monstro's real BloodStart17/42 calls, before its common Base update.
//! Explosion2/77 and category5 remain separate required death-chain consumers.
use crate::{
    actors::{ActorKind, ActorRef},
    audio::{AudioState, SoundRequest},
    familiars::{definition, NpcEffectAdmission, Parent, Sprite, SpriteEffect},
    items,
    motion::WorldVector,
    mt::Mt19937,
    phase::Phase,
    State,
};
use serde::{Deserialize, Serialize};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Controller {
    Poof04,
    BloodGush,
}

/// GetNull4107f0 returns the authored current NullId0 frame, not a null pointer.
pub(crate) struct Guide {
    pub position: WorldVector,
    pub scale: WorldVector,
}
pub(crate) struct Emission {
    pub parent: Parent,
    pub position: WorldVector,
    pub guide: Guide,
    pub tint: [f32; 4],
    pub offsets: [f32; 3],
}

pub(crate) fn direction(shared: &mut Mt19937) -> WorldVector {
    let angle = (f64::from(shared.uniform()) * std::f64::consts::PI * 2.0) as f32;
    WorldVector {
        x: crate::fmath::cos(angle),
        y: crate::fmath::sin(angle),
    }
}

fn offset(guide: &Guide, shared: &mut Mt19937) -> WorldVector {
    // Source draws U before Direction; each vector helper rounds separately.
    let uniform = shared.uniform();
    let direction = direction(shared);
    guide.position
        + WorldVector {
            x: ((direction.x * uniform) * guide.scale.x) * 4.0,
            y: ((direction.y * uniform) * guide.scale.y) * 4.0,
        }
}

impl NpcEffectAdmission<'_> {
    fn admit_blood_effect(
        &mut self,
        variant: u32,
        seed: u32,
        position: WorldVector,
        controller: Controller,
    ) -> Result<u32, String> {
        if !position.is_finite() || !matches!(variant, 17 | 42) {
            return Err("BloodStart requires its real17/42 factory and finite position".into());
        }
        let config = definition(1000, variant).ok_or("BloodStart authored effect absent")?;
        let id = *self.next_effect;
        *self.next_effect = id.checked_add(1).ok_or("effect handle exhausted")?;
        self.effects.push(SpriteEffect {
            id,
            variant,
            subtype: 0,
            spawn_seed: seed,
            // These actual GameSpawn calls pass NULL; linking42 happens later.
            spawner_type: 0,
            spawner_variant: 0,
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
            render_order_offset: 0,
            lifetime_divisor: 0,
            time_to_live: -1,
            raw_7f4: 1,
            raw_7f8: 0.0,
            raw_7d8: false,
            sprite: Sprite::new(config),
            full_updates: 0,
            blood_effect: Some(controller),
            footprint: false,
            critter: None,
            creep_effect: None,
            fly_effect: None,
            epic_fetus: None,
            explosion_effect: None,
            poop_effect: None,
        });
        self.scheduler.admit(ActorRef {
            kind: ActorKind::Effect,
            id,
        })?;
        Ok(id)
    }

    pub(crate) fn blood_start(
        &mut self,
        emission: Emission,
        shared: &mut Mt19937,
        audio: &mut AudioState,
        system_counter: u32,
    ) -> Result<(), String> {
        let Emission {
            parent,
            position,
            guide,
            tint,
            offsets,
        } = emission;
        if tint
            .iter()
            .chain(offsets.iter())
            .any(|value| !value.is_finite())
        {
            return Err("BloodStart inherited ColorMod must be finite".into());
        }
        if parent.entity_type != 20 || parent.variant != 0 {
            return Err("BloodStart currently owns ordinary Monstro20/0 only".into());
        }
        if shared.bounded(2) == 0 {
            let seed = shared.next_u32();
            let id = self.admit_blood_effect(17, seed, position, Controller::Poof04)?;
            let sprite_offset = offset(&guide, shared);
            let rotation = shared.bounded(4) as f32 * 90.0;
            let scale_x = if shared.bounded(2) == 0 { 1.0 } else { -1.0 };
            let scale_y = if shared.bounded(2) == 0 { 1.0 } else { -1.0 };
            let scale = 1.0 - shared.uniform() * 0.4;
            let effect = self
                .effects
                .iter_mut()
                .find(|effect| effect.id == id)
                .unwrap();
            effect.sprite.offset = sprite_offset;
            effect.sprite.rotation = rotation;
            effect.sprite.scale = WorldVector {
                x: scale_x * scale,
                y: scale_y * scale,
            };
            effect.raw_7f8 = 50.0;
            effect.sprite.tint = tint;
            effect.sprite.offsets = offsets;
            let volume = shared.uniform() * 0.2 + 0.5;
            audio.admit(
                shared,
                system_counter,
                SoundRequest {
                    id: 72,
                    volume,
                    delay: 2,
                    looping: false,
                    pitch: 1.0,
                },
            );
        }
        //4fc641 rejoins here after the complete17/SFX path. Both outcomes draw.
        if shared.bounded(5) == 0 {
            let seed = shared.next_u32();
            let id = self.admit_blood_effect(42, seed, position, Controller::BloodGush)?;
            let sprite_offset = offset(&guide, shared);
            let flip = shared.bounded(2) == 0;
            let scale = 1.0 - shared.uniform() * 0.4;
            let effect = self
                .effects
                .iter_mut()
                .find(|effect| effect.id == id)
                .unwrap();
            // Actual447fc0, not GameSpawn's cached caller metadata.
            effect.parent = Some(parent);
            effect.sprite.offset = sprite_offset;
            effect.sprite.flip_x = flip || effect.sprite.flip_x;
            effect.sprite.scale = WorldVector { x: scale, y: scale };
            effect.sprite.tint = tint;
            effect.sprite.offsets = offsets;
            effect.raw_7f8 = 50.0;
            effect.time_to_live = 20;
            effect.raw_7f4 = 20;
        }
        Ok(())
    }
}

pub(crate) fn advance(state: &mut State, phase: Phase, id: u32) -> Result<(), String> {
    let index = state
        .sprite_effects
        .iter()
        .position(|effect| effect.id == id)
        .ok_or("BloodStart callback requires its admitted actor")?;
    if phase == Phase::Skipped || !state.sprite_effects[index].active {
        return Ok(());
    }
    let parent_dead = state.sprite_effects[index].parent.is_none_or(|parent| {
        !state
            .room
            .npcs
            .iter()
            .any(|npc| npc.id == parent.actor.id && !npc.dead)
    });
    let effect = &mut state.sprite_effects[index];
    if phase == Phase::Interpolate {
        effect.presented_position =
            effect.position + (effect.velocity * effect.current_friction) * 0.5;
        return Ok(());
    }
    //45aa84 calls Base before every variant-specific controller.
    effect.full_updates = effect
        .full_updates
        .checked_add(1)
        .ok_or("effect full count exhausted")?;
    effect
        .sprite
        .advance(definition(1000, effect.variant).ok_or("effect config absent")?)?;
    effect.velocity = effect.velocity * effect.current_friction;
    effect.position = effect.position + effect.velocity;
    effect.presented_position = effect.position;
    effect.current_friction = effect.configured_friction;
    match effect
        .blood_effect
        .as_mut()
        .ok_or("BloodStart controller absent")?
    {
        Controller::Poof04 => {
            if effect.lifetime_divisor == 0 {
                let flip_x = state.boot_lifecycle.shared.bounded(2) == 0;
                let flip_y = state.boot_lifecycle.shared.bounded(2) == 0;
                effect.sprite.layer_flips.insert(0, [flip_x, flip_y]);
                effect.lifetime_divisor = state.boot_lifecycle.shared.bounded(3) as i32 + 6;
            }
            if effect.sprite.frame.floor() as i32 == effect.lifetime_divisor {
                // 45e58f pairs flags8 with +7d8 (sequence render key).
                effect.raw_7d8 = true;
                effect.flags |= 8; //17 does not acquire2's0x20000 preservation bit.
            }
            if !effect.sprite.playing {
                effect.active = false;
                effect.removed = true;
            }
        }
        Controller::BloodGush => {
            if !effect.sprite.playing {
                // Authored42 has only loopingIdle; external clip replacement is
                // outside this leaf, never a guessed successful"Loop" playback.
                return Err("BloodGush finished-current Loop callback is not admitted".into());
            }
            if effect.removed {
                effect.active = false;
            } else if parent_dead {
                effect.removed = true;
            }
        }
    }
    if effect.time_to_live > 0 {
        effect.time_to_live -= 1;
        if effect.time_to_live == 0 {
            effect.removed = true;
        }
    }
    Ok(())
}

/// Manager45543e invokes NPC.Uninit47eb80 ->448040 after actor/background
/// callbacks and before4545d0 contacts.448087 nulls each registered child link;
/// inactive effects also clear their own link through447fc0(NULL).
pub(crate) fn clear_retired_links(state: &mut State) {
    for effect in &mut state.sprite_effects {
        if effect.blood_effect != Some(Controller::BloodGush) {
            continue;
        }
        let parent_retired = effect.parent.is_some_and(|parent| {
            state
                .room
                .npcs
                .iter()
                .any(|npc| npc.id == parent.actor.id && npc.removed)
        });
        if !effect.active || parent_retired {
            effect.parent = None;
        }
    }
}
