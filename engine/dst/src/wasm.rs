//! Differential execution of the shipped WASM engine.
//!
//! A game tape replays through `web/generated/engine.wasm` exactly as
//! `web/engine.ts` drives it (reset, request buffer, command, snapshot) in a
//! deterministic pure-Rust interpreter. Every command outcome and sampled
//! published snapshots must equal the native engine's, so properties proven
//! natively speak for the artifact browsers run.
use crate::json_diff;
use crate::props;
use crate::tape::{Reset, Tape};
use crate::units::game::saves::{published_io, serve};
use rebirth_cleanroom::save_host::SaveIo;
use serde_json::{json, Value};
use std::hash::{DefaultHasher, Hash, Hasher};
use wasmi::{Engine, Instance, Linker, Memory, Store, TypedFunc};

pub struct Module {
    engine: Engine,
    module: wasmi::Module,
    pub path: String,
}

struct Exports {
    store: Store<()>,
    memory: Memory,
    reset: TypedFunc<(u32, u32), u32>,
    reset_with_audio_preferences: TypedFunc<(u32, u32, f32, f32), u32>,
    reset_with_fullscreen_preferences: TypedFunc<(u32, u32, u32, u32, f32, f32), u32>,
    reset_with_input_configs: TypedFunc<(u32, u32), u32>,
    request_ptr: TypedFunc<u32, u32>,
    command: TypedFunc<(), u32>,
    snapshot_ptr: TypedFunc<(), u32>,
    snapshot_len: TypedFunc<(), u32>,
    error_len: TypedFunc<(), u32>,
    public_keys_ptr: TypedFunc<(), u32>,
    public_keys_len: TypedFunc<(), u32>,
    save_input_ptr: TypedFunc<u32, u32>,
    load_save_data: TypedFunc<(), u32>,
    enable_saved_game_io: TypedFunc<u32, u32>,
    complete_saved_game_write: TypedFunc<(u32, u32, u32), u32>,
    complete_save_menu_readmission: TypedFunc<(u32, u32, u32), u32>,
    save_data_ptr: TypedFunc<(), u32>,
}

/// Snapshots compared per tape besides the final one; interpretation is
/// far slower than native dispatch.
const SAMPLED_SNAPSHOTS: usize = 6;

impl Module {
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let engine = Engine::default();
        let module = wasmi::Module::new(&engine, &bytes).map_err(|error| error.to_string())?;
        if module.imports().len() != 0 {
            return Err("the browser instantiates engine.wasm without imports".into());
        }
        Ok(Self {
            engine,
            module,
            path: path.display().to_string(),
        })
    }

    fn instantiate(&self) -> Result<Exports, String> {
        let mut store = Store::new(&self.engine, ());
        let instance: Instance = Linker::<()>::new(&self.engine)
            .instantiate_and_start(&mut store, &self.module)
            .map_err(|error| error.to_string())?;
        let abi_version: TypedFunc<(), u32> = instance
            .get_typed_func(&store, "abi_version")
            .map_err(|error| format!("abi_version: {error}"))?;
        let version = abi_version
            .call(&mut store, ())
            .map_err(|error| error.to_string())?;
        if version != rebirth_cleanroom::ABI_VERSION {
            return Err(format!(
                "engine.wasm ABI {version} differs from native ABI {}; rebuild with npm run build:wasm",
                rebirth_cleanroom::ABI_VERSION
            ));
        }
        macro_rules! export {
            ($name:literal) => {
                instance
                    .get_typed_func(&store, $name)
                    .map_err(|error| format!("{}: {error}", $name))?
            };
        }
        Ok(Exports {
            memory: instance
                .get_memory(&store, "memory")
                .ok_or("engine.wasm exports no memory")?,
            reset: export!("reset"),
            reset_with_audio_preferences: export!("reset_with_audio_preferences"),
            reset_with_fullscreen_preferences: export!("reset_with_fullscreen_preferences"),
            reset_with_input_configs: export!("reset_with_input_configs"),
            request_ptr: export!("request_ptr"),
            command: export!("command"),
            snapshot_ptr: export!("snapshot_ptr"),
            snapshot_len: export!("snapshot_len"),
            error_len: export!("error_len"),
            public_keys_ptr: export!("public_keys_ptr"),
            public_keys_len: export!("public_keys_len"),
            save_input_ptr: export!("save_input_ptr"),
            load_save_data: export!("load_save_data"),
            enable_saved_game_io: export!("enable_saved_game_io"),
            complete_saved_game_write: export!("complete_saved_game_write"),
            complete_save_menu_readmission: export!("complete_save_menu_readmission"),
            save_data_ptr: export!("save_data_ptr"),
            store,
        })
    }

    /// Replays `tape`; `trace` holds the native outcome and state digest of
    /// each command. Every compared observation must agree.
    pub fn differential(&self, tape: &Tape, trace: &[(bool, u64)]) {
        match self.run(tape, trace) {
            Ok(None) => {
                props::always!(true, "wasm: the shipped engine agrees with native dispatch");
            }
            Ok(Some(divergence)) => {
                props::always!(
                    false,
                    "wasm: the shipped engine agrees with native dispatch",
                    json!({ "wasm": self.path, "divergence": divergence })
                );
            }
            Err(error) => {
                props::always!(
                    false,
                    "wasm: the shipped engine agrees with native dispatch",
                    json!({ "wasm": self.path, "error": error })
                );
            }
        }
    }

    fn run(&self, tape: &Tape, trace: &[(bool, u64)]) -> Result<Option<serde_json::Value>, String> {
        let mut wasm = self.instantiate()?;
        let trap = |error: wasmi::Error| format!("engine.wasm trapped: {error}");
        // The browser reads its key surface from this export before reset.
        let keys = wasm.read(wasm.public_keys_ptr, wasm.public_keys_len)?;
        // SAFETY: public_keys_ptr/public_keys_len describe a thread-local buffer.
        let native = unsafe {
            std::slice::from_raw_parts(
                rebirth_cleanroom::public_keys_ptr(),
                rebirth_cleanroom::public_keys_len(),
            )
        };
        if keys != native {
            return Ok(Some(
                json!({ "step": "publicKeys", "wasm": String::from_utf8_lossy(&keys) }),
            ));
        }
        let seeds = (tape.initial_seed, tape.boot_epoch_seed);
        let reset = match tape.reset() {
            Reset::Plain => wasm.reset.call(&mut wasm.store, seeds),
            Reset::Audio {
                sfx_volume,
                music_volume,
            } => wasm.reset_with_audio_preferences.call(
                &mut wasm.store,
                (seeds.0, seeds.1, sfx_volume, music_volume),
            ),
            Reset::Display {
                fullscreen,
                has_audio,
                sfx_volume,
                music_volume,
            } => wasm.reset_with_fullscreen_preferences.call(
                &mut wasm.store,
                (
                    seeds.0,
                    seeds.1,
                    fullscreen,
                    has_audio,
                    sfx_volume,
                    music_volume,
                ),
            ),
            Reset::InputConfigs { request } => {
                let pointer = wasm
                    .request_ptr
                    .call(&mut wasm.store, request.len() as u32)
                    .map_err(trap)?;
                if pointer == 0 {
                    return Ok(Some(
                        json!({ "step": "reset", "reason": "request allocation" }),
                    ));
                }
                wasm.memory
                    .write(&mut wasm.store, pointer as usize, &request)
                    .map_err(|error| error.to_string())?;
                wasm.reset_with_input_configs.call(&mut wasm.store, seeds)
            }
        };
        if reset.map_err(trap)? != 0 {
            return Ok(Some(json!({ "step": "reset" })));
        }
        // The browser declares save IO and loads its stored document before
        // the first command, then answers the save IO each command leaves.
        let host = tape.save_host.as_ref();
        let mut store = match host.map(|host| host.boot(&mut wasm)).transpose() {
            Ok(store) => store,
            Err(error) => return Ok(Some(json!({ "step": "saveHost", "error": error }))),
        };
        let count = tape.commands.len();
        let stride = (count / SAMPLED_SNAPSHOTS).max(1);
        for (index, command) in tape.commands.iter().enumerate() {
            let bytes = serde_json::to_vec(command).map_err(|error| error.to_string())?;
            let pointer = wasm
                .request_ptr
                .call(&mut wasm.store, bytes.len() as u32)
                .map_err(trap)?;
            if pointer == 0 {
                return Ok(Some(
                    json!({ "step": index, "reason": "request allocation" }),
                ));
            }
            wasm.memory
                .write(&mut wasm.store, pointer as usize, &bytes)
                .map_err(|error| error.to_string())?;
            let failed = wasm.command.call(&mut wasm.store, ()).map_err(trap)? != 0;
            let error_published = wasm.error_len.call(&mut wasm.store, ()).map_err(trap)? != 0;
            let (native_ok, native_digest) = trace[index];
            if failed == native_ok || failed != error_published {
                return Ok(Some(
                    json!({ "step": index, "command": command, "wasmFailed": failed, "nativeOk": native_ok }),
                ));
            }
            if !failed {
                if let Err(error) = serve(&mut store, host, &mut wasm, command) {
                    return Ok(Some(
                        json!({ "step": index, "command": command, "saveHost": error }),
                    ));
                }
            }
            if index % stride == stride - 1 || index + 1 == count {
                let snapshot = wasm.read(wasm.snapshot_ptr, wasm.snapshot_len)?;
                let mut hasher = DefaultHasher::new();
                snapshot.hash(&mut hasher);
                if hasher.finish() != native_digest {
                    return Ok(Some(json!({
                        "step": index,
                        "command": command,
                        "reason": "snapshot",
                        "firstDifference": first_difference(tape, index, &snapshot),
                    })));
                }
            }
        }
        Ok(None)
    }
}

impl Exports {
    /// Reads a buffer the module describes with a pointer and a length export.
    fn read(
        &mut self,
        pointer: TypedFunc<(), u32>,
        length: TypedFunc<(), u32>,
    ) -> Result<Vec<u8>, String> {
        let trap = |error: wasmi::Error| format!("engine.wasm trapped: {error}");
        let pointer = pointer.call(&mut self.store, ()).map_err(trap)?;
        let length = length.call(&mut self.store, ()).map_err(trap)?;
        let mut bytes = vec![0; length as usize];
        self.memory
            .read(&self.store, pointer as usize, &mut bytes)
            .map_err(|error| error.to_string())?;
        Ok(bytes)
    }

    fn call<P: wasmi::WasmParams>(
        &mut self,
        function: TypedFunc<P, u32>,
        parameters: P,
    ) -> Result<u32, String> {
        function
            .call(&mut self.store, parameters)
            .map_err(|error| format!("engine.wasm trapped: {error}"))
    }

    /// Copies `bytes` into the buffer `save_input_ptr` allocates.
    fn save_input(&mut self, bytes: &[u8]) -> Result<(), String> {
        let pointer = self.call(self.save_input_ptr, bytes.len() as u32)?;
        if pointer == 0 {
            return Err("save input allocation".into());
        }
        self.memory
            .write(&mut self.store, pointer as usize, bytes)
            .map_err(|error| error.to_string())
    }
}

/// The shipped WASM's save transport, as `web/engine.ts` calls it.
impl SaveIo for Exports {
    fn enable(&mut self) -> Result<(), String> {
        match self.call(self.enable_saved_game_io, 1)? {
            0 => Ok(()),
            _ => Err("enable_saved_game_io refused".into()),
        }
    }
    fn import(&mut self, document: &[u8]) -> Result<(), String> {
        self.save_input(document)?;
        match self.call(self.load_save_data, ())? {
            0 => Ok(()),
            _ => Err("load_save_data refused".into()),
        }
    }
    fn pending(&mut self) -> Result<(bool, bool, u64), String> {
        let snapshot = self.read(self.snapshot_ptr, self.snapshot_len)?;
        published_io(&snapshot)
    }
    fn export(&mut self) -> Result<Vec<u8>, String> {
        let document = self.read(self.save_data_ptr, self.snapshot_len)?;
        if document.is_empty() {
            return Err("save_data_ptr refused".into());
        }
        Ok(document)
    }
    fn complete_write(&mut self, revision: u64, succeeded: bool) -> Result<(), String> {
        let parameters = (
            revision as u32,
            (revision >> 32) as u32,
            u32::from(succeeded),
        );
        match self.call(self.complete_saved_game_write, parameters)? {
            0 => Ok(()),
            _ => Err("complete_saved_game_write refused".into()),
        }
    }
    fn complete_readmission(
        &mut self,
        revision: u64,
        document: Option<&[u8]>,
    ) -> Result<bool, String> {
        if let Some(document) = document {
            self.save_input(document)?;
        }
        let parameters = (
            revision as u32,
            (revision >> 32) as u32,
            u32::from(document.is_some()),
        );
        match self.call(self.complete_save_menu_readmission, parameters)? {
            0 => Ok(true),
            2 => Ok(false),
            _ => Err("complete_save_menu_readmission refused".into()),
        }
    }
}

/// Names a snapshot divergence: replays the native engine through `step` and
/// returns the first JSON path whose value differs from the WASM snapshot.
fn first_difference(tape: &Tape, step: usize, wasm: &[u8]) -> Value {
    let Ok((mut game, mut store)) = tape.session() else {
        return Value::Null;
    };
    for command in &tape.commands[..=step] {
        if game.command(command.clone()).is_ok() {
            let _ = serve(&mut store, tape.save_host.as_ref(), &mut game, command);
        }
    }
    // Both sides parse the same published JSON text; `to_value` would widen
    // f32 fields instead of keeping their shortest representation.
    let native = serde_json::to_vec(&game.state).expect("state serializes");
    let (Ok(native), Ok(wasm)) = (
        serde_json::from_slice::<Value>(&native),
        serde_json::from_slice::<Value>(wasm),
    ) else {
        return Value::Null;
    };
    match json_diff::first(&native, &wasm) {
        Some((path, native, wasm)) => json!({ "path": path, "native": native, "wasm": wasm }),
        None => Value::Null,
    }
}
