//! Released System poll of actual physical keys through the one Keyboard owner.
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
    pub held: [bool; 24],
    pub triggered: [bool; 24],
    pub raw_key: Option<i32>,
    physical_was_down: BTreeSet<String>,
}
impl PublicActions {
    ///System5950E0 -> InputPoll60FFE0 runs on both presented parities and is
    ///not advanced on Phase::Skipped. Trigger is per physical key, not the
    ///edge of the OR of an action's two columns (Space-held + Enter-fresh).
    pub fn advance(&mut self, pressed: &BTreeSet<String>, keyboard: &crate::keyboard::Keyboard) {
        for action in 0..24 {
            self.held[action] = keyboard.held(action, pressed);
            self.triggered[action] = keyboard.triggered(action, pressed, &self.physical_was_down);
        }
        //62B420 scans0..348 in native numeric order, independent of DOM or
        //public command arrival order. Escape256 remains a binding candidate.
        self.raw_key = (0..=crate::input::GLFW_KEY_LAST).find(|key| {
            crate::input::code_for_glfw(*key).is_some_and(|code| {
                pressed.contains(code) && !self.physical_was_down.contains(code)
            })
        });
        self.menu_confirm = self.triggered[14];
        self.menu_back = self.triggered[15];
        self.active_pressed = self.held[9];
        self.active_triggered = self.triggered[9];
        self.pocket_pressed = self.held[10];
        self.pocket_triggered = self.triggered[10];
        self.physical_was_down.clone_from(pressed);
    }
}
