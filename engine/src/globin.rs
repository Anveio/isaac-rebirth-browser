//! Globin 24.0 and Gazing Globin 24.1: ai 484190. A Globin chases the
//! player with FindGridPath, groans while it sees the player and now and then
//! bleeds a red splat. Killed while walking, it clears its own death, comes
//! back with 10 HP and plays ReGen as a heap (a Gazing Globin's heap creeps
//! toward a near player), then walks again; killed as a heap, it dies. The AI
//! never draws the NPC's own RNG.
use crate::{
    audio::SoundRequest,
    content,
    entities::{Birth, Family, NpcWorld, SpawnContext},
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    pathfinder::{self, Pathfinder},
    phase::Phase,
    plain_npc,
    room_spawns::{SpawnCategory, SpawnSelector},
};

pub(crate) const FAMILY: Family = Family {
    kinds: &[24],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    // Dank Globin 24.2 is Afterbirth content.
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 24
        || selector.variant > 1
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Globin requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(24, selector.variant.into());
    let mut npc = Npc {
        // Config47f573 SetFrame(default, 0) leaves the default clip stopped.
        animation: if selector.variant == 1 {
            "WalkHori"
        } else {
            "WalkVert"
        }
        .into(),
        actor: config.actor.into(),
        pathfinder: Some(Pathfinder::new()),
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
    _: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    match npc.state {
        0 => {
            // 484255: straight to walking.
            npc.state = 4;
            Ok(())
        }
        4 => walk(npc, random, world, friction),
        _ => {
            heap(npc, world, friction);
            Ok(())
        }
    }
}

/// 484270: walking, which also runs on the update that kills it.
fn walk(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    friction: &mut f32,
) -> Result<(), String> {
    if random.bounded(10) == 0 {
        // 484530: 480fe0's red Blood7 splat scaled 0.2..0.7, no SetColor.
        let scale = random.uniform() * 0.5 + 0.2;
        let seed = random.next_u32();
        let id = world
            .effects
            .admit_blood(seed, npc.position, [1.0; 4], [0.0; 3])?;
        world.effects.set_blood_scale(id, scale)?;
        world.effects.advance_blood_initial(id, random)?;
    }
    // 4807e0 reads the velocity the previous update left.
    plain_npc::walk_frame(npc, "WalkHori", "WalkVert", 0.1);
    // PORT GAP: fear (+319 & 8) evades the player instead; the port admits
    // no fear status.
    let speed = (npc.variant as f32 + 1.0) * 0.35 + 0.4;
    pathfinder::find_grid_path(npc, random, world, world.player, speed, 0, friction)?;
    let direct = npc.pathfinder.as_ref().is_some_and(|path| path.direct);
    // 484354: bounded(3) on every update with pf.direct.
    if direct && random.bounded(3) == 0 {
        world.audio.admit(
            random,
            world.system_counter,
            SoundRequest {
                id: 165,
                volume: 1.0,
                delay: 40,
                looping: false,
                pitch: 1.0,
            },
        );
    }
    *friction *= 0.97;
    npc.flip_x = 0.0 > npc.velocity.x;
    if npc.dead {
        // 484397: the death is undone; ReGen plays as a 10 HP heap.
        npc.dead = false;
        npc.hp = 10.0;
        plain_npc::play(npc, "ReGen", false);
        npc.state = 3;
    }
    Ok(())
}

/// 4841b2: the heap. A Gazing Globin's heap pushes toward a player within
/// 200 px; once ReGen finishes the Globin walks again.
fn heap(npc: &mut Npc, world: &NpcWorld, friction: &mut f32) {
    if npc.variant == 1 {
        let delta = WorldVector {
            x: world.player.x - npc.position.x,
            y: world.player.y - npc.position.y,
        };
        let distance = (delta.x * delta.x + delta.y * delta.y).sqrt();
        if 200.0 > distance {
            let push = delta * (0.5 / distance);
            npc.velocity = npc.velocity
                + WorldVector {
                    x: push.x / *friction,
                    y: push.y / *friction,
                };
        }
        *friction *= 0.83;
    } else {
        *friction *= 0.7;
    }
    if plain_npc::finished(npc) && npc.animation == "ReGen" {
        npc.state = 4;
    }
}
