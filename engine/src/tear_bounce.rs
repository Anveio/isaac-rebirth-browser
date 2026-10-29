//! Bouncing tears (flag 0x80000, Rubber Cement 221).
//!
//! Tear.Update 542C78 runs when the tear is still airborne: with the bounce
//! flag it sets grid collision class 6 (542C8A) instead of keeping class 4,
//! and when the previous Full grid pass set +34C it takes 543480 in place of
//! 542F70's grid death. 543480 hurts the grid cell under each collision
//! point of position - velocity (5CBB30, damage 1 + (+48C > 10), stopping at
//! the first one that takes it), rescales the reflected velocity to the
//! length of the pre-collision velocity +350 (612820, 612790), clears the
//! 808/818 hit registry and goes on with the ordinary update.
//!
//! Class 6 is resolved in the Full grid pass 454B10 (442B90 -> 442C3B room
//! clamp -> 4434B0 samples): collision > 1 is solid, the normal component
//! is reflected, and only an inward velocity sets +34C and copies the old
//! velocity to +350 (443628). Interpolate calls the same query as preview.
//! The tear's collision points are SetSize 446840 from 5418E0: radius
//! config +24 times scale +7E4, sprite scale +478 (1, 1), config points +34.
use crate::{
    attack_grid, content,
    grid::{Collider, CollisionGeometry, Grid},
    knives::{normalized, squared_length},
    motion::WorldVector,
    projectiles::Lifetime,
    State, Tear,
};

pub(crate) const BOUNCE: u64 = 0x8_0000;

pub(crate) fn bounces(tear: &Tear) -> bool {
    tear.controlled.is_none() && tear.attack_flags.0 & BOUNCE != 0
}

fn collider(tear: &Tear) -> Result<Collider, String> {
    let variant = tear
        .source_variant
        .ok_or("bouncing tear has no source variant for its collision points")?;
    let config = content::ENTITIES
        .iter()
        .find(|config| config.id == 2 && config.variant == variant)
        .ok_or("bouncing tear variant is not authored in entities2.xml")?;
    Collider::new(CollisionGeometry {
        radius: tear.collision_radius,
        scale: WorldVector { x: 1.0, y: 1.0 },
        points: config.grid_points,
    })
}

/// 543480: the bounce taken at the start of Tear.Update when +34C is set.
/// Returns true when the bounce ran (542F70's grid death is then skipped).
pub(crate) fn update(state: &mut State, grid: Option<&Grid>, id: u32) -> Result<bool, String> {
    let Some(tear) = state.tears.iter().find(|tear| {
        tear.id == id
            && bounces(tear)
            && tear.motion.lifetime == Lifetime::Alive
            && tear.grid_contact
            && tear.motion.height <= -5.0
    }) else {
        return Ok(false);
    };
    let previous = tear
        .grid_previous_velocity
        .ok_or("bouncing tear +34C is set without its +350 velocity")?;
    let (position, velocity) = (tear.motion.position, tear.motion.velocity);
    let damage = 1 + i32::from(tear.collision_damage > 10.0);
    let samples = collider(tear)?.samples().to_vec();
    if let Some(grid) = grid {
        for sample in samples {
            let point = WorldVector {
                x: position.x - velocity.x,
                y: position.y - velocity.y,
            } + sample;
            let Some(index) = grid.index(point) else {
                continue;
            };
            let outcome = attack_grid::callback_at_index(state, index, damage)?;
            outcome.record_frontier(state, "tearBounceGrid");
            if outcome.taken() == Some(true) {
                break;
            }
        }
    }
    let tear = state
        .tears
        .iter_mut()
        .find(|tear| tear.id == id)
        .ok_or("bouncing tear vanished")?;
    tear.motion.velocity = normalized(tear.motion.velocity) * squared_length(previous).sqrt();
    if !tear.motion.velocity.is_finite() {
        return Err("bouncing tear velocity overflowed".into());
    }
    if let Some(registry) = tear.persistent_contacts.as_mut() {
        registry.hit_sequences.clear();
    }
    Ok(true)
}

/// 454B10 class 6 for every live bouncing tear, newborns included.
pub(crate) fn full_grid_contacts(state: &mut State, grid: &Grid) -> Result<(), String> {
    for tear in &mut state.tears {
        if !bounces(tear) || tear.motion.lifetime == Lifetime::Removed {
            continue;
        }
        let collider = collider(tear)?;
        let mut position = grid.clamp_room(tear.motion.position, 0.0);
        let mut velocity = tear.motion.velocity;
        let contact =
            collider.resolve_nonplayer_no_pits(grid, &mut position, &mut velocity, false)?;
        tear.motion.position = position;
        tear.motion.presented_position = position;
        tear.motion.velocity = velocity;
        tear.grid_contact = false;
        if let Some(contact) = contact.filter(|contact| contact.velocity_changed) {
            tear.grid_contact = true;
            tear.grid_previous_velocity = Some(contact.previous_velocity);
        }
        tear.project();
    }
    Ok(())
}

/// Interpolate's preview query: position correction only.
pub(crate) fn preview(tear: &mut Tear, grid: &Grid) -> Result<(), String> {
    if !bounces(tear) || tear.motion.lifetime == Lifetime::Removed {
        return Ok(());
    }
    let collider = collider(tear)?;
    let mut position = grid.clamp_room(tear.motion.presented_position, 0.0);
    let mut velocity = tear.motion.velocity;
    collider.resolve_nonplayer_no_pits(grid, &mut position, &mut velocity, true)?;
    tear.motion.presented_position = position;
    Ok(())
}
