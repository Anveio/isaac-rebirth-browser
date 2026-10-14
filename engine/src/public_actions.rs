//! Fresh default keyboard action triggers owned by the released System poll.
//! Native Keyboard defaults bind action14 to both Space and Return, and15 to
//! Escape. Each physical key keeps its own previous state, so pressing Return
//! while holding Space still triggers the same source action.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicActions {
    pub menu_confirm: bool,
    pub menu_back: bool,
    pub active_pressed: bool,
    pub active_triggered: bool,
    pub pocket_pressed: bool,
    pub pocket_triggered: bool,
    enter_was_down: bool,
    space_was_down: bool,
    escape_was_down: bool,
    pocket_was_down: bool,
}

impl PublicActions {
    /// System5950e0 -> InputPoll60ffe0 runs on both presentations with the
    /// declared default interpolation. Disabled interpolation skips the odd
    /// System poll; callers must not advance this owner on Phase::Skipped.
    pub fn advance(&mut self, pressed: &BTreeSet<String>) {
        let enter = pressed.contains("Enter");
        let space = pressed.contains("Space");
        let escape = pressed.contains("Escape");
        let pocket = pressed.contains("KeyQ");
        self.menu_confirm = (enter && !self.enter_was_down) || (space && !self.space_was_down);
        self.menu_back = escape && !self.escape_was_down;
        self.active_pressed = space;
        self.active_triggered = space && !self.space_was_down;
        self.pocket_pressed = pocket;
        self.pocket_triggered = pocket && !self.pocket_was_down;
        self.enter_was_down = enter;
        self.space_was_down = space;
        self.escape_was_down = escape;
        self.pocket_was_down = pocket;
    }
}
