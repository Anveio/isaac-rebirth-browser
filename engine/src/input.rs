//! Canonical public physical keyboard catalog and native GLFW key numbers.
//!
//! `Command::Key` and the browser use these exact `KeyboardEvent.code` strings;
//! no logical-key aliases or rewritten WASD events enter physical pressed state.
//! The public list retains its original fifteen entries and ABI order, then
//! appends the remaining mapped keys in ascending native GLFW numeric order.
//! Raw binding priority must scan numeric GLFW 0..=GLFW_KEY_LAST, not this list.
//!
//! GLFW -1 is the unbound sentinel. Unnamed numeric gaps, WORLD_1/WORLD_2
//! (161/162: platform-dependent non-US keys), and F25 (314: no standard DOM
//! physical code) deliberately have no public code. They must not be replaced
//! with invented aliases. Browser/OS-reserved keys may never be delivered.

pub const GLFW_KEY_LAST: i32 = 348;

// One source table emits the public catalog and both directions. The macro
// preserves the fixed-array Rust surface; serialize PUBLIC_KEYS.as_slice().
macro_rules! physical_keys {
    ($( $code:literal => $glfw:literal ),+ $(,)?) => {
        pub const PUBLIC_KEYS: [&str; [$( $code ),+].len()] = [$( $code ),+];

        /// Native GLFW key number for an admitted physical browser code.
        pub fn glfw_key(code: &str) -> Option<i32> {
            match code {
                $( $code => Some($glfw), )+
                _ => None,
            }
        }

        /// Physical browser code for a mapped native key, never a guessed alias.
        pub fn code_for_glfw(key: i32) -> Option<&'static str> {
            match key {
                $( $glfw => Some($code), )+
                _ => None,
            }
        }
    };
}

physical_keys! {
    // Existing public ABI indices 0..14, unchanged.
    "Enter" => 257,
    "Escape" => 256,
    "Tab" => 258,
    "Backspace" => 259,
    "ArrowUp" => 265,
    "ArrowDown" => 264,
    "ArrowLeft" => 263,
    "ArrowRight" => 262,
    "KeyW" => 87,
    "KeyA" => 65,
    "KeyS" => 83,
    "KeyD" => 68,
    "KeyE" => 69,
    "Space" => 32,
    "KeyQ" => 81,
    // New entries append in ascending GLFW numeric order.
    "Quote" => 39,
    "Comma" => 44,
    "Minus" => 45,
    "Period" => 46,
    "Slash" => 47,
    "Digit0" => 48,
    "Digit1" => 49,
    "Digit2" => 50,
    "Digit3" => 51,
    "Digit4" => 52,
    "Digit5" => 53,
    "Digit6" => 54,
    "Digit7" => 55,
    "Digit8" => 56,
    "Digit9" => 57,
    "Semicolon" => 59,
    "Equal" => 61,
    "KeyB" => 66,
    "KeyC" => 67,
    "KeyF" => 70,
    "KeyG" => 71,
    "KeyH" => 72,
    "KeyI" => 73,
    "KeyJ" => 74,
    "KeyK" => 75,
    "KeyL" => 76,
    "KeyM" => 77,
    "KeyN" => 78,
    "KeyO" => 79,
    "KeyP" => 80,
    "KeyR" => 82,
    "KeyT" => 84,
    "KeyU" => 85,
    "KeyV" => 86,
    "KeyX" => 88,
    "KeyY" => 89,
    "KeyZ" => 90,
    "BracketLeft" => 91,
    "Backslash" => 92,
    "BracketRight" => 93,
    "Backquote" => 96,
    "Insert" => 260,
    "Delete" => 261,
    "PageUp" => 266,
    "PageDown" => 267,
    "Home" => 268,
    "End" => 269,
    "CapsLock" => 280,
    "ScrollLock" => 281,
    "NumLock" => 282,
    "PrintScreen" => 283,
    "Pause" => 284,
    "F1" => 290,
    "F2" => 291,
    "F3" => 292,
    "F4" => 293,
    "F5" => 294,
    "F6" => 295,
    "F7" => 296,
    "F8" => 297,
    "F9" => 298,
    "F10" => 299,
    "F11" => 300,
    "F12" => 301,
    "F13" => 302,
    "F14" => 303,
    "F15" => 304,
    "F16" => 305,
    "F17" => 306,
    "F18" => 307,
    "F19" => 308,
    "F20" => 309,
    "F21" => 310,
    "F22" => 311,
    "F23" => 312,
    "F24" => 313,
    "Numpad0" => 320,
    "Numpad1" => 321,
    "Numpad2" => 322,
    "Numpad3" => 323,
    "Numpad4" => 324,
    "Numpad5" => 325,
    "Numpad6" => 326,
    "Numpad7" => 327,
    "Numpad8" => 328,
    "Numpad9" => 329,
    "NumpadDecimal" => 330,
    "NumpadDivide" => 331,
    "NumpadMultiply" => 332,
    "NumpadSubtract" => 333,
    "NumpadAdd" => 334,
    "NumpadEnter" => 335,
    "NumpadEqual" => 336,
    "ShiftLeft" => 340,
    "ControlLeft" => 341,
    "AltLeft" => 342,
    "MetaLeft" => 343,
    "ShiftRight" => 344,
    "ControlRight" => 345,
    "AltRight" => 346,
    "MetaRight" => 347,
    "ContextMenu" => 348,
}

pub fn is_public_key(code: &str) -> bool {
    glfw_key(code).is_some()
}
