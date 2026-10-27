//! Entity status owners reached by Mom's Bra (AddFreeze 444970).
//! This first admission is the unlinked, unchampioned, living plain NPC path.
//! Other status operators and frozen death remain explicit frontiers.
use crate::{npcs::Npc, sprite_color::Color};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Frontier {
    FrozenDeath445c60,
    UnsupportedActor,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NpcStatus {
    /// Source Entity+470 after an observed mapped BaseUpdate reset. The
    /// legacy Npc.friction field stores AI-scaled friction before admission.
    /// None does not license substituting an assumed base friction.
    pub current_friction: Option<f32>,
    /// Signed Entity+400, independent of the timed colour stack.
    pub freeze_timer: i32,
    /// Signed Entity+420, shared by status applications (not a freeze cooldown).
    pub application_lockout: i32,
    /// Entity+318 bit0x20: sampled before this full callback counts down timers.
    pub frozen: bool,
    /// A successful application admitted the source BaseUpdate/Fast path.
    /// Retain it after expiry; an expired timer does not restore the old motion.
    pub base_update_admitted: bool,
    pub frontier: Option<Frontier>,
}

impl NpcStatus {
    pub(crate) fn flags(&self) -> u32 {
        if self.frozen {
            0x20
        } else {
            0
        }
    }

    /// BaseUpdate447446/447716, before the pending damage drain.
    pub(crate) fn advance(&mut self) {
        if self.freeze_timer > 0 {
            self.freeze_timer -= 1;
            if self.freeze_timer == 0 {
                self.frozen = false;
            }
        }
        if self.application_lockout > 0 {
            self.application_lockout -= 1;
        }
    }
}

/// The callbacks reviewed for this living path. Boss/champion, linked,
/// appearance, morph and custom-base branches need their own admission.
pub(crate) fn plain_shape(npc: &Npc) -> bool {
    matches!((npc.kind, npc.variant), (10, 0 | 1) | (12 | 13 | 18, 0))
        && npc.subtype == 0
        && !npc.boss
        && npc.champion.is_none()
        && npc.parent.is_none()
        && npc.child.is_none()
}

pub(crate) fn living_plain(npc: &Npc) -> bool {
    plain_shape(npc)
        && !npc.dead
        && !npc.removed
        && npc.state != 1
        && npc.state != 16
        && npc.status.frontier.is_none()
}

/// Preflight every eligible room-list candidate before mutating any status.
/// A known immune unlinked target is an owned no-op, including other families.
pub(crate) fn can_add_freeze(npc: &Npc, flags: u32, duration: i32) -> bool {
    if duration == 0 {
        return true;
    }
    if npc.parent.is_some() || npc.child.is_some() {
        return false;
    }
    flags & 1 != 0
        || npc.status.application_lockout > 0
        || (living_plain(npc) && npc.status.current_friction.is_some())
}

/// AddFreeze444970 after the complete room walk has passed preflight.
/// Positive durations add then clamp to120; negative durations extend to at
/// least their magnitude. The public Bra route supplies config39's220.
pub(crate) fn add_freeze(npc: &mut Npc, flags: u32, duration: i32) {
    if duration == 0 || flags & 1 != 0 || npc.status.application_lockout > 0 {
        return;
    }
    npc.status.freeze_timer = if duration > 0 {
        npc.status.freeze_timer.wrapping_add(duration).min(120)
    } else {
        npc.status.freeze_timer.max(duration.wrapping_neg())
    };
    npc.status.frozen = true;
    npc.status.base_update_admitted = true;
    // The prior mapped Full proved +470's reset value independently of
    // legacy Npc.friction. AddFreeze itself does not change Native friction;
    // this transfers that proven owner before the first reached Full/Fast.
    npc.friction = npc
        .status
        .current_friction
        .expect("complete freeze preflight requires source current friction");
    // 42f510 multiplies integer40 by the stored binary32 1/255.
    let color = Color::tinted([0.22, 0.22, 0.22, 1.0], 40.0 * (1.0 / 255.0));
    let base = npc.base_color();
    npc.sprite_color
        .set(color, npc.status.freeze_timer, 255, base);
    // No boss or linked actor is admitted by living_plain. Their root walk,
    // shared240 lockout and child propagation are deliberately not inferred.
}

pub(crate) fn refuse(npc: &mut Npc, frontier: Frontier) -> String {
    npc.status.frontier = Some(frontier);
    format!("NPC {} freeze source frontier: {frontier:?}", npc.id)
}
