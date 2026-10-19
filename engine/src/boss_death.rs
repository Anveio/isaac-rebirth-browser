//! Entity_NPC::Update's dying-boss state 16 (4fa56d), shared by boss families.
//!
//! A boss whose HP drained below zero enters state 16 on its next full update
//! instead of being removed: friction drops to 0.01, the family's death clip
//! plays (unforced, when the dying frame is 0), and when the clip finishes
//! 4f2d10 runs the boss callback 5c8430 (boss count, rewards) and removes it.
//! The Explosion event's Effect 1000.2/1000.77 and gib owners are not admitted
//! yet; the callback itself is recorded as pending rather than invented.
use crate::npcs::Npc;

/// One dying update: enters state 16 with frame 0, scales `friction` by 0.01
/// and advances `frame` (Entity_NPC +f58). Returns the frame this update ran
/// with; the caller plays its death clip when it is 0.
pub(crate) fn step(npc: &mut Npc, frame: &mut u32, friction: &mut f32) -> Result<u32, String> {
    if npc.state != 16 {
        npc.state = 16;
        *frame = 0;
    }
    *friction *= 0.01;
    let current = *frame;
    *frame = current.checked_add(1).ok_or("boss Death frame exhausted")?;
    Ok(current)
}

/// 4f2d10 -> 5c8430 once the death clip has finished: the boss callback
/// (boss count, final rewards) runs and the actor is removed.
pub(crate) fn finish_if_done(npc: &mut Npc) -> bool {
    if npc.animation_playing {
        return false;
    }
    // Chub's non-boss body segments die through state 16 without it.
    npc.boss_callback_pending = npc.boss;
    npc.removed = true;
    npc.visible = false;
    true
}
