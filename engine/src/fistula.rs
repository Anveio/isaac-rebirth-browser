//! Small Fistula 73.0: ai 488b80's type-73 path (shared with the bigger
//! Fistulas 71/72 and the Teratomas, not admitted). It drifts ever faster in
//! the direction it already moves on each axis, bouncing off walls and doors
//! only (grid class 3), and dies into a Charger 23.0 at three quarters HP.
use crate::{
    audio::SoundRequest,
    content,
    entities::{Birth, Family, NpcWorld, SpawnContext, SpawnedNpc},
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    phase::Phase,
    plain_npc,
    room_spawns::{SpawnCategory, SpawnSelector},
};

pub(crate) const FAMILY: Family = Family {
    kinds: &[73],
    spawn,
    advance,
    post_contact_grid: crate::entities::wall_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    // Teratoma 73.1 (Spider births) and subtype 1 (Boom Fly) stay out.
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 73
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err(
            "Small Fistula requires bounded authored position and nonzero spawn seed".into(),
        );
    }
    let config = content::entity(73, 0);
    let mut npc = Npc {
        // Config47f573 SetFrame(default "Small", 0); the AI never plays a clip.
        animation: "Small".into(),
        actor: config.actor.into(),
        ..Npc::authored(id, config, selector)
    };
    npc.max_hp = content::initial_hp(config, context.stage);
    npc.hp = npc.max_hp;
    Ok(Some(npc))
}

fn advance(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    plain_npc::advance_dead_ai(npc, phase, random, world, births, ai)
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.state == 0 {
        // 488bb2: grid class 3 (walls); no clip, no draws.
        npc.state = 4;
        npc.grid_collision_class = 3;
        return Ok(());
    }
    // 488bd8: any other state, the dying update included.
    *friction *= 0.9;
    // PORT GAP: confusion wanders and fear adds a push away from the
    // player; the port admits neither status.
    let push = 0.56 / *friction;
    let step = |value: f32| {
        if value > 0.0 {
            value + push
        } else {
            value - push
        }
    };
    npc.velocity = WorldVector {
        x: step(npc.velocity.x),
        y: step(npc.velocity.y),
    };
    if !npc.dead {
        return Ok(());
    }
    // 488cf1: the split angle is drawn for every size, unused by the small.
    random.uniform();
    world.audio.admit(
        random,
        world.system_counter,
        SoundRequest {
            id: 265,
            volume: 0.8,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
    let costs = world
        .path_costs
        .as_deref()
        .ok_or("Small Fistula death requires source Room.Init path costs")?;
    let pit = match world.grid.index(npc.position) {
        Some(index) => costs.get(index)? == 3000,
        None => false,
    };
    if pit || world.grid.collision_at(npc.position) != 0 {
        return Ok(());
    }
    let spawn_seed = random.next_u32();
    births.push(Birth::Spawned(SpawnedNpc {
        selector: SpawnSelector {
            group_index: 0,
            grid_index: 0,
            category: SpawnCategory::Entity,
            entity_type: 23,
            variant: 0,
            subtype: 0,
            position: npc.position,
            group_seed: spawn_seed,
            spawn_seed,
        },
        velocity: WorldVector { x: 0.0, y: 0.0 },
        spawner_type: 0,
        boss_spawned: false,
        scale: 1.0,
        // 488e30: HP x0.75, max HP untouched.
        health_scale: 0.75,
        max_health_scale: 1.0,
        health_delta: 0.0,
        appear: false,
    }));
    Ok(())
}
