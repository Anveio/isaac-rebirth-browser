//! Envy 51.0 and Super Envy 51.1: ai 48c630. A head drifts at a constant
//! speed per axis and bounces off the walls through its grid class 3
//! collider (it passes over rocks). On its dead update a head of size level
//! 0..2 splits: Envy into two heads of the next level, Super Envy into two
//! or three. Runtime variants are `v + 10·level`, spawned from config row
//! `v % 10`.
use crate::{
    content,
    entities::{Birth, Family, NpcWorld, SpawnContext},
    grid::Grid,
    motion::WorldVector,
    mt::Mt19937,
    npcs::Npc,
    phase::Phase,
    plain_npc,
    room_spawns::{SpawnCategory, SpawnSelector},
};

pub(crate) const FAMILY: Family = Family {
    kinds: &[51],
    spawn,
    advance,
    post_contact_grid: walls,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 51
        || selector.variant > 1
        || selector.subtype != 0
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Envy requires bounded authored position and nonzero spawn seed".into());
    }
    let config = content::entity(51, selector.variant.into());
    let mut npc = Npc {
        // Config47f573 SetFrame(default, 0) leaves the default clip stopped.
        animation: "Walk0".into(),
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

fn level(npc: &Npc) -> usize {
    (npc.variant / 10).min(3) as usize
}

fn walk_clip(npc: &Npc) -> &'static str {
    let level = level(npc);
    if npc.variant % 10 == 1 {
        ["Walk0", "Walk2", "Walk3", "Walk4"][level]
    } else {
        ["Walk0", "Walk1", "Walk2", "Walk3"][level]
    }
}

fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    _: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    let level = level(npc);
    if npc.state == 0 {
        // 48c64c INIT: smaller heads keep the sprite but shrink the shadow
        // and divide the HP.
        match level {
            3 => npc.shadow_size *= 0.6,
            2 => npc.shadow_size *= 0.8,
            _ => {}
        }
        let clip = walk_clip(npc);
        plain_npc::play(npc, clip, false);
        match level {
            1 => {
                npc.hp /= 3.0;
                npc.max_hp /= 3.0;
            }
            2 => {
                npc.hp *= 0.25;
                npc.max_hp *= 0.25;
            }
            3 => {
                npc.hp /= 5.0;
                npc.max_hp /= 5.0;
            }
            _ => {}
        }
        npc.grid_collision_class = 3;
        npc.state = 4;
        npc.velocity = crate::npcs::direction(random);
        return Ok(());
    }
    if npc.dead && npc.variant <= 29 {
        // 48c8f0: the split offset, then two children at ±offset whose
        // nested Updates run before Super Envy's third-child roll.
        let offset = crate::npcs::direction(random) * 10.0;
        let variant = npc.variant + 10;
        for position in [npc.position + offset, npc.position + offset * -1.0] {
            births.push(Birth::EnvyChild {
                position,
                variant,
                seed: npc.spawn_seed,
            });
        }
        if npc.variant % 10 == 1 {
            births.push(Birth::EnvyThird {
                position: npc.position + offset * -1.0,
                variant,
                seed: npc.spawn_seed,
                level: level as u32,
            });
        }
    }
    *friction *= 0.9;
    let push = [0.25_f32, f32::from_bits(0x3eb3_3333), f32::from_bits(0x3ee6_6666), f32::from_bits(0x3eb3_3333)][level];
    let dx = if 0.0 > npc.velocity.x { -push } else { push };
    let dy = if 0.0 > npc.velocity.y { -push } else { push };
    npc.velocity = npc.velocity
        + WorldVector {
            x: dx / *friction,
            y: dy / *friction,
        };
    npc.flip_x = 0.0 > npc.velocity.x;
    Ok(())
}

/// A runtime child from config row `v % 10` with the split variant and the
/// parent's spawn seed, APPEAR cleared (48c9d0).
pub(crate) fn child_selector(position: WorldVector, variant: u32, seed: u32) -> SpawnSelector {
    SpawnSelector {
        group_index: 0,
        grid_index: 0,
        category: SpawnCategory::Entity,
        entity_type: 51,
        variant: (variant % 10) as u16,
        subtype: 0,
        position,
        group_seed: seed,
        spawn_seed: seed,
    }
}

/// The class 3 wall collider for 12 points, keyed on the config row.
fn walls(npc: &mut Npc, grid: &Grid) -> Result<(), String> {
    let points = content::entity(51, npc.variant % 10).grid_points;
    if npc.grid_collision_class != 3 || points == 0 {
        return crate::entities::clamp_walls(npc, grid);
    }
    let collider = crate::grid::Collider::new(crate::grid::CollisionGeometry {
        radius: npc.collision_radius,
        scale: WorldVector { x: 1.0, y: 1.0 },
        points,
    })?;
    npc.grid_collided = collider
        .resolve_nonplayer_walls(grid, &mut npc.position, &mut npc.velocity, false)?
        .is_some();
    npc.presented_position = npc.position;
    Ok(())
}
