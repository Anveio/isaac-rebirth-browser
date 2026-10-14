//! The exported C ABI transport (`engine/src/lib.rs`, consumed by
//! `web/engine.ts`): request framing, malformed requests, failed resets and
//! the per-thread engine slot.
use crate::entropy::Entropy;
use crate::harness::Case;
use crate::props;
use rebirth_cleanroom as abi;
use rebirth_cleanroom::{Command, State, ABI_VERSION};
use serde_json::json;

/// web/engine.ts refuses larger requests before calling `request_ptr`.
const MAX_REQUEST: usize = 65_536;

fn send(bytes: &[u8]) -> Option<u32> {
    let pointer = abi::request_ptr(bytes.len());
    if pointer.is_null() {
        return None;
    }
    // SAFETY: request_ptr returned a writable buffer of exactly bytes.len().
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), pointer, bytes.len()) };
    Some(abi::command())
}

fn snapshot() -> Vec<u8> {
    let pointer = abi::snapshot_ptr();
    // SAFETY: snapshot_ptr/snapshot_len describe the thread-local buffer.
    unsafe { std::slice::from_raw_parts(pointer, abi::snapshot_len()) }.to_vec()
}

fn public_keys() -> Vec<u8> {
    // SAFETY: public_keys_ptr/public_keys_len describe the thread-local buffer.
    unsafe { std::slice::from_raw_parts(abi::public_keys_ptr(), abi::public_keys_len()) }.to_vec()
}

fn error() -> String {
    // SAFETY: error_ptr/error_len describe the thread-local buffer.
    let bytes = unsafe { std::slice::from_raw_parts(abi::error_ptr(), abi::error_len()) };
    String::from_utf8_lossy(bytes).into_owned()
}

/// Requests the bridge can deliver but the engine must reject.
fn malformed(e: &mut Entropy) -> Vec<u8> {
    match e.below(8) {
        0 => br#"{"type":"tick","count":1,"extra":true}"#.to_vec(),
        1 => br#"{"type":"tick","count":-1}"#.to_vec(),
        2 => br#"{"type":"audioVolume","sfx_volume":1e999}"#.to_vec(),
        3 => br#"{"type":"teleport"}"#.to_vec(),
        4 => br#"{"type":"key","code":"Enter"}"#.to_vec(),
        5 => {
            let valid = serde_json::to_vec(&Command::Tick {
                count: 1,
                timestamp_millis: None,
            })
            .expect("command");
            valid[..e.index(valid.len())].to_vec()
        }
        6 => {
            // Arbitrary bytes can encode a valid command. Keep all drawn
            // bytes, but prefix invalid UTF-8 so this arm is truly malformed.
            let mut request = vec![0xff];
            request.extend((0..e.range(1, 64)).map(|_| e.u32() as u8));
            request
        }
        _ => vec![0xff, 0xfe, b'{'],
    }
}

pub fn run(case: &mut Case) {
    props::always!(
        abi::abi_version() == ABI_VERSION,
        "abi: the exported ABI version is current"
    );

    // A fresh thread has no engine: commands fail and snapshots publish null,
    // while the key surface the browser reads before reset is available.
    let unset = std::thread::spawn(|| {
        let result = send(br#"{"type":"tick","count":1}"#);
        (result, error(), snapshot(), public_keys())
    })
    .join();
    props::always!(
        matches!(&unset, Ok((Some(1), message, snapshot, _)) if !message.is_empty() && snapshot == b"null"),
        "abi: commands before reset fail and snapshots publish null"
    );
    let published = public_keys();
    let decoded: Result<Vec<String>, _> = serde_json::from_slice(&published);
    props::always!(
        decoded
            .as_ref()
            .is_ok_and(|keys| keys.iter().map(String::as_str).eq(abi::input::PUBLIC_KEYS))
            && matches!(&unset, Ok((.., before_reset)) if *before_reset == published),
        "abi: the public_keys export publishes exactly the engine's key surface",
        json!({ "published": String::from_utf8_lossy(&published) })
    );

    let e = &mut *case.entropy;
    let seed = e.nonzero_u32();
    props::always!(
        abi::reset(seed, e.u32()) == 0 && abi::error_len() == 0,
        "abi: reset with a nonzero seed succeeds and clears the error",
        json!({ "seed": seed })
    );
    let booted = snapshot();
    props::always!(
        serde_json::from_slice::<State>(&booted).is_ok_and(|state| state.initial_seed == seed),
        "abi: the published snapshot is a valid state of the reset seed",
        json!({ "seed": seed })
    );
    props::always!(
        abi::reset(0, e.u32()) == 1 && !error().is_empty() && snapshot() == booted,
        "abi: a failed reset reports an error and keeps the previous game"
    );
    props::always!(
        abi::request_ptr(MAX_REQUEST + 1 + e.index(1024)).is_null()
            && !abi::request_ptr(MAX_REQUEST).is_null(),
        "abi: request buffers are bounded at 64 KiB"
    );

    let mut current = booted;
    for _ in 0..(case.steps / 16).max(32) {
        if e.chance(1, 2) {
            let request = malformed(e);
            let result = send(&request);
            props::always!(
                result == Some(1) && !error().is_empty() && snapshot() == current,
                "abi: malformed requests fail with an error and leave the snapshot unchanged",
                json!({ "request": String::from_utf8_lossy(&request) })
            );
        } else {
            let command = match e.below(4) {
                3 => Command::Tick {
                    count: rebirth_cleanroom::MAX_COMMAND_TICKS + 1,
                    timestamp_millis: None,
                },
                0 => Command::Tick {
                    count: e.range(0, 30),
                    timestamp_millis: None,
                },
                1 => Command::Key {
                    code: (*e.pick(&["Enter", "Escape", "ArrowLeft", "KeyD"])).into(),
                    down: e.chance(1, 2),
                },
                _ => Command::AudioVolume {
                    sfx_volume: e.range(0, 100) as f32 / 100.0,
                },
            };
            let bytes = serde_json::to_vec(&command).expect("command serializes");
            let result = send(&bytes);
            let accepted = result == Some(0);
            props::always!(
                result.is_some() && accepted == (abi::error_len() == 0),
                "abi: the return code and error buffer agree",
                json!({ "command": command, "result": result, "error": error() })
            );
            let published = snapshot();
            props::always!(
                accepted || published == current,
                "abi: rejected commands leave the snapshot unchanged",
                json!({ "command": command })
            );
            props::always!(
                serde_json::from_slice::<State>(&published).is_ok(),
                "abi: every published snapshot is a valid state"
            );
            props::sometimes!(!accepted, "abi: a well-formed command is rejected");
            current = published;
        }
    }
}
