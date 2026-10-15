//! State-aware public input generation.
//!
//! The driver only emits commands the browser bridge can send (keys, seed
//! text, ticks, volumes and music backend callbacks) plus adversarial
//! variants of them. It reads the published state to steer toward doors,
//! pickups and NPCs so ordinary local budgets reach deep gameplay; Antithesis
//! steers the same decisions further.
use crate::entropy::Entropy;
use rebirth_cleanroom::bombs::player_world;
use rebirth_cleanroom::grid::Grid;
use rebirth_cleanroom::input::{self, PUBLIC_KEYS};
use rebirth_cleanroom::motion::WorldVector;
use rebirth_cleanroom::music::BackendEvent;
use rebirth_cleanroom::run_admission::MAX_SAFE_TIMESTAMP_MS;
use rebirth_cleanroom::traversal;
use rebirth_cleanroom::{Command, PausePhase, Screen, State, MAX_COMMAND_TICKS};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, VecDeque};

/// Unrelated browser key codes; `foreign_key` keeps only those outside the
/// engine's public surface.
const OTHER_KEYS: [&str; 7] = [
    "KeyR",
    "Spacebar",
    "ShiftLeft",
    "",
    "F5",
    "Digit1",
    "Numpad0",
];
const MOVES: [&str; 4] = ["KeyW", "KeyA", "KeyS", "KeyD"];
const AIMS: [&str; 4] = ["ArrowUp", "ArrowLeft", "ArrowDown", "ArrowRight"];

/// Each case follows one persona so ordinary budgets cover deep gameplay,
/// the menus and the seed dialog rather than averaging them away.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Persona {
    Direct,
    Explorer,
    Seeded,
}

pub struct Driver {
    queue: VecDeque<Command>,
    persona: Persona,
    menu_budget: u32,
    /// A steering target held for several macros in one room, so routes
    /// complete instead of oscillating between doors and pickups.
    goal: Option<Goal>,
    /// Browser frames tick once with `Date.now()`; this case's host clock.
    host_clock: Option<u64>,
    host_frames: bool,
    /// Where this case travels through the floor's door graph, if anywhere.
    travel: Option<Travel>,
    /// The engine's doorway admission per descriptor, for the floor it was
    /// computed on (boss and miniboss checks decode authored room assets).
    admitted: Option<((u32, u32), Vec<bool>)>,
    /// Pickups the player reached without collecting (a full-health heart,
    /// say), by room and id, with how often; they stop being targets.
    touched: BTreeMap<(usize, u32), u32>,
}

#[derive(Clone, Copy, PartialEq)]
enum Travel {
    /// The boss room.
    Boss,
    /// The nearest unvisited room the player may enter: special, large and
    /// miniboss rooms are otherwise never on a route.
    Explore,
}

#[derive(Clone, Copy)]
struct Goal {
    room: usize,
    position: WorldVector,
    remaining: u32,
    /// A hidden door: bomb it on arrival.
    bomb: bool,
}

fn key(code: &str, down: bool) -> Command {
    Command::Key {
        code: code.into(),
        down,
    }
}

/// A controlled step without a host clock, as test harnesses send.
/// A code outside the engine's public key surface: a near miss of a public
/// key (case, whitespace) or an unrelated key, derived from the engine's own
/// declaration so new public keys never become "foreign".
fn foreign_key(e: &mut Entropy) -> String {
    let public = *e.pick(&PUBLIC_KEYS);
    let candidates: Vec<String> = [
        public.to_lowercase(),
        public.to_uppercase(),
        format!("{public} "),
        format!(" {public}"),
        (*e.pick(&OTHER_KEYS)).to_owned(),
    ]
    .into_iter()
    .filter(|code| !input::is_public_key(code))
    .collect();
    e.pick(&candidates).clone()
}

fn tick(count: u32) -> Command {
    Command::Tick {
        count,
        timestamp_millis: None,
    }
}

impl Driver {
    pub fn new(e: &mut Entropy) -> Self {
        Self {
            queue: VecDeque::new(),
            persona: match e.below(4) {
                0 => Persona::Explorer,
                1 => Persona::Seeded,
                _ => Persona::Direct,
            },
            menu_budget: e.range(40, 400),
            goal: None,
            host_clock: None,
            host_frames: e.chance(3, 4),
            travel: match e.below(3) {
                0 => Some(Travel::Boss),
                1 => Some(Travel::Explore),
                _ => None,
            },
            admitted: None,
            touched: BTreeMap::new(),
        }
    }

    pub fn next(&mut self, state: &State, e: &mut Entropy) -> Command {
        if self.host_clock.is_none() {
            let clock = u64::try_from(state.run_admission.clock.milliseconds).unwrap_or(0);
            self.host_clock = Some(clock + u64::from(e.range(0, 900)));
        }
        if self.queue.is_empty() {
            self.plan(state, e);
        }
        self.queue.pop_front().unwrap_or_else(|| tick(1))
    }

    fn press(&mut self, code: &str, hold: u32) {
        self.queue.push_back(key(code, true));
        if hold > 0 {
            self.queue.push_back(tick(hold));
        }
        self.queue.push_back(key(code, false));
    }

    fn ticks(&mut self, e: &mut Entropy) {
        let count = match e.below(32) {
            0 => e.range(60, 600),
            1..=8 => e.range(8, 60),
            _ => e.range(1, 8),
        };
        if self.host_frames && count <= 8 {
            for _ in 0..count {
                let timestamp = self.next_host_time(e);
                self.queue.push_back(Command::Tick {
                    count: 1,
                    timestamp_millis: Some(timestamp),
                });
            }
        } else {
            self.queue.push_back(tick(count));
        }
    }

    /// Frame-paced host time with jitter, hitches and rare rollbacks (host
    /// clock adjustments), starting near the boot epoch as browsers do.
    fn next_host_time(&mut self, e: &mut Entropy) -> u64 {
        let now = self.host_clock.unwrap_or(0);
        let next = match e.below(64) {
            0 => now.saturating_sub(u64::from(e.range(1, 4_000))),
            1 => now + u64::from(e.range(250, 6_000)),
            2..=7 => now + 33,
            _ => now + u64::from(e.range(16, 17)),
        };
        self.host_clock = Some(next);
        next
    }

    fn plan(&mut self, state: &State, e: &mut Entropy) {
        if e.chance(1, 25) {
            return self.adversarial(state, e);
        }
        let request = &state.run_start_request;
        if state.screen != Screen::Playing
            && (request.input_pending || request.fading || request.system_pending)
        {
            // Inputs queued during the confirmation fade must be ignored.
            if e.chance(1, 6) {
                let code = *e.pick(&PUBLIC_KEYS);
                return self.press(code, 0);
            }
            return self.queue.push_back(tick(e.range(1, 24)));
        }
        match state.screen {
            Screen::Title => match e.below(10) {
                0..=6 => self.press("Enter", e.range(0, 2)),
                _ => self.ticks(e),
            },
            Screen::Profile => match e.below(20) {
                0..=9 => self.press("Enter", 0),
                10..=13 => self.press(e.pick(&["ArrowLeft", "ArrowRight", "KeyA", "KeyD"]), 0),
                14 => self.press("Escape", 0),
                _ => self.ticks(e),
            },
            Screen::Main => {
                if state.main_selection == 0 {
                    match e.below(20) {
                        0..=10 => self.press("Enter", 0),
                        11..=14 => self.press(e.pick(&["ArrowUp", "ArrowDown", "KeyW", "KeyS"]), 0),
                        15 => self.press("Escape", 0),
                        _ => self.ticks(e),
                    }
                } else {
                    match e.below(20) {
                        // Challenge opens a menu; the other actions are rejected.
                        0..=1 => self.press("Enter", 0),
                        2..=13 => self.press(e.pick(&["ArrowUp", "ArrowDown"]), 0),
                        14 => self.press("Escape", 0),
                        _ => self.ticks(e),
                    }
                }
            }
            Screen::Challenge => match e.below(20) {
                0..=5 => self.press("Enter", 0),
                6..=13 => self.press(e.pick(&["ArrowUp", "ArrowDown", "KeyW", "KeyS"]), 0),
                14..=16 => self.press("Escape", 0),
                _ => self.ticks(e),
            },
            Screen::Character if self.persona == Persona::Seeded && !state.seeded_run => {
                match e.below(10) {
                    0..=6 => self.press("Tab", 0),
                    7 => self.press(e.pick(&["ArrowUp", "ArrowDown"]), 0),
                    _ => self.ticks(e),
                }
            }
            Screen::Character if self.persona == Persona::Explorer && self.menu_budget > 0 => {
                self.menu_budget -= 1;
                match e.below(40) {
                    0..=2 => self.press("Enter", 0),
                    3..=14 => self.press(e.pick(&["ArrowLeft", "ArrowRight", "KeyA", "KeyD"]), 0),
                    15..=22 => self.press(e.pick(&["ArrowUp", "ArrowDown", "KeyW", "KeyS"]), 0),
                    23..=31 => self.press("Tab", 0),
                    32..=34 => self.press("Escape", 0),
                    _ => self.ticks(e),
                }
            }
            Screen::Character => match e.below(40) {
                0..=13 if state.difficulty != "normal" && e.chance(3, 4) => {
                    self.press(e.pick(&["ArrowUp", "ArrowDown"]), 0)
                }
                0..=13 => self.press("Enter", 0),
                14..=19 => self.press(e.pick(&["ArrowLeft", "ArrowRight", "KeyA", "KeyD"]), 0),
                20..=23 => self.press(e.pick(&["ArrowUp", "ArrowDown", "KeyW", "KeyS"]), 0),
                24..=28 => self.press("Tab", 0),
                29 => self.press("Escape", 0),
                _ => self.ticks(e),
            },
            Screen::Seed if self.persona != Persona::Direct && e.chance(1, 2) => {
                // Replace the dialog text with one complete code, then confirm.
                for _ in 0..state.seed_text.len() {
                    self.press("Backspace", 0);
                }
                let code = match e.below(8) {
                    0 => (*e.pick(&["T1PK6Y0W", "QNXPTVT1", "0YAZCZ7N", "BHX1EF8A", "KMAGB3XH"]))
                        .into(),
                    1 => (0..8)
                        .map(|_| *e.pick(b"ABCDEFGHJKLMNPQRSTWXYZ0123456789") as char)
                        .collect(),
                    _ => rebirth_cleanroom::seed::encode(e.nonzero_u32()).unwrap_or_default(),
                };
                self.queue.push_back(Command::Text { text: code });
                if e.chance(4, 5) {
                    self.press("Enter", 0);
                }
            }
            Screen::Seed => match e.below(20) {
                0..=7 => {
                    let text = seed_text(state, e);
                    self.queue.push_back(Command::Text { text });
                }
                8..=9 => self.press("Backspace", 0),
                10..=14 => self.press("Enter", 0),
                15..=16 => self.press(e.pick(&["Escape", "Tab"]), 0),
                _ => self.ticks(e),
            },
            Screen::Playing => self.play(state, e),
        }
    }

    fn play(&mut self, state: &State, e: &mut Entropy) {
        if state.paused {
            return match e.below(10) {
                0..=2 => self.press(e.pick(&["ArrowUp", "ArrowDown", "KeyW", "KeyS"]), 0),
                3..=4 if state.pause_menu.phase == PausePhase::Idle => self.press("Escape", 0),
                5 => self.press("Enter", 0),
                _ => self.queue.push_back(tick(e.range(1, 16))),
            };
        }
        // The GameOver popup reads released confirm (Enter/Space) and back
        // (Escape) edges at a tick; other keys must be ignored.
        if state.player_death.game_over.is_some() {
            return match e.below(8) {
                0..=2 => self.press(e.pick(&["Enter", "Space"]), e.range(1, 3)),
                3..=4 => self.press("Escape", e.range(1, 3)),
                5 => self.press(e.pick(&PUBLIC_KEYS), 0),
                _ => self.ticks(e),
            };
        }
        // Release held keys now and then so stale holds never dominate.
        if !state.pressed.is_empty() && e.chance(1, 6) {
            for code in &state.pressed {
                self.queue.push_back(key(code, false));
            }
            return;
        }
        let player = player_world(state);
        let hostile = state
            .room
            .npcs
            .iter()
            .filter(|npc| !npc.removed && npc.hp > 0.0)
            .map(|npc| npc.position)
            .min_by(|a, b| distance(player, *a).total_cmp(&distance(player, *b)));
        // Play the room before leaving it: fight what is alive (a boss on
        // nearly every macro, since its HP outlasts short cases), then collect
        // what the engine says can be collected.
        let boss_fight = state
            .room
            .npcs
            .iter()
            .any(|npc| npc.boss && !npc.removed && npc.hp > 0.0);
        let fight_odds = if boss_fight { (4, 5) } else { (1, 2) };
        if let Some(npc) = hostile.filter(|_| e.chance(fight_odds.0, fight_odds.1)) {
            return self.fight(state, player, npc, boss_fight, e);
        }
        if hostile.is_none() && e.chance(2, 3) {
            if let Some(pickup) = self.nearest_collectable(state, player) {
                return self.go(state, player, pickup, false, hostile, e);
            }
        }
        // Travellers spend most macros travelling.
        let choice = if self.travel.is_some() && e.chance(1, 2) {
            5 + e.below(40)
        } else {
            e.below(100)
        };
        match choice {
            0..=4 if state.inventory.bombs > 0 => {
                // Demolition: walk beside the nearest intact ordinary rock.
                match nearest_rock(state, player) {
                    Some(rock) if distance(player, rock) < 46.0 => self.bomb_rock(player, rock, e),
                    Some(rock) => {
                        let waypoint = match route(state, player, rock) {
                            Some(Route::Walk(waypoint)) => waypoint,
                            _ => rock,
                        };
                        self.steer(player, waypoint, None, e);
                    }
                    None => self.ticks(e),
                }
            }
            0..=44 => {
                let held = self.goal.filter(|goal| {
                    goal.room == state.current_room
                        && goal.remaining > 0
                        && distance(player, goal.position) > 8.0
                });
                let goal = match held {
                    Some(goal) => Goal {
                        remaining: goal.remaining - 1,
                        ..goal
                    },
                    None => {
                        let (position, bomb) = self.target(state, player, e);
                        Goal {
                            room: state.current_room,
                            position,
                            remaining: e.range(6, 24),
                            bomb,
                        }
                    }
                };
                self.goal = Some(goal);
                self.go(state, player, goal.position, goal.bomb, hostile, e);
            }
            45..=59 => {
                let aim = match hostile {
                    Some(npc) if e.chance(3, 4) => aim_toward(player, npc),
                    _ => *e.pick(&AIMS),
                };
                self.press(aim, e.range(1, 40));
            }
            // The run starts with one bomb; keep it for blocked routes more often.
            60..=64 if state.inventory.bombs > 1 || e.chance(1, 40) => {
                // Placement samples the held key at a tick, so a tap released
                // before any tick places nothing; usually hold across one.
                let hold = if e.chance(1, 8) { 0 } else { e.range(1, 3) };
                self.press("KeyE", hold)
            }
            65..=67 => self.press("Escape", 0),
            68..=69 => self.music_backend(state, e),
            // Active item and pocket inputs; triggers read held edges at a tick.
            70..=71 => self.press(e.pick(&["Space", "KeyQ"]), e.range(0, 3)),
            _ => self.ticks(e),
        }
    }

    /// A collectable pickup, an open door or a random point; true when the
    /// target is a hidden door to bomb.
    fn target(
        &mut self,
        state: &State,
        player: WorldVector,
        e: &mut Entropy,
    ) -> (WorldVector, bool) {
        let pickups: Vec<WorldVector> = self
            .collectable(state, player)
            .into_iter()
            .map(|(_, position)| position)
            .collect();
        let doors: Vec<WorldVector> = state
            .room
            .appearance
            .doors
            .iter()
            .filter(|door| door.state == "Opened")
            .map(|door| door.world_position)
            .collect();
        if let Some(travel) = self.travel.filter(|_| e.chance(3, 4)) {
            if let Some(door) = self.toward(state, travel) {
                return door;
            }
        }
        let position = match e.below(10) {
            0..=6 if !pickups.is_empty() => *e.pick(&pickups),
            0..=7 if !doors.is_empty() => *e.pick(&doors),
            _ => WorldVector {
                x: player.x + (e.range(0, 400) as f32 - 200.0),
                y: player.y + (e.range(0, 300) as f32 - 150.0),
            },
        };
        (position, false)
    }

    /// Pickups the engine marks collectable that an open route, or a bomb,
    /// reaches, minus those the player already stood on without collecting.
    fn collectable(&self, state: &State, player: WorldVector) -> Vec<(u32, WorldVector)> {
        state
            .room
            .pickups
            .iter()
            .filter(|pickup| !pickup.removed && !pickup.collected && pickup.can_collect)
            .filter(|pickup| {
                self.touched
                    .get(&(state.current_room, pickup.id))
                    .is_none_or(|touches| *touches < 6)
            })
            .map(|pickup| (pickup.id, pickup.position))
            .filter(|(_, position)| match route(state, player, *position) {
                Some(Route::Walk(_)) => true,
                Some(Route::Blocked { .. }) => state.inventory.bombs > 0,
                None => false,
            })
            .collect()
    }

    fn nearest_collectable(&mut self, state: &State, player: WorldVector) -> Option<WorldVector> {
        let (id, position) = self
            .collectable(state, player)
            .into_iter()
            .min_by(|a, b| distance(player, a.1).total_cmp(&distance(player, b.1)))?;
        if distance(player, position) < 14.0 {
            *self.touched.entry((state.current_room, id)).or_default() += 1;
        }
        Some(position)
    }

    /// Walks a route to `target`, bombing the first rock when every route is
    /// blocked; a hidden-door target is bombed on arrival.
    fn go(
        &mut self,
        state: &State,
        player: WorldVector,
        target: WorldVector,
        bomb: bool,
        hostile: Option<WorldVector>,
        e: &mut Entropy,
    ) {
        if bomb && state.inventory.bombs > 0 && distance(player, target) < 44.0 {
            return self.bomb_rock(player, target, e);
        }
        match route(state, player, target) {
            Some(Route::Walk(waypoint)) => self.steer(player, waypoint, hostile, e),
            Some(Route::Blocked { stand, rock }) if state.inventory.bombs > 0 => {
                if distance(player, stand) < 14.0 {
                    self.bomb_rock(player, rock, e);
                } else {
                    let waypoint = match route(state, player, stand) {
                        Some(Route::Walk(waypoint)) => waypoint,
                        _ => stand,
                    };
                    self.steer(player, waypoint, None, e);
                }
            }
            _ => self.steer(player, target, hostile, e),
        }
    }

    /// Lines up with `npc` on one axis and fires along it: tears fly
    /// straight, so alignment is what lands hits. Keeps a range band (wider
    /// against a boss), never stands still against a boss, turns away from
    /// walls instead of backing into a corner, and steps across the path of
    /// the nearest enemy projectile.
    fn fight(
        &mut self,
        state: &State,
        player: WorldVector,
        npc: WorldVector,
        boss: bool,
        e: &mut Entropy,
    ) {
        let (dx, dy) = (npc.x - player.x, npc.y - player.y);
        let horizontal = dx.abs() >= dy.abs();
        let fire = aim_toward(player, npc);
        // The published room bounds share the player's screen coordinates.
        let room = &state.room;
        let (x, y) = (state.player.x, state.player.y);
        let walled = |code: &str| match code {
            "KeyA" => x - room.left < 40.0,
            "KeyD" => room.right - x < 40.0,
            "KeyW" => y - room.top < 40.0,
            _ => room.bottom - y < 40.0,
        };
        let opposite = |code: &str| match code {
            "KeyA" => "KeyD",
            "KeyD" => "KeyA",
            "KeyW" => "KeyS",
            _ => "KeyW",
        };
        let (toward, away) = match fire {
            "ArrowRight" => ("KeyD", "KeyA"),
            "ArrowLeft" => ("KeyA", "KeyD"),
            "ArrowDown" => ("KeyS", "KeyW"),
            _ => ("KeyW", "KeyS"),
        };
        let (near, far) = if boss { (120.0, 220.0) } else { (70.0, 400.0) };
        let range = distance(player, npc);
        // One key per axis: along the firing line and across it.
        let mut along = if range < near && !walled(away) {
            Some(away)
        } else if range > far {
            Some(toward)
        } else {
            None
        };
        let cross = if horizontal { dy } else { dx };
        let (positive, negative) = if horizontal {
            ("KeyS", "KeyW")
        } else {
            ("KeyD", "KeyA")
        };
        let cornered = range < near && walled(away);
        let mut across = if cross.abs() > 8.0 {
            Some(if cross > 0.0 { positive } else { negative })
        } else if boss || cornered {
            Some(if e.chance(1, 2) { positive } else { negative })
        } else {
            None
        };
        if let Some(code) = across.filter(|code| walled(code)) {
            across = Some(opposite(code));
        }
        if let Some(shot) = state
            .enemy_projectiles
            .iter()
            .filter(|shot| distance(player, shot.position) < 90.0)
            .min_by(|a, b| distance(player, a.position).total_cmp(&distance(player, b.position)))
        {
            let shot_horizontal = shot.velocity.x.abs() >= shot.velocity.y.abs();
            let dodge = match (
                shot_horizontal,
                player.y >= shot.position.y,
                player.x >= shot.position.x,
            ) {
                (true, true, _) => "KeyS",
                (true, false, _) => "KeyW",
                (false, _, true) => "KeyD",
                (false, _, false) => "KeyA",
            };
            let dodge = if walled(dodge) {
                opposite(dodge)
            } else {
                dodge
            };
            // A dodge on the firing axis replaces the range move.
            if matches!(dodge, "KeyA" | "KeyD") == horizontal {
                along = Some(dodge);
            } else {
                across = Some(dodge);
            }
        }
        let held: Vec<&str> = std::iter::once(fire).chain(along).chain(across).collect();
        for code in &held {
            self.queue.push_back(key(code, true));
        }
        self.queue
            .push_back(tick(if boss { e.range(4, 14) } else { e.range(3, 20) }));
        for code in &held {
            self.queue.push_back(key(code, false));
        }
    }

    fn steer(
        &mut self,
        player: WorldVector,
        target: WorldVector,
        hostile: Option<WorldVector>,
        e: &mut Entropy,
    ) {
        let dx = target.x - player.x;
        let dy = target.y - player.y;
        let mut held = Vec::new();
        if dx.abs() > 6.0 {
            held.push(if dx > 0.0 { "KeyD" } else { "KeyA" });
        }
        if dy.abs() > 6.0 {
            held.push(if dy > 0.0 { "KeyS" } else { "KeyW" });
        }
        if held.is_empty() || e.chance(1, 10) {
            held = vec![*e.pick(&MOVES)];
        }
        if let Some(npc) = hostile {
            if e.chance(1, 2) {
                held.push(aim_toward(player, npc));
            }
        }
        for code in &held {
            self.queue.push_back(key(code, true));
        }
        self.queue.push_back(tick(e.range(2, 18)));
        for code in &held {
            self.queue.push_back(key(code, false));
        }
    }

    /// Plants a bomb beside a blocking rock and retreats from the blast.
    fn bomb_rock(&mut self, player: WorldVector, rock: WorldVector, e: &mut Entropy) {
        // Placement samples the held key at a tick; hold across one.
        self.press("KeyE", e.range(1, 3));
        let away = aim_toward(rock, player);
        let code = match away {
            "ArrowRight" => "KeyD",
            "ArrowLeft" => "KeyA",
            "ArrowDown" => "KeyS",
            _ => "KeyW",
        };
        self.press(code, e.range(20, 40));
        self.queue.push_back(tick(e.range(40, 90)));
    }

    fn music_backend(&mut self, state: &State, e: &mut Entropy) {
        let channel = e.range(0, 3) as u8;
        let generation = state
            .music
            .channels
            .get(usize::from(channel))
            .and_then(|channel| channel.generation)
            .filter(|_| e.chance(3, 4))
            .unwrap_or_else(|| e.u64() % 64);
        let event = if e.chance(1, 2) {
            BackendEvent::Ended
        } else {
            BackendEvent::Loop
        };
        self.queue.push_back(Command::MusicBackend {
            channel,
            generation,
            event,
        });
    }

    fn adversarial(&mut self, state: &State, e: &mut Entropy) {
        match e.below(9) {
            0 => self.press(&foreign_key(e), 0),
            1 => {
                // Releases and repeats never re-trigger handlers.
                let code = *e.pick(&PUBLIC_KEYS);
                self.queue.push_back(key(code, e.chance(1, 2)));
            }
            2 => self.queue.push_back(Command::Text {
                text: (*e.pick(&["abc!", "12345678901", "ÄÖ", "  ", "zz zz", "T1PK6Y0W", "-"]))
                    .into(),
            }),
            3 => self.queue.push_back(Command::AudioVolume {
                sfx_volume: *e.pick(&[0.0, 1.0, 0.5, -0.25, 1.5, 1.0e9, -0.0]),
            }),
            4 => self.queue.push_back(Command::MusicVolume {
                volume: *e.pick(&[0.0, 1.0, 0.25, -1.0, 1.0001, 3.0e8]),
            }),
            5 => self
                .queue
                .push_back(tick(*e.pick(&[0, MAX_COMMAND_TICKS + 1, u32::MAX]))),
            6 if e.chance(1, 40) => self.queue.push_back(tick(MAX_COMMAND_TICKS)),
            6 => {
                // Out-of-range host clocks and timestamps on batched steps.
                let (count, timestamp) = if e.chance(1, 2) {
                    (1, MAX_SAFE_TIMESTAMP_MS + 1 + u64::from(e.u32()))
                } else {
                    (e.range(2, 30), self.next_host_time(e))
                };
                self.queue.push_back(Command::Tick {
                    count,
                    timestamp_millis: Some(timestamp),
                });
            }
            7 => self.music_backend(state, e),
            _ => {
                for code in &state.pressed {
                    self.queue.push_back(key(code, false));
                }
            }
        }
    }
}

/// Valid fragments, documented display codes and characters the dialog
/// must reject, so both acceptance and checksum outcomes occur.
fn seed_text(state: &State, e: &mut Entropy) -> String {
    const GLYPHS: &[u8] = b"ABCDEFGHJKLMNPQRSTWXYZ0123456789abcdefIOU";
    match e.below(6) {
        0 => {
            let value = e.nonzero_u32();
            let code = rebirth_cleanroom::seed::encode(value).unwrap_or_default();
            let available = 8usize.saturating_sub(state.seed_text.len());
            code[..available.min(8)].to_string()
        }
        1 => (*e.pick(&["T1PK 6Y0W", "QNXPTVT1", "0YAZCZ7N", "BHX1EF8A", "KMAG B3XH"])).into(),
        2 => "!?".into(),
        _ => (0..e.range(1, 4))
            .map(|_| *e.pick(GLYPHS) as char)
            .collect(),
    }
}

impl Driver {
    /// Doorway admission for every descriptor of the current floor, asked of
    /// the engine's own rule so routes follow it as rooms are admitted.
    fn admitted(&mut self, state: &State) -> &[bool] {
        let floor = (state.floor.stage, state.floor.stage_seed);
        if self.admitted.as_ref().map(|(key, _)| *key) != Some(floor) {
            let admitted = state
                .floor
                .descriptors
                .iter()
                .map(traversal::admits)
                .collect();
            self.admitted = Some((floor, admitted));
        }
        &self.admitted.as_ref().expect("computed above").1
    }

    /// The door on a shortest path through admitted rooms to the nearest
    /// room `travel` seeks, and whether it is hidden and needs a bomb;
    /// explorers without one head for the boss.
    fn toward(&mut self, state: &State, travel: Travel) -> Option<(WorldVector, bool)> {
        let mut admitted = self.admitted(state).to_vec();
        // A key opens a KeyClosed door on contact and a bomb reveals a
        // hidden one; without them those neighbours are out of reach.
        for door in &state.room.appearance.doors {
            let passable = match door.state.as_str() {
                "Opened" => true,
                "KeyClosed" => state.inventory.keys > 0,
                "Hidden" => state.inventory.bombs > 0,
                _ => false,
            };
            if !passable {
                if let Some(room) = admitted.get_mut(door.target_room as usize) {
                    *room = false;
                }
            }
        }
        let descriptors = &state.floor.descriptors;
        let boss = |room: usize| descriptors[room].template.room_type == 5;
        let unvisited = |room: usize| !state.visited_rooms.contains(&room);
        let next = match travel {
            Travel::Boss => first_step(state, &admitted, boss),
            Travel::Explore => first_step(state, &admitted, unvisited)
                .or_else(|| first_step(state, &admitted, boss)),
        }?;
        state
            .room
            .appearance
            .doors
            .iter()
            .find(|door| door.target_room as usize == next)
            .map(|door| (door.world_position, door.state == "Hidden"))
    }
}

/// The neighbour of the current room that starts a shortest path through
/// admitted rooms to the nearest room satisfying `goal`.
fn first_step(state: &State, admitted: &[bool], goal: impl Fn(usize) -> bool) -> Option<usize> {
    let descriptors = &state.floor.descriptors;
    let start = state.current_room;
    let mut first = vec![None; descriptors.len()];
    let mut seen = vec![false; descriptors.len()];
    seen[start] = true;
    let mut queue = VecDeque::from([start]);
    while let Some(room) = queue.pop_front() {
        for neighbor in descriptors[room].neighbors.iter().flatten().copied() {
            if seen[neighbor] || !admitted[neighbor] {
                continue;
            }
            seen[neighbor] = true;
            first[neighbor] = if room == start {
                Some(neighbor)
            } else {
                first[room]
            };
            if goal(neighbor) {
                return first[neighbor];
            }
            queue.push_back(neighbor);
        }
    }
    None
}

fn nearest_rock(state: &State, player: WorldVector) -> Option<WorldVector> {
    let grid = Grid::new(state.room.grid.clone()?).ok()?;
    (0..grid.descriptor().cells.len())
        .filter(|cell| {
            grid.descriptor().cells[*cell]
                .entity
                .as_ref()
                .is_some_and(|entity| entity.kind == 2 && entity.state == 1)
        })
        .filter_map(|cell| grid.position(cell))
        .min_by(|a, b| distance(player, *a).total_cmp(&distance(player, *b)))
}

enum Route {
    /// The next cell centre on a rock-free path.
    Walk(WorldVector),
    /// Every path crosses a destructible rock: stand beside the first one.
    Blocked {
        stand: WorldVector,
        rock: WorldVector,
    },
}

/// Shortest grid route to `target` among those crossing the fewest rocks
/// (Dijkstra over cells), so steering routes around obstacles and bombs
/// through enclosures only when no open route exists.
fn route(state: &State, player: WorldVector, target: WorldVector) -> Option<Route> {
    const ROCK_COST: u32 = 1_000;
    let grid = Grid::new(state.room.grid.clone()?).ok()?;
    let width = usize::from(grid.descriptor().width);
    let cells = grid.descriptor().cells.len();
    let start = grid.index(player)?;
    let goal = grid.index(target)?;
    let rock = |cell: usize| {
        grid.descriptor().cells[cell]
            .entity
            .as_ref()
            .is_some_and(|entity| matches!(entity.kind, 2 | 5 | 6) && entity.collision_class == 3)
    };
    let mut cost = vec![u32::MAX; cells];
    let mut previous = vec![usize::MAX; cells];
    cost[start] = 0;
    previous[start] = start;
    let mut frontier = BinaryHeap::from([Reverse((0u32, start))]);
    while let Some(Reverse((reached, cell))) = frontier.pop() {
        if reached > cost[cell] {
            continue;
        }
        let (x, y) = (cell % width, cell / width);
        let neighbors = [
            (x > 0).then(|| cell - 1),
            (x + 1 < width).then(|| cell + 1),
            (y > 0).then(|| cell - width),
            (cell + width < cells).then(|| cell + width),
        ];
        for next in neighbors.into_iter().flatten() {
            let step = if next == goal || grid.collision(next) == 0 {
                1
            } else if rock(next) {
                ROCK_COST
            } else {
                continue;
            };
            if reached + step < cost[next] {
                cost[next] = reached + step;
                previous[next] = cell;
                frontier.push(Reverse((reached + step, next)));
            }
        }
    }
    if previous[goal] == usize::MAX {
        return None;
    }
    let mut path = vec![goal];
    while *path.last()? != start {
        path.push(previous[*path.last()?]);
    }
    path.reverse();
    if let Some(blocked) = path
        .iter()
        .position(|cell| *cell != goal && rock(*cell))
        .filter(|blocked| *blocked > 0)
    {
        return Some(Route::Blocked {
            stand: grid.position(path[blocked - 1])?,
            rock: grid.position(path[blocked])?,
        });
    }
    Some(match path.get(1) {
        Some(next) if *next != goal => Route::Walk(grid.position(*next)?),
        _ => Route::Walk(target),
    })
}

fn distance(a: WorldVector, b: WorldVector) -> f32 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}

fn aim_toward(from: WorldVector, to: WorldVector) -> &'static str {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    if dx.abs() >= dy.abs() {
        if dx > 0.0 {
            "ArrowRight"
        } else {
            "ArrowLeft"
        }
    } else if dy > 0.0 {
        "ArrowDown"
    } else {
        "ArrowUp"
    }
}
