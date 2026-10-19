use rebirth_cleanroom::{
    fullscreen::FullscreenPreferences, options_menu::AudioPreferences, save_host::SaveHost,
    Command, Game, MAX_REPLAY_COMMANDS, MAX_REPLAY_TICKS,
};
use serde::Deserialize;
use std::io::{self, Read};
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Replay {
    initial_seed: u32,
    #[serde(default)]
    boot_epoch_seed: u32,
    #[serde(default)]
    boot_audio_preferences: Option<AudioPreferences>,
    #[serde(default)]
    boot_fullscreen_preferences: Option<FullscreenPreferences>,
    #[serde(default)]
    boot_input_configs: Option<rebirth_cleanroom::keyboard::InputConfigs>,
    #[serde(default)]
    boot_map_preferences: Option<rebirth_cleanroom::map_presentation::MapPreferences>,
    #[serde(default)]
    boot_gamma_preferences: Option<rebirth_cleanroom::display_settings::GammaPreferences>,
    /// The browser's own-save storage; absent, the game has no save IO.
    #[serde(default)]
    save_host: Option<SaveHost>,
    commands: Vec<Command>,
}
fn main() {
    let mut input = String::new();
    if let Err(error) = io::stdin()
        .take(4 * 1024 * 1024 + 1)
        .read_to_string(&mut input)
    {
        eprintln!("{error}");
        std::process::exit(1);
    }
    if input.len() > 4 * 1024 * 1024 {
        eprintln!("replay exceeds 4 MiB");
        std::process::exit(1);
    }
    let replay: Replay = match serde_json::from_str(&input) {
        Ok(replay) => replay,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    if replay.commands.len() > MAX_REPLAY_COMMANDS {
        eprintln!("replay exceeds 36,000 commands");
        std::process::exit(1);
    }
    let total_ticks: u64 = replay
        .commands
        .iter()
        .filter_map(|command| match command {
            Command::Tick { count, .. } => Some(u64::from(*count)),
            _ => None,
        })
        .sum();
    if total_ticks > u64::from(MAX_REPLAY_TICKS) {
        eprintln!("replay exceeds 72,000 total simulation steps");
        std::process::exit(1);
    }
    let mut game = match Game::new_with_settings_preferences(
        replay.initial_seed,
        replay.boot_epoch_seed,
        None,
        replay.boot_audio_preferences,
        replay.boot_fullscreen_preferences,
        replay.boot_input_configs,
        replay.boot_map_preferences,
        replay.boot_gamma_preferences,
    ) {
        Ok(game) => game,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    let mut store = match replay
        .save_host
        .as_ref()
        .map(|host| host.boot(&mut game))
        .transpose()
    {
        Ok(store) => store,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    println!("{}", game.json());
    for command in replay.commands {
        if let Err(error) = game.command(command.clone()) {
            eprintln!("{error}");
            std::process::exit(1);
        }
        if let (Some(host), Some(store)) = (&replay.save_host, &mut store) {
            if let Err(error) = store.service(host, &mut game, &command) {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
        println!("{}", game.json());
    }
}
