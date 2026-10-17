//! Actual base-Rebirth Keyboard61FDA0/6207E0/620830 binding owner.
//! Numeric GLFW key values remain distinct from public physical code strings.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const ACTIONS: usize = 24;
pub const CONFIGURABLE: [usize; 17] = [2, 3, 0, 1, 6, 7, 4, 5, 8, 9, 10, 11, 13, 17, 16, 18, 12];
pub const DEFAULT_PAIRS: [(usize, i32); 25] = [
    (0, 65),
    (1, 68),
    (2, 87),
    (3, 83),
    (4, 263),
    (5, 262),
    (6, 265),
    (7, 264),
    (8, 69),
    (9, 32),
    (10, 81),
    (11, 341),
    (12, 80),
    (13, 258),
    (14, 32),
    (14, 257),
    (15, 256),
    (16, 82),
    (17, 70),
    (18, 77),
    (19, 257),
    (20, 263),
    (21, 262),
    (22, 265),
    (23, 264),
];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Keyboard {
    pub bindings: [[i32; 2]; ACTIONS],
}
impl Default for Keyboard {
    fn default() -> Self {
        let mut owner = Self {
            bindings: [[-1; 2]; ACTIONS],
        };
        owner.reset();
        owner
    }
}
impl Keyboard {
    ///61FDA0 inserts each default into the first free column. This differs
    ///from the XML loader's Name=Keyboard preset of both columns per pair.
    pub fn reset(&mut self) {
        self.bindings = [[-1; 2]; ACTIONS];
        for (action, key) in DEFAULT_PAIRS {
            if let Some(column) = self.bindings[action].iter_mut().find(|slot| **slot == -1) {
                *column = key;
            }
        }
    }
    pub fn get(&self, action: usize) -> [i32; 2] {
        self.bindings[action]
    }
    pub fn set(&mut self, action: usize, column: usize, key: i32) -> Result<(), String> {
        if action >= ACTIONS || column >= 2 || !(-1..=crate::input::GLFW_KEY_LAST).contains(&key) {
            return Err("Keyboard binding is outside action24/column2/GLFW0..348".into());
        }
        self.bindings[action][column] = key;
        Ok(())
    }
    pub fn bind(&mut self, action: usize, column: usize, key: i32) -> Result<(), String> {
        if !CONFIGURABLE.contains(&action)
            || column >= 2
            || !(0..=crate::input::GLFW_KEY_LAST).contains(&key)
        {
            return Err("Raw keyboard binding is outside its configurable owner".into());
        }
        //5A4BC0 clears only this configurable family, BOTH columns. The
        //fixed menu bindings retain the same physical key independently.
        for configured in CONFIGURABLE {
            for slot in &mut self.bindings[configured] {
                if *slot == key {
                    *slot = -1;
                }
            }
        }
        self.set(action, column, key)
    }
    pub fn held(&self, action: usize, pressed: &BTreeSet<String>) -> bool {
        self.bindings[action]
            .iter()
            .any(|key| crate::input::code_for_glfw(*key).is_some_and(|code| pressed.contains(code)))
    }
    pub fn triggered(
        &self,
        action: usize,
        pressed: &BTreeSet<String>,
        previous: &BTreeSet<String>,
    ) -> bool {
        self.bindings[action].iter().any(|key| {
            crate::input::code_for_glfw(*key)
                .is_some_and(|code| pressed.contains(code) && !previous.contains(code))
        })
    }
    pub fn complete(&self) -> bool {
        CONFIGURABLE
            .iter()
            .all(|action| self.bindings[*action] != [-1, -1])
    }
}

///Independent inputconfigs.dat startup input, never Options.ini or live state.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeviceConfig {
    pub name: String,
    pub bindings: [[i32; 2]; ACTIONS],
}
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InputConfigs {
    pub devices: Vec<DeviceConfig>,
}
impl InputConfigs {
    pub fn validate(self) -> Result<Self, String> {
        if self.devices.len() > 33
            || self
                .devices
                .iter()
                .filter(|device| device.name != "Keyboard")
                .count()
                > 32
        {
            return Err(
                "Input configuration exceeds32 retained non-Keyboard devices plus Keyboard".into(),
            );
        }
        let mut names = BTreeSet::new();
        for device in &self.devices {
            if device.name.is_empty()
                || device.name.len() > 256
                || device.name.chars().any(char::is_control)
                || !names.insert(&device.name)
            {
                return Err("Input configuration has invalid or duplicate device names".into());
            }
            if device.name == "Keyboard"
                && device
                    .bindings
                    .iter()
                    .flatten()
                    .any(|key| !(-1..=crate::input::GLFW_KEY_LAST).contains(key))
            {
                return Err(
                    "Input configuration contains an unsupported signed GLFW binding".into(),
                );
            }
        }
        Ok(self)
    }
    pub fn keyboard(&self) -> Keyboard {
        self.devices
            .iter()
            .find(|device| device.name == "Keyboard")
            .map_or_else(Keyboard::default, |device| Keyboard {
                bindings: device.bindings,
            })
    }
    ///5959D0 updates the selected device by name before43D0E0 serializes ALL
    ///retained devices. Retention does not make an unconnected device usable.
    pub fn update_keyboard(&mut self, keyboard: &Keyboard) {
        if let Some(device) = self
            .devices
            .iter_mut()
            .find(|device| device.name == "Keyboard")
        {
            device.bindings = keyboard.bindings;
        } else {
            self.devices.push(DeviceConfig {
                name: "Keyboard".into(),
                bindings: keyboard.bindings,
            });
        }
    }
}
