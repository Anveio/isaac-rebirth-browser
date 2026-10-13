//! A fresh, narrow foundation driven by observed base-Rebirth menu/run captures.
//! World positions use logical pixels in a 480 × 270 viewport; simulation is 60 Hz.
//! Sprite offsets and stage effects are explicit state separate from ground positions.
//! Public seed encoding is supported; original room RNG routing and simulation
//! ticks remain unverified. Scene descriptors are ordinary generated state.
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::BTreeSet;
mod bombs;
pub mod combat;
mod content;
mod effects;
pub mod grid;
pub mod layout;
pub mod motion;
pub mod mt;
pub mod phase;
pub mod projectiles;
mod room;
pub mod room_templates;
pub mod seed;
pub use bombs::{Bomb, Explosion};
pub use effects::{Point, StageEffects};
pub use room::Room;

pub const TICK_RATE: u32 = 60;
pub const ABI_VERSION: u32 = 2;
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
pub struct RunIntroduction {
    pub age: u32,
    pub active: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MenuFade {
    pub active: bool,
    pub alpha: f64,
    pub color: [f64; 4],
}
impl MenuFade {
    fn first_title() -> Self {
        // MenuInit starts at 1, then its first rendered update subtracts .025.
        // Store expanded f32 values so native and JavaScript snapshots agree.
        Self {
            active: true,
            alpha: f64::from(1.0_f32 - 0.025_f32),
            color: [1.0, 1.0, 1.0, f64::from(0.95000017_f32)],
        }
    }
    fn advance(&mut self) {
        if self.active {
            let alpha = self.alpha as f32 - 0.025_f32;
            self.active = alpha > 0.0;
            self.alpha = f64::from(alpha.max(0.0));
        }
    }
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PausePhase {
    Hidden,
    Appearing,
    Idle,
    Disappearing,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PauseMenu {
    pub phase: PausePhase,
    pub age: u32,
    pub selection: u8,
    pub stat_frames: [u8; 6],
}
impl PauseMenu {
    fn hidden() -> Self {
        Self {
            phase: PausePhase::Hidden,
            age: 0,
            selection: 1,
            stat_frames: [2, 1, 2, 2, 2, 1],
        }
    }
    fn advance(&mut self) {
        match self.phase {
            PausePhase::Appearing | PausePhase::Disappearing => {
                self.age += 1;
                let end = if self.phase == PausePhase::Appearing {
                    26
                } else {
                    22
                };
                if self.age >= end {
                    self.phase = if self.phase == PausePhase::Appearing {
                        PausePhase::Idle
                    } else {
                        PausePhase::Hidden
                    };
                    self.age = 0;
                }
            }
            PausePhase::Hidden | PausePhase::Idle => {}
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct State {
    pub schema: u32,
    pub tick_rate: u32,
    pub tick: u32,
    pub menu_animation_tick: u32,
    pub title_animation_frame: u32,
    pub menu_fade: MenuFade,
    pub system_clock: phase::Clock,
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
    pub boot_epoch_seed: u32,
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
    pub bombs: Vec<Bomb>,
    pub explosions: Vec<Explosion>,
    pub next_bomb: u32,
    pub damage_cooldown: u32,
    pub run_introduction: RunIntroduction,
    pub pause_menu: PauseMenu,
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
        Self::new_with_boot_epoch(initial_seed, 0)
    }
    pub fn new_with_boot_epoch(initial_seed: u32, boot_epoch_seed: u32) -> Result<Self, String> {
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
                title_animation_frame: 0,
                menu_fade: MenuFade::first_title(),
                // The public runtime starts at the first rendered Title,
                // after the omitted native introductory cutscene. Its next
                // system iteration is a full update; this clock is relative
                // to that boundary, not the source's elapsed boot counter.
                system_clock: phase::Clock::new(0, true),
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
                boot_epoch_seed,
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
                bombs: Vec::new(),
                explosions: Vec::new(),
                next_bomb: 1,
                damage_cooldown: 0,
                run_introduction: RunIntroduction {
                    age: 0,
                    active: false,
                },
                pause_menu: PauseMenu::hidden(),
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
                        | "KeyE"
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
                self.state
                    .system_clock
                    .system_counter
                    .checked_add(count)
                    .ok_or("system counter exhausted")?;
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
                if code == "Escape"
                    || (self.state.paused
                        && code == "Enter"
                        && self.state.pause_menu.selection == 1)
                {
                    self.state.paused = !self.state.paused;
                    self.state.pause_menu.phase = if self.state.paused {
                        PausePhase::Appearing
                    } else {
                        PausePhase::Disappearing
                    };
                    self.state.pause_menu.age = 0;
                    if self.state.paused {
                        self.state.pause_menu.selection = 1;
                    }
                } else if self.state.paused {
                    match code {
                        "ArrowUp" | "KeyW" => {
                            self.state.pause_menu.selection =
                                (self.state.pause_menu.selection + 2) % 3
                        }
                        "ArrowDown" | "KeyS" => {
                            self.state.pause_menu.selection =
                                (self.state.pause_menu.selection + 1) % 3
                        }
                        _ => {}
                    }
                } else if code == "KeyE" {
                    bombs::place(&mut self.state)?;
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
        self.state.stage_effects.add_bomb_shadows(&self.state.bombs);
        self.state
            .stage_effects
            .add_explosion_lights(&self.state.explosions);
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
        self.state.system_clock.align_run_start()?;
        self.state.screen = Screen::Playing;
        self.state.paused = false;
        self.state.run_tick = 0;
        self.state.tears.clear();
        self.state.impacts.clear();
        self.state.next_tear = 1;
        self.state.fire_cooldown = 0;
        self.state.bombs.clear();
        self.state.explosions.clear();
        self.state.next_bomb = 1;
        self.state.damage_cooldown = 0;
        self.state.run_introduction = RunIntroduction {
            age: 0,
            active: true,
        };
        self.state.pause_menu = PauseMenu::hidden();
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
        let phase = self
            .state
            .system_clock
            .advance()
            .expect("counter preflight");
        if self.state.screen == Screen::Title && phase == phase::Phase::Update {
            self.state.title_animation_frame = (self.state.title_animation_frame + 1) % 40;
        }
        self.state.menu_fade.advance();
        self.state.tick += 1;
        self.state.menu_animation_tick += 1;
        if self.state.screen == Screen::Playing {
            self.state.pause_menu.advance();
        }
        if self.state.screen != Screen::Playing || self.state.paused {
            return;
        }
        self.state.run_tick += 1;
        if self.state.run_introduction.active {
            self.state.run_introduction.age += 1;
            self.state.run_introduction.active = self.state.run_introduction.age < 140;
        }
        let dx = i32::from(self.held("KeyD")) - i32::from(self.held("KeyA"));
        let dy = i32::from(self.held("KeyS")) - i32::from(self.held("KeyW"));
        // Native blast impulses apply before movement integration and drag in
        // their first visible phase; stationary zero-separation blasts stay put.
        bombs::tick(&mut self.state);
        // Shooting inherits the velocity entering this phase, before input acceleration.
        let inherited_velocity = (self.state.player.vx, self.state.player.vy);
        motion::advance(&mut self.state.player, &self.state.room, dx, dy);
        let player = &mut self.state.player;
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
        for tear in &mut self.state.tears {
            tear.x += tear.vx;
            tear.y += tear.vy;
            tear.age += 1;
        }
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
                let eye_offset = if self.state.next_tear.is_multiple_of(2) {
                    6.0
                } else {
                    0.0
                };
                // Native stored velocity projects at 0.65 logical pixels per
                // world unit for the birth offset; observed presentation
                // displacement uses half that amount. Moving shots recover 60% of
                // the player's previous presentation velocity in its motion.
                // Motion against the aim direction is excluded; sideways
                // drift and forward motion are inherited. Independent native
                // perpendicular and opposite-motion shots verify this rule.
                let backward = (inherited_velocity.0 * dx + inherited_velocity.1 * dy).min(0.0);
                let vx = dx * 3.25 + (inherited_velocity.0 - dx * backward) * 0.6;
                let ground_vy = dy * 3.25 + (inherited_velocity.1 - dy * backward) * 0.6;
                self.state.tears.push(Tear {
                    id: self.state.next_tear,
                    x: self.state.player.x
                        + vx * 2.0
                        + if dy != 0.0 { eye_offset - 3.0 } else { 0.0 },
                    y: self.state.player.y + self.state.player.render_offset.y - 20.75
                        + ground_vy * 2.0
                        + if dx != 0.0 { eye_offset } else { 0.0 },
                    vx,
                    // Height/range still uses the provisional projected drift.
                    // The independently observed native ground/height/falling
                    // fields are tracked for the next projectile workstream.
                    vy: ground_vy + 0.125,
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
        let bounds = &self.state.room.projectile_bounds;
        let mut surviving = Vec::new();
        for tear in std::mem::take(&mut self.state.tears) {
            if tear.age >= 45
                || tear.x < bounds.left
                || tear.x > bounds.right
                || tear.y < bounds.top - 21.25
                || tear.y > bounds.bottom
            {
                self.state.impacts.push(Impact {
                    id: tear.id,
                    x: tear.x.clamp(bounds.left, bounds.right),
                    y: tear.y.clamp(bounds.top - 21.25, bounds.bottom),
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
pub extern "C" fn reset(initial_seed: u32, boot_epoch_seed: u32) -> u32 {
    match Game::new_with_boot_epoch(initial_seed, boot_epoch_seed) {
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
        assert_eq!(reset(1, 0), 0);
        let before = GAME.with(|game| game.borrow().clone());
        assert_eq!(reset(0, 0), 1);
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
        assert!(
            game.state.player.vx > 0.0 && game.state.player.vx < 0.001,
            "native release preserves subpixel inertia"
        );
        assert_eq!(game.state.player.facing, Direction::Down);
    }
    #[test]
    fn introduction_allows_movement_and_freezes_while_pause_animation_advances() {
        let mut game =
            run("introduction_allows_movement_and_freezes_while_pause_animation_advances");
        let initial = game.state.player.x;
        game.command(Command::Key {
            code: "KeyD".into(),
            down: true,
        })
        .unwrap();
        game.command(Command::Tick { count: 10 }).unwrap();
        assert!(game.state.player.x > initial);
        assert_eq!(
            game.state.run_introduction,
            RunIntroduction {
                age: 10,
                active: true
            }
        );
        press(&mut game, "Escape");
        let paused_run = game.state.run_tick;
        game.command(Command::Tick { count: 26 }).unwrap();
        assert_eq!(game.state.run_tick, paused_run);
        assert_eq!(game.state.run_introduction.age, 10);
        assert_eq!(game.state.pause_menu.phase, PausePhase::Idle);
        assert_eq!(game.state.pause_menu.selection, 1);
        press(&mut game, "ArrowDown");
        assert_eq!(game.state.pause_menu.selection, 2);
        press(&mut game, "ArrowUp");
        press(&mut game, "Enter");
        assert!(!game.state.paused);
        assert_eq!(game.state.pause_menu.phase, PausePhase::Disappearing);
        game.command(Command::Tick { count: 22 }).unwrap();
        assert_eq!(game.state.pause_menu.phase, PausePhase::Hidden);
        game.command(Command::Tick { count: 108 }).unwrap();
        assert_eq!(
            game.state.run_introduction,
            RunIntroduction {
                age: 140,
                active: false
            }
        );
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
        for counter in ["tick", "menu", "system", "run", "tear"] {
            let mut game =
                run("exhausted_replay_counters_fail_without_partial_ticks_or_key_changes");
            match counter {
                "tick" => game.state.tick = u32::MAX,
                "menu" => game.state.menu_animation_tick = u32::MAX,
                "system" => game.state.system_clock.system_counter = u32::MAX,
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
    fn native_shots_inherit_prior_forward_and_sideways_motion_but_exclude_backward_motion() {
        // Independent moving/perpendicular/opposite-shot native captures on
        // KGVM KYA2, YPL8 ERQD, and DXZX P818 respectively. Native tear
        // velocity14.14933967590332 projects to4.598535394668579 per phase;
        // perpendicular velocity4.14933967590332 projects to1.348535394668579.
        for (movement, aim, expected_vx, expected_birth_x) in [
            (
                "KeyD",
                "ArrowRight",
                4.598535394668579,
                Some(267.0927291870117),
            ),
            ("KeyD", "ArrowUp", 1.348535394668579, None),
            ("KeyA", "ArrowRight", 3.25, Some(228.60435028076173)),
        ] {
            let mut game = run("native_shots_inherit_prior_forward_and_sideways_motion_but_exclude_backward_motion");
            game.command(Command::Key {
                code: movement.into(),
                down: true,
            })
            .unwrap();
            game.command(Command::Tick { count: 12 }).unwrap();
            game.command(Command::Key {
                code: aim.into(),
                down: true,
            })
            .unwrap();
            game.command(Command::Tick { count: 1 }).unwrap();
            assert_eq!(game.state.tears.len(), 1);
            let tear = &game.state.tears[0];
            assert_eq!(
                tear.age, 0,
                "native new tears are born after live-tear integration"
            );
            assert!(
                (tear.vx - expected_vx).abs() < 0.000001,
                "movement {movement}, aim {aim}"
            );
            if let Some(expected) = expected_birth_x {
                assert!(
                    (tear.x - expected).abs() < 0.0001,
                    "native moving-shot birth position"
                );
            }
            let emitted_velocity = tear.vx;
            let emitted_x = tear.x;
            game.command(Command::Tick { count: 1 }).unwrap();
            assert_eq!(game.state.tears[0].age, 1);
            assert!((game.state.tears[0].x - emitted_x - emitted_velocity).abs() < 1e-10);
            assert_eq!(
                game.state.tears[0].vx, emitted_velocity,
                "later player acceleration does not steer a fired tear"
            );
        }
    }
    #[test]
    fn native_direction_length_holdout_matches_numeric_player_observations() {
        // Independent native motion-holdout capture on seed 0DNW 1DMX,
        // recorded again with a preferred read-only entity candidate. Values
        // are world f32 positions projected to logical ground coordinates.
        // The bound covers at most four native f32 ULPs, not sprite rounding.
        for seed in test_seeds::samples(
            "native_direction_length_holdout_matches_numeric_player_observations",
            8,
        ) {
            let mut game = Game::new(seed).unwrap();
            for _ in 0..4 {
                press(&mut game, "Enter");
            }
            for (keys, held, expected) in [
                (vec!["KeyA"], 40, [125.7505485534668, 214.0]),
                (vec!["KeyW"], 20, [125.45559082031251, 157.09796295166015]),
                (vec!["KeyD"], 30, [211.0206481933594, 156.82596588134766]),
                (
                    vec!["KeyD", "KeyS"],
                    20,
                    [251.65743103027344, 197.17250213623046],
                ),
            ] {
                for code in &keys {
                    game.command(Command::Key {
                        code: (*code).into(),
                        down: true,
                    })
                    .unwrap();
                }
                game.command(Command::Tick { count: held }).unwrap();
                for code in &keys {
                    game.command(Command::Key {
                        code: (*code).into(),
                        down: false,
                    })
                    .unwrap();
                }
                game.command(Command::Tick { count: 30 }).unwrap();
                assert!(
                    (game.state.player.x - expected[0]).abs() < 0.0001,
                    "seed {seed}, keys {keys:?}: ground x {} vs {}",
                    game.state.player.x,
                    expected[0]
                );
                assert!(
                    (game.state.player.y - expected[1]).abs() < 0.0001,
                    "seed {seed}, keys {keys:?}: ground y {} vs {}",
                    game.state.player.y,
                    expected[1]
                );
            }
        }
    }
    #[test]
    fn native_immediate_direction_reversal_matches_independent_numeric_observations() {
        // Fresh original ZVSW NLEY: D20/A20/release30, no settling between turns.
        let mut game =
            run("native_immediate_direction_reversal_matches_independent_numeric_observations");
        game.command(Command::Key {
            code: "KeyD".into(),
            down: true,
        })
        .unwrap();
        game.command(Command::Tick { count: 20 }).unwrap();
        game.command(Command::Key {
            code: "KeyD".into(),
            down: false,
        })
        .unwrap();
        game.command(Command::Key {
            code: "KeyA".into(),
            down: true,
        })
        .unwrap();
        game.command(Command::Tick { count: 1 }).unwrap();
        assert!((game.state.player.x - 277.9258575439453).abs() < 0.0001);
        assert!((game.state.player.vx - 1.892010533809662).abs() < 0.000001);
        game.command(Command::Tick { count: 19 }).unwrap();
        assert!((game.state.player.x - 258.5896270751953).abs() < 0.0001);
        game.command(Command::Key {
            code: "KeyA".into(),
            down: false,
        })
        .unwrap();
        game.command(Command::Tick { count: 30 }).unwrap();
        assert!((game.state.player.x - 238.45638885498047).abs() < 0.0001);
        assert!((game.state.player.vx - -0.05382454320788383).abs() < 0.000001);
    }
    #[test]
    fn native_player_position_uses_previous_velocity_and_release_preserves_inertia() {
        let mut game =
            run("native_player_position_uses_previous_velocity_and_release_preserves_inertia");
        game.command(Command::Key {
            code: "KeyD".into(),
            down: true,
        })
        .unwrap();
        game.command(Command::Tick { count: 1 }).unwrap();
        assert_eq!(
            game.state.player.x, 240.0,
            "first native presentation updates velocity before displacement"
        );
        assert!((game.state.player.vx - 0.34333292543888094).abs() < 0.000001);
        game.command(Command::Tick { count: 29 }).unwrap();
        assert!((game.state.player.x - 302.62315979003906).abs() < 0.0001);
        assert!((game.state.player.vx - 2.806_552_362_442_017).abs() < 0.000001);
        game.command(Command::Key {
            code: "KeyD".into(),
            down: false,
        })
        .unwrap();
        game.command(Command::Tick { count: 55 }).unwrap();
        assert!((game.state.player.x - 326.0565246582031).abs() < 0.0001);
        assert!((game.state.player.vx - 0.002534948627000212).abs() < 0.000001);
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
