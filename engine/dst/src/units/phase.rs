//! System iteration phases (`engine/src/phase.rs`) against a counter model.
use crate::harness::Case;
use crate::props;
use rebirth_cleanroom::phase::{Clock, Phase};
use serde_json::json;

pub fn run(case: &mut Case) {
    let e = &mut *case.entropy;
    for _ in 0..(case.steps / 64).max(16) {
        let start = match e.below(4) {
            0 => u32::MAX - e.range(0, 8),
            1 => e.range(0, 8),
            _ => e.u32(),
        };
        let mut clock = Clock::new(start, e.chance(3, 4));
        let mut model = u64::from(start);
        let mut window_player_updates = 0u32;
        let mut window_nonplayer_updates = 0u32;
        let mut window_phases = 0u32;
        for _ in 0..e.range(1, 64) {
            let before = clock;
            if e.chance(1, 8) {
                let result = clock.align_run_start();
                if model % 2 == 1 && model == u64::from(u32::MAX) {
                    props::always!(
                        result.is_err() && clock == before,
                        "phase: exhausted run-start alignment fails without mutation"
                    );
                } else {
                    let expected = model + model % 2;
                    props::always!(
                        result.is_ok() && u64::from(clock.system_counter) == expected,
                        "phase: run start aligns to the next full phase without rewinding",
                        json!({ "before": model, "after": clock.system_counter })
                    );
                    if model % 2 == 1 {
                        // Alignment restarts the two-phase window.
                        window_phases = 0;
                        window_player_updates = 0;
                        window_nonplayer_updates = 0;
                    }
                    model = expected;
                    props::always!(
                        clock.next_phase() == Phase::Update,
                        "phase: an aligned clock's next phase is a full update"
                    );
                }
                continue;
            }
            let predicted = clock.next_phase();
            match clock.advance() {
                Ok(phase) => {
                    let expected = if model % 2 == 0 {
                        Phase::Update
                    } else if clock.interpolation_enabled {
                        Phase::Interpolate
                    } else {
                        Phase::Skipped
                    };
                    props::always!(
                        phase == expected
                            && phase == predicted
                            && u64::from(clock.system_counter) == model + 1,
                        "phase: even counters run full updates and odd counters interpolate or skip",
                        json!({ "counter": model, "phase": phase, "interpolation": clock.interpolation_enabled })
                    );
                    model += 1;
                    if phase == Phase::Update {
                        window_phases = 0;
                        window_player_updates = 0;
                        window_nonplayer_updates = 0;
                    }
                    window_phases += 1;
                    window_player_updates +=
                        u32::from(phase.player_updates(clock.interpolation_enabled));
                    window_nonplayer_updates += u32::from(phase.updates_nonplayers());
                    if window_phases == 2 {
                        // Interpolation changes presentation, never the rate.
                        props::always!(
                            window_player_updates == 2 && window_nonplayer_updates == 1,
                            "phase: every two-phase window runs two player and one nonplayer update",
                            json!({ "player": window_player_updates, "nonplayer": window_nonplayer_updates })
                        );
                        props::sometimes!(
                            !clock.interpolation_enabled,
                            "phase: a window without interpolation is exercised"
                        );
                    }
                }
                Err(_) => {
                    props::always!(
                        model == u64::from(u32::MAX) && clock == before,
                        "phase: only an exhausted counter fails, without mutation",
                        json!({ "counter": model })
                    );
                    props::sometimes!(true, "phase: counter exhaustion is exercised");
                    break;
                }
            }
        }
    }
}
