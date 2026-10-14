//! Per-step state invariants and per-command transition contracts.
use super::driver::SUPPORTED_KEYS;
use crate::props;
use rebirth_cleanroom::actors::ActorKind;
use rebirth_cleanroom::run_admission::{MAX_SAFE_TIMESTAMP_MS, PRESENTATION_INTERVAL_NS};
use rebirth_cleanroom::traversal::TransitionPhase;
use rebirth_cleanroom::{seed, Command, PausePhase, Screen, State, MAX_COMMAND_TICKS};
use serde_json::json;
use std::collections::BTreeSet;

/// Inputs the engine documents as rejected (state unchanged, error
/// returned). Everything else must be accepted.
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
        Command::Key { code, .. } if !SUPPORTED_KEYS.contains(&code.as_str()) => {
            Some("unsupported key")
        }
        Command::Key { code, down } => {
            if !*down || state.pressed.contains(code) {
                return None;
            }
            let request = &state.run_start_request;
            if state.screen != Screen::Playing && (request.input_pending || request.fading) {
                return None;
            }
            match (state.screen, code.as_str()) {
                (Screen::Main, "Enter") if state.main_selection != 0 => {
                    Some("unimplemented main menu action")
                }
                (Screen::Character, "Enter") if state.difficulty != "normal" => {
                    Some("unimplemented hard mode")
                }
                (Screen::Character, "Enter") if !startable(state) => Some("unstartable character"),
                _ => None,
            }
        }
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

/// Fresh profiles unlock Isaac only, and only Isaac's runtime exists; Random
/// resolves among the profile's unlocked characters.
fn startable(state: &State) -> bool {
    let unlocked = &state.profiles[usize::from(state.profile - 1)].unlocked_characters;
    if state.character == "Random" {
        !unlocked.is_empty() && unlocked.iter().all(|character| character == "Isaac")
    } else {
        state.character == "Isaac" && unlocked.contains(&state.character)
    }
}

pub fn outcome(state: &State, command: &Command, result: &Result<(), String>) {
    let expected = expected_rejection(state, command);
    let details = || json!({ "command": command, "result": result, "expected": expected });
    props::always!(
        result.is_err() == expected.is_some(),
        "game: exactly the documented invalid inputs are rejected",
        details()
    );
    if matches!(command, Command::Tick { .. }) {
        props::always!(
            expected.is_some() || result.is_ok(),
            "game: valid Tick commands always succeed",
            details()
        );
    }
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
            Some("unimplemented main menu action") => {
                props::sometimes!(true, "game: an unimplemented main menu action is rejected")
            }
            Some("unimplemented hard mode") => {
                props::sometimes!(true, "game: an unimplemented hard-mode run is rejected")
            }
            Some("unstartable character") => {
                props::sometimes!(
                    true,
                    "game: a locked or unimplemented character is rejected"
                )
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

fn menu_edge(from: Screen, to: Screen) -> bool {
    matches!(
        (from, to),
        (Screen::Title, Screen::Profile)
            | (Screen::Profile, Screen::Main)
            | (Screen::Profile, Screen::Title)
            | (Screen::Main, Screen::Profile)
            | (Screen::Main, Screen::Character)
            | (Screen::Character, Screen::Main)
            | (Screen::Character, Screen::Seed)
            | (Screen::Seed, Screen::Character)
    )
}

/// Contracts relating the state before and after an accepted command.
pub fn transition(before: &State, command: &Command, after: &State) {
    let run_started = before.screen != Screen::Playing && after.screen == Screen::Playing;
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
            props::always!(
                before.screen == after.screen || run_started,
                "game: Tick changes the screen only by starting a run",
                json!({ "before": before.screen, "after": after.screen })
            );
            if before.screen == Screen::Playing && !before.paused {
                props::always!(
                    u64::from(after.run_tick) == u64::from(before.run_tick) + u64::from(count),
                    "game: unpaused gameplay advances the run clock once per tick",
                    json!({ "count": count, "before": before.run_tick, "after": after.run_tick })
                );
            }
            if before.screen == Screen::Playing && after.screen == Screen::Playing {
                props::always!(
                    after.game_update_count >= before.game_update_count
                        && after.floor == before.floor,
                    "game: within a run the floor is fixed and full updates never rewind"
                );
            }
            if before.paused && after.paused && count > 0 {
                let frozen = after.player == before.player
                    && after.tears == before.tears
                    && after.bombs == before.bombs
                    && after.room == before.room
                    && after.inventory == before.inventory
                    && after.current_room == before.current_room
                    && after.run_tick == before.run_tick
                    && after.boot_lifecycle.shared == before.boot_lifecycle.shared;
                props::always!(
                    frozen,
                    "game: a paused run freezes gameplay and the shared stream",
                    json!({ "count": count })
                );
                props::sometimes!(true, "game: ticks elapse while paused");
            }
            if run_started {
                props::reachable!("game: a run starts through the public menus");
                let floor = rebirth_cleanroom::floor::generate_basement(seed::stage1_seed(
                    after.run_seed_value,
                ));
                props::always!(
                    floor.as_ref().ok() == Some(&after.floor)
                        && after.current_room == after.floor.starting_descriptor
                        && after.run_tick <= count
                        && after.visited_rooms == BTreeSet::from([after.current_room]),
                    "game: a run starts in the starting room of its seed's floor",
                    json!({ "runSeed": after.run_seed })
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
                let mut ignored = after.clone();
                ignored.pressed = before.pressed.clone();
                ignored.audio.events = before.audio.events.clone();
                ignored.music.events = before.music.events.clone();
                ignored.stage_effects = before.stage_effects.clone();
                ignored == *before
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
            {
                props::always!(
                    after.paused && after.pause_menu.phase == PausePhase::Appearing,
                    "game: Escape opens the pause menu",
                    json!({ "transition": before.room_transition, "pause": after.pause_menu })
                );
                props::sometimes!(true, "game: the pause menu opens");
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
    let details = || json!({ "screen": state.screen, "tick": state.tick });
    let catalog = state.character_catalog.len();
    props::always!(
        (1..=3).contains(&state.profile)
            && state.main_selection < 5
            && usize::from(state.character_selection) < catalog
            && (state.difficulty == "normal" || state.difficulty == "hard")
            && state.profiles.len() == 3,
        "game: menu selections stay within their menus",
        json!({ "profile": state.profile, "main": state.main_selection, "character": state.character_selection, "difficulty": state.difficulty })
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
    props::always!(
        seed::encode(state.run_seed_value).as_deref() == Ok(state.run_seed.as_str())
            && state.stage_seed == seed::stage1_seed(state.run_seed_value)
            && state.seeded_run != state.achievement_eligibility,
        "game: the run seed code, value, stage seed and eligibility agree",
        json!({ "runSeed": state.run_seed, "value": state.run_seed_value, "stage": state.stage_seed })
    );
    props::always!(
        state
            .pressed
            .iter()
            .all(|code| SUPPORTED_KEYS.contains(&code.as_str())),
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
    // Player+7d4 is cleared by the run-start fade and by every doorway or
    // boss-scene transition, and set again when both have finished.
    let controls_follow_holds = if state.screen == Screen::Playing {
        state.player.controls_enabled
            == (admission.initial_fade == 0.0 && state.room_transition.is_none())
            && admission.requested_time_anchor_ms.is_some()
    } else {
        state.player.controls_enabled && admission.initial_fade == 0.0
    };
    props::always!(
        (0.0..=1.0).contains(&admission.initial_fade)
            && (admission.fade_rate == 0.0 || admission.fade_rate == 0.04)
            && admission.clock.sub_millisecond_nanoseconds < 1_000_000
            && controls_follow_holds,
        "game: player controls are disabled exactly while the run-start fade or a room transition holds",
        json!({ "admission": admission, "controls": state.player.controls_enabled, "transition": state.room_transition })
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
    props::always!(
        state.inventory.coins <= 99 && state.inventory.bombs <= 99 && state.inventory.keys <= 99,
        "game: consumables are capped at 99",
        json!({ "inventory": state.inventory })
    );
    props::always!(
        state.fire_cooldown >= -1
            && state.fire_cooldown <= state.max_fire_delay
            && state.head_timer >= -1,
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
            && allocated(
                state
                    .fire_particles
                    .iter()
                    .map(|particle| particle.id)
                    .chain(state.sprite_effects.iter().map(|effect| effect.id)),
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
        ActorKind::Effect => {
            state
                .fire_particles
                .iter()
                .any(|particle| particle.id == actor.id)
                || state
                    .sprite_effects
                    .iter()
                    .any(|effect| effect.id == actor.id)
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
            props::always!(
                transition.from_room < rooms
                    && transition.target_room < rooms
                    && state.floor.descriptors[transition.from_room]
                        .neighbors
                        .contains(&Some(transition.target_room)),
                "game: room transitions follow floor neighbor links",
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
    props::always!(
        current_cells.len() == 1 && (current_cells[0].x, current_cells[0].y) == expected,
        "game: the minimap marks exactly the current room",
        json!({ "cells": state.minimap.cells, "expected": expected })
    );
    props::sometimes!(
        state.current_room != state.floor.starting_descriptor,
        "game: the player enters another room"
    );
    props::sometimes!(!state.tears.is_empty(), "game: a tear is in flight");
    props::sometimes!(
        !state.impacts.is_empty(),
        "game: a tear impact is presented"
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
