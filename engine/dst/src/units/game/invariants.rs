//! Per-step state invariants and per-command transition contracts.
use crate::props;
use rebirth_cleanroom::actors::ActorKind;
use rebirth_cleanroom::floor::Floor;
use rebirth_cleanroom::input;
use rebirth_cleanroom::motion::WorldVector;
use rebirth_cleanroom::player_death;
use rebirth_cleanroom::run_admission::{MAX_SAFE_TIMESTAMP_MS, PRESENTATION_INTERVAL_NS};
use rebirth_cleanroom::traversal::TransitionPhase;
use rebirth_cleanroom::{seed, Command, PausePhase, Screen, State, MAX_COMMAND_TICKS};
use serde_json::json;
use std::collections::BTreeSet;

/// Malformed inputs the public API documents as rejected (state unchanged,
/// error returned). Whether a well-formed input is accepted is the engine's
/// call: a menu entry, mode, Challenge or character the port has not
/// implemented yet may refuse its input, and pinning that refusal would turn
/// each unported feature into a failing property. A refusal still leaves the
/// game unchanged and a valid Tick still always succeeds.
pub fn expected_rejection(state: &State, command: &Command) -> Option<&'static str> {
    match command {
        Command::Tick {
            count,
            timestamp_millis,
        } => {
            if timestamp_millis.is_some_and(|timestamp| timestamp > MAX_SAFE_TIMESTAMP_MS) {
                Some("host timestamp range")
            } else if timestamp_millis.is_some() && *count != 1 {
                Some("timestamped batch")
            } else {
                (*count > MAX_COMMAND_TICKS).then_some("tick budget")
            }
        }
        Command::AudioVolume { sfx_volume: volume } | Command::MusicVolume { volume } => {
            (!volume.is_finite() || !(0.0..=1.0).contains(volume)).then_some("volume range")
        }
        Command::Key { code, .. } => (!input::is_public_key(code)).then_some("unsupported key"),
        Command::Text { .. } if state.screen != Screen::Seed => Some("text outside seed dialog"),
        Command::Text { text } => {
            let compact: String = text.chars().filter(|glyph| *glyph != ' ').collect();
            (!compact.bytes().all(|glyph| glyph.is_ascii_alphanumeric())
                || compact.len() + state.seed_text.len() > 8)
                .then_some("invalid seed text")
        }
        Command::MusicBackend { .. } => None,
    }
}

/// Stage `stage`'s seed from the run seed's startup draws (Seeds::SetStartSeed
/// 5e1470): the (stage+1)-th draw.
fn startup_stage_seed(run_seed_value: u32, stage: u32) -> u32 {
    (0..=stage).fold(run_seed_value, |value, _| seed::startup_next(value))
}

/// Whether two floors share the layout Level::Init generates: stage, seed,
/// room placement, templates and links. A room's runtime state (its
/// persistent-spawn map, descriptor+88) is not layout.
fn same_layout(left: &Floor, right: &Floor) -> bool {
    (
        left.stage,
        left.stage_seed,
        &left.grid,
        left.starting_descriptor,
    ) == (
        right.stage,
        right.stage_seed,
        &right.grid,
        right.starting_descriptor,
    ) && left.descriptors.len() == right.descriptors.len()
        && left
            .descriptors
            .iter()
            .zip(&right.descriptors)
            .all(|(a, b)| {
                (a.grid_index, &a.template, a.neighbors) == (b.grid_index, &b.template, b.neighbors)
            })
}

/// The source holds that clear the player's controls (Player+7d4).
pub fn controls_hold(state: &State) -> bool {
    state.run_admission.initial_fade != 0.0
        || state.room_transition.is_some()
        || state.stage_transition.active()
}

/// A character the profile has not unlocked; Random resolves among the
/// unlocked ones, so it is never locked.
fn locked(state: &State) -> bool {
    let unlocked = &state.profiles[usize::from(state.profile - 1)].unlocked_characters;
    state.character != "Random" && !unlocked.contains(&state.character)
}

fn run_requested(state: &State) -> bool {
    let request = &state.run_start_request;
    state.screen == Screen::Playing
        || request.input_pending
        || request.fading
        || request.system_pending
}

pub fn outcome(state: &State, command: &Command, result: &Result<(), String>, after: &State) {
    let expected = expected_rejection(state, command);
    let details = || json!({ "command": command, "result": result, "expected": expected });
    props::always!(
        expected.is_none() || result.is_err(),
        "game: malformed inputs are rejected",
        details()
    );
    // Locked characters cannot be played in the original. Refusing or
    // ignoring the press is the engine's choice; either way no run starts.
    if let Command::Key { code, down: true } = command {
        if state.screen == Screen::Character
            && code == "Enter"
            && !state.pressed.contains(code)
            && locked(state)
            && !run_requested(state)
        {
            props::always!(
                !run_requested(after),
                "game: confirming a locked character never starts a run",
                details()
            );
            props::sometimes!(true, "game: a locked character is confirmed");
        }
    }
    if matches!(command, Command::Tick { .. }) {
        props::always!(
            expected.is_some() || result.is_ok(),
            "game: valid Tick commands always succeed",
            details()
        );
    }
    props::sometimes!(
        matches!(command, Command::Tick { count, .. } if *count == MAX_COMMAND_TICKS)
            && result.is_ok(),
        "game: a Tick of exactly the command budget is accepted"
    );
    if result.is_err() {
        match expected {
            Some("tick budget") => props::sometimes!(true, "game: an over-budget Tick is rejected"),
            Some("host timestamp range") => {
                props::sometimes!(true, "game: an out-of-range host timestamp is rejected")
            }
            Some("timestamped batch") => {
                props::sometimes!(true, "game: a host timestamp on a batched Tick is rejected")
            }
            Some("volume range") => {
                props::sometimes!(true, "game: an out-of-range volume is rejected")
            }
            Some("unsupported key") => {
                props::sometimes!(true, "game: an unsupported key is rejected")
            }
            Some("text outside seed dialog") => {
                props::sometimes!(true, "game: text outside the seed dialog is rejected")
            }
            Some("invalid seed text") => {
                props::sometimes!(true, "game: invalid seed text is rejected")
            }
            _ => {}
        }
    }
}

/// The original's key-driven menu graph. Edges the port has yet to
/// implement (Continue, the pause menu's Exit) are included so that porting
/// them is not a failure. Key handlers move most menus at the key event;
/// Options acts on keys held at a tick.
fn menu_edge(from: Screen, to: Screen) -> bool {
    matches!(
        (from, to),
        (Screen::Main, Screen::Playing)
            | (Screen::Playing, Screen::Main)
            | (Screen::Title, Screen::Profile)
            | (Screen::Profile, Screen::Main)
            | (Screen::Profile, Screen::Title)
            | (Screen::Main, Screen::Profile)
            | (Screen::Main, Screen::Character)
            | (Screen::Main, Screen::Challenge)
            | (Screen::Challenge, Screen::Main)
            | (Screen::Main, Screen::Options)
            | (Screen::Options, Screen::Main)
            | (Screen::Character, Screen::Main)
            | (Screen::Character, Screen::Seed)
            | (Screen::Seed, Screen::Character)
    )
}

/// Contracts relating the state before and after an accepted command.
pub fn transition(before: &State, command: &Command, after: &State) {
    death(before, command, after);
    room_clear(before, command, after);
    // A shot sets the fire cooldown to the max fire delay it fires with, and
    // the cooldown then only counts down. The max delay itself may drop under
    // a running cooldown: a collectible queued during its pickup animation
    // flushes at 531f01, after the shot (52f..530) on the frame the animation
    // ends, so a shot fired with the old delay outlasts a lowered one.
    props::always!(
        after.fire_cooldown
            <= before
                .fire_cooldown
                .max(before.max_fire_delay)
                .max(after.max_fire_delay),
        "game: the fire cooldown never rises above a max fire delay the command held",
        json!({
            "before": [before.fire_cooldown, before.max_fire_delay],
            "after": [after.fire_cooldown, after.max_fire_delay],
        })
    );
    // A run starts from the menus, or restarts from the GameOver popup without
    // leaving the playing screen; both go through the engine's run start.
    let restarted = before.player_death.game_over.is_some()
        && after.screen == Screen::Playing
        && after.player_death.game_over.is_none();
    let menu_start = before.screen != Screen::Playing && after.screen == Screen::Playing;
    let run_started = menu_start || restarted;
    match command {
        Command::Tick {
            count,
            timestamp_millis,
        } => {
            let count = *count;
            wall_clock(before, count, *timestamp_millis, after);
            run_hold(before, count, after, run_started);
            let clock_delta = u64::from(after.system_clock.system_counter)
                - u64::from(before.system_clock.system_counter);
            props::always!(
                u64::from(after.tick) == u64::from(before.tick) + u64::from(count)
                    && (clock_delta == u64::from(count)
                        || (run_started && clock_delta == u64::from(count) + 1)),
                "game: Tick advances the tick and system clocks by its count (plus run-start alignment)",
                json!({ "count": count, "tick": [before.tick, after.tick], "clock": [before.system_clock, after.system_clock] })
            );
            let menu_clock_ok = if after.screen == before.screen {
                u64::from(after.menu_animation_tick)
                    == u64::from(before.menu_animation_tick) + u64::from(count)
            } else {
                after.menu_animation_tick == 0
            };
            props::always!(
                menu_clock_ok,
                "game: the menu animation clock advances with Tick and restarts on screen changes",
                json!({ "count": count, "before": before.menu_animation_tick, "after": after.menu_animation_tick })
            );
            // A Tick may leave gameplay (GameOver, and the endings once
            // ported). Between menus, a menu that polls its keys at a tick
            // (Options5A7840) moves along the key graph on a held key.
            props::always!(
                before.screen == after.screen
                    || run_started
                    || before.screen == Screen::Playing
                    || (menu_edge(before.screen, after.screen) && !before.pressed.is_empty()),
                "game: a Tick moves between menu screens only by starting a run or along the key graph on a held key",
                json!({ "before": before.screen, "after": after.screen, "pressed": before.pressed })
            );
            options(before, after);
            if before.screen == Screen::Playing
                && !before.paused
                && before.player_death.game_over.is_none()
                && after.player_death.game_over.is_none()
            {
                // The nightmare scene a stage transition requests (System
                // state 4, 5950a0) owns its ticks before the Game update and
                // its run clock; it is requested only as the stage advances.
                let scene = |state: &State| {
                    state.nightmares.system_active || state.nightmares.request_pending
                };
                let ticks =
                    u64::from(after.run_tick) - u64::from(before.run_tick.min(after.run_tick));
                props::always!(
                    after.run_tick >= before.run_tick
                        && if scene(before) || scene(after) || after.stage != before.stage {
                            ticks <= u64::from(count)
                        } else {
                            ticks == u64::from(count)
                        },
                    "game: unpaused gameplay advances the run clock once per tick, except while the nightmare scene plays",
                    json!({ "count": count, "before": before.run_tick, "after": after.run_tick })
                );
                if before.nightmares.system_active && after.nightmares.system_active {
                    props::always!(
                        after.run_tick == before.run_tick,
                        "game: the nightmare scene between stages holds the run clock",
                        json!({ "count": count, "before": before.run_tick, "after": after.run_tick })
                    );
                }
            }
            if before.screen == Screen::Playing && after.screen == Screen::Playing && !restarted {
                // Level::Init generates a floor's layout from its stage and
                // seed (docs/stage-progression.md); its rooms keep runtime
                // state of their own. A descent (SetNextStage, then Level::Init
                // up to a nightmare scene later) may skip a stage, as Curse of
                // the Labyrinth's XL floors do, and takes that stage's startup
                // seed. Forget Me Now and the five-pip Dice Room regenerate the
                // same stage under a new seed, so only a held stage and seed
                // pin the layout.
                let deeper = after.floor.stage > before.floor.stage;
                let descended = after.floor.stage == after.stage
                    && after.floor.stage_seed == after.stage_seed
                    && after.stage_seed == startup_stage_seed(after.run_seed_value, after.stage);
                let held = after.floor.stage == before.floor.stage
                    && after.floor.stage_seed == before.floor.stage_seed;
                props::always!(
                    after.game_update_count >= before.game_update_count
                        && after.stage >= before.stage
                        && after.floor.stage >= before.floor.stage
                        && (!deeper || descended)
                        && (!held || same_layout(&after.floor, &before.floor)),
                    "game: within a run the floor only deepens, keeps its layout while its stage and seed hold, and full updates never rewind",
                    json!({ "stage": [before.stage, after.stage], "stageSeed": [before.stage_seed, after.stage_seed] })
                );
                props::sometimes!(deeper, "game: a run descends to a deeper stage's floor");
            }
            // A Quit chosen from the pause menu fades the run out, and an
            // active stage transition keeps priority over that fade (54F9DA,
            // 54FB88), so a quitting run may still move.
            let quitting = |state: &State| {
                state.run_quit.active()
                    || state.run_quit.input_pending
                    || state.run_quit.system_pending
            };
            if before.paused && after.paused && count > 0 && !quitting(before) && !quitting(after) {
                // The shared stream is not frozen: Game::Render 54fcd0 runs
                // Room render 5d1a70 on every presented frame, pause menu or
                // not, and its light pass flickers lit fires with shared draws
                // (5d31b5, 5d31cb).
                let frozen = after.player == before.player
                    && after.tears == before.tears
                    && after.bombs == before.bombs
                    && after.room == before.room
                    && after.inventory == before.inventory
                    && after.current_room == before.current_room
                    && after.run_tick == before.run_tick;
                props::always!(
                    frozen,
                    "game: a paused run freezes gameplay and the run clock",
                    json!({ "count": count })
                );
                props::sometimes!(true, "game: ticks elapse while paused");
            }
            if menu_start {
                props::reachable!("game: a run starts through the public menus");
            }
            if run_started {
                let floor = rebirth_cleanroom::floor::generate_basement(seed::stage1_seed(
                    after.run_seed_value,
                ));
                // Held keys survive a GameOver restart, so a long tick can
                // carry the new run's player on through doors once its start
                // hold ends: the rooms it visits are a walk from the start.
                props::always!(
                    floor
                        .as_ref()
                        .is_ok_and(|floor| same_layout(floor, &after.floor))
                        && after.stage == 1
                        && after.stage_seed == startup_stage_seed(after.run_seed_value, 1)
                        && after.run_tick <= count
                        && walk_from_start(after),
                    "game: a run starts in the starting room of its seed's floor",
                    json!({
                        "runSeed": after.run_seed,
                        "currentRoom": after.current_room,
                        "visitedRooms": after.visited_rooms,
                    })
                );
                props::sometimes!(
                    after.current_room != after.floor.starting_descriptor,
                    "game: a new run's player walks on from the starting room within its first command"
                );
                props::sometimes!(after.seeded_run, "game: a seeded run starts");
                props::sometimes!(
                    before.character == "Random",
                    "game: a Random character run starts"
                );
            }
        }
        Command::Key { code, down } => {
            props::always!(
                after.tick == before.tick
                    && after.system_clock == before.system_clock
                    && after.run_tick == before.run_tick
                    && after.player == before.player
                    && after.boot_lifecycle.shared == before.boot_lifecycle.shared,
                "game: key events never advance simulation or consume the shared stream",
                json!({ "code": code, "down": down })
            );
            props::always!(
                after.pressed.contains(code) == *down,
                "game: key events update the held-key set",
                json!({ "code": code, "down": down })
            );
            if before.screen != after.screen {
                props::always!(
                    *down
                        && !before.pressed.contains(code)
                        && menu_edge(before.screen, after.screen),
                    "game: menu screens change only along the documented key graph",
                    json!({ "code": code, "from": before.screen, "to": after.screen })
                );
            }
            // Only the held-key set and per-command outputs may change.
            let ignored = || {
                let (mut ignored, mut earlier) = (after.clone(), before.clone());
                ignored.begin_command();
                earlier.begin_command();
                ignored.pressed = before.pressed.clone();
                ignored.stage_effects = before.stage_effects.clone();
                ignored == earlier
            };
            let request = &before.run_start_request;
            if before.screen != Screen::Playing && (request.input_pending || request.fading) {
                props::always!(
                    ignored(),
                    "game: menu inputs during the run-start fade are ignored",
                    json!({ "code": code })
                );
                props::sometimes!(*down, "game: a key arrives during the run-start fade");
            }
            // Game54f760 returns from its key handler while a boss Scene or
            // its cleanup is active.
            let boss_scene = before.screen == Screen::Playing
                && before.room_transition.as_ref().is_some_and(|transition| {
                    matches!(
                        transition.phase,
                        TransitionPhase::BossScene | TransitionPhase::BossCleanup
                    )
                });
            if boss_scene {
                props::always!(
                    ignored(),
                    "game: key presses during a boss scene are ignored",
                    json!({ "code": code, "transition": before.room_transition })
                );
                props::sometimes!(*down, "game: a key arrives during a boss scene");
            }
            if code == "Escape"
                && *down
                && !before.pressed.contains(code)
                && before.screen == Screen::Playing
                && !before.paused
                && !boss_scene
                && before.player_death.game_over.is_none()
            {
                if before.nightmares.system_active || before.nightmares.request_pending {
                    // Game54f760 returns while the nightmare scene owns System
                    // state 4, before any menu key handling.
                    props::always!(
                        !after.paused && after.pause_menu == before.pause_menu,
                        "game: Escape during the nightmare scene does not pause",
                        json!({ "nightmares": before.nightmares.system_active, "pause": after.pause_menu })
                    );
                } else if before.room_transition.is_some() {
                    // The original ignores Escape until the transition ends (#168).
                    props::always!(
                        !after.paused && after.pause_menu == before.pause_menu,
                        "game: Escape during a room transition does not pause",
                        json!({ "transition": before.room_transition, "pause": after.pause_menu })
                    );
                    props::sometimes!(true, "game: Escape arrives during a room transition");
                } else {
                    props::always!(
                        after.paused && after.pause_menu.phase == PausePhase::Appearing,
                        "game: Escape opens the pause menu",
                        json!({ "pause": after.pause_menu })
                    );
                    props::sometimes!(true, "game: the pause menu opens");
                }
            }
        }
        Command::Text { text } => {
            let compact: String = text.chars().filter(|glyph| *glyph != ' ').collect();
            props::always!(
                after.seed_text == format!("{}{}", before.seed_text, compact.to_ascii_uppercase()),
                "game: seed text appends its uppercase compact glyphs",
                json!({ "before": before.seed_text, "text": text, "after": after.seed_text })
            );
        }
        Command::AudioVolume { sfx_volume } => props::always!(
            after.audio.sfx_volume == *sfx_volume,
            "game: SFX volume commands set the SFX volume"
        ),
        Command::MusicVolume { volume } => props::always!(
            after.music.volume == *volume && after.music.music_enabled == (*volume > 0.0),
            "game: music volume commands set the music volume"
        ),
        Command::MusicBackend {
            channel,
            generation,
            ..
        } => {
            let current = before
                .music
                .channels
                .get(usize::from(*channel))
                .and_then(|channel| channel.generation);
            if current != Some(*generation) {
                // Events are each command's output and always restart.
                let mut unchanged = before.music.clone();
                unchanged.events = after.music.events.clone();
                props::always!(
                    after.music == unchanged,
                    "game: stale music backend callbacks are ignored",
                    json!({ "channel": channel, "generation": generation })
                );
                props::sometimes!(true, "game: a stale music backend callback arrives");
            } else {
                props::sometimes!(true, "game: a current music backend callback arrives");
            }
        }
    }
    let (held, now) = (&before.inventory, &after.inventory);
    props::sometimes!(
        now.coins > held.coins || now.keys > held.keys || now.bombs > held.bombs,
        "game: a pickup is collected"
    );
    if before.seed_text != after.seed_text || before.screen == Screen::Seed {
        props::sometimes!(
            before.screen == Screen::Seed
                && after.screen == Screen::Character
                && after.seeded_run
                && !before.seeded_run,
            "game: a valid seed code is committed through the dialog"
        );
        props::sometimes!(
            before.screen == Screen::Seed
                && after.screen == Screen::Character
                && !after.seeded_run
                && !before.seed_valid
                && !before.seed_text.is_empty(),
            "game: an invalid seed code is dismissed"
        );
    }
}

/// The visited rooms are the starting room and rooms reached from it
/// through visited neighbours, the current room among them.
fn walk_from_start(state: &State) -> bool {
    let start = state.floor.starting_descriptor;
    let visited = &state.visited_rooms;
    if !visited.contains(&start) || !visited.contains(&state.current_room) {
        return false;
    }
    let mut reached = BTreeSet::from([start]);
    let mut frontier = vec![start];
    while let Some(room) = frontier.pop() {
        let Some(descriptor) = state.floor.descriptors.get(room) else {
            return false;
        };
        for next in descriptor.neighbors.iter().flatten().copied() {
            if visited.contains(&next) && reached.insert(next) {
                frontier.push(next);
            }
        }
    }
    reached == *visited
}

/// Room.Update (`npcs::advance_room_clear`, source 5d149b/5d1487) runs once
/// per nonplayer pass: countable actors reset a ten-pass countdown, and only
/// an empty pass entered with the countdown expired marks the room clear and
/// reopens the doors combat shut. A bomb may reveal a secret door mid-combat.
fn room_clear(before: &State, command: &Command, after: &State) {
    let same_room = before.screen == Screen::Playing
        && after.screen == Screen::Playing
        && before.current_room == after.current_room
        && before.stage_seed == after.stage_seed
        && before.room_transition.is_none()
        && before.player_death.game_over.is_none();
    if !same_room {
        return;
    }
    let room_type = after.floor.descriptors[after.current_room]
        .template
        .room_type;
    let cleared = !before.room.combat_clear && after.room.combat_clear;
    if let (Command::Tick { count, .. }, true) = (command, cleared) {
        // The command's nonplayer passes, from the engine's own phase clock.
        let mut clock = before.system_clock;
        let passes = (0..*count)
            .filter(|_| {
                clock
                    .advance()
                    .is_ok_and(|phase| phase.updates_nonplayers())
            })
            .count();
        let countdown = before.room.combat_clear_countdown as usize;
        props::always!(
            passes > countdown
                && (passes > 1
                    || (countdown == 0 && after.room.manager_counts.countable_total() <= 0)),
            "game: a room is marked clear only on an empty pass after its countdown expires",
            json!({
                "room": after.current_room,
                "passes": passes,
                "countdown": countdown,
                "counts": after.room.manager_counts,
            })
        );
    }
    let reopened = before.room.appearance.doors.iter().filter(|door| {
        door.state == "Closed"
            && before.room.combat_shut_door_slots.contains(&door.slot)
            && after
                .room
                .appearance
                .doors
                .iter()
                .any(|after_door| after_door.slot == door.slot && after_door.state == "Opened")
    });
    let mut unexplained = reopened.filter(|door| {
        let target = door.target_room as usize;
        let connection = [
            after.current_room.min(target),
            after.current_room.max(target),
        ];
        !after.room.combat_clear && !after.revealed_secret_connections.contains(&connection)
    });
    props::always!(
        unexplained.next().is_none(),
        "game: combat-shut doors reopen only when the room clears or a secret is revealed",
        json!({ "room": after.current_room, "command": command })
    );
    props::sometimes!(cleared && room_type == 5, "game: a boss room is cleared");
    props::sometimes!(
        cleared && room_type == 6,
        "game: a miniboss room is cleared"
    );
}

/// Options5A7840 on a Tick. Left and Right step only the setting on the
/// selected row (SFX on row 1, music on row 2; Up and Down take priority, so
/// the row holds while a setting steps). Its back action saves the settings
/// it leaves with (Settings5B8DB0 writes %.4f) before returning to Main.
fn options(before: &State, after: &State) {
    if before.screen != Screen::Options {
        return;
    }
    let row = before.options_menu.selection;
    props::always!(
        (after.audio.sfx_volume == before.audio.sfx_volume || row == 1)
            && (after.music.volume == before.music.volume || row == 2),
        "game: an Options tick changes only the volume on its selected row",
        json!({
            "row": row,
            "sfx": [before.audio.sfx_volume, after.audio.sfx_volume],
            "music": [before.music.volume, after.music.volume],
        })
    );
    if after.screen == Screen::Main {
        let saved = after.options_menu.save_requests.last();
        props::always!(
            saved.is_some_and(|saved| {
                saved.sfx_volume == format!("{:.4}", after.audio.sfx_volume)
                    && saved.music_volume == format!("{:.4}", after.music.volume)
                    && saved.fullscreen == after.fullscreen.requested
            }),
            "game: leaving Options saves the settings it leaves with",
            json!({
                "saved": saved,
                "sfx": after.audio.sfx_volume,
                "music": after.music.volume,
                "fullscreen": after.fullscreen.requested,
            })
        );
        props::sometimes!(true, "game: Options saves and returns to the main menu");
    }
}

/// Normal Isaac death (`engine/src/player_death.rs`): GameOver freezes
/// gameplay (Game54f760 returns before the run tick), the menu key handler
/// ignores input, and the released System popup's Restart or Exit request is
/// consumed by the next System dispatch.
fn death(before: &State, command: &Command, after: &State) {
    props::sometimes!(
        !before.player.dead && after.player.dead,
        "game: the player dies"
    );
    props::sometimes!(
        before.player_death.game_over.is_none() && after.player_death.game_over.is_some(),
        "game: a death reaches GameOver"
    );
    if before.screen != Screen::Playing || before.player_death.game_over.is_none() {
        return;
    }
    match command {
        Command::Key { .. } => props::always!(
            after.screen == before.screen && after.player_death == before.player_death,
            "game: keys never navigate while GameOver shows",
            json!({ "command": command, "before": before.player_death, "after": after.player_death })
        ),
        Command::Tick { count, .. } if *count > 0 => {
            if after.player_death.game_over.is_some() {
                props::always!(
                    after.run_tick == before.run_tick
                        && after.player.x == before.player.x
                        && after.player.y == before.player.y,
                    "game: GameOver freezes gameplay",
                    json!({ "runTick": [before.run_tick, after.run_tick], "count": count })
                );
                return;
            }
            let restarted = after.screen == Screen::Playing && !after.player.dead;
            let exited = after.screen == Screen::Main
                && after.player_death == player_death::PlayerDeath::default();
            props::always!(
                restarted || exited,
                "game: GameOver ends only by Restart or Exit",
                json!({
                    "pending": before.player_death.system_pending,
                    "screen": after.screen,
                    "dead": after.player.dead,
                })
            );
            props::sometimes!(restarted, "game: GameOver restarts the run");
            props::sometimes!(exited, "game: GameOver exits to the main menu");
        }
        _ => {}
    }
}

/// A host timestamp sets the elapsed clock; otherwise each step adds the
/// declared presentation interval in exact nanoseconds.
fn wall_clock(before: &State, count: u32, timestamp: Option<u64>, after: &State) {
    let (start, end) = (&before.run_admission.clock, &after.run_admission.clock);
    let expected = match timestamp {
        Some(timestamp) => (i128::from(timestamp), 0),
        None => {
            let nanoseconds = i128::from(start.milliseconds) * 1_000_000
                + i128::from(start.sub_millisecond_nanoseconds)
                + i128::from(count) * i128::from(PRESENTATION_INTERVAL_NS);
            (nanoseconds / 1_000_000, nanoseconds % 1_000_000)
        }
    };
    props::always!(
        (
            i128::from(end.milliseconds),
            i128::from(end.sub_millisecond_nanoseconds)
        ) == expected
            && end.nanoseconds_per_step == PRESENTATION_INTERVAL_NS,
        "game: the elapsed clock follows host timestamps or exact presentation intervals",
        json!({ "count": count, "timestamp": timestamp, "before": start, "after": end })
    );
    props::sometimes!(timestamp.is_some(), "game: a host timestamp drives a tick");
}

/// After run start, full updates hold until the host clock passes the
/// confirmation anchor by 3 s, then fade over 25 full updates.
fn run_hold(before: &State, count: u32, after: &State, run_started: bool) {
    let (held, now) = (&before.run_admission, &after.run_admission);
    if run_started || before.screen != Screen::Playing {
        return;
    }
    let past_anchor = now
        .requested_time_anchor_ms
        .is_some_and(|anchor| now.clock.milliseconds.saturating_sub(anchor) > 3000);
    props::always!(
        now.initial_fade <= held.initial_fade
            && (now.initial_fade == held.initial_fade || past_anchor),
        "game: the run-start fade only falls, and only past the 3 s clock anchor",
        json!({ "count": count, "before": held, "after": now })
    );
    if held.initial_fade > 0.0 && now.initial_fade > 0.0 && !before.paused {
        props::always!(
            after.game_update_count == before.game_update_count,
            "game: the run-start hold blocks full game updates",
            json!({ "count": count, "fade": now.initial_fade })
        );
        props::sometimes!(
            now.clock.milliseconds < held.clock.milliseconds,
            "game: the host clock rolls back during the run-start hold"
        );
    }
    props::sometimes!(
        held.initial_fade > 0.0 && now.initial_fade == 0.0,
        "game: the run-start fade completes and controls return"
    );
}

/// Invariants of every published state.
pub fn state(state: &State) {
    // Every "unchanged" and "same game" comparison relies on this; a field
    // that can hold NaN breaks it.
    props::always!(
        PartialEq::eq(state, state),
        "game: state equality is reflexive",
        json!({ "screen": state.screen })
    );
    let details = || json!({ "screen": state.screen, "tick": state.tick });
    let catalog = state.character_catalog.len();
    props::always!(
        (1..=3).contains(&state.profile)
            && state.main_selection < 5
            && state.options_menu.selection < 8
            && usize::from(state.character_selection) < catalog
            && (state.difficulty == "normal" || state.difficulty == "hard")
            && state.profiles.len() == 3,
        "game: menu selections stay within their menus",
        json!({ "profile": state.profile, "main": state.main_selection, "options": state.options_menu.selection, "character": state.character_selection, "difficulty": state.difficulty })
    );
    if state.screen != Screen::Playing {
        props::always!(
            state.character_catalog[usize::from(state.character_selection) % catalog.max(1)].name
                == state.character,
            "game: the highlighted character is the selected character",
            json!({ "selection": state.character_selection, "character": state.character })
        );
    }
    props::always!(
        state.seed_text.len() <= 8
            && state
                .seed_text
                .bytes()
                .all(|glyph| glyph.is_ascii_uppercase() || glyph.is_ascii_digit())
            && state.seed_valid == seed::decode(&state.seed_text).is_ok(),
        "game: dialog seed text is at most eight uppercase glyphs and its validity is its decode",
        json!({ "text": state.seed_text, "valid": state.seed_valid })
    );
    // Seeds::SetStartSeed5e1470 gives stage s the (s+1)-th startup draw
    // (stage 1's is `stage1_seed`); a run start and a descent take it (see
    // `transition`), and a forgotten stage seed may replace it later. A run's
    // floor is its stage's; between SetNextStage and Level::Init an active
    // stage transition still holds the previous stage's floor. The menus
    // keep the last run's stage and floor until the next run starts.
    let transition = state.stage_transition.active();
    let level_stage = if transition && state.floor.stage < state.stage {
        state.floor.stage
    } else {
        state.stage
    };
    props::always!(
        seed::encode(state.run_seed_value).as_deref() == Ok(state.run_seed.as_str())
            && startup_stage_seed(state.run_seed_value, 1)
                == seed::stage1_seed(state.run_seed_value)
            && (state.screen != Screen::Playing
                || (state.floor.stage == level_stage
                    && state.floor.stage_seed == state.stage_seed))
            && state.seeded_run != state.achievement_eligibility,
        "game: the run seed code, value, stage seeds, floor and eligibility agree",
        json!({ "screen": state.screen, "runSeed": state.run_seed, "value": state.run_seed_value, "stage": state.stage, "stageSeed": state.stage_seed, "floor": [state.floor.stage, state.floor.stage_seed], "transition": state.stage_transition })
    );
    props::always!(
        state.pressed.iter().all(|code| input::is_public_key(code)),
        "game: only supported keys are ever held",
        json!({ "pressed": state.pressed })
    );
    let pause = &state.pause_menu;
    let pause_ok = pause.selection < 3
        && pause.age <= 22
        && pause.age.is_multiple_of(2)
        && if state.screen == Screen::Playing {
            state.paused == (pause.phase != PausePhase::Hidden)
        } else {
            !state.paused && pause.phase == PausePhase::Hidden
        };
    props::always!(
        pause_ok,
        "game: pause state agrees with the pause menu phase",
        json!({ "paused": state.paused, "pause": pause })
    );
    props::always!(
        (0.0..=1.0).contains(&state.audio.sfx_volume) && (0.0..=1.0).contains(&state.music.volume),
        "game: volumes stay within the unit interval"
    );
    let introduction = &state.run_introduction;
    props::always!(
        introduction.age <= 140 && (!introduction.active || introduction.age < 140),
        "game: the run introduction lasts at most 140 ticks",
        json!({ "introduction": introduction })
    );
    let admission = &state.run_admission;
    // Player+7d4 is cleared by the run-start fade, by every doorway or
    // boss-scene transition and by StageTransition::Start (5e3800). Other
    // source holds may clear it once ported, so only this direction is
    // pinned; the simulation checks that lost controls come back.
    let held_but_controlled =
        state.screen == Screen::Playing && controls_hold(state) && state.player.controls_enabled;
    props::always!(
        (0.0..=1.0).contains(&admission.initial_fade)
            && (admission.fade_rate == 0.0 || admission.fade_rate == 0.04)
            && admission.clock.sub_millisecond_nanoseconds < 1_000_000
            && (state.screen != Screen::Playing || admission.requested_time_anchor_ms.is_some())
            && !held_but_controlled,
        "game: player controls are disabled while the run-start fade, a room transition or a stage transition holds",
        json!({ "admission": admission, "controls": state.player.controls_enabled, "transition": state.room_transition, "stageTransition": state.stage_transition })
    );
    let request = &state.run_start_request;
    props::always!(
        state.screen != Screen::Playing || !(request.input_pending || request.system_pending),
        "game: no run-start request is pending during gameplay",
        json!({ "request": request })
    );
    if state.screen != Screen::Playing {
        return;
    }
    let player = &state.player;
    props::always!(
        [player.x, player.y, player.vx, player.vy]
            .iter()
            .all(|value| value.is_finite())
            && player.hp <= player.max_hp
            && player.max_hp <= 24
            && player.soul_hp >= 0,
        "game: player motion is finite and health is bounded",
        json!({ "player": player })
    );
    // Grid collision reflects a body's motion into solid cells, so the centre
    // of a colliding NPC, pickup or bomb never moves out of the room interior
    // (`Grid::clamp_room`), whatever its speed or family. The ground solver
    // (443117) never moves a body at rest, so one born where the player
    // stands in an open doorway (a bomb dropped there) rests on the door
    // cell; the boundary walls and the space beyond them stay out of reach.
    if state.room_transition.is_none() {
        let grid = state
            .room
            .grid
            .clone()
            .map(rebirth_cleanroom::grid::Grid::new);
        if let Some(Ok(grid)) = grid {
            let npcs = state.room.npcs.iter().filter(|npc| !npc.removed);
            let npcs = npcs
                .filter(|npc| npc.grid_collision_class != 0)
                .map(|npc| ("npc", npc.id, npc.position));
            let pickups = state.room.pickups.iter().filter(|pickup| !pickup.removed);
            let pickups = pickups.map(|pickup| ("pickup", pickup.id, pickup.position));
            let bombs = state.bombs.iter();
            let bombs = bombs.map(|bomb| ("bomb", bomb.id, bomb.motion.position));
            let doorway = |position: WorldVector| {
                grid.index(position)
                    .and_then(|index| grid.descriptor().cells[index].entity.as_ref())
                    .is_some_and(|entity| entity.kind == 16)
            };
            let escaped = npcs.chain(pickups).chain(bombs).find(|(_, _, position)| {
                grid.clamp_room(*position, 0.0) != *position && !doorway(*position)
            });
            props::always!(
                escaped.is_none(),
                "game: grid-colliding bodies stay inside the room's walls",
                json!({ "body": escaped })
            );
        }
    }
    let unfinite = unfinite_body(state);
    props::always!(
        unfinite.is_none(),
        "game: every room body has finite motion",
        json!({ "body": unfinite })
    );
    props::always!(
        state.inventory.coins <= 99 && state.inventory.bombs <= 99 && state.inventory.keys <= 99,
        "game: consumables are capped at 99",
        json!({ "inventory": state.inventory })
    );
    props::always!(
        state.fire_cooldown >= -1 && state.head_timer >= -1,
        "game: fire cooldown and head timers stay in range",
        json!({ "fireCooldown": state.fire_cooldown, "headTimer": state.head_timer })
    );

    // Identities are unique and below their allocators.
    fn allocated(ids: impl Iterator<Item = u32>, next: u32) -> bool {
        let mut seen = BTreeSet::new();
        ids.into_iter().all(|id| id < next && seen.insert(id))
    }
    let persisted_pickups = state
        .persisted_rooms
        .values()
        .flat_map(|room| room.pickups.iter().map(|pickup| pickup.id))
        .all(|id| id < state.next_pickup);
    props::always!(
        allocated(state.tears.iter().map(|tear| tear.id), state.next_tear)
            && allocated(state.bombs.iter().map(|bomb| bomb.id), state.next_bomb)
            && allocated(
                state.knives.iter().map(|knife| knife.id),
                state.attack_runtime.next_knife
            )
            && allocated(
                state.room.pickups.iter().map(|pickup| pickup.id),
                state.next_pickup
            )
            && persisted_pickups
            && allocated(
                state
                    .enemy_projectiles
                    .iter()
                    .map(|projectile| projectile.id),
                state.next_enemy_projectile
            )
            && allocated(
                state.familiars.iter().map(|familiar| familiar.id),
                state.next_familiar
            )
            && allocated(state.lasers.iter().map(|laser| laser.id), state.next_laser)
            && allocated(
                state
                    .fire_particles
                    .iter()
                    .map(|particle| particle.id)
                    .chain(state.sprite_effects.iter().map(|effect| effect.id))
                    .chain(laser_impacts(state)),
                state.next_effect
            ),
        "game: entity identities are unique and below their allocators",
        details()
    );
    let npcs_unique = {
        let mut seen = BTreeSet::new();
        state.room.npcs.iter().all(|npc| seen.insert(npc.id))
    };
    props::always!(
        npcs_unique,
        "game: room NPC identities are unique",
        details()
    );
    // Coverage the laser and miniboss contracts depend on.
    props::sometimes!(
        state.screen == Screen::Playing
            && state.floor.descriptors[state.current_room]
                .template
                .room_type
                == 6,
        "game: the player is in a miniboss room"
    );
    props::sometimes!(!state.lasers.is_empty(), "game: a laser is live");
    if state.screen == Screen::Playing {
        let room_type = state
            .floor
            .descriptors
            .get(state.current_room)
            .map(|room| room.template.room_type);
        props::sometimes!(
            matches!(room_type, Some(7 | 8)),
            "game: the player is in a secret room"
        );
        props::sometimes!(
            !state.revealed_secret_connections.is_empty(),
            "game: a secret door is revealed"
        );
        props::sometimes!(
            state
                .player_items
                .as_ref()
                .is_some_and(|items| items.owned_counts.iter().any(|count| *count > 0)),
            "game: the player owns a collectible"
        );
    }
    // Every engine path that adds consumables clamps at 99, as the original
    // HUD's two digits do.
    props::always!(
        state.inventory.coins <= 99 && state.inventory.bombs <= 99 && state.inventory.keys <= 99,
        "game: consumable counts stay within 99",
        json!({ "inventory": state.inventory })
    );

    // Every scheduled handle names one live entity of its kind.
    let scheduled = state.actor_scheduler.full_phase();
    let unique = {
        let mut seen = BTreeSet::new();
        scheduled
            .iter()
            .all(|actor| seen.insert((actor.kind as u8, actor.id)))
    };
    let live = scheduled.iter().all(|actor| match actor.kind {
        ActorKind::Player => actor.id == 0,
        ActorKind::Familiar => state
            .familiars
            .iter()
            .any(|familiar| familiar.id == actor.id),
        ActorKind::Tear => state.tears.iter().any(|tear| tear.id == actor.id),
        ActorKind::Bomb => state.bombs.iter().any(|bomb| bomb.id == actor.id),
        ActorKind::Knife => state.knives.iter().any(|knife| knife.id == actor.id),
        ActorKind::Pickup => state
            .room
            .pickups
            .iter()
            .any(|pickup| pickup.id == actor.id),
        ActorKind::Npc => state.room.npcs.iter().any(|npc| npc.id == actor.id),
        ActorKind::Projectile => state
            .enemy_projectiles
            .iter()
            .any(|projectile| projectile.id == actor.id),
        ActorKind::Laser => state.lasers.iter().any(|laser| laser.id == actor.id),
        ActorKind::Effect => {
            state
                .fire_particles
                .iter()
                .any(|particle| particle.id == actor.id)
                || state
                    .sprite_effects
                    .iter()
                    .any(|effect| effect.id == actor.id)
                || laser_impacts(state).any(|id| id == actor.id)
        }
    });
    let player_scheduled = scheduled
        .iter()
        .any(|actor| actor.kind == ActorKind::Player);
    let tears_and_bombs_scheduled = state.tears.iter().all(|tear| {
        scheduled
            .iter()
            .any(|actor| actor.kind == ActorKind::Tear && actor.id == tear.id)
    }) && state.bombs.iter().all(|bomb| {
        scheduled
            .iter()
            .any(|actor| actor.kind == ActorKind::Bomb && actor.id == bomb.id)
    });
    props::always!(
        unique && live && player_scheduled && tears_and_bombs_scheduled,
        "game: the actor schedule names exactly the live player, tears, bombs and room actors",
        json!({ "scheduled": scheduled, "tears": state.tears.iter().map(|tear| tear.id).collect::<Vec<_>>(), "bombs": state.bombs.iter().map(|bomb| bomb.id).collect::<Vec<_>>() })
    );

    // Floor and room bookkeeping index the generated floor.
    let rooms = state.floor.descriptors.len();
    props::always!(
        state.current_room < rooms
            && state.room_display_flags.len() == rooms
            && state.visited_rooms.contains(&state.current_room)
            && state.visited_rooms.iter().all(|room| *room < rooms)
            && state.persisted_rooms.keys().all(|room| *room < rooms)
            && state
                .revealed_secret_connections
                .iter()
                .all(|[a, b]| *a < rooms && *b < rooms),
        "game: room bookkeeping indexes the run's floor",
        json!({ "current": state.current_room, "rooms": rooms })
    );
    match &state.room_transition {
        Some(transition) => {
            // A transition through one of the room's doors leads to the
            // neighbor behind it. Teleports (Teleport!, the Fool card, the
            // I AM ERROR room) change rooms without a door once ported.
            props::always!(
                transition.from_room < rooms
                    && transition.target_room < rooms
                    && state.floor.descriptors[transition.from_room]
                        .neighbors
                        .get(usize::from(transition.from_slot))
                        .is_none_or(|neighbor| {
                            neighbor.is_none_or(|room| room == transition.target_room)
                        }),
                "game: a door transition leads to the neighbor behind its door",
                json!({ "transition": transition })
            );
            props::sometimes!(
                transition.phase == TransitionPhase::In,
                "game: a room transition reaches its inbound phase"
            );
        }
        None => {
            props::always!(
                state
                    .room
                    .descriptor
                    .as_ref()
                    .is_some_and(|descriptor| descriptor.index == state.current_room),
                "game: the loaded room is the current descriptor's room",
                json!({ "current": state.current_room, "loaded": state.room.descriptor.as_ref().map(|descriptor| descriptor.index) })
            );
        }
    }
    let current_cells: Vec<_> = state
        .minimap
        .cells
        .iter()
        .filter(|cell| cell.visibility == "current")
        .collect();
    let descriptor = &state.floor.descriptors[state.current_room.min(rooms.saturating_sub(1))];
    let expected = (
        descriptor.grid_index as i32 % 13 - 6,
        descriptor.grid_index as i32 / 13 - 6,
    );
    // A minimap may mark a large room by its origin cell or its whole
    // footprint; either way only cells of the current room are current.
    let (columns, rows) = (
        (i32::from(descriptor.template.width) / 13).max(1),
        (i32::from(descriptor.template.height) / 7).max(1),
    );
    props::always!(
        !current_cells.is_empty()
            && current_cells.iter().all(|cell| {
                (expected.0..expected.0 + columns).contains(&cell.x)
                    && (expected.1..expected.1 + rows).contains(&cell.y)
            }),
        "game: the minimap marks the current room and no other",
        json!({ "cells": state.minimap.cells, "expected": expected })
    );
    props::sometimes!(
        state.current_room != state.floor.starting_descriptor,
        "game: the player enters another room"
    );
    props::sometimes!(!state.tears.is_empty(), "game: a tear is in flight");
    // Tear.Update 542530's dead branch spawns a sized TearPoof effect (wall
    // 12, ground 13, small 0x4f/0x50) where the renderer once drew an impact.
    props::sometimes!(
        state
            .sprite_effects
            .iter()
            .any(|effect| matches!(effect.variant, 12 | 13 | 0x4f | 0x50)),
        "game: a tear poof is presented"
    );
    props::sometimes!(!state.bombs.is_empty(), "game: a bomb is live");
    props::sometimes!(!state.explosions.is_empty(), "game: a bomb explodes");
    props::sometimes!(
        state.player.hp < state.player.max_hp,
        "game: the player takes damage"
    );
    props::sometimes!(
        state
            .room
            .pickups
            .iter()
            .any(|pickup| !pickup.removed && !pickup.collected),
        "game: the player shares a room with a collectible pickup"
    );
    props::sometimes!(
        state.room.npcs.iter().any(|npc| !npc.removed),
        "game: the player shares a room with an NPC"
    );
    props::sometimes!(
        state.visited_rooms.len() >= 3,
        "game: three rooms are visited in one run"
    );
    props::sometimes!(
        state.room.grid.as_ref().is_some_and(|grid| {
            grid.cells.iter().any(|cell| {
                cell.entity
                    .as_ref()
                    .is_some_and(|entity| entity.kind == 2 && entity.state == 2)
            })
        }),
        "game: a bomb destroys an ordinary rock"
    );
}

/// Laser impacts are effects whose handles come from the effect allocator.
/// The first body whose motion is NaN or infinite, as (kind, id). One bad
/// value (a normalized zero vector, say) spreads through every later
/// contact, so every family's bodies are checked, new families included.
fn unfinite_body(state: &State) -> Option<(&'static str, u32)> {
    let finite = |vectors: &[WorldVector], scalars: &[f64]| {
        vectors.iter().all(|vector| vector.is_finite())
            && scalars.iter().all(|value| value.is_finite())
    };
    let npcs = state.room.npcs.iter().map(|npc| {
        let ok = finite(
            &[npc.position, npc.presented_position, npc.velocity],
            &[f64::from(npc.hp), f64::from(npc.max_hp)],
        );
        ("npc", npc.id, ok)
    });
    let pickups = state.room.pickups.iter().map(|pickup| {
        let vectors = [pickup.position, pickup.presented_position, pickup.velocity];
        ("pickup", pickup.id, finite(&vectors, &[pickup.x, pickup.y]))
    });
    let tears = state.tears.iter().map(|tear| {
        let motion = &tear.motion;
        let vectors = [motion.position, motion.presented_position, motion.velocity];
        let scalars = [tear.x, tear.y, tear.vx, tear.vy];
        ("tear", tear.id, finite(&vectors, &scalars))
    });
    let projectiles = state.enemy_projectiles.iter().map(|shot| {
        let vectors = [shot.position, shot.presented_position, shot.velocity];
        ("enemy projectile", shot.id, finite(&vectors, &[]))
    });
    let familiars = state.familiars.iter().map(|familiar| {
        let vectors = [
            familiar.position,
            familiar.presented_position,
            familiar.velocity,
        ];
        let health = [f64::from(familiar.hp), f64::from(familiar.max_hp)];
        ("familiar", familiar.id, finite(&vectors, &health))
    });
    let bombs = state.bombs.iter().map(|bomb| {
        let motion = &bomb.motion;
        let vectors = [motion.position, motion.presented_position, motion.velocity];
        ("bomb", bomb.id, finite(&vectors, &[bomb.x, bomb.y]))
    });
    let lasers = state.lasers.iter().map(|laser| {
        (
            "laser",
            laser.id,
            finite(&[laser.position, laser.velocity], &[]),
        )
    });
    let knives = state.knives.iter().map(|knife| {
        (
            "knife",
            knife.id,
            finite(&[knife.position, knife.velocity], &[]),
        )
    });
    npcs.chain(pickups)
        .chain(tears)
        .chain(projectiles)
        .chain(familiars)
        .chain(bombs)
        .chain(lasers)
        .chain(knives)
        .find(|(_, _, ok)| !ok)
        .map(|(kind, id, _)| (kind, id))
}

fn laser_impacts(state: &State) -> impl Iterator<Item = u32> + '_ {
    state
        .lasers
        .iter()
        .filter_map(|laser| laser.impact.as_ref().map(|impact| impact.id))
}
