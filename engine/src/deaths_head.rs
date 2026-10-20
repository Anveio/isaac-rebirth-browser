//! Death's Head 212.0: ai 493aa0. An invincible skull that bounces
//! diagonally off walls and doors only (grid class 3) at a steady push, and
//! dies once no countable enemy is left in the room. The AI never draws the
//! NPC's own RNG.
use crate::{
    audio::SoundRequest,
    content,
    entities::{Birth, Family, NpcWorld, SpawnContext},
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    phase::Phase,
    plain_npc,
    room_spawns::{SpawnCategory, SpawnSelector},
    sprite_color::Color,
};

pub(crate) const FAMILY: Family = Family {
    kinds: &[212],
    spawn,
    advance,
    post_contact_grid: crate::entities::wall_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 212
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Death's Head requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(212, 0);
    let mut npc = Npc {
        animation: "Idle".into(),
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
    // NPC::Update diverts state 16 to the dying handler before the AI,
    // alive or dead (4fa56d): Death plays and the skull is removed.
    if npc.state == 16 && phase == Phase::Update && !npc.removed {
        return plain_npc::dying(npc, npc.base_friction);
    }
    plain_npc::advance_dead_ai(npc, phase, random, world, births, ai)
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    _births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.state == 0 {
        // 493be8: invincible, walls only, a random heading and a dark red
        // splat. PORT GAP: the +318 bit 0 it also sets is not admitted.
        npc.state = 4;
        npc.grid_collision_class = 3;
        npc.invincible = true;
        npc.velocity = crate::npcs::direction(random);
        npc.splat_color = Color {
            tint: [0.15, 0.0, 0.0, 1.0],
            offset: [0.1, 0.1, 0.1],
            ..Color::WHITE
        };
        return Ok(());
    }
    *friction *= 0.9;
    if npc.state == 4 {
        plain_npc::play(npc, "Idle", false);
    }
    if world.countable_total <= 0 && npc.state != 16 {
        // 493af6: no enemy left; SFX 316 and the dying state.
        npc.state = 16;
        npc.death_frame = 0;
        world.audio.admit(
            random,
            world.system_counter,
            SoundRequest {
                id: 316,
                volume: 1.0,
                delay: 60,
                looping: false,
                pitch: 1.0,
            },
        );
    }
    // PORT GAP: confusion (+319 & 2) wanders on the pathfinder instead.
    // 493b43: a 0.45 push along the signs of the velocity.
    let push = WorldVector {
        x: if 0.0 > npc.velocity.x { -0.45 } else { 0.45 },
        y: if 0.0 > npc.velocity.y { -0.45 } else { 0.45 },
    };
    npc.velocity = npc.velocity
        + WorldVector {
            x: push.x / *friction,
            y: push.y / *friction,
        };
    Ok(())
}
