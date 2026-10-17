use rebirth_cleanroom::{
    fullscreen::FullscreenPreferences, options_menu::AudioPreferences, Command, Game,
    MAX_COMMAND_TICKS, MAX_REPLAY_COMMANDS,
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
    if total_ticks > u64::from(MAX_COMMAND_TICKS) {
        eprintln!("replay exceeds 36,000 total simulation steps");
        std::process::exit(1);
    }
    let mut game = match Game::new_with_input_configs(
        replay.initial_seed,
        replay.boot_epoch_seed,
        None,
        replay.boot_audio_preferences,
        replay.boot_fullscreen_preferences,
        replay.boot_input_configs,
    ) {
        Ok(game) => game,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    println!("{}", game.json());
    for command in replay.commands {
        if let Err(error) = game.command(command) {
            eprintln!("{error}");
            std::process::exit(1);
        }
        println!("{}", game.json());
    }
}
