//! Gusher 11.0 and Pacer 11.1: ai 4e8890 (shared with Blubber 210, Half Sack
//! 211 and Splasher 238, not admitted). Both wander the room and trail blood
//! splats on the global ten-frame beat; the Gusher's neck fountains (Blood
//! overlay) and it spits along its own velocity. Authored, or Morphed from a
//! dead Gaper (`gapers`). The type-11 path has no dead test, so the dead-AI
//! update runs the whole body once more.
use crate::{
    audio::SoundRequest,
    content,
    enemy_projectiles::ProjectileBirth,
    entities::{Birth, Family, NpcWorld, SpawnContext},
    motion::WorldVector,
    mt::Mt19937,
    npcs::{Npc, Overlay},
    pathfinder::{self, Pathfinder},
    phase::Phase,
    plain_npc::{self, ShooterTimers},
    room_spawns::{SpawnCategory, SpawnSelector},
};

pub(crate) const FAMILY: Family = Family {
    kinds: &[11],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 11
        || selector.variant > 1
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Gusher requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(11, selector.variant.into());
    let mut npc = Npc {
        // Config47f573 SetFrame(default, 0) leaves the default clip stopped.
        animation: "WalkHori".into(),
        actor: config.actor.into(),
        pathfinder: Some(Pathfinder::new()),
        shooter: Some(ShooterTimers::default()),
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
        // 4e889e: INIT only chooses state 4.
        npc.state = 4;
        return Ok(());
    }
    plain_npc::walk_frame(npc, "WalkHori", "WalkVert", 0.1);
    if npc.variant == 0 {
        gush(npc, random, world, births);
    }
    *friction *= 0.75;
    pathfinder::wander(npc, random, world, *friction)?;
    // 4e89ca is_frame(self, 10, 0): the global beat, without the actor's
    // manager sequence.
    if world.game_counter % 10 == 0 {
        let scale = random.uniform() * 0.3 + 0.2;
        let seed = random.next_u32();
        let id = world.effects.admit_blood(seed, npc.position, [1.0; 4], [0.0; 3])?;
        world.effects.set_blood_scale(id, scale)?;
        world.effects.advance_blood_initial(id, random)?;
    }
    Ok(())
}

/// 4e8c38: the Blood overlay, and a mode-0 shot along the Gusher's own
/// velocity every 41..60 moving updates. +f68 survives a Gaper's Morph.
fn gush(npc: &mut Npc, random: &mut Mt19937, world: &mut NpcWorld, births: &mut Vec<Birth>) {
    // PlayOverlay("Blood", unforced): the looping clip starts once.
    if !npc.overlay.as_ref().is_some_and(|overlay| overlay.animation == "Blood" && overlay.playing) {
        npc.overlay = Some(Overlay {
            animation: "Blood".into(),
            frame: 0,
            clock: 0.0,
            playing: true,
        });
    }
    let velocity = npc.velocity;
    let timers = npc.shooter.get_or_insert_with(ShooterTimers::default);
    if timers.cooldown > 0 {
        timers.cooldown -= 1;
        return;
    }
    let length = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
    if length <= 0.0 {
        return;
    }
    let direction = WorldVector {
        x: velocity.x / length,
        y: velocity.y / length,
    };
    timers.cooldown = random.bounded(20) as i32 + 40;
    let parameter_angle = (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
    let spawn_seed = random.next_u32();
    let initial_falling_speed = (random.uniform() - 0.5) * 0.2;
    // Mode0's post loop stores -3/0.1, then adds params +8 -5 and +c 0.3.
    births.push(Birth::Projectile(ProjectileBirth {
        position: npc.position,
        velocity: direction * 5.0,
        spawn_seed,
        initial_falling_speed,
        parameter_angle,
        falling_speed: -3.0_f32 + -5.0,
        falling_acceleration: 0.1_f32 + 0.3,
        scale: 1.0,
        height: -23.0,
    }));
    world.audio.admit(
        random,
        world.system_counter,
        SoundRequest {
            id: 178,
            volume: 1.0,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
}
