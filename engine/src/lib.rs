//! A fresh, narrow foundation driven by observed base-Rebirth menu/run captures.
//! World positions use logical pixels in a 480 × 270 viewport; simulation is 60 Hz.
//! Sprite offsets and stage effects are explicit state separate from ground positions.
//! Public seed encoding is supported; original room RNG routing and simulation
//! ticks remain unverified. Scene descriptors are ordinary generated state.
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::BTreeSet;
mod content;
mod effects;
mod room;
pub mod seed;
pub use effects::{Point, StageEffects};
pub use room::Room;

pub const TICK_RATE: u32 = 60;
pub const ABI_VERSION: u32 = 1;
pub const MAX_COMMAND_TICKS: u32 = 36_000;
pub const MAX_REPLAY_COMMANDS: usize = 36_000;
const MAX_REQUEST: usize = 65_536;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Screen {
    Title,
    Profile,
    Main,
    Character,
    Seed,
    Playing,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Player {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub hp: u8,
    pub max_hp: u8,
    pub facing: Direction,
    pub body_facing: Direction,
    pub walk_frame: u8,
    pub head_frame: u8,
    pub shooting: bool,
    pub render_offset: Point,
}
impl Player {
    fn starting() -> Self {
        Self {
            x: 240.0,
            y: 214.0,
            vx: 0.0,
            vy: 0.0,
            hp: 6,
            max_hp: 6,
            facing: Direction::Down,
            body_facing: Direction::Down,
            walk_frame: 0,
            head_frame: 0,
            shooting: false,
            // Authored sprite roots and shadow centers are one logical pixel
            // above the physical ground/light anchor in public GL submissions.
            render_offset: Point { x: 0.0, y: -1.0 },
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Tear {
    pub id: u32,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub age: u32,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Impact {
    pub id: u32,
    pub x: f64,
    pub y: f64,
    pub age: u32,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Inventory {
    pub coins: u32,
    pub bombs: u32,
    pub keys: u32,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Profile {
    pub slot: u8,
    pub unlocked_characters: Vec<String>,
    pub selected_character: String,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterDefinition {
    pub name: String,
    pub animation: String,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MinimapCell {
    pub x: i32,
    pub y: i32,
    pub visibility: String,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Minimap {
    pub cells: Vec<MinimapCell>,
}
fn character_catalog() -> Vec<CharacterDefinition> {
    [
        ("Isaac", "01_Isaac"),
        ("Magdalene", "02_Magdalene"),
        ("Cain", "03_Cain"),
        ("Judas", "04_Judas"),
        ("Eve", "05_Eve"),
        ("Samson", "07_Samson"),
        ("Azazel", "08_Azazel"),
        ("Lazarus", "09_Lazarus"),
        ("Eden", "10_Eden"),
        ("Random", "00_Random"),
    ]
    .into_iter()
    .map(|(name, animation)| CharacterDefinition {
        name: name.into(),
        animation: animation.into(),
    })
    .collect()
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct State {
    pub schema: u32,
    pub tick_rate: u32,
    pub tick: u32,
    pub menu_animation_tick: u32,
    pub run_tick: u32,
    pub screen: Screen,
    pub profile: u8,
    pub main_selection: u8,
    pub character_selection: u8,
    pub character: String,
    pub difficulty: String,
    pub seed_text: String,
    pub seed_valid: bool,
    pub run_seed: String,
    pub initial_seed: u32,
    pub run_seed_value: u32,
    pub stage: u32,
    pub stage_seed: u32,
    pub seeded_run: bool,
    pub achievement_eligibility: bool,
    pub profiles: Vec<Profile>,
    pub character_catalog: Vec<CharacterDefinition>,
    pub paused: bool,
    pub pressed: BTreeSet<String>,
    pub player: Player,
    pub room: Room,
    pub stage_effects: StageEffects,
    pub inventory: Inventory,
    pub minimap: Minimap,
    pub tears: Vec<Tear>,
    pub impacts: Vec<Impact>,
    pub fire_cooldown: u32,
    pub next_tear: u32,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Command {
    Key { code: String, down: bool },
    Text { text: String },
    Tick { count: u32 },
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Game {
    pub state: State,
}

impl Game {
    pub fn new(initial_seed: u32) -> Result<Self, String> {
        let run_seed = seed::encode(initial_seed)?;
        let player = Player::starting();
        let room = room::generate_room(initial_seed);
        let stage_effects = StageEffects::basement(&player, &room.appearance.fx, 0);
        let mut game = Self {
            state: State {
                schema: 1,
                tick_rate: TICK_RATE,
                tick: 0,
                menu_animation_tick: 0,
                run_tick: 0,
                screen: Screen::Title,
                profile: 1,
                main_selection: 0,
                character_selection: 0,
                character: "Isaac".into(),
                difficulty: "normal".into(),
                seed_text: String::new(),
                seed_valid: false,
                run_seed,
                initial_seed,
                run_seed_value: initial_seed,
                stage: 1,
                stage_seed: seed::stage1_seed(initial_seed),
                seeded_run: false,
                achievement_eligibility: true,
                profiles: (1..=3)
                    .map(|slot| Profile {
                        slot,
                        unlocked_characters: vec!["Isaac".into()],
                        selected_character: "Isaac".into(),
                    })
                    .collect(),
                character_catalog: character_catalog(),
                paused: false,
                pressed: BTreeSet::new(),
                player,
                room,
                stage_effects,
                inventory: Inventory {
                    coins: 0,
                    bombs: 1,
                    keys: 0,
                },
                minimap: Minimap { cells: Vec::new() },
                tears: Vec::new(),
                impacts: Vec::new(),
                fire_cooldown: 0,
                next_tear: 1,
            },
        };
        game.refresh_minimap();
        Ok(game)
    }
    /// Invalid commands leave all state, including held keys, unchanged.
    pub fn command(&mut self, command: Command) -> Result<(), String> {
        let mut next = self.clone();
        next.apply(command)?;
        if next.state.screen != self.state.screen {
            next.state.menu_animation_tick = 0;
        }
        next.refresh_stage_effects();
        *self = next;
        Ok(())
    }
    fn apply(&mut self, command: Command) -> Result<(), String> {
        match command {
            Command::Key { code, down } => {
                if !matches!(
                    code.as_str(),
                    "Enter"
                        | "Escape"
                        | "Tab"
                        | "Backspace"
                        | "ArrowUp"
                        | "ArrowDown"
                        | "ArrowLeft"
                        | "ArrowRight"
                        | "KeyW"
                        | "KeyA"
                        | "KeyS"
                        | "KeyD"
                ) {
                    return Err(format!("unsupported key code: {code}"));
                }
                if down {
                    if self.state.pressed.insert(code.clone()) {
                        self.key_press(&code)?;
                    }
                } else {
                    self.state.pressed.remove(&code);
                }
            }
            Command::Text { text } => {
                if self.state.screen != Screen::Seed {
                    return Err("text entry requires the seed dialog".into());
                }
                let compact: String = text.chars().filter(|character| *character != ' ').collect();
                if !compact.is_ascii()
                    || !compact
                        .bytes()
                        .all(|character| character.is_ascii_alphanumeric())
                    || compact.len() + self.state.seed_text.len() > 8
                {
                    return Err("seed entry requires at most eight ASCII letters/digits".into());
                }
                self.state.seed_text.push_str(&compact.to_ascii_uppercase());
                self.state.seed_valid = seed::decode(&self.state.seed_text).is_ok();
            }
            Command::Tick { count } => {
                if count > MAX_COMMAND_TICKS {
                    return Err("tick command exceeds 36,000 steps".into());
                }
                self.state
                    .tick
                    .checked_add(count)
                    .ok_or("tick counter exhausted")?;
                self.state
                    .menu_animation_tick
                    .checked_add(count)
                    .ok_or("menu animation counter exhausted")?;
                if self.state.screen == Screen::Playing && !self.state.paused {
                    self.state
                        .run_tick
                        .checked_add(count)
                        .ok_or("run tick counter exhausted")?;
                    self.state
                        .next_tear
                        .checked_add(count)
                        .ok_or("tear IDs exhausted")?;
                }
                for _ in 0..count {
                    self.tick();
                }
            }
        }
        Ok(())
    }
    fn key_press(&mut self, code: &str) -> Result<(), String> {
        match self.state.screen {
            Screen::Title => {
                if code == "Enter" {
                    self.state.screen = Screen::Profile;
                }
            }
            Screen::Profile => match code {
                "Enter" => {
                    self.state.character = self.state.profiles[usize::from(self.state.profile - 1)]
                        .selected_character
                        .clone();
                    self.state.character_selection = self
                        .state
                        .character_catalog
                        .iter()
                        .position(|character| character.name == self.state.character)
                        .unwrap_or(0) as u8;
                    self.state.main_selection = 0;
                    self.state.screen = Screen::Main;
                }
                "Escape" => self.state.screen = Screen::Title,
                "ArrowLeft" | "KeyA" => self.state.profile = (self.state.profile + 1) % 3 + 1,
                "ArrowRight" | "KeyD" => self.state.profile = self.state.profile % 3 + 1,
                _ => {}
            },
            Screen::Main => match code {
                "Escape" => self.state.screen = Screen::Profile,
                "ArrowUp" | "KeyW" => {
                    self.state.main_selection = (self.state.main_selection + 4) % 5
                }
                "ArrowDown" | "KeyS" => {
                    self.state.main_selection = (self.state.main_selection + 1) % 5
                }
                "Enter" if self.state.main_selection == 0 => self.state.screen = Screen::Character,
                "Enter" => return Err("this foundation implements the New Run menu action".into()),
                _ => {}
            },
            Screen::Character => match code {
                "Escape" => self.state.screen = Screen::Main,
                "Tab" => self.state.screen = Screen::Seed,
                "ArrowUp" | "ArrowDown" | "KeyW" | "KeyS" => {
                    self.state.difficulty = if self.state.difficulty == "normal" {
                        "hard"
                    } else {
                        "normal"
                    }
                    .into()
                }
                "ArrowLeft" | "KeyA" => self.select_character(-1),
                "ArrowRight" | "KeyD" => self.select_character(1),
                "Enter" if self.state.difficulty == "normal" => self.start_run()?,
                "Enter" => {
                    return Err("only the observed normal-mode starting room is implemented".into())
                }
                _ => {}
            },
            Screen::Seed => match code {
                "Escape" | "Tab" => self.state.screen = Screen::Character,
                "Backspace" => {
                    self.state.seed_text.pop();
                    self.state.seed_valid = seed::decode(&self.state.seed_text).is_ok();
                }
                "Enter" if self.state.seed_valid => {
                    self.state.run_seed_value = seed::decode(&self.state.seed_text)?;
                    self.state.run_seed = seed::encode(self.state.run_seed_value)?;
                    self.state.stage_seed = seed::stage1_seed(self.state.run_seed_value);
                    self.state.seeded_run = true;
                    self.state.achievement_eligibility = false;
                    self.state.screen = Screen::Character;
                }
                "Enter" => {
                    // Invalid codes close the dialog without committing a seed;
                    // this applies equally to bad checksums and reserved zero.
                    self.state.seed_text.clear();
                    self.state.seed_valid = false;
                    self.state.run_seed_value = self.state.initial_seed;
                    self.state.run_seed = seed::encode(self.state.initial_seed)?;
                    self.state.stage_seed = seed::stage1_seed(self.state.initial_seed);
                    self.state.seeded_run = false;
                    self.state.achievement_eligibility = true;
                    self.state.screen = Screen::Character;
                }
                _ => {}
            },
            Screen::Playing => {
                if code == "Escape" {
                    self.state.paused = !self.state.paused;
                }
            }
        }
        Ok(())
    }
    fn select_character(&mut self, direction: i32) {
        self.state.character_selection = (i32::from(self.state.character_selection) + direction)
            .rem_euclid(self.state.character_catalog.len() as i32)
            as u8;
        self.state.character = self.state.character_catalog
            [usize::from(self.state.character_selection)]
        .name
        .clone();
        self.state.profiles[usize::from(self.state.profile - 1)].selected_character =
            self.state.character.clone();
    }
    fn refresh_minimap(&mut self) {
        let mut cells = vec![MinimapCell {
            x: 0,
            y: 0,
            visibility: "current".into(),
        }];
        for door in &self.state.room.appearance.doors {
            let (x, y) = match door.side.as_str() {
                "left" => (-1, 0),
                "right" => (1, 0),
                "up" => (0, -1),
                "down" => (0, 1),
                _ => continue,
            };
            if !cells.iter().any(|cell| cell.x == x && cell.y == y) {
                cells.push(MinimapCell {
                    x,
                    y,
                    visibility: "unvisited".into(),
                });
            }
        }
        self.state.minimap.cells = cells;
    }
    fn refresh_stage_effects(&mut self) {
        self.state.stage_effects = StageEffects::basement(
            &self.state.player,
            &self.state.room.appearance.fx,
            self.state.run_tick,
        );
    }
    fn start_run(&mut self) -> Result<(), String> {
        let unlocked =
            &self.state.profiles[usize::from(self.state.profile - 1)].unlocked_characters;
        let chosen = if self.state.character == "Random" {
            unlocked
                .get(
                    (seed::startup_next(self.state.run_seed_value) as usize)
                        % unlocked.len().max(1),
                )
                .cloned()
                .ok_or("profile has no unlocked character")?
        } else {
            self.state.character.clone()
        };
        if !unlocked.contains(&chosen) {
            return Err("selected character is locked in this profile".into());
        }
        if chosen != "Isaac" {
            return Err("selected character runtime is not implemented".into());
        }
        self.state.character = chosen;
        self.state.screen = Screen::Playing;
        self.state.paused = false;
        self.state.run_tick = 0;
        self.state.tears.clear();
        self.state.impacts.clear();
        self.state.next_tear = 1;
        self.state.fire_cooldown = 0;
        self.state.player = Self::new(self.state.run_seed_value)?.state.player;
        self.state.room = room::generate_room(self.state.run_seed_value);
        self.state.inventory = Inventory {
            coins: 0,
            bombs: 1,
            keys: 0,
        };
        self.state.stage_seed = seed::stage1_seed(self.state.run_seed_value);
        self.refresh_minimap();
        Ok(())
    }
    fn held(&self, code: &str) -> bool {
        self.state.pressed.contains(code)
    }
    fn tick(&mut self) {
        self.state.tick += 1;
        self.state.menu_animation_tick += 1;
        if self.state.screen != Screen::Playing || self.state.paused {
            return;
        }
        self.state.run_tick += 1;
        let dx = i32::from(self.held("KeyD")) - i32::from(self.held("KeyA"));
        let dy = i32::from(self.held("KeyS")) - i32::from(self.held("KeyW"));
        // Rendered-frame calibration: native-baseline/measurements.json.
        // A .9 retention / 3-pixel target reproduces the observed +80-pixel
        // movement after 27 held samples and settling. Native ticks are unknown.
        let diagonal = if dx != 0 && dy != 0 {
            std::f64::consts::FRAC_1_SQRT_2
        } else {
            1.0
        };
        let speed = 3.0;
        let player = &mut self.state.player;
        player.vx = player.vx * 0.9 + f64::from(dx) * diagonal * speed * 0.1;
        player.vy = player.vy * 0.9 + f64::from(dy) * diagonal * speed * 0.1;
        if player.vx.abs() < 0.05 && dx == 0 {
            player.vx = 0.0;
        }
        if player.vy.abs() < 0.05 && dy == 0 {
            player.vy = 0.0;
        }
        player.x = (player.x + player.vx).clamp(self.state.room.left, self.state.room.right);
        player.y = (player.y + player.vy).clamp(self.state.room.top, self.state.room.bottom);
        if dx != 0 || dy != 0 {
            player.body_facing = if dx < 0 {
                Direction::Left
            } else if dx > 0 {
                Direction::Right
            } else if dy < 0 {
                Direction::Up
            } else {
                Direction::Down
            };
            player.facing = player.body_facing;
            // Walk* has 24 authored ANM2 frames. Playback-rate calibration is
            // separate from the verified stopped/idle checkpoint pose.
            player.walk_frame = (self.state.run_tick % 24) as u8;
        } else {
            player.walk_frame = 0;
            player.body_facing = Direction::Down;
            if self.state.fire_cooldown == 0 {
                player.facing = Direction::Down;
            }
        }
        let aim = if self.held("ArrowLeft") {
            Some(Direction::Left)
        } else if self.held("ArrowRight") {
            Some(Direction::Right)
        } else if self.held("ArrowUp") {
            Some(Direction::Up)
        } else if self.held("ArrowDown") {
            Some(Direction::Down)
        } else {
            None
        };
        self.state.player.shooting = aim.is_some();
        self.state.fire_cooldown = self.state.fire_cooldown.saturating_sub(1);
        if let Some(direction) = aim {
            self.state.player.facing = direction;
            self.state.player.head_frame = if self.state.fire_cooldown < 5 { 2 } else { 0 };
            if self.state.fire_cooldown == 0 {
                let (dx, dy) = match direction {
                    Direction::Left => (-1.0, 0.0),
                    Direction::Right => (1.0, 0.0),
                    Direction::Up => (0.0, -1.0),
                    Direction::Down => (0.0, 1.0),
                };
                // The captured right-facing volleys alternate between two
                // rendered eye heights about six logical pixels apart.
                let eye_offset = if self.state.next_tear % 2 == 0 {
                    6.0
                } else {
                    0.0
                };
                self.state.tears.push(Tear {
                    id: self.state.next_tear,
                    x: self.state.player.x
                        + dx * 9.5
                        + if dy != 0.0 { eye_offset - 3.0 } else { 0.0 },
                    y: self.state.player.y + self.state.player.render_offset.y - 20.75
                        + dy * 10.0
                        + if dx != 0.0 { eye_offset } else { 0.0 },
                    vx: dx * 3.25,
                    vy: dy * 3.25 + 0.125,
                    age: 0,
                });
                self.state.next_tear += 1;
                // Three observed spawns are about 22 nominal rendered samples apart.
                self.state.fire_cooldown = 22;
            }
        } else {
            self.state.player.head_frame = 0;
        }
        for impact in &mut self.state.impacts {
            impact.age += 1;
        }
        self.state.impacts.retain(|impact| impact.age < 12);
        for tear in &mut self.state.tears {
            tear.x += tear.vx;
            tear.y += tear.vy;
            tear.age += 1;
        }
        let room = &self.state.room;
        let mut surviving = Vec::new();
        for tear in std::mem::take(&mut self.state.tears) {
            if tear.age >= 45
                || tear.x < room.left
                || tear.x > room.right
                || tear.y < room.top - 21.25
                || tear.y > room.bottom
            {
                self.state.impacts.push(Impact {
                    id: tear.id,
                    x: tear.x.clamp(room.left, room.right),
                    y: tear.y.clamp(room.top - 21.25, room.bottom),
                    age: 0,
                });
            } else {
                surviving.push(tear);
            }
        }
        self.state.tears = surviving;
    }
    pub fn snapshot(&self) -> State {
        self.state.clone()
    }
    pub fn json(&self) -> String {
        serde_json::to_string(&self.state).expect("state contains finite values")
    }
}

thread_local! {
    static GAME: RefCell<Option<Game>> = const { RefCell::new(None) };
    static REQUEST: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static SNAPSHOT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static ERROR: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}
#[no_mangle]
pub extern "C" fn abi_version() -> u32 {
    ABI_VERSION
}
#[no_mangle]
pub extern "C" fn reset(initial_seed: u32) -> u32 {
    match Game::new(initial_seed) {
        Ok(game) => {
            GAME.with(|state| *state.borrow_mut() = Some(game));
            ERROR.with(|error| error.borrow_mut().clear());
            0
        }
        Err(message) => {
            ERROR.with(|error| *error.borrow_mut() = message.into_bytes());
            1
        }
    }
}
#[no_mangle]
pub extern "C" fn request_ptr(length: usize) -> *mut u8 {
    if length > MAX_REQUEST {
        return std::ptr::null_mut();
    }
    REQUEST.with(|request| {
        let mut bytes = request.borrow_mut();
        bytes.resize(length, 0);
        bytes.as_mut_ptr()
    })
}
#[no_mangle]
pub extern "C" fn command() -> u32 {
    let result = REQUEST
        .with(|request| {
            serde_json::from_slice::<Command>(&request.borrow()).map_err(|error| error.to_string())
        })
        .and_then(|command| {
            GAME.with(|game| {
                game.borrow_mut()
                    .as_mut()
                    .ok_or_else(|| "initialize the engine with an entropy seed first".to_string())?
                    .command(command)
            })
        });
    ERROR.with(|error| {
        *error.borrow_mut() = result
            .as_ref()
            .err()
            .map(|message| message.as_bytes().to_vec())
            .unwrap_or_default();
    });
    u32::from(result.is_err())
}
#[no_mangle]
pub extern "C" fn snapshot_ptr() -> *const u8 {
    SNAPSHOT.with(|snapshot| {
        *snapshot.borrow_mut() = GAME.with(|game| {
            game.borrow()
                .as_ref()
                .map(Game::json)
                .unwrap_or_else(|| "null".into())
                .into_bytes()
        });
        snapshot.borrow().as_ptr()
    })
}
#[no_mangle]
pub extern "C" fn snapshot_len() -> usize {
    SNAPSHOT.with(|snapshot| snapshot.borrow().len())
}
#[no_mangle]
pub extern "C" fn error_ptr() -> *const u8 {
    ERROR.with(|error| error.borrow().as_ptr())
}
#[no_mangle]
pub extern "C" fn error_len() -> usize {
    ERROR.with(|error| error.borrow().len())
}

#[cfg(test)]
mod test_seeds;

#[cfg(test)]
mod tests {
    use super::*;
    fn press(game: &mut Game, code: &str) {
        game.command(Command::Key {
            code: code.into(),
            down: true,
        })
        .unwrap();
        game.command(Command::Key {
            code: code.into(),
            down: false,
        })
        .unwrap();
    }
    fn run(test: &str) -> Game {
        let mut game = Game::new(test_seeds::one(test)).unwrap();
        for _ in 0..4 {
            press(&mut game, "Enter");
        }
        game
    }
    fn observed_run() -> Game {
        let mut game = Game::new(test_seeds::one("observed_run_boot")).unwrap();
        for _ in 0..3 {
            press(&mut game, "Enter");
        }
        press(&mut game, "Tab");
        game.command(Command::Text {
            text: "T1PK 6Y0W".into(),
        })
        .unwrap();
        press(&mut game, "Enter");
        press(&mut game, "Enter");
        game
    }
    #[test]
    fn observed_menu_seed_path_enters_run() {
        let mut game = Game::new(test_seeds::one("observed_menu_seed_path_enters_run")).unwrap();
        for screen in [Screen::Profile, Screen::Main, Screen::Character] {
            press(&mut game, "Enter");
            assert_eq!(game.state.screen, screen);
        }
        press(&mut game, "Tab");
        assert_eq!(game.state.screen, Screen::Seed);
        game.command(Command::Text {
            text: "fxsw jzbm".into(),
        })
        .unwrap();
        assert!(game.state.seed_valid);
        press(&mut game, "Enter");
        assert_eq!(game.state.screen, Screen::Character);
        press(&mut game, "Enter");
        assert_eq!(game.state.screen, Screen::Playing);
        assert_eq!(game.state.run_seed, "FXSWJZBM");
    }
    #[test]
    fn held_enter_cannot_skip_multiple_menus() {
        let mut game = Game::new(test_seeds::one("held_enter_cannot_skip_multiple_menus")).unwrap();
        for _ in 0..3 {
            game.command(Command::Key {
                code: "Enter".into(),
                down: true,
            })
            .unwrap();
        }
        assert_eq!(game.state.screen, Screen::Profile);
    }
    #[test]
    fn invalid_commands_are_atomic() {
        let mut game = Game::new(test_seeds::one("invalid_commands_are_atomic")).unwrap();
        let before = game.clone();
        assert!(game
            .command(Command::Key {
                code: "Unknown".into(),
                down: true
            })
            .is_err());
        assert!(game
            .command(Command::Text {
                text: "T1PK6Y0W".into()
            })
            .is_err());
        assert!(game
            .command(Command::Tick {
                count: MAX_COMMAND_TICKS + 1
            })
            .is_err());
        assert_eq!(game, before);
    }
    #[test]
    fn unsupported_mode_is_rejected_and_invalid_seed_cancels_to_entropy_run() {
        let mut game = Game::new(test_seeds::one(
            "unsupported_mode_is_rejected_and_invalid_seed_cancels_to_entropy_run",
        ))
        .unwrap();
        for _ in 0..3 {
            press(&mut game, "Enter");
        }
        press(&mut game, "ArrowDown");
        let before = game.clone();
        assert!(game
            .command(Command::Key {
                code: "Enter".into(),
                down: true
            })
            .is_err());
        assert_eq!(game, before);
        press(&mut game, "ArrowUp");
        press(&mut game, "Tab");
        game.command(Command::Text {
            text: "AAAAAAAA".into(),
        })
        .unwrap();
        press(&mut game, "Enter");
        assert_eq!(game.state.screen, Screen::Character);
        assert!(game.state.seed_text.is_empty());
        assert!(!game.state.seeded_run);
        assert!(game.state.achievement_eligibility);
        assert_eq!(game.state.run_seed_value, game.state.initial_seed);
        press(&mut game, "Enter");
        assert_eq!(game.state.screen, Screen::Playing);
    }
    #[test]
    fn reserved_zero_boot_is_rejected_atomically() {
        assert!(Game::new(0).is_err());
        assert_eq!(reset(1), 0);
        let before = GAME.with(|game| game.borrow().clone());
        assert_eq!(reset(0), 1);
        GAME.with(|game| assert_eq!(*game.borrow(), before));
        let mut game = Game::new(1).unwrap();
        for _ in 0..3 {
            press(&mut game, "Enter");
        }
        press(&mut game, "Tab");
        game.command(Command::Text {
            text: "B91199JA".into(),
        })
        .unwrap();
        assert!(!game.state.seed_valid);
        press(&mut game, "Enter");
        assert_eq!(game.state.screen, Screen::Character);
        assert_eq!(game.state.run_seed_value, 1);
        assert!(!game.state.seeded_run);
    }
    #[test]
    fn arbitrary_valid_seeds_are_accepted_and_start_new_generated_state() {
        for value in test_seeds::samples(
            "arbitrary_valid_seeds_are_accepted_and_start_new_generated_state",
            128,
        ) {
            let initial = value.rotate_left(11);
            let mut game = Game::new(initial).unwrap();
            for _ in 0..3 {
                press(&mut game, "Enter");
            }
            press(&mut game, "Tab");
            game.command(Command::Text {
                text: seed::encode(value).unwrap(),
            })
            .unwrap();
            assert!(game.state.seed_valid);
            press(&mut game, "Enter");
            press(&mut game, "Enter");
            assert_eq!(game.state.initial_seed, initial);
            assert_eq!(game.state.run_seed_value, value);
            assert_eq!(game.state.run_seed, seed::encode(value).unwrap());
            assert_eq!(game.state.room, room::generate_room(value));
            assert_eq!(game.state.stage_seed, seed::stage1_seed(value));
            assert!(game.state.seeded_run);
            assert!(!game.state.achievement_eligibility);
        }
    }
    #[test]
    fn entropy_seed_and_profile_state_control_an_unseeded_run() {
        let mut game = Game::new(test_seeds::one(
            "entropy_seed_and_profile_state_control_an_unseeded_run",
        ))
        .unwrap();
        press(&mut game, "Enter");
        press(&mut game, "ArrowRight");
        assert_eq!(game.state.profile, 2);
        press(&mut game, "Enter");
        press(&mut game, "Enter");
        press(&mut game, "ArrowRight");
        assert_eq!(game.state.character, "Magdalene");
        assert_eq!(game.state.profiles[1].selected_character, "Magdalene");
        let locked = game.clone();
        assert!(game
            .command(Command::Key {
                code: "Enter".into(),
                down: true
            })
            .is_err());
        assert_eq!(game, locked);
        press(&mut game, "ArrowLeft");
        press(&mut game, "Enter");
        assert!(!game.state.seeded_run);
        assert!(game.state.achievement_eligibility);
        assert_eq!(game.state.run_seed_value, game.state.initial_seed);
        assert_eq!(game.state.inventory.bombs, 1);
        assert_eq!(game.state.player.max_hp, game.state.player.hp);
        assert_eq!(
            game.state.minimap.cells.len(),
            game.state.room.appearance.doors.len() + 1
        );
    }
    #[test]
    fn menu_animation_time_resets_on_transition_and_fire_pose_reaches_second_frame() {
        let mut game = Game::new(test_seeds::one(
            "menu_animation_time_resets_on_transition_and_fire_pose_reaches_second_frame",
        ))
        .unwrap();
        game.command(Command::Tick { count: 50 }).unwrap();
        press(&mut game, "Enter");
        assert_eq!(game.state.tick, 50);
        assert_eq!(game.state.menu_animation_tick, 0);
        game.command(Command::Tick { count: 12 }).unwrap();
        assert_eq!(game.state.menu_animation_tick, 12);
        let mut game =
            run("menu_animation_time_resets_on_transition_and_fire_pose_reaches_second_frame");
        game.command(Command::Key {
            code: "ArrowRight".into(),
            down: true,
        })
        .unwrap();
        game.command(Command::Tick { count: 1 }).unwrap();
        assert_eq!(
            game.state.player.head_frame, 2,
            "authored Head* first pose has Delay=2"
        );
        game.command(Command::Tick { count: 2 }).unwrap();
        assert_eq!(game.state.player.head_frame, 0);
    }
    #[test]
    fn movement_pause_and_release_are_controlled() {
        let mut game = run("movement_pause_and_release_are_controlled");
        let initial = game.state.player.x;
        game.command(Command::Key {
            code: "KeyD".into(),
            down: true,
        })
        .unwrap();
        game.command(Command::Tick { count: 10 }).unwrap();
        assert!(game.state.player.x > initial);
        press(&mut game, "Escape");
        let paused = game.state.player.clone();
        game.command(Command::Tick { count: 10 }).unwrap();
        assert_eq!(game.state.player, paused);
        press(&mut game, "Escape");
        game.command(Command::Key {
            code: "KeyD".into(),
            down: false,
        })
        .unwrap();
        game.command(Command::Tick { count: 10 }).unwrap();
        assert!(
            game.state.player.vx > 0.0,
            "released movement retains observed inertia"
        );
        game.command(Command::Tick { count: 60 }).unwrap();
        assert_eq!(game.state.player.vx, 0.0);
        assert_eq!(game.state.player.facing, Direction::Down);
    }
    #[test]
    fn stage_effects_track_motion_and_freeze_with_the_paused_run() {
        let mut game = run("stage_effects_track_motion_and_freeze_with_the_paused_run");
        game.command(Command::Key {
            code: "KeyD".into(),
            down: true,
        })
        .unwrap();
        game.command(Command::Tick { count: 13 }).unwrap();
        let effects = &game.state.stage_effects;
        assert_eq!(effects.lights[0].x, game.state.player.x);
        assert_eq!(effects.lights[0].y, game.state.player.y);
        assert_eq!(
            effects.shadow_casters[0].y,
            game.state.player.y + game.state.player.render_offset.y
        );
        assert_eq!(effects.overlay.simulation_tick, game.state.run_tick);
        press(&mut game, "Escape");
        let paused = game.state.stage_effects.clone();
        game.command(Command::Tick { count: 100 }).unwrap();
        assert_eq!(game.state.stage_effects, paused);
        press(&mut game, "Escape");
        game.command(Command::Tick { count: 1 }).unwrap();
        assert_ne!(game.state.stage_effects.lights, paused.lights);
        assert_eq!(
            game.state.stage_effects.overlay.simulation_tick,
            paused.overlay.simulation_tick + 1
        );
    }
    #[test]
    fn exhausted_replay_counters_fail_without_partial_ticks_or_key_changes() {
        for counter in ["tick", "menu", "run", "tear"] {
            let mut game =
                run("exhausted_replay_counters_fail_without_partial_ticks_or_key_changes");
            match counter {
                "tick" => game.state.tick = u32::MAX,
                "menu" => game.state.menu_animation_tick = u32::MAX,
                "run" => game.state.run_tick = u32::MAX,
                "tear" => game.state.next_tear = u32::MAX,
                _ => unreachable!(),
            }
            let before = game.clone();
            assert!(game.command(Command::Tick { count: 1 }).is_err());
            assert_eq!(game, before, "counter {counter}");
        }
    }
    #[test]
    fn shooting_advances_tears_and_collides_with_room_bounds() {
        let mut game = run("shooting_advances_tears_and_collides_with_room_bounds");
        game.command(Command::Key {
            code: "ArrowRight".into(),
            down: true,
        })
        .unwrap();
        game.command(Command::Tick { count: 1 }).unwrap();
        assert_eq!(game.state.tears.len(), 1);
        let position = game.state.tears[0].x;
        game.command(Command::Tick { count: 1 }).unwrap();
        assert!(game.state.tears[0].x > position);
        game.command(Command::Key {
            code: "ArrowRight".into(),
            down: false,
        })
        .unwrap();
        game.command(Command::Tick { count: 60 }).unwrap();
        assert!(game.state.tears.is_empty());
    }
    #[test]
    fn observed_move_fire_checkpoints_remain_near_rendered_measurements() {
        let mut game = observed_run();
        game.command(Command::Key {
            code: "KeyD".into(),
            down: true,
        })
        .unwrap();
        game.command(Command::Tick { count: 27 }).unwrap();
        game.command(Command::Key {
            code: "KeyD".into(),
            down: false,
        })
        .unwrap();
        game.command(Command::Tick { count: 30 }).unwrap();
        assert!(
            (game.state.player.x - 320.0).abs() < 0.5,
            "native stable movement foot estimate: x=320 logical"
        );
        assert_eq!(game.state.player.facing, Direction::Down);
        game.command(Command::Key {
            code: "ArrowRight".into(),
            down: true,
        })
        .unwrap();
        game.command(Command::Tick { count: 54 }).unwrap();
        game.command(Command::Key {
            code: "ArrowRight".into(),
            down: false,
        })
        .unwrap();
        assert_eq!(game.state.tears.len(), 1);
        assert_eq!(game.state.impacts.len(), 1);
        assert!(
            (game.state.tears[0].x - 362.5).abs() < 1.0,
            "native regular tear rendered estimate"
        );
        assert!((game.state.tears[0].y - 193.5).abs() < 0.5);
        assert_eq!(game.state.player.facing, Direction::Right);
        game.command(Command::Tick { count: 30 }).unwrap();
        assert_eq!(game.state.player.facing, Direction::Down);
        assert!(game.state.tears.is_empty());
    }
    #[test]
    fn replay_and_serialized_continuation_are_deterministic() {
        let mut first = run("replay_and_serialized_continuation_are_deterministic");
        first
            .command(Command::Key {
                code: "KeyD".into(),
                down: true,
            })
            .unwrap();
        first
            .command(Command::Key {
                code: "ArrowUp".into(),
                down: true,
            })
            .unwrap();
        first.command(Command::Tick { count: 12 }).unwrap();
        let mut second: Game =
            serde_json::from_str(&serde_json::to_string(&first).unwrap()).unwrap();
        for _ in 0..180 {
            let command = Command::Tick { count: 1 };
            first.command(command.clone()).unwrap();
            second.command(command).unwrap();
            assert_eq!(first, second);
        }
    }
}
