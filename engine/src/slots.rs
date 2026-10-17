//! Entity_Slot (type 6, vtable 671130): machines and beggars.
//!
//! Init 53a460 and the idle Update 53c450 draw nothing from the shared MT.
//! Idle slots exist, render their Idle clip and push back through the
//! ordinary mass response. Payouts, damage and every partner callback that
//! would reach a resource gate, TakeDamage or a pair this owner has not
//! admitted are explicit errors, never a silent pass-through.
use crate::{
    actors::{ActorKind, ActorRef},
    combat::{self, ContactBody},
    motion::WorldVector,
    phase::Phase,
    State,
};
use serde::{Deserialize, Serialize};

/// Neither Player nor Entity_Slot writes sprite rotation +2cc.
const UNROTATED: f32 = 0.0;
const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Slot {
    pub id: u32,
    pub variant: u32,
    pub subtype: u32,
    ///Base Init441c80 seeds the own RNG+7c0 from it; idle never draws it.
    pub spawn_seed: u32,
    pub position: WorldVector,
    pub presented_position: WorldVector,
    pub velocity: WorldVector,
    ///+440 TargetPosition. Init clears it; the first Update anchors it.
    pub target_position: WorldVector,
    ///+458 PositionOffset, a render offset only (42d9d0 ignores it).
    pub position_offset: WorldVector,
    ///+478 SizeMulti. Anything but (1,1) selects the capsule pair 4454a0.
    pub size_multi: WorldVector,
    pub collision_radius: f32,
    pub collision_mass: f32,
    ///+46c configured and +470 current friction.
    pub configured_friction: f32,
    pub current_friction: f32,
    ///+7d4. Init writes1 (idle); 0 and 2..5 belong to unadmitted branches.
    pub state: u32,
    pub actor: String,
    pub animation: String,
    pub animation_frame: u32,
}

impl Slot {
    /// Room SpawnEntity5ce6b0: spawnConfig5c5468 forces variant3 (Fortune
    /// Teller) in a super secret room (type8), drawing nothing for type6.
    /// Game550a40 then calls Init before writing position and zero velocity.
    pub(crate) fn from_selector(
        id: u32,
        selector: &crate::room_spawns::SpawnSelector,
        room_type: u32,
    ) -> Result<Option<Self>, String> {
        if selector.entity_type != 6 {
            return Ok(None);
        }
        let variant = if room_type == 8 {
            3
        } else {
            u32::from(selector.variant)
        };
        //53a77d: Donation Machine Init reads Game+10eae0 0x20000/0x40000 and
        //Game+10ecd0 (Remove, or a shared MT.bounded(4) CoinJam clip) and the
        //Persistent+130 donation counter for its digit layers. None has an
        //owner here, so 6.8 stays an unadmitted factory. Variants above 8 log
        //670f48 "Slot type %d is not implemented!" in the original.
        // 666134/66ad10/668978: per-variant PositionOffset y and SizeMulti.
        let (offset_y, size_multi) = match variant {
            1..=3 => (-8.0, (1.5, 0.75)),
            4 | 5 | 7 => (8.0, (1.0, 1.0)),
            6 => (-8.0, (3.0, 0.75)),
            _ => return Ok(None),
        };
        let config = crate::content::entity(6, variant);
        if selector.spawn_seed == 0 || !selector.position.is_finite() {
            return Err("Slot Init requires an actual seed and finite position".into());
        }
        Ok(Some(Self {
            id,
            variant,
            subtype: u32::from(selector.subtype),
            spawn_seed: selector.spawn_seed,
            position: selector.position,
            presented_position: selector.position,
            velocity: ZERO,
            target_position: ZERO,
            // Blood Donation alone shifts its sprite left by5.
            position_offset: WorldVector {
                x: if variant == 2 { -5.0 } else { 0.0 },
                y: offset_y,
            },
            size_multi: WorldVector {
                x: size_multi.0,
                y: size_multi.1,
            },
            // entities2.xml: radius12, mass3, friction1 for every variant.
            collision_radius: config.collision_radius,
            collision_mass: config.collision_mass,
            configured_friction: config.friction,
            current_friction: config.friction,
            state: 1,
            actor: config.actor.into(),
            // Init plays the anm2 default clip, Idle in all eight.
            animation: "Idle".into(),
            animation_frame: 0,
        }))
    }

    fn body(&self) -> ContactBody {
        ContactBody {
            id: self.id,
            entity_type: 6,
            position: self.position,
            velocity: self.velocity,
            radius: self.collision_radius,
            mass: self.collision_mass,
        }
    }
}

/// Room.Save keeps type6 records; RoomLoad5c5860 respawns each at its saved
/// position with zero velocity, so Init clears TargetPosition and friction
/// again and the admission Update re-anchors at that position.
pub(crate) fn readmit_saved(state: &mut State) -> Vec<u32> {
    for slot in &mut state.room.slots {
        slot.presented_position = slot.position;
        slot.velocity = ZERO;
        slot.target_position = ZERO;
        slot.current_friction = slot.configured_friction;
    }
    state.room.slots.iter().map(|slot| slot.id).collect()
}

/// Update53c450 in idle state1 with no timer, then BaseUpdate447190.
pub(crate) fn advance_actor(state: &mut State, phase: Phase, id: u32) -> Result<(), String> {
    let Some(slot) = state.room.slots.iter_mut().find(|slot| slot.id == id) else {
        return Ok(());
    };
    if slot.state != 1 {
        return Err("Slot Update outside idle state1 is unadmitted".into());
    }
    if phase == Phase::Interpolate {
        // Base4446a0 predicts from stored velocity and current friction.
        slot.presented_position = slot.position + (slot.velocity * slot.current_friction) * 0.5;
        return Ok(());
    }
    if !phase.updates_nonplayers() {
        return Ok(());
    }
    if slot.target_position.x == 0.0 && slot.target_position.y == 0.0 {
        slot.target_position = slot.position;
        slot.velocity = ZERO;
    }
    // Every idle variant: velocity = TargetPosition - position, then
    // friction*0.8 (669044) before BaseUpdate applies it: a spring back home.
    slot.velocity = WorldVector {
        x: slot.target_position.x - slot.position.x,
        y: slot.target_position.y - slot.position.y,
    };
    slot.current_friction *= 0.8;
    // 447190: the one-frame looping Idle clip stays on frame0.
    slot.velocity = slot.velocity * slot.current_friction;
    slot.position = slot.position + slot.velocity;
    slot.current_friction = slot.configured_friction;
    slot.presented_position = slot.position;
    if !slot.position.is_finite() || !slot.velocity.is_finite() {
        return Err("Slot motion became nonfinite".into());
    }
    Ok(())
}

/// The full manager contact pass, after the Player/Pickup callbacks. Player
/// callback509160 returns false at50917b for a partner failing the
/// vulnerable-enemy test444810 (types10..999), and Slot539690 always
/// returns false, so only the resource gates inside 539690 can matter.
/// ASSUMPTION: the source pair order interleaves slots and pickups by
/// manager order; this pass runs every slot after every pickup.
pub(crate) fn player_contacts(state: &mut State) -> Result<(), String> {
    for index in 0..state.room.slots.len() {
        //4453a0 rejects a dead Player before either callback.
        if state.player.dead {
            break;
        }
        let slot = &state.room.slots[index];
        let mut player = ContactBody {
            id: 0,
            entity_type: 1,
            position: crate::bombs::player_world(state),
            velocity: crate::bombs::player_velocity(state),
            radius: crate::content::PLAYER_COLLISION_RADIUS_WORLD as f32,
            mass: crate::content::PLAYER_COLLISION_MASS_WORLD as f32,
        };
        let mut body = slot.body();
        let size_multi = slot.size_multi;
        if !combat::slot_overlap(&player, &body, size_multi, UNROTATED)? {
            continue;
        }
        // 539690 with partner type1, state1 and timer+7e4 <= 0.
        let gate = match slot.variant {
            2 => Some("Blood Donation Player contact (TakeDamage, no resource gate)"),
            5 => Some("Devil Beggar Player contact (no resource gate)"),
            7 if state.inventory.keys > 0 => Some("Key Master contact with keys"),
            1 | 3 | 4 | 6 if state.inventory.coins > 0 => Some("Slot contact with coins"),
            _ => None,
        };
        if let Some(gate) = gate {
            return Err(format!("{gate} is unadmitted"));
        }
        let resolution = if size_multi.x != 1.0 || size_multi.y != 1.0 {
            combat::resolve_capsule_contact(
                &mut player,
                WorldVector { x: 1.0, y: 1.0 },
                &mut body,
                size_multi,
                UNROTATED,
            )?
        } else {
            //445710 retains its Player grid-shift veto and half share here.
            if state.player.flying != Some(false) || state.player.grid_collision_class != Some(5) {
                return Err("beggar contact requires the ground Player collider owner".into());
            }
            let grid = crate::grid::Grid::new(
                state
                    .room
                    .grid
                    .clone()
                    .ok_or("beggar contact requires admitted room grid")?,
            )?;
            static COLLIDER: std::sync::OnceLock<crate::grid::Collider> =
                std::sync::OnceLock::new();
            let collider = COLLIDER.get_or_init(|| {
                crate::grid::Collider::new(crate::grid::CollisionGeometry {
                    radius: crate::content::PLAYER_COLLISION_RADIUS_WORLD as f32,
                    scale: WorldVector { x: 1.0, y: 1.0 },
                    points: 40,
                })
                .expect("authored normal Isaac geometry")
            });
            let original = player.position;
            combat::resolve_player_beggar_contact(&mut player, &mut body, |shift| {
                collider.blocks_player_shift(&grid, original, shift)
            })?
        };
        if !resolution.colliding {
            continue;
        }
        state.player.x = f64::from(player.position.x) * 0.65 + 32.0;
        state.player.y = f64::from(player.position.y) * 0.65 - 46.0;
        state.player.vx = f64::from(player.velocity.x) * 0.65;
        state.player.vy = f64::from(player.velocity.y) * 0.65;
        let slot = &mut state.room.slots[index];
        slot.position = body.position;
        slot.presented_position = body.position;
        slot.velocity = body.velocity;
    }
    Ok(())
}

/// Entity class4 slots also pair with NPCs, pickups and bombs, each with the
/// mass response. None of those pairs is admitted yet: reaching one stops the
/// frame. Ordinary tears pass through: in the original a Tear spawned
/// overlapping a Beggar neither dies nor moves it (PDTGS0Y1 door1 shared-MT
/// trace: no tear-death draws at the overlap), so tears do not pair here.
pub(crate) fn unadmitted_contacts(state: &State) -> Result<(), String> {
    if state.room.slots.is_empty() {
        return Ok(());
    }
    let mut partners = Vec::new();
    for bomb in &state.bombs {
        if bomb.motion.lifetime == crate::bombs::BombLifetime::Alive {
            // bombs::contact_body: the planted Bomb4 body has radius16.
            partners.push(("Bomb", 4, bomb.motion.position, 16.0));
        }
    }
    for pickup in &state.room.pickups {
        if pickup.removed
            || pickup.collected
            || pickup
                .regular_collision
                .as_ref()
                .is_some_and(|collision| collision.entity_collision_class == 0)
        {
            continue;
        }
        partners.push(("Pickup", 5, pickup.position, pickup.collision_radius));
    }
    for npc in &state.room.npcs {
        if !(npc.dead || npc.removed || npc.entity_collision_class == 0) {
            partners.push(("NPC", npc.kind, npc.position, npc.collision_radius));
        }
    }
    for slot in &state.room.slots {
        let body = slot.body();
        for &(name, entity_type, position, radius) in &partners {
            let partner = ContactBody {
                id: 0,
                entity_type,
                position,
                velocity: ZERO,
                radius,
                mass: 1.0,
            };
            if combat::slot_overlap(&partner, &body, slot.size_multi, UNROTATED)? {
                return Err(format!("{name}/Slot contact is unadmitted"));
            }
        }
    }
    Ok(())
}

/// Game554490 reaches Slot TakeDamage53bd80 with flags4: machines break or
/// pay out and beggars die, all unadmitted (same radius rule as pickups).
pub(crate) fn bomb_hit(
    state: &State,
    center: WorldVector,
    blast_radius: f32,
    line_clear: &dyn Fn(WorldVector) -> bool,
) -> Result<(), String> {
    for slot in &state.room.slots {
        let delta = WorldVector {
            x: slot.position.x - center.x,
            y: slot.position.y - center.y,
        };
        let squared = delta.x * delta.x + delta.y * delta.y;
        let radius = blast_radius + slot.collision_radius;
        if squared < radius * radius && line_clear(slot.position) {
            return Err("explosion damage to a Slot is unadmitted".into());
        }
    }
    Ok(())
}

pub(crate) fn actor(id: u32) -> ActorRef {
    ActorRef {
        kind: ActorKind::Slot,
        id,
    }
}
