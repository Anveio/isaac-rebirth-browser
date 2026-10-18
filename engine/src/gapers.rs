//! Frowning Gaper 10.0 and Gaper 10.1: ai 4e9330 (shared with Fatty 208 and
//! Nulls 252, not admitted). The Gaper chases the player with FindGridPath,
//! groans while it sees the player, and the Frowning Gaper's head overlay
//! opens once on first sight and speeds it up. A dead Gaper may Morph into a
//! Gusher or Pacer (`gushers`).
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
    kinds: &[10],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 10
        || selector.variant > 1
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Gaper requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(10, selector.variant.into());
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
    plain_npc::advance_dead_ai(npc, phase, random, world, births, ai)?;
    if phase == Phase::Interpolate && !npc.removed && npc.grid_collision_class == 5 {
        // Base Fast444746 calls442b90(true): correct only the preview,
        // preserving the full position, velocity and prior grid feedback.
        let collider = crate::grid::Collider::new(crate::grid::CollisionGeometry {
            radius: npc.collision_radius,
            scale: crate::motion::WorldVector { x: 1.0, y: 1.0 },
            points: npc.config().grid_points,
        })?;
        let mut position = world.grid.clamp_room(npc.presented_position, 0.0);
        let mut velocity = npc.velocity;
        collider.resolve_nonplayer_ground(world.grid, &mut position, &mut velocity, true)?;
        npc.presented_position = position;
    }
    Ok(())
}

/// GetOverlayFrame 410430: the overlay's frame, or -1 without one.
fn overlay_frame(npc: &Npc) -> i32 {
    npc.overlay
        .as_ref()
        .map_or(-1, |overlay| overlay.frame as i32)
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    _: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.dead {
        return death_morph(npc, random);
    }
    if npc.state == 0 {
        // 4e934c: SetOverlayFrame("Head", 0) and SetFrame("WalkVert", 0).
        npc.state = 4;
        npc.overlay = Some(Overlay {
            animation: "Head".into(),
            frame: 0,
            clock: 0.0,
            playing: false,
        });
        plain_npc::set_frame(npc, "WalkVert", 0);
        return Ok(());
    }
    // 4807e0 reads the velocity the previous update left.
    plain_npc::walk_frame(npc, "WalkHori", "WalkVert", 0.1);
    let speed = if npc.variant == 1 {
        f32::from_bits(0x3efe_f9db)
    } else if overlay_frame(npc) > 0 {
        0.6_f32 * 1.2
    } else {
        0.6
    };
    let target = world.player;
    pathfinder::find_grid_path(npc, random, world, target, speed, 900, friction)?;
    let direct = npc.pathfinder.as_ref().is_some_and(|path| path.direct);
    // 4e95a8: bounded(3) on every update with pf.direct.
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
    if npc.variant == 0 && direct && overlay_frame(npc) <= 0 {
        // 4e95b5 PlayOverlay("Head", force).
        npc.overlay = Some(Overlay {
            animation: "Head".into(),
            frame: 0,
            clock: 0.0,
            playing: true,
        });
    }
    Ok(())
}

/// 4e9418: unless overkilled by 25, half the dead Gapers Morph into a Pacer
/// 11.1 (1/3) or a Gusher 11.0 (2/3) at full HP.
fn death_morph(npc: &mut Npc, random: &mut Mt19937) -> Result<(), String> {
    if npc.hp <= -25.0 || random.bounded(2) != 0 {
        return Ok(());
    }
    let variant = u32::from(random.bounded(3) == 0);
    npc.morph(11, variant, "WalkHori", random)?;
    npc.dead = false;
    npc.hp = npc.max_hp;
    Ok(())
}
