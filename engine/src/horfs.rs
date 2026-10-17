//! Horf12 and Psychic Horf248: ai_horf4bfdb0 on the plain NPC base.
use crate::{
    audio::SoundRequest,
    content,
    entities::{Birth, Family, NpcWorld, SpawnContext},
    mt::Mt19937,
    npcs::Npc,
    phase::Phase,
    plain_npc::{self, ShooterTimers, ShotMode, ShotParameters},
    room_spawns::{SpawnCategory, SpawnSelector},
};

pub(crate) const FAMILY: Family = Family {
    kinds: &[12],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

fn spawn(id: u32, selector: &SpawnSelector, _: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 12
        || selector.variant != 0
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Horf requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(12, 0);
    Ok(Some(Npc {
        // Config47f573 SetFrame(default,0) leaves the default clip stopped.
        animation: "Shake".into(),
        actor: config.actor.into(),
        shooter: Some(ShooterTimers::default()),
        ..Npc::authored(id, config, selector)
    }))
}

fn advance(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    //4FA3EB dispatches Horf AI with31F set before the next-Full retirement.
    //Keep the living callback's existing no-APPEAR behavior.
    if npc.dead {
        return plain_npc::advance_dead_ai(npc, phase, random, world, births, ai);
    }
    plain_npc::advance(npc, phase, random, world, births, ai)
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    *friction *= 0.7;
    let parameter_angle = (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
    let shot = ShotParameters {
        mode: ShotMode::Single,
        range: 200.0,
        cooldown: 4,
        delay: 7,
        speed: 7.0,
        parameter_angle,
        lead: 0.0,
        adjust: None,
    };
    if plain_npc::try_shoot_player(npc, random, world, births, shot)? {
        plain_npc::play(npc, "Attack", true);
    }
    if npc.shooter.as_ref().is_some_and(|timers| timers.delay == 0) {
        world.audio.admit(
            random,
            world.system_counter,
            SoundRequest {
                id: 146,
                volume: 1.0,
                delay: 2,
                looping: false,
                pitch: 1.0,
            },
        );
    }
    if plain_npc::finished(npc) {
        plain_npc::play(npc, "Shake", false);
    }
    Ok(())
}
