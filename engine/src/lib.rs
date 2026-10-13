//! A fresh, narrow foundation driven by observed base-Rebirth menu/run captures.
//! World positions use logical pixels in a 480 × 270 viewport; public stepping
//! follows full/intermediate source phases at nominal 60 presentations per second.
//! Sprite offsets and stage effects are explicit state separate from ground positions.
//! Public seed encoding is supported; original room RNG routing and simulation
//! ticks remain unverified. Scene descriptors are ordinary generated state.
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
pub mod actors;
pub mod bombs;
pub mod combat;
mod content;
mod effects;
pub mod floor;
pub mod grid;
pub mod layout;
pub mod motion;
pub mod mt;
pub mod phase;
pub mod pickups;
pub mod projectiles;
mod room;
pub mod room_assets;
pub mod room_spawns;
pub mod room_templates;
pub mod seed;
pub mod traversal;
pub use bombs::{Bomb, Explosion};
pub use effects::{Point, StageEffects};
pub use pickups::Pickup;
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
    fn advance(&mut self, phase: phase::Phase) {
        if phase != phase::Phase::Update {
            return;
        }
        match self.phase {
            PausePhase::Appearing | PausePhase::Disappearing => {
                // The source retains the final authored frame, then dispatches
                // its completion on the following full controller update.
                if self.age >= 22 {
                    self.phase = if self.phase == PausePhase::Appearing {
                        PausePhase::Idle
                    } else {
                        PausePhase::Hidden
                    };
                    self.age = 0;
                } else {
                    self.age += 2;
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
    pub floor: floor::Floor,
    pub current_room: usize,
    pub visited_rooms: BTreeSet<usize>,
    pub persisted_rooms: BTreeMap<usize, Room>,
    pub room_transition: Option<traversal::RoomTransition>,
    pub stage_effects: StageEffects,
    pub inventory: Inventory,
    pub next_pickup: u32,
    pub minimap: Minimap,
    pub tears: Vec<Tear>,
    pub impacts: Vec<Impact>,
    pub fire_cooldown: i32,
    pub max_fire_delay: i32,
    pub head_timer: i32,
    pub actor_scheduler: actors::Scheduler,
    pub next_tear: u32,
    pub bombs: Vec<Bomb>,
    pub explosions: Vec<Explosion>,
    pub next_bomb: u32,
    pub damage_cooldown: u32,
    pub bomb_placement_cooldown: bombs::BombPlacementCooldown,
    pub bomb_action_was_down: bool,
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
        let floor = floor::generate_basement(seed::stage1_seed(initial_seed))?;
        let current_room = floor.starting_descriptor;
        let room = Room::from_starting_floor(&floor);
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
                floor,
                current_room,
                visited_rooms: BTreeSet::from([current_room]),
                persisted_rooms: BTreeMap::new(),
                room_transition: None,
                stage_effects,
                inventory: Inventory {
                    coins: 0,
                    bombs: 1,
                    keys: 0,
                },
                next_pickup: 1,
                minimap: Minimap { cells: Vec::new() },
                tears: Vec::new(),
                impacts: Vec::new(),
                fire_cooldown: -1,
                max_fire_delay: 10,
                head_timer: -1,
                actor_scheduler: actors::Scheduler::default(),
                next_tear: 1,
                bombs: Vec::new(),
                explosions: Vec::new(),
                next_bomb: 1,
                damage_cooldown: 0,
                bomb_placement_cooldown: bombs::BombPlacementCooldown::default(),
                bomb_action_was_down: false,
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
                    self.tick()?;
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
                    if !self.state.paused {
                        self.state.paused = true;
                        self.state.pause_menu.phase = PausePhase::Appearing;
                        self.state.pause_menu.age = 0;
                        self.state.pause_menu.selection = 1;
                    } else if self.state.pause_menu.phase != PausePhase::Disappearing {
                        self.state.pause_menu.phase = PausePhase::Disappearing;
                        self.state.pause_menu.age = 0;
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
        let mut shown = self.state.visited_rooms.clone();
        for index in &self.state.visited_rooms {
            let descriptor = &self.state.floor.descriptors[*index];
            for neighbor in descriptor.neighbors.iter().flatten() {
                let target = &self.state.floor.descriptors[*neighbor];
                if !matches!(target.template.room_type, 7 | 8) {
                    shown.insert(*neighbor);
                }
            }
        }
        self.state.minimap.cells = shown
            .into_iter()
            .map(|index| {
                let descriptor = &self.state.floor.descriptors[index];
                MinimapCell {
                    x: descriptor.grid_index as i32 % 13 - 6,
                    y: descriptor.grid_index as i32 / 13 - 6,
                    visibility: if index == self.state.current_room {
                        "current"
                    } else if self.state.visited_rooms.contains(&index) {
                        "visited"
                    } else {
                        "unvisited"
                    }
                    .into(),
                }
            })
            .collect();
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
            .add_pickup_shadows(&self.state.room.pickups);
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
        self.state.fire_cooldown = -1;
        self.state.max_fire_delay = 10;
        self.state.head_timer = -1;
        self.state.actor_scheduler = actors::Scheduler::default();
        self.state.actor_scheduler.admit(actors::ActorRef {
            kind: actors::ActorKind::Player,
            id: 0,
        })?;
        self.state.bombs.clear();
        self.state.explosions.clear();
        self.state.next_bomb = 1;
        self.state.damage_cooldown = 0;
        self.state.bomb_placement_cooldown = bombs::BombPlacementCooldown::default();
        self.state.bomb_action_was_down = self.held("KeyE");
        self.state.run_introduction = RunIntroduction {
            age: 0,
            active: true,
        };
        self.state.pause_menu = PauseMenu::hidden();
        self.state.player = Player::starting();
        self.state.floor = floor::generate_basement(seed::stage1_seed(self.state.run_seed_value))?;
        self.state.current_room = self.state.floor.starting_descriptor;
        self.state.visited_rooms = BTreeSet::from([self.state.current_room]);
        self.state.persisted_rooms.clear();
        self.state.room_transition = None;
        self.state.room = Room::from_starting_floor(&self.state.floor);
        self.state.inventory = Inventory {
            coins: 0,
            bombs: 1,
            keys: 0,
        };
        self.state.next_pickup = 1;
        self.state.stage_seed = seed::stage1_seed(self.state.run_seed_value);
        self.refresh_minimap();
        Ok(())
    }
    fn held(&self, code: &str) -> bool {
        self.state.pressed.contains(code)
    }
    /// The room/loot generator supplies a concrete authored selector. This is
    /// an internal gameplay admission API, not a browser debug-spawn command.
    pub fn admit_pickup(
        &mut self,
        variant: u32,
        subtype: u32,
        position: motion::WorldVector,
        appear: bool,
    ) -> Result<u32, String> {
        let id = self.state.next_pickup;
        let next = id
            .checked_add(1)
            .ok_or("pickup identity counter exhausted")?;
        let pickup = Pickup::spawn(id, variant, subtype, position, appear)?;
        self.state.room.pickups.push(pickup);
        self.state.actor_scheduler.admit(actors::ActorRef {
            kind: actors::ActorKind::Pickup,
            id,
        })?;
        self.state.next_pickup = next;
        self.refresh_stage_effects();
        Ok(id)
    }
    /// Fresh authored contents admit before the player, preserving the native
    /// room's actor order. Unsupported selectors stay with the room generator.
    pub fn admit_room_pickups(
        &mut self,
        selectors: &[room_spawns::SpawnSelector],
    ) -> Result<(), String> {
        let mut admitted = Vec::new();
        let mut next = self.state.next_pickup;
        for selector in selectors {
            // Shops and special rooms add prices/pools/replacement rules. Their
            // selectors remain typed until those owners admit that context.
            if self
                .state
                .room
                .descriptor
                .as_ref()
                .is_some_and(|descriptor| descriptor.template.room_type != 1)
            {
                continue;
            }
            if let Some(pickup) = Pickup::from_selector(next, selector, true)? {
                next = next
                    .checked_add(1)
                    .ok_or("pickup identity counter exhausted")?;
                admitted.push(pickup);
            }
        }
        self.state.room.pickups = admitted;
        self.state.next_pickup = next;
        self.rebuild_room_actor_schedule()?;
        self.refresh_stage_effects();
        Ok(())
    }
    /// Restored Room clones retain the collected animation and removed state;
    /// their source handles precede Player0 just as on initial admission.
    pub fn rebuild_room_actor_schedule(&mut self) -> Result<(), String> {
        self.state.actor_scheduler = actors::Scheduler::default();
        for pickup in &self.state.room.pickups {
            if !pickup.removed {
                self.state.actor_scheduler.admit(actors::ActorRef {
                    kind: actors::ActorKind::Pickup,
                    id: pickup.id,
                })?;
            }
        }
        self.state.actor_scheduler.admit(actors::ActorRef {
            kind: actors::ActorKind::Player,
            id: 0,
        })?;
        Ok(())
    }
    fn tick(&mut self) -> Result<(), String> {
        let phase = self.state.system_clock.advance()?;
        let bomb_down = self.held("KeyE");
        let bomb_triggered = bomb_down && !self.state.bomb_action_was_down;
        self.state.bomb_action_was_down = bomb_down;
        if self.state.screen == Screen::Title && phase == phase::Phase::Update {
            self.state.title_animation_frame = (self.state.title_animation_frame + 1) % 40;
        }
        self.state.menu_fade.advance();
        self.state.tick += 1;
        self.state.menu_animation_tick += 1;
        let gameplay_was_paused = self.state.paused;
        if self.state.screen == Screen::Playing {
            self.state.pause_menu.advance(phase);
            if gameplay_was_paused && self.state.pause_menu.phase == PausePhase::Hidden {
                self.state.paused = false;
            }
        }
        if self.state.screen != Screen::Playing || gameplay_was_paused {
            return Ok(());
        }
        self.state.run_tick = self
            .state
            .run_tick
            .checked_add(1)
            .ok_or("run tick counter exhausted")?;
        if self.state.run_introduction.active {
            self.state.run_introduction.age += 1;
            self.state.run_introduction.active = self.state.run_introduction.age < 140;
        }
        let room_before = self.state.current_room;
        let restoring = self
            .state
            .room_transition
            .as_ref()
            .is_some_and(|transition| {
                self.state
                    .persisted_rooms
                    .contains_key(&transition.target_room)
            });
        if traversal::advance_transition(&mut self.state, phase) {
            if room_before != self.state.current_room {
                if restoring {
                    self.rebuild_room_actor_schedule()?;
                } else {
                    let selectors = self
                        .state
                        .room
                        .spawns
                        .as_ref()
                        .expect("first entered descriptor has authored admission")
                        .selectors
                        .clone();
                    self.admit_room_pickups(&selectors)?;
                    // Source load_room admits one full room update before the
                    // inbound transition freezes nonplayer animation.
                    for pickup in &mut self.state.room.pickups {
                        pickup.advance(phase);
                    }
                }
            }
            self.refresh_minimap();
            return Ok(());
        }
        let explosion_fence = self.state.explosions.len();
        let existing = self.state.actor_scheduler.full_phase();
        if phase == phase::Phase::Update {
            for actor in existing.iter().copied() {
                self.advance_actor(actor, phase, false, bomb_triggered)?;
            }
        } else if phase == phase::Phase::Interpolate {
            let mut index = 0;
            while let Some(actor) = self.state.actor_scheduler.get(index) {
                self.advance_actor(actor, phase, !existing.contains(&actor), bomb_triggered)?;
                index += 1;
            }
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
        bombs::advance_explosions(&mut self.state, phase, explosion_fence);
        bombs::retire_removed(&mut self.state);
        self.state.room.pickups.retain(|pickup| !pickup.removed);
        if phase.updates_nonplayers() {
            traversal::check_departure(&mut self.state);
        }
        let surviving = &self.state.tears;
        let pickups = &self.state.room.pickups;
        self.state.actor_scheduler.retain(|actor| match actor.kind {
            actors::ActorKind::Tear => surviving.iter().any(|tear| tear.id == actor.id),
            actors::ActorKind::Pickup => pickups.iter().any(|pickup| pickup.id == actor.id),
            _ => true,
        });
        Ok(())
    }
    fn advance_actor(
        &mut self,
        actor: actors::ActorRef,
        phase: phase::Phase,
        newborn: bool,
        bomb_triggered: bool,
    ) -> Result<(), String> {
        match actor.kind {
            actors::ActorKind::Player => self.update_player(phase, bomb_triggered)?,
            actors::ActorKind::Tear => {
                if let Some(tear) = self.state.tears.iter_mut().find(|tear| tear.id == actor.id) {
                    // Projected height/range remains the existing provisional
                    // representation; native ground/height kernels land next.
                    tear.x += tear.vx;
                    tear.y += tear.vy;
                    if !newborn {
                        tear.age = tear.age.checked_add(1).ok_or("tear age exhausted")?;
                    }
                }
            }
            actors::ActorKind::Bomb => bombs::advance_actor(&mut self.state, phase, actor.id)?,
            actors::ActorKind::Pickup => {
                if let Some(pickup) = self
                    .state
                    .room
                    .pickups
                    .iter_mut()
                    .find(|pickup| pickup.id == actor.id)
                {
                    pickup.advance(phase);
                }
            }
            _ => return Err("actor kind has no admitted runtime yet".into()),
        }
        Ok(())
    }
    fn update_player(&mut self, phase: phase::Phase, bomb_triggered: bool) -> Result<(), String> {
        let dx = i32::from(self.held("KeyD")) - i32::from(self.held("KeyA"));
        let dy = i32::from(self.held("KeyS")) - i32::from(self.held("KeyW"));
        self.state.damage_cooldown = self.state.damage_cooldown.saturating_sub(1);
        let bomb_admitted = self.state.bomb_placement_cooldown.advance(phase);
        // Shooting inherits the velocity entering this phase, before input acceleration.
        let inherited_velocity = (self.state.player.vx, self.state.player.vy);
        motion::advance(&mut self.state.player, &self.state.room, dx, dy);
        if bomb_triggered && bomb_admitted {
            bombs::place(&mut self.state, phase)?;
        }
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
            if self.state.head_timer < 0 {
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
        if phase == phase::Phase::Update {
            self.state.player.head_frame = if self.state.head_timer > 8 { 2 } else { 0 };
            if self.state.fire_cooldown >= 0 {
                self.state.fire_cooldown -= 1;
            }
            if self.state.head_timer >= 0 {
                self.state.head_timer -= 1;
            }
        }
        if let Some(direction) = aim {
            self.state.player.facing = direction;
            if self.state.fire_cooldown < 0 {
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
                self.state.actor_scheduler.admit(actors::ActorRef {
                    kind: actors::ActorKind::Tear,
                    id: self.state.next_tear,
                })?;
                self.state.next_tear = self
                    .state
                    .next_tear
                    .checked_add(1)
                    .ok_or("tear IDs exhausted")?;
                self.state.fire_cooldown = self.state.max_fire_delay;
                self.state.head_timer =
                    ((self.state.max_fire_delay as f32 * 0.45_f32) as i32 + 7).max(11);
            }
        }
        Ok(())
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
