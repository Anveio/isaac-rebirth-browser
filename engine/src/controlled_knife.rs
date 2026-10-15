//! Selected subtype0/bit33 transition in473590→47359D.
//! Caller admission owns the actual ParentPlayer and empty4A0 child-chain
//! proof. This kernel reads current actor/parent/count/grid inputs and mutates
//! only that actor; allocation, item queries, RNG, slots and contacts stay out.
use crate::{
    grid::Grid,
    knives::{self, Knife, ParentView},
    motion::WorldVector,
    phase::Phase,
};

/// True requests the real grid receiver followed by Base447190. The caller
/// must keep those effects in that order; a fast callback only predicts448.
pub(crate) fn advance(
    knife: &mut Knife,
    phase: Phase,
    parent: ParentView,
    room: &Grid,
    game_age: u32,
    countable_total: i32,
) -> Result<bool, String> {
    if phase == Phase::Skipped {
        return Ok(false);
    }
    // Numeric failure is separate from the caller's valid-unmapped frontier.
    // No admission, Remove or controller write precedes these checks.
    if !knife.position.is_finite()
        || !knife.velocity.is_finite()
        || !knife.current_friction.is_finite()
        || !knife.sprite.rotation.is_finite()
        || !parent.position.is_finite()
    {
        return Err("controlled Knife requires finite current source fields".into());
    }
    if knife.subtype != 0 || knife.attack_flags.0 & knives::CONTROLLED == 0 {
        return Err("controlled Knife requires the admitted subtype0 bit33 path".into());
    }
    if knife.attack_flags.0 & (4 | 0x400 | 0x800000) != 0 {
        return Err("controlled Knife orbital, spectral or attraction path is unadmitted".into());
    }

    //47267F/473214: the fast callback sets322 before entering18;473200
    //scales the current470 only when that real flag is false. System's
    //445370 restore/reset is a separate preceding owner, not forced here.
    if phase == Phase::Interpolate {
        knife.fast = true;
    }
    if !knife.fast {
        knife.current_friction *= f32::from_bits(0x3f33_3333);
    }
    //4735AE..5BE requests Remove without clearing the registeredC18 slot;
    //the following controller/tail still runs in this actual callback.
    if parent.primary_c08 != 4 {
        knife.remove();
    }
    knife.raw_814 = false;

    //4735D2/474B6B: squared length, then612700 and42EEB0. Preserve the
    //source0.24 f32 coefficient and the actual shortest-angle operation.
    let velocity_squared = knives::squared_length(knife.velocity);
    if velocity_squared > f32::from_bits(0x3dcc_cccd) {
        let desired = knives::angle_degrees(knife.velocity) - 90.0;
        knife.sprite.rotation = crate::source_angles::approach(
            knife.sprite.rotation,
            desired,
            f32::from_bits(0x3e75_c28f),
        );
    }
    //47360A writes Layer0+22 only; preserve the independently ownedY flip.
    knife.sprite.layer_flips.entry(0).or_insert([false, false])[0] = knife.sprite.rotation > 0.0;
    //5C7E50 with all four margins zero precedes the empty-room return test.
    knife.position = room.clamp_room(knife.position, 0.0);

    let toward_parent = WorldVector {
        x: parent.position.x - knife.position.x,
        y: parent.position.y - knife.position.y,
    };
    //474767..7BD uses signed current16AC+pending16B0 supplied by its real
    //owner.474CCF/DA8 accepts slow motion or a positive current velocity dot.
    if countable_total <= 0
        && knives::squared_length(toward_parent) > 10_000.0
        && (velocity_squared < f32::from_bits(0x3dcc_cccd)
            || knife.velocity.x * toward_parent.x + knife.velocity.y * toward_parent.y > 0.0)
        && knife.current_friction != 0.0
    {
        //612860 normalizes with a f32 reciprocal,612A70 multiplies each
        //component by0.3,612B30 divides each by actual470,6129B0 adds464.
        let acceleration = knives::normalized(toward_parent) * f32::from_bits(0x3e99_999a);
        knife.velocity = knife.velocity
            + WorldVector {
                x: acceleration.x / knife.current_friction,
                y: acceleration.y / knife.current_friction,
            };
    }
    //444890(period3,offset0) respects intermediate318bit4000. Contact's
    //separate period5/own-sequence admission stays with the contact owner.
    if knife.entity_flags & 0x4000 == 0 && (game_age as i32) % 3 == 0 {
        knife.hit_sequences.clear();
    }
    if knife.fast {
        //472727→saved450 AFTER18, then (464×470)/2 added to physical448.
        knife.saved_position = knife.position;
        knife.position = knife.position + (knife.velocity * knife.current_friction) * 0.5;
    }
    if !knife.position.is_finite()
        || !knife.velocity.is_finite()
        || !knife.current_friction.is_finite()
        || !knife.sprite.rotation.is_finite()
    {
        return Err("controlled Knife transition overflowed source f32 fields".into());
    }
    Ok(!knife.fast)
}
