//! Eternal Fly96.0: ai_eternalfly483340. An invincible fly that circles its
//! parent, or its own spawn point, and turns into an Attack Fly18 when the
//! parent dies or the poop it sits on is broken.
use crate::{
    content,
    entities::{Birth, Family, NpcWorld, SpawnContext},
    fmath,
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    phase::Phase,
    plain_npc,
    room_spawns::{SpawnCategory, SpawnSelector},
};
use serde::{Deserialize, Serialize};

pub(crate) const FAMILY: Family = Family {
    kinds: &[96],
    spawn,
    advance,
    post_contact_grid: crate::entities::clamp_walls,
    entity_flags,
    occupancy: crate::entities::Occupancy::None,
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EternalFlyState {
    /// Entity_NPC+f58: Random(10) phase slot on the circle.
    pub orbit: u32,
    /// Entity+42c: Game+ce9f4 at Init (441d34).
    pub init_counter: u32,
    /// +318 after state0: bit 1 set, APPEAR cleared.
    pub entity_flags: u32,
}

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 96
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Eternal Fly requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(96, 0);
    let mut npc = Npc {
        // Config47f573 SetFrame(default "Fly", 0) leaves the clip stopped.
        animation: "Fly".into(),
        actor: config.actor.into(),
        eternal_fly: Some(EternalFlyState {
            orbit: 0,
            init_counter: context.game_update_count,
            entity_flags: 4,
        }),
        ..Npc::authored(id, config, selector)
    };
    npc.max_hp = content::initial_hp(config, context.stage);
    npc.hp = npc.max_hp;
    Ok(Some(npc))
}

/// The Eternal Fly a corn poop's Init spawns (`GridEntity::corn_fly`):
/// Spawn550a40 with no spawner, then the poop writes its state0 (+f50), so
/// its first update initializes it without the Appear hide.
pub(crate) fn from_corn_poop(
    id: u32,
    fly: &crate::grid_records::CornFly,
    context: &SpawnContext,
) -> Result<Npc, String> {
    let selector = SpawnSelector {
        group_index: 0,
        grid_index: 0,
        category: SpawnCategory::Entity,
        entity_type: 96,
        variant: 0,
        subtype: 0,
        position: fly.position,
        group_seed: fly.seed,
        spawn_seed: fly.seed,
    };
    let mut npc = spawn(id, &selector, context)?.ok_or("Eternal Fly spawn refused its selector")?;
    npc.state = 0;
    Ok(npc)
}

fn entity_flags(npc: &Npc) -> u32 {
    npc.eternal_fly
        .as_ref()
        .map_or(4, |state| state.entity_flags)
}

fn advance(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    // State0 clears APPEAR, so Entity_NPC::Update skips the hide/Poof and runs
    // the AI again in the same age-0 update.
    plain_npc::advance_with(npc, phase, random, world, births, ai, false)
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    _: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    let mut state = npc
        .eternal_fly
        .take()
        .ok_or("Eternal Fly requires its AI state")?;
    let result = orbit(npc, &mut state, random, world, friction);
    if npc.kind == 96 {
        npc.eternal_fly = Some(state);
    }
    result
}

fn orbit(
    npc: &mut Npc,
    state: &mut EternalFlyState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.state == 0 {
        // 483620: grid class0, entity class2 (player and his objects only),
        // invincible, a random circle slot, and the spawn point as centre.
        state.entity_flags = (state.entity_flags | 1) & !4;
        npc.entity_collision_class = 2;
        npc.grid_collision_class = 0;
        npc.invincible = true;
        npc.state = 4;
        state.orbit = random.bounded(10);
        npc.target = npc.position;
    }
    plain_npc::play(npc, "Fly", false);
    // The parent handle (+498) is nulled when the parent is purged.
    let parent = npc
        .parent
        .and_then(|id| world.peers.get(id))
        .map(|parent| (parent.kind, parent.position, parent.dead));
    let morph = match parent {
        Some((_, position, dead)) => {
            npc.target = position;
            dead
        }
        // 483710: with no parent, an empty cell under the centre or a broken
        // poop there (grid class0) releases it; outside the grid it stays.
        None => world.grid.index(npc.target).is_some_and(|index| {
            world.grid.descriptor().cells[index]
                .entity
                .as_ref()
                .is_none_or(|entity| entity.kind == 14 && entity.collision_class == 0)
        }),
    };
    let parent_kind = parent.map(|(kind, _, _)| kind);
    let wide = parent_kind.is_none_or(|kind| kind == 90);
    if morph {
        // 4836ba: grid class3, then Morph480750 to an Attack Fly.
        npc.grid_collision_class = 3;
        npc.morph(18, 0, "Fly", random)?;
        npc.invincible = false;
        return Ok(());
    }
    let (radius, speed) = if wide {
        (WorldVector { x: 60.0, y: 50.0 }, 0.02_f32)
    } else {
        (WorldVector { x: 25.0, y: 20.0 }, 0.04_f32)
    };
    let pi = std::f64::consts::PI;
    let mut angle = (f64::from(world.game_counter as i32 as f32 * speed) * pi
        + f64::from(state.orbit) * pi / 10.0) as f32;
    if state.orbit & 1 == 0 {
        angle = -angle;
    }
    let direction = WorldVector {
        x: fmath::cos(angle),
        y: fmath::sin(angle),
    };
    // 442b30: frames since Init, ramped over 90 (30 around a type-90 parent).
    let period: i32 = if parent_kind == Some(90) { 30 } else { 90 };
    let age = world.game_counter.wrapping_sub(state.init_counter) as i32;
    let ramp = age.min(period) as f32 / period as f32;
    let pull = if parent_kind == Some(102) { 1.0 } else { 0.5 };
    let offset = WorldVector {
        x: direction.x * radius.x,
        y: direction.y * radius.y,
    } * ramp;
    let goal = npc.target + offset;
    let step = WorldVector {
        x: goal.x - npc.position.x,
        y: goal.y - npc.position.y,
    } * pull;
    npc.position = npc.position + step;
    *friction *= 0.9;
    Ok(())
}
