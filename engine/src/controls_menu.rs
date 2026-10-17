//! Menu9 device select5A3390 and keyboard Menu10 callback5A4FF0.
//! Keyboard is the only actually admitted device; saved disconnected names
//! are retained by persistence, never advertised as connected controllers.
use crate::{
    keyboard::{InputConfigs, Keyboard, CONFIGURABLE},
    public_actions::PublicActions,
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ControlsMenu {
    pub device_selection: usize,
    pub device_held: u32,
    pub selected_device: Option<u32>,
    pub selection: usize,
    pub column: usize,
    pub waiting: bool,
    pub next_save_sequence: u64,
    pub save_requests: Vec<SaveRequest>,
    pub unavailable_actions: Vec<usize>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveRequest {
    pub sequence: u64,
    pub configs: InputConfigs,
}
#[derive(Clone, Copy)]
pub enum Action {
    None,
    Keyboard,
    Devices,
    Options,
    Refused,
}
impl ControlsMenu {
    pub fn begin_command(&mut self) {
        self.save_requests.clear();
        self.unavailable_actions.clear();
    }
    pub fn devices(&mut self, actions: &PublicActions) -> Result<Action, String> {
        //Actual connected-device list presently contains Keyboard ID0 only.
        //The repeat counter is owned here, independent of Options counters.
        let result = if actions.menu_confirm {
            self.selected_device = Some(0);
            Action::Keyboard
        } else if actions.menu_back {
            Action::Options
        } else {
            Action::None
        };
        self.device_held = if actions.held[22] || actions.held[23] {
            self.device_held
                .checked_add(1)
                .ok_or("Controls device repeat counter exhausted")?
        } else {
            0
        };
        Ok(result)
    }
    pub fn configure(
        &mut self,
        actions: &PublicActions,
        keyboard: &mut Keyboard,
        configs: &mut InputConfigs,
    ) -> Result<Action, String> {
        if self.selected_device != Some(0) || self.selection > 17 || self.column > 1 {
            return Err("Keyboard configuration lacks a valid actual device/row/column".into());
        }
        if self.waiting {
            if let Some(key) = actions.raw_key {
                keyboard.bind(CONFIGURABLE[self.selection], self.column, key)?;
                self.waiting = false;
            }
        } else if actions.menu_confirm {
            if self.selection == 17 {
                keyboard.reset();
            } else {
                self.waiting = true;
            }
        } else if actions.menu_back {
            if !keyboard.complete() {
                return Ok(Action::Refused);
            }
            self.selection = 0;
            self.column = 0;
            self.waiting = false;
            self.selected_device = None;
            configs.update_keyboard(keyboard);
            let sequence = self.next_save_sequence;
            self.next_save_sequence = sequence
                .checked_add(1)
                .ok_or("Input configuration save sequence exhausted")?;
            self.save_requests.push(SaveRequest {
                sequence,
                configs: configs.clone(),
            });
            return Ok(Action::Devices);
        } else if actions.triggered[22] {
            self.selection = (self.selection + 17) % 18;
        } else if actions.triggered[23] {
            self.selection = (self.selection + 1) % 18;
        }
        //Native LEFT selects column1, RIGHT column0; preserve this source
        //getter/setter index even if its screen layout appears surprising.
        else if actions.triggered[20] {
            self.column = 1;
        } else if actions.triggered[21] {
            self.column = 0;
        }
        Ok(Action::None)
    }
    pub fn keyboard_scroll(&self) -> f32 {
        //59CC60(row,21,12), reset row17 uses(13,27,10).
        if self.selection <= 2 {
            -1890.0
        } else if self.selection == 17 {
            -1890.0 - 27.0 * 10.0
        } else {
            -1890.0 - 21.0 * (self.selection.min(12) as f32 - 2.0)
        }
    }
}
