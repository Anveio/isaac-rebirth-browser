//! Skinny 226.0: ai 4b7f40's type-226 path (shared with Rotty 226.1 and
//! Bony 227, not admitted). A Skinny walks at the player with FindGridPath
//! under its looping Head overlay, moans now and then, and every seventh
//! frame leaves a small red creep pool. It never draws its own RNG.
use crate::{
    audio::SoundRequest,
    content,
    entities::{Birth, Family, NpcWorld, SpawnContext},
    mt::Mt19937,
    npcs::{Npc, Overlay},
    pathfinder::{self, Pathfinder},
    phase::Phase,
    plain_npc,
    room_spawns::{SpawnCategory, SpawnSelector},
};

pub(crate) const FAMILY: Family = Family {
    kinds: &[226],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    // Rotty 226.1 first appears in Depths; it and its Bony morph stay out.
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 226
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Skinny requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(226, 0);
    let mut npc = Npc {
        // Config47f573 SetFrame(default, 0) leaves the default clip stopped.
        animation: "WalkHori".into(),
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

/// The walk body also runs on the dead-AI update (4b7f40 has no dead test for
/// 226.0), so a dying Skinny may still moan and leave a last pool.
fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    _: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.state == 0 {
        // 4b7f63: state 4 and PlayOverlay("Head"); no tail.
        npc.state = 4;
        npc.overlay = Some(Overlay {
            animation: "Head".into(),
            frame: 0,
            clock: 0.0,
            playing: true,
        });
        return Ok(());
    }
    if npc.state == 4 {
        if plain_npc::on_frame(npc, world, 20)? && random.bounded(3) == 0 {
            world.audio.admit(
                random,
                world.system_counter,
                SoundRequest {
                    id: 167,
                    volume: 0.8,
                    delay: 120,
                    looping: false,
                    pitch: 1.0,
                },
            );
        }
        pathfinder::find_grid_path(npc, random, world, world.player, 0.5, 900, friction)?;
        // After the path: the walk clip follows this update's velocity.
        plain_npc::walk_frame(npc, "WalkHori", "WalkVert", 0.1);
        if plain_npc::on_frame(npc, world, 7)? {
            // 4b858b: a spawnerless 1000.22 at scale 0.1, size 0.5, 30/30,
            // first updated by the scheduler.
            let seed = random.next_u32();
            world
                .effects
                .admit_sized_creep(seed, npc.position, 0.1, 0.5, (30, 30))?;
        }
    }
    *friction *= 0.8;
    Ok(())
}
