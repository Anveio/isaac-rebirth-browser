/// Exact444810 inputs. Flags require a concrete family callback; an unknown
/// family must not borrow Npc::entity_flags' legacy default4.
pub(crate) struct TargetEligibility {
    pub kind: u32,
    pub collision_class: u32,
    pub raw_494: bool,
    pub dead: bool,
    pub entity_flags: Option<u32>,
}

pub(crate) fn eligible_target(target: TargetEligibility) -> Result<bool, String> {
    if !(10..=999).contains(&target.kind)
        || matches!(target.collision_class, 0 | 1)
        || target.raw_494
        || target.dead
    {
        return Ok(false);
    }
    Ok(target
        .entity_flags
        .ok_or("homing target Entity318 owner is unknown")?
        & 0x10
        == 0)
}
