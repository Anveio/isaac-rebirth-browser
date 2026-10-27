//! Mom 45.0 and Mom Stomp 45.10: ai 4d3960 (subtype 0 only).
//!
//! Mom's room authors five 45.0 actors: one at each door and one in the
//! centre. Each initializer (state 0) removes every door (Room::RemoveDoor
//! 5cbee0) and clears the descriptor's door mask. A door Mom snaps 15 px
//! into its doorway (table 8ecba0) and turns to face the room. The centre
//! Mom morphs into Mom Stomp 45.10, adopts the other four as children (+f88
//! numbers them 1..4), and moves the player to (320, 370).
//!
//! The foot (parent) stays pinned to its target. With no cooldown (+f68)
//! it rolls Random(40) every update, or every update anyway in the first 60
//! room frames. It then stomps where the player stands, or waits and calls
//! out when the player is near a door. The stomp clip's frame 28 blasts 300
//! damage in a 50 px radius. The door parts share the foot's HP. On their
//! is_frame(4) they reach out with ArmOpen at a near player, look with Eye,
//! or open the door (Fat01/Fat02) to drop one of twelve enemies.
//!
//! When any part dies the whole body dies. The foot's dead update damages
//! every countable actor by 100. Subtypes 1 and 2 (bosscolors.xml) are not
//! admitted: the port never rolls boss colours.
use crate::{
    audio::SoundRequest,
    combat::{DamageEvent, DamageKind, DamageSource},
    content,
    entities::{Birth, Family, NpcWorld, SpawnContext, SpawnedNpc},
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    phase::Phase,
    plain_npc,
    room_spawns::{SpawnCategory, SpawnSelector},
};
use serde::{Deserialize, Serialize};

pub(crate) const FAMILY: Family = Family {
    kinds: &[45],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags,
    occupancy: crate::entities::Occupancy::None,
};

/// +318 NO_DEATH_TRIGGER: set by the initializer, cleared by the dead foot.
const NO_DEATH_TRIGGER: u32 = 0x40000;

/// 8ecba0: the door anchors, left, top, right and bottom.
const DOORS: [WorldVector; 4] = [
    WorldVector { x: 40.0, y: 280.0 },
    WorldVector { x: 320.0, y: 120.0 },
    WorldVector { x: 600.0, y: 280.0 },
    WorldVector { x: 320.0, y: 440.0 },
];

/// 66ed00: the summons; 25 stands for Red Maw 26.1.
const SUMMONS: [u32; 12] = [10, 11, 12, 14, 15, 16, 18, 21, 23, 24, 25, 26];

/// Degrees to radians, as 6126c0 multiplies them.
const RADIANS: f32 = 0.017_453_292;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MomState {
    /// +f88: 0 for the foot, 1..4 for the door parts it adopted.
    pub part: i32,
    /// +f68: the action cooldown.
    pub cooldown: i32,
    /// +f6c on the foot: door parts currently playing Eye.
    pub eyes: i32,
    /// +f58: updates into the current stomp.
    pub stomp: i32,
    /// Sprite rotation +2cc of a door part, in degrees.
    pub rotation: f32,
    /// Entity+318 as this family writes it.
    pub entity_flags: u32,
}

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 45
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Mom requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(45, 0);
    let mut npc = Npc {
        // Init47f500 loads the default clip stopped at frame 0.
        animation: "Eye".into(),
        actor: config.actor.into(),
        mom: Some(MomState {
            part: 0,
            cooldown: 0,
            eyes: 0,
            stomp: 0,
            rotation: 0.0,
            entity_flags: 4,
        }),
        ..Npc::authored(id, config, selector)
    };
    npc.max_hp = content::initial_hp(config, context.stage);
    npc.hp = npc.max_hp;
    Ok(Some(npc))
}

fn entity_flags(npc: &Npc) -> u32 {
    npc.mom.as_ref().map_or(4, |state| state.entity_flags)
}

fn advance(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    plain_npc::advance_dead_ai_deferred_die(npc, phase, random, world, births, ai)
}

fn sound(world: &mut NpcWorld, random: &mut Mt19937, id: u32, volume: f32, delay: i32) {
    world.audio.admit(
        random,
        world.system_counter,
        SoundRequest {
            id,
            volume,
            delay,
            looping: false,
            pitch: 1.0,
        },
    );
}

/// 6126c0: the unit vector of an angle in degrees.
fn angle_vector(degrees: f32) -> WorldVector {
    let radians = degrees * RADIANS;
    WorldVector {
        x: crate::fmath::cos(radians),
        y: crate::fmath::sin(radians),
    }
}

fn squared(vector: WorldVector) -> f32 {
    vector.x * vector.x + vector.y * vector.y
}

fn room_frame(world: &NpcWorld) -> Result<i32, String> {
    world
        .room_frame
        .ok_or_else(|| "Mom requires the Room.Init frame stamp".into())
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.subtype != 0 {
        return Err("Mom subtypes 1 and 2 are not admitted".into());
    }
    let mut state = npc.mom.take().ok_or("Mom requires its AI state")?;
    let result = step(npc, &mut state, random, world, births, friction);
    npc.mom = Some(state);
    result
}

fn step(
    npc: &mut Npc,
    state: &mut MomState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    // 4d3974: a pending hit above Random(20)+10 groans (SFX 97).
    let threshold = random.bounded(20) + 10;
    if npc.damage.pending_damage() > threshold as f32 {
        let delay = random.bounded(30) as i32 + 25;
        sound(world, random, 97, 1.0, delay);
    }
    let entry = npc.state;
    if entry == 0 {
        init(npc, state, random, world, births)?;
        zero_own_hits(npc);
        return Ok(());
    }
    if state.part != 0 {
        let parent_alive = npc
            .parent
            .and_then(|id| world.peers.get(id))
            .is_some_and(|parent| !parent.dead);
        if !parent_alive {
            // 4d3ae1 Kill47fc80: a boss skips Die here.
            npc.dead = true;
            tail(npc, state, friction);
            return Ok(());
        }
    }
    // 4d39cc: pinned to the target; a finished clip returns to idle.
    npc.position = npc.target;
    if plain_npc::finished(npc) {
        npc.state = 3;
    }
    if state.part == 0 {
        foot(npc, state, random, world, births)?;
    } else {
        door(npc, state, entry, random, world, births)?;
    }
    tail(npc, state, friction);
    Ok(())
}

/// 4d3a99: the cooldown ticks, friction x0.8, then 4d3cda.
fn tail(npc: &mut Npc, state: &mut MomState, friction: &mut f32) {
    if state.cooldown > 0 {
        state.cooldown -= 1;
    }
    *friction *= 0.8;
    zero_own_hits(npc);
}

/// 4d3cda: with damage pending, queued hits whose source is a Mom (type
/// 45) are zeroed; the stomp never hurts the body.
fn zero_own_hits(npc: &mut Npc) {
    if npc.damage.pending_damage() > 0.0 {
        npc.damage
            .zero_events(|event| event.source.entity_type == 45);
    }
}

/// State 0 (4d3af8).
fn init(
    npc: &mut Npc,
    state: &mut MomState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    sound(world, random, 101, 1.0, 45);
    state.entity_flags |= NO_DEATH_TRIGGER;
    let mut centre = true;
    for (slot, anchor) in DOORS.iter().enumerate() {
        let offset = WorldVector {
            x: anchor.x - npc.position.x,
            y: anchor.y - npc.position.y,
        };
        if 2500.0 > squared(offset) {
            // 4d44e2: into the doorway, facing the room.
            npc.grid_collision_class = 0;
            let angle = slot as f32 * 90.0;
            state.rotation = angle - 90.0;
            npc.position = *anchor + angle_vector(angle) * 15.0;
            centre = false;
            break;
        }
    }
    births.push(Birth::RemoveDoors);
    npc.target = npc.position;
    let parented = npc.parent.is_some();
    npc.shadow_size = 0.0;
    npc.entity_collision_class = 0;
    if !centre {
        npc.state = 3;
        return Ok(());
    }
    if !parented {
        adopt(npc, state, random, world)?;
    }
    // 4d3bf4: every player stands below the centre, then grid class 5.
    births.push(Birth::PlacePlayer(WorldVector { x: 320.0, y: 370.0 }));
    npc.grid_collision_class = 5;
    npc.state = 3;
    Ok(())
}

/// 4d4b3a: the centre Mom becomes the foot and adopts every other Mom
/// without a parent, in room order, as door part 1, 2, ...
fn adopt(
    npc: &mut Npc,
    state: &mut MomState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
) -> Result<(), String> {
    npc.morph(45, 10, "Stomp", random)?;
    state.part = 0;
    state.eyes = 0;
    npc.hp = npc.max_hp;
    let mut part = 1;
    for other in world
        .peers
        .before
        .iter_mut()
        .chain(world.peers.after.iter_mut())
    {
        if other.removed || other.kind != 45 || other.parent.is_some() {
            continue;
        }
        let variant = other.variant;
        let default = if variant == 10 { "Stomp" } else { "Eye" };
        other.morph(45, variant, default, random)?;
        other.hp = other.max_hp;
        other.parent = Some(npc.id);
        other
            .mom
            .as_mut()
            .ok_or("adopted Mom requires its AI state")?
            .part = part;
        part += 1;
    }
    npc.shadow_size = 0.0;
    npc.entity_collision_class = 0;
    Ok(())
}

/// The foot (+f88 0), 4d3a42.
fn foot(
    npc: &mut Npc,
    state: &mut MomState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    if (state.cooldown == 0 && foot_decides(npc, state, random, world)?) || npc.state == 7 {
        stomp_step(npc, state, random, world, births);
    } else {
        npc.shadow_size = 0.0;
        npc.entity_collision_class = 0;
    }
    if npc.dead {
        died(npc, state, random, world, births)?;
    }
    Ok(())
}

/// 4d3fa0: whether the foot starts a stomp this update. It waits (SFX 101
/// with a long delay, cooldown 30) while the player is within 70 px of a
/// door anchor, or on a 1-in-3 pass after the first 60 room frames.
fn foot_decides(
    npc: &mut Npc,
    state: &mut MomState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
) -> Result<bool, String> {
    if random.bounded(40) != 0 && room_frame(world)? > 59 {
        return Ok(false);
    }
    if npc.state != 3 {
        return Ok(false);
    }
    let mut attack = true;
    if random.bounded(3) != 0 {
        attack = 60 > room_frame(world)?;
    }
    let near_door = DOORS.iter().any(|anchor| {
        let offset = WorldVector {
            x: anchor.x - world.player.x,
            y: anchor.y - world.player.y,
        };
        4900.0 > squared(offset)
    });
    if near_door || !attack {
        let delay = random.bounded(120) as i32 + 45;
        sound(world, random, 101, 1.0, delay);
        state.cooldown = 30;
        return Ok(false);
    }
    sound(world, random, 93, 1.0, 2);
    if room_frame(world)? > 59 {
        npc.target = world.player;
    } else {
        npc.target = npc.position;
    }
    npc.position = npc.target;
    plain_npc::play(npc, "Stomp", true);
    npc.state = 7;
    state.stomp = 0;
    state.cooldown = 80;
    Ok(true)
}

/// State 7 (4d4113): the shadow shrinks over 30 updates; frames 27..61 are
/// solid and frame 28 lands (SFX 52, the blast and a render-only shockwave).
fn stomp_step(
    npc: &mut Npc,
    state: &mut MomState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) {
    let shrink = state.stomp.min(30) as f32 * 0.5 / 30.0;
    npc.shadow_size = (1.33 - shrink) * npc.config().shadow_size;
    let frame = npc.animation_frame;
    if frame > 26 && frame <= 61 {
        npc.entity_collision_class = 4;
        if frame == 28 {
            sound(world, random, 52, 1.0, 2);
            births.push(Birth::MomStomp {
                position: npc.position,
                source: npc.id,
            });
            // PORT GAP: Room::SetShockwave 5cc640 is a render-only floor
            // distortion.
        }
    } else {
        npc.entity_collision_class = 0;
    }
    state.stomp += 1;
}

/// 4d41f0: the dead foot's cry, 100 damage to every countable actor, the
/// Room+2480 particle rain, and NO_DEATH_TRIGGER cleared.
fn died(
    npc: &mut Npc,
    state: &mut MomState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    sound(world, random, 82, 1.0, 2);
    // vtable+0x70 reads +103c, the countable byte. TakeDamage(100, flags 0,
    // no source, countdown 30) only queues. Dead actors are skipped.
    let hit = |other: &mut Npc| -> Result<(), String> {
        if other.removed || other.dead || !other.can_shut_doors {
            return Ok(());
        }
        let health = other.health();
        other.damage.enqueue(
            health,
            DamageEvent {
                amount: 100.0,
                kind: DamageKind::RoomEffect,
                source: DamageSource {
                    id: 0,
                    entity_type: 0,
                },
                copied_source: None,
            },
        )?;
        Ok(())
    };
    for other in world.peers.before.iter_mut() {
        hit(other)?;
    }
    hit(npc)?;
    for other in world.peers.after.iter_mut() {
        hit(other)?;
    }
    // Room+2480 = 20, +2484 = 6: Room::Update 5d1537 rains Blood Particle
    // 1000.5 along the walls for 20 updates. Not ported.
    births.push(Birth::UnportedFactory {
        entity_type: 1000,
        variant: 5,
        subtype: 0,
    });
    state.entity_flags &= !NO_DEATH_TRIGGER;
    Ok(())
}

/// A door part (+f88 1..4), 4d3e18.
fn door(
    npc: &mut Npc,
    state: &mut MomState,
    entry: u32,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    let parent_id = npc.parent.ok_or("Mom door part requires its foot")?;
    {
        let parent = world
            .peers
            .get_mut(parent_id)
            .ok_or("Mom door part requires its live foot")?;
        // 4d3e60: the lower HP wins on both.
        if parent.hp > npc.hp {
            parent.hp = npc.hp;
        } else {
            npc.hp = parent.hp;
        }
        if npc.dead {
            // 4d479e Kill47fc80 on the foot.
            parent.dead = true;
        }
    }
    if npc.state == 9 && npc.animation_frame == 5 {
        sound(world, random, 48, 1.5, 2);
    }
    if plain_npc::on_frame(npc, world, 4)? {
        decide(npc, state, parent_id, random, world)?;
    }
    if npc.state == 3 {
        npc.entity_collision_class = 0;
    } else {
        npc.entity_collision_class = 4;
        if npc.state == 10 && npc.animation_frame == 7 {
            summon(npc, random, world, births)?;
            state.cooldown = 100;
        }
    }
    if entry != npc.state && entry == 8 {
        foot_state(world, parent_id)?.eyes -= 1;
    }
    Ok(())
}

fn foot_state<'a>(world: &'a mut NpcWorld, id: u32) -> Result<&'a mut MomState, String> {
    world
        .peers
        .get_mut(id)
        .and_then(|parent| parent.mom.as_mut())
        .ok_or_else(|| "Mom door part requires its foot's AI state".into())
}

/// 4d45ae: on is_frame(4) a door part reaches for a player within 75 px
/// (100 px when aligned within 20), may look (Eye) while at most one other
/// part looks, then with few enemies opens its door to summon.
fn decide(
    npc: &mut Npc,
    state: &MomState,
    parent_id: u32,
    random: &mut Mt19937,
    world: &mut NpcWorld,
) -> Result<(), String> {
    let offset = WorldVector {
        x: npc.position.x - world.player.x,
        y: npc.position.y - world.player.y,
    };
    let distance = squared(offset);
    let near = 5625.0 > distance
        || (10000.0 > distance && (20.0 > offset.x.abs() || 20.0 > offset.y.abs()));
    if near
        && npc.state != 9
        && foot_state(world, parent_id)?.cooldown == 0
        && random.bounded(15) == 0
        && room_frame(world)? > 60
    {
        foot_state(world, parent_id)?.cooldown = 80;
        plain_npc::play(npc, "ArmOpen", true);
        npc.state = 9;
        return Ok(());
    }
    if random.bounded(100) == 0 && npc.state == 3 {
        let foot = foot_state(world, parent_id)?;
        if foot.eyes <= 1 {
            plain_npc::play(npc, "Eye", true);
            npc.state = 8;
            foot.eyes += 1;
            return Ok(());
        }
    }
    let count = world.countable_total;
    if count > 7 {
        return Ok(());
    }
    if random.bounded(50) != 0 && (count > 5 || random.bounded(4) != 0) {
        return Ok(());
    }
    if state.cooldown > 0 || foot_state(world, parent_id)?.cooldown != 0 || room_frame(world)? <= 60
    {
        return Ok(());
    }
    let clip = if random.bounded(2) != 0 {
        "Fat02"
    } else {
        "Fat01"
    };
    plain_npc::play(npc, clip, false);
    npc.state = 10;
    Ok(())
}

/// 4d4934: Fat frame 7 drops a Random(12) summon 18 px inside the door,
/// toward the room centre, with SFX 265 louder by 0.05 per index.
fn summon(
    npc: &Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    // Room+1c/+24: the interior bounds; their midpoint is the centre.
    let descriptor = world.grid.descriptor();
    let minimum = WorldVector { x: 60.0, y: 140.0 };
    let maximum = WorldVector {
        x: 60.0 + f32::from(descriptor.width - 2) * 40.0,
        y: 140.0 + f32::from(descriptor.height - 2) * 40.0,
    };
    let centre = minimum
        + WorldVector {
            x: maximum.x - minimum.x,
            y: maximum.y - minimum.y,
        } * 0.5;
    let outward = WorldVector {
        x: npc.position.x - centre.x,
        y: npc.position.y - centre.y,
    };
    let length = squared(outward).sqrt();
    let inset = outward * (18.0 / length);
    let index = random.bounded(12);
    let (entity_type, variant) = match SUMMONS[index as usize] {
        25 => (26, 1),
        kind => (kind, 0),
    };
    sound(world, random, 265, index as f32 * 0.05 + 0.8, 2);
    let spawn_seed = random.next_u32();
    let position = WorldVector {
        x: npc.position.x - inset.x,
        y: npc.position.y - inset.y,
    };
    if !crate::entities::FAMILIES
        .iter()
        .any(|family| family.kinds.contains(&entity_type))
    {
        // Clotty 15 and (Red) Maw 26 have no family yet.
        births.push(Birth::UnportedFactory {
            entity_type,
            variant,
            subtype: 0,
        });
        return Ok(());
    }
    births.push(Birth::Spawned(SpawnedNpc {
        selector: SpawnSelector {
            group_index: 0,
            grid_index: 0,
            category: SpawnCategory::Entity,
            entity_type: entity_type as u16,
            variant: variant as u16,
            subtype: 0,
            position,
            group_seed: spawn_seed,
            spawn_seed,
        },
        velocity: WorldVector { x: 0.0, y: 0.0 },
        spawner_type: 45,
        boss_spawned: false,
        scale: 1.0,
        health_scale: 1.0,
        max_health_scale: 1.0,
        health_delta: 0.0,
        appear: true,
    }));
    Ok(())
}
