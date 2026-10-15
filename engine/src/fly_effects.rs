//! The Duke of Flies' effects: Tiny Fly 1000.33 (45b790), its dark smoke
//! 1000.1 (45ab80) and the Blood Explosion 1000.2 a killed tiny fly leaves.
//! A tiny fly draws bounded(5) on every live update, drifts toward its
//! parent's rim (or the player when orphaned), flees the player inside 60,
//! dies on a tear, knife or fireplace (1000.2 at once) or a bomb blast, then
//! falls onto its shadow and leaves a 1000.7 splat.
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

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Controller {
    TinyFly {
        /// Main layer offset (LayerState+0x34): the dying body's fall.
        fall: f32,
        /// +440: the clamped position of the previous update.
        previous: WorldVector,
        /// +498 parent NPC handle.
        parent: Option<u32>,
    },
    Smoke {
        /// +814 rotation magnitude.
        angle: f32,
    },
    BloodExplosion {
        /// +7d4 frame on which the floor stamp is taken.
        stamp_frame: u32,
    },
}

/// What a tiny fly's update sees of the room.
pub(crate) struct Probe<'a> {
    pub player: WorldVector,
    pub game_counter: u32,
    pub sequence: u32,
    pub bounds: (WorldVector, WorldVector),
    /// The parent's position and collision radius while it is listed.
    pub parent: Option<(WorldVector, f32)>,
    /// The tear partition from the last contact pass, at live positions.
    pub tears: &'a [(WorldVector, f32)],
    pub knives: &'a [(WorldVector, f32)],
    pub fires: &'a [(WorldVector, f32)],
}

fn base(id: u32, config: u32, seed: u32, position: WorldVector, game_update: u32) -> SpriteEffect {
    let definition = definition(1000, config).expect("authored effect config");
    SpriteEffect {
        id,
        variant: config,
        subtype: 0,
        spawn_seed: seed,
        spawner_type: 0,
        spawner_variant: 0,
        parent: None,
        birth_game_update: game_update,
        actor_rng: items::Rng::new(seed, [4, 3, 17]),
        position,
        presented_position: position,
        render_offset: ZERO,
        velocity: ZERO,
        configured_friction: definition.friction,
        current_friction: definition.friction,
        collision_radius: definition.collision_radius,
        collision_mass: definition.collision_mass,
        collision_damage: 22.0,
        hp: definition.base_hp,
        max_hp: definition.base_hp,
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
        sprite: Sprite::new(definition),
        full_updates: 0,
        poop_effect: None,
        blood_effect: None,
        creep_effect: None,
        fly_effect: None,
        epic_fetus: None,
    }
}

impl NpcEffectAdmission<'_> {
    fn next_id(&mut self) -> Result<u32, String> {
        let id = *self.next_effect;
        *self.next_effect = id.checked_add(1).ok_or("effect handle exhausted")?;
        Ok(id)
    }

    fn admit_actor(&mut self, effect: SpriteEffect) -> Result<u32, String> {
        let id = effect.id;
        self.effects.push(effect);
        self.scheduler.admit(ActorRef {
            kind: ActorKind::Effect,
            id,
        })?;
        Ok(id)
    }

    /// Spawn550a40 of 1000.33 by the Duke (spawner type 67); Init draws
    /// nothing. The Cathedral "White" clip is not admitted.
    pub(crate) fn admit_tiny_fly(
        &mut self,
        seed: u32,
        position: WorldVector,
        velocity: WorldVector,
        parent: Option<u32>,
    ) -> Result<u32, String> {
        let id = self.next_id()?;
        let mut effect = base(
            id,
            33,
            seed,
            position,
            self.game_update_count,
        );
        effect.spawner_type = 67;
        effect.velocity = velocity;
        effect.entity_collision_class = 4;
        effect.fly_effect = Some(Controller::TinyFly {
            fall: 0.0,
            previous: ZERO,
            parent,
        });
        self.admit_actor(effect)
    }

    /// Variant 33 records still listed (dying ones included).
    pub(crate) fn tiny_fly_count(&self) -> usize {
        self.effects
            .iter()
            .filter(|effect| effect.variant == 33 && effect.active)
            .count()
    }

    /// The spawner's immediate vtable+0x18 update of a fresh tiny fly.
    pub(crate) fn advance_tiny_fly_initial(
        &mut self,
        id: u32,
        random: &mut Mt19937,
        probe: &Probe,
    ) -> Result<(), String> {
        let index = self
            .effects
            .iter()
            .position(|effect| effect.id == id)
            .ok_or("tiny fly first update requires its actor")?;
        tiny_fly(self, index, random, probe)
    }

    /// Attack03: push every tiny fly away from `origin`.
    pub(crate) fn push_tiny_flies(&mut self, mut push: impl FnMut(WorldVector, &mut WorldVector, f32)) {
        for effect in self
            .effects
            .iter_mut()
            .filter(|effect| effect.variant == 33 && effect.active)
        {
            let friction = effect.current_friction;
            push(effect.position, &mut effect.velocity, friction);
        }
    }

    /// 1000.1 from the Duke: Init draws its own FlipX, then the spawner
    /// tints it black at half alpha, lifts it 40 and updates it at once.
    pub(crate) fn admit_duke_smoke(
        &mut self,
        seed: u32,
        position: WorldVector,
        angle: f32,
        random: &mut Mt19937,
    ) -> Result<u32, String> {
        let id = self.next_id()?;
        let mut effect = base(
            id,
            1,
            seed,
            position,
            self.game_update_count,
        );
        effect.sprite.flip_x = random.bounded(2) == 0;
        effect.spawner_type = 67;
        effect.sprite.tint = [0.0, 0.0, 0.0, 0.5];
        effect.render_offset = WorldVector { x: 0.0, y: -40.0 };
        effect.fly_effect = Some(Controller::Smoke { angle });
        let id = self.admit_actor(effect)?;
        let index = self.effects.len() - 1;
        smoke(&mut self.effects[index])?;
        Ok(id)
    }

    /// A killed tiny fly's 1000.2: seed, scale 0.3, then its first update
    /// (both flips and the stamp frame).
    fn blood_explosion(&mut self, seed: u32, position: WorldVector, random: &mut Mt19937) -> Result<(), String> {
        let id = self.next_id()?;
        let mut effect = base(
            id,
            2,
            seed,
            position,
            self.game_update_count,
        );
        effect.sprite.scale = WorldVector { x: 0.3, y: 0.3 };
        self.admit_actor(effect)?;
        let index = self.effects.len() - 1;
        let effect = &mut self.effects[index];
        base_update(effect)?;
        let flip_x = random.bounded(2) == 0;
        let flip_y = random.bounded(2) == 0;
        // 1000.002 has four layers (body and three poof sheets).
        for layer in 0..4 {
            effect.sprite.layer_flips.insert(layer, [flip_x, flip_y]);
        }
        effect.fly_effect = Some(Controller::BloodExplosion {
            stamp_frame: random.bounded(3) + 6,
        });
        Ok(())
    }
}

/// 447190: sprite step, velocity times friction, position, friction reset.
fn base_update(effect: &mut SpriteEffect) -> Result<(), String> {
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
    Ok(())
}

/// 45ab80 for the Duke's smoke: rotation by its own FlipX; removed when the
/// Explosion clip finishes (spawner 67 skips SFX 182).
fn smoke(effect: &mut SpriteEffect) -> Result<(), String> {
    base_update(effect)?;
    let Some(Controller::Smoke { angle }) = effect.fly_effect else {
        return Err("smoke controller absent".into());
    };
    effect.sprite.rotation = if effect.sprite.flip_x { angle } else { -angle };
    if !effect.sprite.playing {
        effect.removed = true;
        effect.active = false;
    }
    Ok(())
}

fn add_scaled(velocity: &mut WorldVector, impulse: WorldVector, friction: f32) {
    if friction != 0.0 {
        *velocity = *velocity
            + WorldVector {
                x: impulse.x / friction,
                y: impulse.y / friction,
            };
    }
}

fn length(vector: WorldVector) -> f32 {
    (vector.x * vector.x + vector.y * vector.y).sqrt()
}

/// 45b790: one full tiny-fly update of `effects[index]`; kill products are
/// admitted (with their draws) at the check that fires.
fn tiny_fly(
    admission: &mut NpcEffectAdmission,
    index: usize,
    random: &mut Mt19937,
    probe: &Probe,
) -> Result<(), String> {
    let effect = &mut admission.effects[index];
    base_update(effect)?;
    let toward = WorldVector {
        x: probe.player.x - effect.position.x,
        y: probe.player.y - effect.position.y,
    };
    let distance = length(toward);
    let unit = if distance == 0.0 {
        WorldVector {
            x: toward.x / 0.01,
            y: toward.y / 0.01,
        }
    } else {
        WorldVector {
            x: toward.x / distance,
            y: toward.y / distance,
        }
    };
    effect.current_friction *= 0.98;
    let friction = effect.current_friction;
    if effect.removed {
        return dying(admission, index, random, probe);
    }
    if random.bounded(5) == 0 {
        if let Some((parent, radius)) = probe.parent {
            let delta = WorldVector {
                x: parent.x - effect.position.x,
                y: parent.y - effect.position.y,
            };
            let span = length(delta);
            let rim = delta * ((span - radius) / span);
            let strength = random.uniform() * 0.5;
            let rim_length = length(rim);
            let direction = if rim_length > 0.0 { rim * (1.0 / rim_length) } else { ZERO };
            add_scaled(&mut effect.velocity, direction * strength, friction);
        } else {
            let strength = random.uniform() * 0.5;
            add_scaled(&mut effect.velocity, unit * strength, friction);
        }
    }
    if (probe.game_counter.wrapping_add(probe.sequence)).is_multiple_of(5) {
        let strength = random.uniform() * 0.9;
        let angle = crate::npcs::direction(random);
        add_scaled(&mut effect.velocity, angle * strength, friction);
    }
    if 60.0 > distance {
        let push = (1.0 - (distance * distance) / 3600.0) * -0.2;
        add_scaled(&mut effect.velocity, unit * push, friction);
    }
    // 45b97e/45ba9d/45bc31: three independent kill checks.
    let position = effect.position;
    let near = |(other, reach): &(WorldVector, f32), extra: f32| {
        let (dx, dy) = (other.x - position.x, other.y - position.y);
        (extra + reach) * (extra + reach) > dx * dx + dy * dy
    };
    let kills = [
        probe.tears.iter().any(|tear| near(tear, 0.5)),
        probe.knives.iter().any(|(knife, reach)| {
            let (dx, dy) = (knife.x - position.x, knife.y - position.y);
            (1.5 * reach) * (1.5 * reach) > dx * dx + dy * dy
        }),
        probe.fires.iter().any(|fire| near(fire, 2.0)),
    ];
    for kill in kills {
        if kill {
            admission.effects[index].removed = true;
            let seed = random.next_u32();
            admission.blood_explosion(
                seed,
                position + WorldVector { x: 0.0, y: -16.0 },
                random,
            )?;
            admission.effects[index].sprite.playing = false;
        }
    }
    tail(&mut admission.effects[index], probe);
    Ok(())
}

/// 4601bd: the body falls 16 onto its shadow while spiralling, then splats.
fn dying(
    admission: &mut NpcEffectAdmission,
    index: usize,
    random: &mut Mt19937,
    probe: &Probe,
) -> Result<(), String> {
    let effect = &mut admission.effects[index];
    let Some(Controller::TinyFly { fall, .. }) = effect.fly_effect.as_mut() else {
        return Err("tiny fly controller absent".into());
    };
    if *fall > 16.0 {
        let position = effect.position;
        effect.active = false;
        let seed = random.next_u32();
        let splat = admission.admit_blood(seed, position, [1.0; 4], [0.0; 3])?;
        admission.set_blood_scale(splat, 0.1)?;
        admission.advance_blood_initial(splat, random)?;
        return Ok(());
    }
    *fall += 0.25;
    let step = fall.abs() * 0.0625;
    let sign = (probe.sequence & 1) as f32 + (probe.sequence & 1) as f32 - 1.0;
    let degrees = (probe.game_counter as f32 * 15.0) * sign;
    let radians = degrees * 0.017_453_292;
    let spiral = WorldVector {
        x: crate::fmath::cos(radians),
        y: crate::fmath::sin(radians),
    };
    let fallen = *fall;
    effect.position = effect.position + (spiral * step) * 3.0;
    effect.current_friction *= 0.8;
    effect.sprite.layer_offsets.insert(0, [0.0, fallen]);
    tail(effect, probe);
    Ok(())
}

/// 45be15: FlipX by the motion since the previous clamp, then clamp 2 in.
fn tail(effect: &mut SpriteEffect, probe: &Probe) {
    let Some(Controller::TinyFly { previous, .. }) = effect.fly_effect.as_mut() else {
        return;
    };
    effect.sprite.flip_x = 0.0 > effect.position.x - previous.x;
    let (minimum, maximum) = probe.bounds;
    effect.position.x = effect.position.x.min(maximum.x - 2.0).max(minimum.x + 2.0);
    effect.position.y = effect.position.y.min(maximum.y - 2.0).max(minimum.y + 2.0);
    effect.presented_position = effect.position;
    *previous = effect.position;
}

/// One manager-pass update of a Duke effect.
pub(crate) fn advance(state: &mut State, phase: Phase, id: u32) -> Result<(), String> {
    let Some(index) = state.sprite_effects.iter().position(|effect| effect.id == id) else {
        return Ok(());
    };
    if phase == Phase::Skipped || !state.sprite_effects[index].active {
        return Ok(());
    }
    if phase == Phase::Interpolate {
        let effect = &mut state.sprite_effects[index];
        effect.presented_position =
            effect.position + (effect.velocity * effect.current_friction) * 0.5;
        return Ok(());
    }
    match state.sprite_effects[index].fly_effect.clone() {
        Some(Controller::Smoke { .. }) => smoke(&mut state.sprite_effects[index]),
        Some(Controller::BloodExplosion { stamp_frame }) => {
            let effect = &mut state.sprite_effects[index];
            base_update(effect)?;
            // The frame-`stamp_frame` floor stamp (flags 8|0x20000) is not
            // admitted; the explosion plays out and is removed.
            let _ = stamp_frame;
            if !effect.sprite.playing {
                effect.removed = true;
                effect.active = false;
            }
            Ok(())
        }
        Some(Controller::TinyFly { parent, .. }) => {
            let grid = crate::grid::Grid::new(
                state.room.grid.clone().ok_or("tiny fly requires the room grid")?,
            )?;
            let parent = parent.and_then(|parent| {
                state
                    .room
                    .npcs
                    .iter()
                    .find(|npc| npc.id == parent)
                    .map(|npc| (npc.position, npc.collision_radius))
            });
            let tears = tear_partition(state);
            let knives: Vec<_> = state
                .knives
                .iter()
                .map(|knife| (knife.position, knife.collision_radius))
                .collect();
            let fires: Vec<_> = state
                .room
                .npcs
                .iter()
                .filter(|npc| npc.kind == 33)
                .map(|npc| (npc.position, npc.collision_radius))
                .collect();
            let probe = Probe {
                player: crate::bombs::player_world(state),
                game_counter: state.game_update_count.wrapping_add(1),
                sequence: state
                    .actor_scheduler
                    .sequence_id(ActorRef {
                        kind: ActorKind::Effect,
                        id,
                    })
                    .unwrap_or(0),
                bounds: crate::laser_npc_runtime::room_bounds_from_grid(&grid),
                parent,
                tears: &tears,
                knives: &knives,
                fires: &fires,
            };
            let mut admission = NpcEffectAdmission {
                game_update_count: state.game_update_count,
                next_effect: &mut state.next_effect,
                effects: &mut state.sprite_effects,
                scheduler: &mut state.actor_scheduler,
            };
            tiny_fly(&mut admission, index, &mut state.boot_lifecycle.shared, &probe)
        }
        None => Ok(()),
    }
}

/// The tear partition of the last contact pass (live tears and colliding
/// knives) at their current positions.
pub(crate) fn tear_partition(state: &State) -> Vec<(WorldVector, f32)> {
    let mut partition: Vec<_> = state
        .tear_partition
        .iter()
        .filter_map(|id| state.tears.iter().find(|tear| tear.id == *id))
        .map(|tear| (tear.motion.position, tear.collision_radius))
        .collect();
    partition.extend(
        state
            .knives
            .iter()
            .filter(|knife| knife.entity_collision_class != 0)
            .map(|knife| (knife.position, knife.collision_radius)),
    );
    partition
}

/// 554490: a bomb blast kills every tiny fly inside its radius (no 1000.2).
pub(crate) fn bomb_hit(state: &mut State, position: WorldVector, radius: f32, clear: impl Fn(WorldVector) -> bool) {
    for effect in state
        .sprite_effects
        .iter_mut()
        .filter(|effect| effect.variant == 33 && effect.active && !effect.removed)
    {
        let (dx, dy) = (effect.position.x - position.x, effect.position.y - position.y);
        if radius * radius > dx * dx + dy * dy && clear(effect.position) {
            effect.removed = true;
        }
    }
}
