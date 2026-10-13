//! Ordinary Fireplace33 factory and subclass AI49a530.
//! Fire drains its own damage FIFO before common BaseUpdate, clamps to HP1
//! and extinguishes without entering the ordinary NPC death path.
use crate::{
    audio::{AudioState, SoundRequest},
    combat, content,
    grid::{Grid, GridDescriptor},
    motion::WorldVector,
    mt::{uint_to_uniform, Mt19937},
    npcs::Npc,
    phase::Phase,
    room_spawns::{self, SpawnCategory, SpawnSelector},
};
use serde::{Deserialize, Serialize};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };

/// The actual AI producer parameters passed to the Effect66 factory.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ParticleBirth {
    pub parent_id: u32,
    pub spawn_seed: u32,
    pub position: WorldVector,
    pub velocity: WorldVector,
}

/// Actual Fire loot routing before the independent Pickup.Init subtype draw.
/// Modifier-dependent loot and room return are separate supported-actor work.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LootBirth {
    pub parent_id: u32,
    pub variant: u32,
    pub spawn_seed: u32,
    pub position: WorldVector,
}

#[derive(Default)]
pub(crate) struct Produced {
    pub particle: Option<ParticleBirth>,
    pub loot: Option<LootBirth>,
}

/// Ordinary unmodified Isaac Pickup.Init500150 uses a separate local RNG35.
/// This admits the real drop identity/animation; heart collection and item or
/// trinket modifiers belong to the inventory owner's health slice.
pub(crate) fn loot_subtype(variant: u32, spawn_seed: u32) -> Result<u32, String> {
    if variant == 20 {
        return crate::pickups::regular_subtype(variant, 0, spawn_seed);
    }
    if variant != 10 || spawn_seed == 0 {
        return Err("Fire loot requires a real ordinary coin/heart seed".into());
    }
    let mut word = spawn_seed;
    let mut bounded = |bound| {
        word = room_spawns::next35(word);
        word % bound
    };
    let mut subtype = if bounded(2) == 0 { 2 } else { 1 };
    if bounded(50) == 0 {
        subtype = 4;
    }
    if bounded(10) == 0 {
        subtype = if bounded(20) == 0 { 6 } else { 3 };
    }
    Ok(subtype)
}

/// Ordinary Effect1000/66 follows BaseUpdate before its own full-only drift,
/// timeout and manual sprite frame. A dead active particle receives one more
/// full callback, including two real uniforms, before removal441bb0.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Ember {
    pub id: u32,
    pub parent_npc: u32,
    pub parent_type: u32,
    pub parent_variant: u32,
    pub spawn_seed: u32,
    pub birth_game_update: u32,
    pub position: WorldVector,
    pub presented_position: WorldVector,
    pub velocity: WorldVector,
    pub hp: f32,
    pub max_hp: f32,
    pub collision_radius: f32,
    pub collision_mass: f32,
    pub collision_damage: f32,
    pub base_friction: f32,
    pub grid_collision_class: u32,
    pub entity_collision_class: u32,
    pub lifetime_divisor: i32,
    pub time_to_live: i32,
    pub full_updates: u32,
    pub animation_clock: f32,
    pub animation_playing: bool,
    pub dead: bool,
    pub removed: bool,
    pub visible: bool,
}

impl Ember {
    pub(crate) fn birth(
        id: u32,
        birth: ParticleBirth,
        parent_variant: u32,
        game_update: u32,
    ) -> Self {
        Self {
            id,
            parent_npc: birth.parent_id,
            parent_type: 33,
            parent_variant,
            spawn_seed: birth.spawn_seed,
            birth_game_update: game_update,
            position: birth.position,
            presented_position: birth.position,
            velocity: birth.velocity,
            hp: 0.0,
            max_hp: 0.0,
            collision_radius: 0.0,
            collision_mass: 0.0,
            //458b31 overrides the authored contactDamage0 to22 for common
            //Effect.Init; class0 still excludes all entity collision dispatch.
            collision_damage: 22.0,
            base_friction: 1.0,
            grid_collision_class: 0,
            entity_collision_class: 0,
            lifetime_divisor: 0,
            time_to_live: -1,
            full_updates: 0,
            animation_clock: 0.0,
            animation_playing: true,
            dead: false,
            removed: false,
            visible: true,
        }
    }

    pub(crate) fn advance(&mut self, phase: Phase, random: &mut Mt19937) -> Result<(), String> {
        if self.removed {
            return Ok(());
        }
        if phase != Phase::Update {
            self.presented_position = self.position + (self.velocity * self.base_friction) * 0.5;
            return Ok(());
        }
        self.full_updates = self
            .full_updates
            .checked_add(1)
            .ok_or("Ember callback count exhausted")?;
        //447190 updates the Sprite and moves by entering V (friction1)
        //before45b488 chooses new drift, manual frame and retirement.
        if self.animation_playing {
            let frame = self.animation_clock + 1.0;
            if frame.floor() < content::EMBER_IDLE_FRAMES as f32 {
                self.animation_clock = frame;
            } else {
                self.animation_clock = (content::EMBER_IDLE_FRAMES - 1) as f32;
                self.animation_playing = false;
            }
        }
        self.velocity = self.velocity * self.base_friction;
        self.position = self.position + self.velocity;
        self.presented_position = self.position;
        if self.lifetime_divisor == 0 {
            let lifetime = random.bounded(30) as i32;
            self.time_to_live = lifetime + 1;
            self.lifetime_divisor = lifetime + 2;
        }
        let dy = random.uniform() * 0.1 - 0.05;
        let dx = random.uniform() * 0.4 - 0.2;
        self.velocity = WorldVector {
            x: (self.velocity.x + dx) * 0.98,
            y: (self.velocity.y + dy) * 0.97,
        };
        self.animation_clock = (content::EMBER_IDLE_FRAMES as f32
            * (self.time_to_live as f32 / self.lifetime_divisor as f32))
            .trunc()
            .min((content::EMBER_IDLE_FRAMES - 1) as f32);
        if self.dead {
            self.removed = true;
        }
        if self.time_to_live > 0 {
            self.time_to_live -= 1;
            if self.time_to_live == 0 {
                self.dead = true;
            }
        }
        if !self.position.is_finite() || !self.velocity.is_finite() {
            return Err("Ember motion overflowed".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FireplaceState {
    pub anchor: WorldVector,
    pub authored_grid_index: u16,
    pub entity_flags: u32,
    pub countdown: i32,
    /// Entity.BaseInit initializes this local RNG30 to spawnSeed without Next.
    pub actor_rng_state: u32,
    /// Entity+460 is an integer render-sort addend, never a world Y offset.
    pub render_order_offset: i32,
    pub flame_scale: f32,
    pub flame_render_mode: u32,
    pub particle_birth_count: u32,
    pub last_particle_birth: Option<ParticleBirth>,
    pub loot_birth: Option<LootBirth>,
}

pub struct World<'a> {
    pub grid: &'a mut GridDescriptor,
    pub audio: &'a mut AudioState,
    pub system_counter: u32,
}

fn next7(mut word: u32) -> u32 {
    word ^= word >> 1;
    word ^= word << 21;
    word ^= word >> 20;
    word
}

fn next30(word: &mut u32) -> u32 {
    *word ^= *word >> 4;
    *word ^= *word << 3;
    *word ^= *word >> 17;
    *word
}

/// Room.spawnConfig5c5240 substitutes raw1400 using independent local RNG7,
/// then the entity seed comes from RNG35(groupSeed), not the raw grid seed.
/// Red/blue/purple Fire retain their authored selectors until admitted.
pub(crate) fn from_selector(id: u32, selector: &SpawnSelector) -> Result<Option<Npc>, String> {
    let spawn_seed = match (selector.category, selector.entity_type) {
        (SpawnCategory::Grid, 1400) if !next7(selector.group_seed).is_multiple_of(40) => {
            room_spawns::next35(selector.group_seed)
        }
        (SpawnCategory::Entity, 33) if selector.variant == 0 => selector.spawn_seed,
        _ => return Ok(None),
    };
    if !selector.position.is_finite() || spawn_seed == 0 {
        return Err(
            "Fire admission requires bounded authored position and nonzero entity seed".into(),
        );
    }
    Ok(Some(Npc {
        id,
        kind: 33,
        variant: 0,
        subtype: u32::from(selector.subtype),
        spawn_seed,
        position: selector.position,
        presented_position: selector.position,
        velocity: ZERO,
        target: ZERO,
        wander_velocity: ZERO,
        hp: content::NPC_33_BASEHP,
        max_hp: content::NPC_33_BASEHP,
        collision_radius: content::NPC_33_COLLISIONRADIUS,
        collision_mass: content::NPC_33_COLLISIONMASS,
        collision_damage: content::NPC_33_COLLISIONDAMAGE,
        base_friction: content::NPC_33_FRICTION,
        shadow_size: content::NPC_33_SHADOWSIZE,
        grid_collision_class: content::NPC_33_GRID_CLASS,
        entity_collision_class: 4,
        can_shut_doors: content::NPC_33_CAN_SHUT_DOORS,
        boss: content::NPC_33_BOSS,
        state: 1,
        frame_age: 0,
        animation_frame: 0,
        animation: "Idle".into(),
        animation_clock: 0.0,
        animation_playing: false,
        visible: true,
        saved_initialized_state: 0,
        appearance_effect_seed: None,
        death_effect_seed: None,
        dead: false,
        removed: false,
        damage: combat::DamageQueue::default(),
        keeper: None,
        fireplace: Some(FireplaceState {
            anchor: ZERO,
            authored_grid_index: selector.grid_index,
            entity_flags: 4,
            countdown: 0,
            actor_rng_state: spawn_seed,
            render_order_offset: 0,
            flame_scale: 1.0,
            flame_render_mode: 0,
            particle_birth_count: 0,
            last_particle_birth: None,
            loot_birth: None,
        }),
    }))
}

fn clip(base: &str, subtype: u32) -> String {
    match subtype {
        0 => base.into(),
        1 | 2 => format!("{base}{}", subtype + 1),
        _ => unreachable!("local RNG35 bounds ordinary Fire subtype to0..2"),
    }
}

fn play(npc: &mut Npc, base: &str) {
    npc.animation = clip(base, npc.subtype);
    npc.animation_clock = 0.0;
    npc.animation_frame = 0;
    npc.animation_playing = true;
}

fn update_sprite(npc: &mut Npc) {
    if !npc.animation_playing {
        return;
    }
    let (frames, looping) = if npc.animation.starts_with("Flickering") {
        (content::FIRE_FLICKERING_FRAMES, true)
    } else if npc.animation.starts_with("Dissapear") {
        (content::FIRE_DISSAPEAR_FRAMES, false)
    } else if npc.animation.starts_with("NoFire") {
        (content::FIRE_NOFIRE_FRAMES, false)
    } else {
        unreachable!("ordinary Fire plays only authored clips");
    };
    let frame = npc.animation_clock + 1.0;
    if frame.floor() < frames as f32 {
        npc.animation_clock = frame;
        npc.animation_frame = frame.floor() as u32;
    } else if looping {
        npc.animation_clock = 0.0;
        npc.animation_frame = 0;
    } else {
        npc.animation_clock = (frames - 1) as f32;
        npc.animation_frame = frames - 1;
        npc.animation_playing = false;
    }
}

fn set_selector(
    grid: &mut GridDescriptor,
    position: WorldVector,
    selector: u32,
) -> Result<(), String> {
    let index = Grid::new(grid.clone())?
        .index(position)
        .ok_or("Fire lies outside admitted room grid")?;
    grid.cells[index].selector = selector;
    Ok(())
}

fn initialize(
    npc: &mut Npc,
    fire: &mut FireplaceState,
    random: &mut Mt19937,
    world: &mut World<'_>,
) -> Result<(), String> {
    fire.entity_flags |= 0x40000;
    npc.subtype = room_spawns::next35(npc.spawn_seed) % 3;
    fire.anchor = npc.position;
    npc.target = npc.position;
    fire.countdown = (random.bounded(30) + 60) as i32;
    fire.entity_flags = (fire.entity_flags & !4) | 0x11;
    fire.flame_render_mode = 2;
    if npc.hp > 1.5 {
        npc.state = 8;
        set_selector(world.grid, npc.position, 950)?;
        play(npc, "Flickering");
        fire.flame_scale = npc.hp / npc.max_hp;
    } else {
        npc.state = 3;
        npc.entity_collision_class = 0;
        npc.grid_collision_class = 0;
        set_selector(world.grid, npc.position, 900)?;
        play(npc, "NoFire");
        fire.render_order_offset = -1000;
    }
    Ok(())
}

fn extinguish(
    npc: &mut Npc,
    fire: &mut FireplaceState,
    random: &mut Mt19937,
    world: &mut World<'_>,
) -> Result<(), String> {
    set_selector(world.grid, npc.position, 900)?;
    npc.hp = 1.0;
    npc.state = 3;
    npc.entity_collision_class = 0;
    npc.grid_collision_class = 0;
    play(npc, "NoFire");
    fire.render_order_offset = -1000;
    world.audio.admit(
        random,
        world.system_counter,
        SoundRequest {
            id: 261,
            volume: 0.85,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
    play(npc, "Dissapear");
    // This is the real local RNG30 producer. Pickup.Init and spawned loot
    // motion are not silently replaced by a default subtype or a shared skip.
    if uint_to_uniform(next30(&mut fire.actor_rng_state)) < 0.1 {
        let variant = if uint_to_uniform(next30(&mut fire.actor_rng_state)) < 0.25 {
            10
        } else {
            20
        };
        fire.loot_birth = Some(LootBirth {
            parent_id: npc.id,
            variant,
            spawn_seed: next30(&mut fire.actor_rng_state),
            position: npc.position,
        });
    }
    Ok(())
}

fn active_ai(
    npc: &mut Npc,
    fire: &mut FireplaceState,
    random: &mut Mt19937,
    world: &mut World<'_>,
) -> Result<(), String> {
    npc.velocity = WorldVector {
        x: fire.anchor.x - npc.position.x,
        y: fire.anchor.y - npc.position.y,
    };
    if npc.damage.pending_damage() > 0.0 && npc.state == 8 {
        let flags = npc.damage.pending_flags();
        for event in npc.damage.events() {
            if event.amount > 0.0 {
                let draw = random.bounded(3);
                let loss = if event.amount > 10.0 {
                    if draw == 0 {
                        3.0
                    } else {
                        2.0
                    }
                } else if draw == 0 {
                    2.0
                } else {
                    1.0
                };
                npc.hp -= loss;
            }
        }
        npc.hp = npc.hp.max(1.0);
        fire.flame_scale = npc.hp / npc.max_hp;
        if npc.hp < 1.5 || flags & 4 != 0 {
            extinguish(npc, fire, random, world)?;
        }
    }
    npc.damage.clear();
    if fire.countdown > 0 {
        fire.countdown -= 1;
    }
    if npc.state == 8 && random.bounded(7) == 0 {
        let y = -15.0 - npc.hp * 15.0 / 6.0 - random.uniform() * 15.0;
        let x = random.uniform() * 16.0 - 8.0;
        let vy = random.uniform() * -1.6 - 0.8;
        let vx = random.uniform() * 0.4 - 0.2;
        fire.particle_birth_count = fire
            .particle_birth_count
            .checked_add(1)
            .ok_or("Fire particle producer counter exhausted")?;
        fire.last_particle_birth = Some(ParticleBirth {
            parent_id: npc.id,
            spawn_seed: random.next_u32(),
            position: npc.position + WorldVector { x, y },
            velocity: WorldVector { x: vx, y: vy },
        });
    } else if npc.state == 3 && Grid::new(world.grid.clone())?.collision_at(npc.position) != 0 {
        //441bb0 clears active31e and sets dead31f; Sprite visibility31d
        //is a separate field and is not changed by this removal callback.
        npc.dead = true;
        npc.removed = true;
    }
    Ok(())
}

pub(crate) fn advance(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut World<'_>,
) -> Result<Produced, String> {
    if npc.removed {
        return Ok(Produced::default());
    }
    if phase != Phase::Update {
        npc.presented_position = npc.position + (npc.velocity * npc.base_friction) * 0.5;
        return Ok(Produced::default());
    }
    let mut fire = npc
        .fireplace
        .take()
        .ok_or("Fire actor requires admitted subclass state")?;
    let particles_before = fire.particle_birth_count;
    let loot_before = fire.loot_birth.is_some();
    let result = (|| {
        let mut friction = npc.base_friction;
        if npc.state == 1 && fire.entity_flags & 4 != 0 && npc.frame_age == 0 {
            friction *= 0.01;
            npc.state = 0;
            initialize(npc, &mut fire, random, world)?;
            // 4f9a41 initializer clears flag4, then generic4fa3eb invokes
            // the newly initialized AI again within this same full update.
            npc.frame_age = 1;
        }
        active_ai(npc, &mut fire, random, world)?;
        npc.velocity = npc.velocity * friction;
        npc.position = npc.position + npc.velocity;
        npc.presented_position = npc.position;
        update_sprite(npc);
        if !npc.position.is_finite() || !npc.velocity.is_finite() || !npc.hp.is_finite() {
            return Err("Fire update produced nonfinite actor state".into());
        }
        Ok(Produced {
            particle: (fire.particle_birth_count != particles_before)
                .then(|| fire.last_particle_birth.clone())
                .flatten(),
            loot: (!loot_before).then(|| fire.loot_birth.clone()).flatten(),
        })
    })();
    npc.fireplace = Some(fire);
    result
}
