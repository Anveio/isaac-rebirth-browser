//! The public keyboard surface: the only `KeyboardEvent.code` values the
//! browser forwards and `Command::Key` accepts. The browser reads this list
//! through the `public_keys_ptr`/`public_keys_len` exports, so the engine is
//! its single owner; add a key here and both sides follow.
pub const PUBLIC_KEYS: [&str; 15] = [
    "Enter",
    "Escape",
    "Tab",
    "Backspace",
    "ArrowUp",
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight",
    "KeyW",
    "KeyA",
    "KeyS",
    "KeyD",
    "KeyE",
    "Space",
    "KeyQ",
];

pub fn is_public_key(code: &str) -> bool {
    PUBLIC_KEYS.contains(&code)
}
