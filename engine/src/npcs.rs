//! Supported NPC actors from authored room selectors.
//! Full NPC Update owns AI, native BaseUpdate motion and FIFO damage drain.
//! Intermediate presentation predicts half velocity without advancing AI.
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

pub struct World<'a> {
    pub game_counter: u32,
    pub player: WorldVector,
    pub grid: &'a Grid,
    pub audio: &'a mut crate::audio::AudioState,
    pub system_counter: u32,
}

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
    pub damage: combat::DamageQueue,
    pub keeper: Option<crate::keepers::KeeperState>,
    pub fireplace: Option<crate::fireplaces::FireplaceState>,
    pub monstro: Option<crate::monstro::MonstroState>,
    pub level2_fly: Option<crate::level2_flies::Level2FlyState>,
}

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };

/// Ordinary actor explosion collection is strict (75 + collision radius).
/// Velocity changes at admission; HP waits for the next full BaseUpdate.
pub fn bomb_hit(state: &mut State, center: WorldVector, bomb_id: u32) -> Result<(), String> {
    for npc in &mut state.room.npcs {
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
        let radius = 75.0 + npc.collision_radius;
        if squared >= radius * radius {
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
                amount: 60.0,
                kind: combat::DamageKind::Bomb,
                source: combat::DamageSource {
                    id: bomb_id,
                    entity_type: 4,
                },
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
        if npc.dead || npc.removed || npc.entity_collision_class != 4 {
            continue;
        }
        let dx = ground.x - npc.position.x;
        let dy = ground.y - npc.position.y;
        let sum = radius + npc.collision_radius;
        if dx.abs() > sum || dy.abs() > sum || (dx * dx + dy * dy).sqrt() >= sum {
            continue;
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
            return Ok(Some(npc.id));
        }
    }
    Ok(None)
}

pub fn player_contacts(state: &mut State) -> Result<(), String> {
    if state.room.npcs.is_empty() {
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
        let contact_damage = npc.collision_damage;
        if contact_damage > 0.0 {
            combat::try_damage_player(state, contact_damage as u8, 60, 0)?;
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
            if first.dead
                || second.dead
                || first.entity_collision_class != 4
                || second.entity_collision_class != 4
            {
                continue;
            }
            let mut a = body(first);
            let mut b = body(second);
            if combat::resolve_npc_or_bomb_pair(&mut a, &mut b)?.colliding {
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
        match npc.kind {
            20 => crate::monstro::post_contact_grid(npc, &grid)?,
            214 => crate::level2_flies::post_contact_grid(npc, &grid)?,
            244 => crate::roundworm::post_contact_grid(npc, &grid)?,
            // Source class3 clamps the center first. These authored families
            // have zero grid collision points, so their sample loop is empty.
            13 | 18 | 33 if npc.grid_collision_class == 3 => {
                npc.position = grid.clamp_room(npc.position, 0.0);
                npc.presented_position = npc.position;
            }
            _ => {}
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

pub fn advance_room_clear(state: &mut State) {
    // The final callback decrements2490 before Remove. Native Room5d1454
    // also refreshes it from manager e4+e8 BEFORE the full actor pass; do not
    // replace that boundary with a post-retirement NPC-vector recount.
    if state
        .room
        .npcs
        .iter()
        .any(|npc| !npc.removed && npc.can_shut_doors)
    {
        state.room.combat_clear_countdown = 10;
    } else if state.room.combat_clear_countdown > 0 {
        state.room.combat_clear_countdown -= 1;
    } else if !state.room.combat_clear {
        state.room.combat_clear = true;
        change_doors(state, true);
    }
}

fn change_doors(state: &mut State, open: bool) {
    let mut changed = Vec::new();
    for door in &mut state.room.appearance.doors {
        if !open && door.state == "Opened" {
            door.state = "Closed".into();
            if !state.room.combat_shut_door_slots.contains(&door.slot) {
                state.room.combat_shut_door_slots.push(door.slot);
            }
            changed.push(door.world_position);
        } else if open
            && door.state == "Closed"
            && state.room.combat_shut_door_slots.contains(&door.slot)
        {
            door.state = "Opened".into();
            changed.push(door.world_position);
        }
    }
    if open {
        state.room.combat_shut_door_slots.clear();
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
                entity.state = if open { 2 } else { 1 };
                // The next full Door::Update admits opened-door class5.
                entity.collision_class = 4;
            }
        }
    }
}

fn normalized(vector: WorldVector) -> WorldVector {
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
        x: radians.cos(),
        y: radians.sin(),
    }
}

impl Npc {
    pub fn from_selector(id: u32, selector: &SpawnSelector) -> Result<Option<Self>, String> {
        if let Some(fly) = crate::level2_flies::from_selector(id, selector)? {
            return Ok(Some(fly));
        }
        if let Some(monstro) = crate::monstro::from_selector(id, selector)? {
            return Ok(Some(monstro));
        }
        if let Some(fireplace) = crate::fireplaces::from_selector(id, selector)? {
            return Ok(Some(fireplace));
        }
        if selector.category == SpawnCategory::Entity
            && selector.entity_type == 17
            && selector.variant == 1
            && selector.subtype == 0
        {
            return crate::keepers::from_selector(id, selector).map(Some);
        }
        if selector.category != SpawnCategory::Entity
            || !matches!(selector.entity_type, 13 | 18 | 244)
            || selector.variant != 0
            || selector.subtype != 0
        {
            return Ok(None);
        }
        if !selector.position.is_finite() || selector.spawn_seed == 0 {
            return Err(
                "NPC admission requires bounded authored position and nonzero spawn seed".into(),
            );
        }
        let (hp, radius, mass, collision_damage, base_friction, shadow_size) =
            if selector.entity_type == 13 {
                (
                    content::NPC_13_BASEHP,
                    content::NPC_13_COLLISIONRADIUS,
                    content::NPC_13_COLLISIONMASS,
                    content::NPC_13_COLLISIONDAMAGE,
                    content::NPC_13_FRICTION,
                    content::NPC_13_SHADOWSIZE,
                )
            } else if selector.entity_type == 18 {
                (
                    content::NPC_18_BASEHP,
                    content::NPC_18_COLLISIONRADIUS,
                    content::NPC_18_COLLISIONMASS,
                    content::NPC_18_COLLISIONDAMAGE,
                    content::NPC_18_FRICTION,
                    content::NPC_18_SHADOWSIZE,
                )
            } else {
                (
                    content::NPC_244_BASEHP,
                    content::NPC_244_COLLISIONRADIUS,
                    content::NPC_244_COLLISIONMASS,
                    content::NPC_244_COLLISIONDAMAGE,
                    content::NPC_244_FRICTION,
                    content::NPC_244_SHADOWSIZE,
                )
            };
        Ok(Some(Self {
            id,
            kind: u32::from(selector.entity_type),
            variant: 0,
            subtype: 0,
            spawn_seed: selector.spawn_seed,
            position: selector.position,
            presented_position: selector.position,
            velocity: ZERO,
            target: ZERO,
            wander_velocity: ZERO,
            hp,
            max_hp: hp,
            collision_radius: radius,
            collision_mass: mass,
            collision_damage,
            base_friction,
            shadow_size,
            grid_collision_class: match selector.entity_type {
                13 => content::NPC_13_GRID_CLASS,
                18 => content::NPC_18_GRID_CLASS,
                _ => content::NPC_244_GRID_CLASS,
            },
            entity_collision_class: 4,
            can_shut_doors: if selector.entity_type == 13 {
                content::NPC_13_CAN_SHUT_DOORS
            } else if selector.entity_type == 18 {
                content::NPC_18_CAN_SHUT_DOORS
            } else {
                content::NPC_244_CAN_SHUT_DOORS
            },
            boss: if selector.entity_type == 13 {
                content::NPC_13_BOSS
            } else if selector.entity_type == 18 {
                content::NPC_18_BOSS
            } else {
                content::NPC_244_BOSS
            },
            state: 1,
            frame_age: 0,
            animation_frame: 0,
            animation: if selector.entity_type == 244 {
                "Idle"
            } else {
                "Fly"
            }
            .into(),
            animation_clock: 0.0,
            // NPC config47f573 uses Sprite.SetFrame(default,0), which stops
            // playback. Worm waits for the staged forced Appear at counter4.
            animation_playing: selector.entity_type != 244,
            visible: true,
            saved_initialized_state: 0,
            appearance_effect_seed: None,
            death_effect_seed: None,
            dead: false,
            removed: false,
            damage: combat::DamageQueue::default(),
            keeper: None,
            fireplace: None,
            monstro: None,
            level2_fly: None,
        }))
    }

    pub fn health(&self) -> combat::ActorHealth {
        combat::ActorHealth {
            hp: self.hp,
            max_hp: self.max_hp,
        }
    }

    pub fn advance(
        &mut self,
        phase: Phase,
        random: &mut Mt19937,
        world: &mut World<'_>,
    ) -> Result<(), String> {
        if self.kind == 17 {
            return crate::keepers::advance(self, phase);
        }
        if self.removed {
            return Ok(());
        }
        if phase != Phase::Update {
            self.presented_position = self.position + self.velocity * 0.5;
            return Ok(());
        }
        if self.dead {
            // Native ordinary flies retain negative HP/dead state through the
            // next intermediate presentation, then retire at the next full pass.
            self.removed = true;
            return Ok(());
        }
        let mut friction = self.base_friction;
        if self.state == 1 {
            if self.frame_age == 0 {
                // First Appear update temporarily sets AI state0 and executes
                // its initializer at4f9a41 before the real Poof15 seed.
                friction = self.fly_ai(
                    world.game_counter,
                    random,
                    world.player,
                    world.grid,
                    friction * 0.01,
                );
                self.appearance_effect_seed = Some(random.next_u32());
            }
            self.frame_age += 1;
            if self.frame_age > 1 {
                friction *= 0.01;
            }
            if self.frame_age == 21 {
                self.state = 3;
                friction = self.base_friction;
            }
        } else {
            friction = self.fly_ai(
                world.game_counter,
                random,
                world.player,
                world.grid,
                friction,
            );
        }
        self.velocity = self.velocity * friction;
        self.position = self.position + self.velocity;
        self.presented_position = self.position;
        self.animation_frame = self
            .animation_frame
            .checked_add(1)
            .ok_or("NPC animation clock exhausted")?;
        let mut health = self.health();
        self.damage.advance(&mut health, phase)?;
        self.hp = health.hp;
        if self.hp <= 0.0 {
            self.dead = true;
            // Source44641d zero-gib sound precedes the real Blood Splat seed.
            world.audio.admit(
                random,
                world.system_counter,
                crate::audio::SoundRequest {
                    id: 30,
                    volume: 0.8,
                    delay: 2,
                    looping: false,
                    pitch: 1.0,
                },
            );
            self.death_effect_seed = Some(random.next_u32());
        }
        if !self.position.is_finite() || !self.velocity.is_finite() {
            return Err("NPC source motion overflowed native world vectors".into());
        }
        Ok(())
    }

    fn fly_ai(
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
        if self.target != ZERO && counter.is_multiple_of(2) {
            let delta = WorldVector {
                x: self.target.x - self.position.x,
                y: (self.target.y - self.position.y) * 0.5,
            };
            let length = (delta.x * delta.x + delta.y * delta.y).sqrt();
            if self.kind == 13 {
                self.wander_velocity = self.wander_velocity * 0.7;
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
        if counter % 3 == 2 {
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
            self.velocity = self.velocity + self.wander_velocity * (2.0 / friction);
            friction *= 0.6;
        }
        friction
    }
}
