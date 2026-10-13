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
pub mod audio;
pub mod bombs;
pub mod chests;
pub mod combat;
mod content;
mod doors;
mod effects;
pub mod enemy_projectiles;
pub mod floor;
pub mod grid;
mod grid_hazards;
pub mod items;
pub mod keepers;
pub mod layout;
pub mod menu_lifecycle;
pub mod motion;
pub mod mt;
pub mod music;
pub mod npcs;
pub mod phase;
pub mod pickups;
pub mod player_items;
pub mod projectiles;
pub mod rock_blast;
mod room;
pub mod room_assets;
pub mod room_spawns;
pub mod room_templates;
pub mod roundworm;
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
    pub soul_hp: i32,
    pub black_heart_mask: u32,
    pub facing: Direction,
    pub body_facing: Direction,
    pub walk_frame: u8,
    pub head_frame: u8,
    pub shooting: bool,
    pub eye: projectiles::Eye,
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
            soul_hp: 0,
            black_heart_mask: 0,
            facing: Direction::Down,
            body_facing: Direction::Down,
            walk_frame: 0,
            head_frame: 0,
            shooting: false,
            eye: projectiles::Eye::Positive,
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
    pub spawn_seed: u32,
    pub collision_radius: f32,
    pub grid_contact: bool,
    pub motion: projectiles::TearMotion,
}
impl Tear {
    fn project(&mut self) {
        self.x = f64::from(self.motion.presented_position.x) * motion::WORLD_SCALE + 32.0;
        self.y = f64::from(self.motion.presented_position.y + self.motion.render_offset_y)
            * motion::WORLD_SCALE
            - 47.0;
        self.vx = f64::from(self.motion.velocity.x) * motion::WORLD_SCALE;
        self.vy = f64::from(self.motion.velocity.y) * motion::WORLD_SCALE;
    }
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
    pub achievements: Vec<bool>,
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
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunStartRequest {
    /// DOM input awaits the next source menu dispatch; fade runs before input.
    pub input_pending: bool,
    /// MenuController +1 and the distinct fade-out alpha +4.
    pub fading: bool,
    pub fade_alpha: f32,
    /// System pending request is consumed at the following system dispatch.
    pub system_pending: bool,
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
    pub run_start_request: RunStartRequest,
    pub system_clock: phase::Clock,
    pub run_tick: u32,
    pub game_update_count: u32,
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
    pub boot_lifecycle: mt::BootState,
    pub audio: audio::AudioState,
    #[serde(default)]
    pub pending_menu_initializations: Vec<menu_lifecycle::Pending>,
    pub music: music::MusicState,
    pub run_seed_value: u32,
    pub run_seeds: Option<seed::RunSeeds>,
    pub item_pools: Option<items::ItemPools>,
    pub player_items: Option<player_items::PlayerItems>,
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
    pub room_display_flags: Vec<u32>,
    pub revealed_secret_connections: BTreeSet<[usize; 2]>,
    pub persisted_rooms: BTreeMap<usize, Room>,
    pub room_transition: Option<traversal::RoomTransition>,
    pub stage_effects: StageEffects,
    pub inventory: Inventory,
    pub next_pickup: u32,
    pub minimap: Minimap,
    pub tears: Vec<Tear>,
    pub enemy_projectiles: Vec<enemy_projectiles::EnemyProjectile>,
    pub next_enemy_projectile: u32,
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
    Key {
        code: String,
        down: bool,
    },
    Text {
        text: String,
    },
    Tick {
        count: u32,
    },
    AudioVolume {
        sfx_volume: f32,
    },
    MusicVolume {
        volume: f32,
    },
    MusicBackend {
        channel: u8,
        generation: u64,
        event: music::BackendEvent,
    },
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Game {
    pub state: State,
    pub boot_saved_grid: mt::DescriptorSeeds,
    pub boot_level_grid: mt::DescriptorSeeds,
}

impl Game {
    pub fn new(initial_seed: u32) -> Result<Self, String> {
        Self::new_with_boot_epoch(initial_seed, 0)
    }
    pub fn new_with_boot_epoch(initial_seed: u32, boot_epoch_seed: u32) -> Result<Self, String> {
        let run_seed = seed::encode(initial_seed)?;
        let (boot_lifecycle, boot_saved_grid, boot_level_grid) =
            mt::BootLifecycle::new(boot_epoch_seed).into_parts();
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
                run_start_request: RunStartRequest::default(),
                // The public runtime starts at the first rendered Title,
                // after the omitted native introductory cutscene. Its next
                // system iteration is a full update; this clock is relative
                // to that boundary, not the source's elapsed boot counter.
                system_clock: phase::Clock::new(0, true),
                run_tick: 0,
                game_update_count: 0,
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
                boot_lifecycle,
                audio: audio::AudioState::default(),
                pending_menu_initializations: Vec::new(),
                music: music::MusicState::default(),
                run_seed_value: initial_seed,
                run_seeds: None,
                item_pools: None,
                player_items: None,
                stage: 1,
                stage_seed: seed::stage1_seed(initial_seed),
                seeded_run: false,
                achievement_eligibility: true,
                profiles: (1..=3)
                    .map(|slot| Profile {
                        slot,
                        unlocked_characters: vec!["Isaac".into()],
                        selected_character: "Isaac".into(),
                        achievements: vec![false; 179],
                    })
                    .collect(),
                character_catalog: character_catalog(),
                paused: false,
                pressed: BTreeSet::new(),
                player,
                room,
                room_display_flags: vec![0; floor.descriptors.len()],
                floor,
                current_room,
                visited_rooms: BTreeSet::from([current_room]),
                revealed_secret_connections: BTreeSet::new(),
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
                enemy_projectiles: Vec::new(),
                next_enemy_projectile: 1,
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
            boot_saved_grid,
            boot_level_grid,
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
        self.state.audio.begin_command();
        self.state.music.begin_command();
        match command {
            Command::MusicVolume { volume } => {
                if !volume.is_finite() || !(0.0..=1.0).contains(&volume) {
                    return Err("music volume must be finite and within 0..1".into());
                }
                self.state.music.set_volume(volume);
            }
            Command::MusicBackend {
                channel,
                generation,
                event,
            } => self.state.music.backend(channel, generation, event),
            Command::AudioVolume { sfx_volume } => {
                if !sfx_volume.is_finite() || !(0.0..=1.0).contains(&sfx_volume) {
                    return Err("SFX volume must be finite and within 0..1".into());
                }
                self.state.audio.sfx_volume = sfx_volume;
            }
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
        if self.state.screen != Screen::Playing
            && (self.state.run_start_request.input_pending || self.state.run_start_request.fading)
        {
            return Ok(());
        }
        let previous_screen = self.state.screen;
        match self.state.screen {
            Screen::Title => {
                if code == "Enter" {
                    self.state.screen = Screen::Profile;
                }
            }
            Screen::Profile => match code {
                "Enter" => {
                    self.state
                        .pending_menu_initializations
                        .push(menu_lifecycle::Pending {
                            after_audio_requests: self.state.audio.pending.len(),
                            initialization: menu_lifecycle::Initialization::LoadFreshProfile {
                                profile: self.state.profile,
                            },
                        });
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
                "Tab" => {
                    // CharacterUpdate's dialog admission repeats the named
                    // initializer's empty-code/nonzero raw seed field route.
                    self.state.seed_text.clear();
                    self.state.seed_valid = false;
                    self.state
                        .pending_menu_initializations
                        .push(menu_lifecycle::Pending {
                            after_audio_requests: self.state.audio.pending.len(),
                            initialization: menu_lifecycle::Initialization::ResetCharacterSeed,
                        });
                    self.state.screen = Screen::Seed;
                }
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
                "Enter" if self.state.difficulty == "normal" => {
                    self.state.run_start_request.input_pending = true;
                }
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
                    // Keep the decoded setter after every earlier Clear/Reset
                    // even when public inputs queue before the first tick.
                    self.state
                        .pending_menu_initializations
                        .push(menu_lifecycle::Pending {
                            after_audio_requests: self.state.audio.pending.len(),
                            initialization: menu_lifecycle::Initialization::CommitCharacterSeed {
                                seed: self.state.run_seed_value,
                            },
                        });
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
                        for tear in &mut self.state.tears {
                            tear.motion.revert_prediction();
                            tear.project();
                        }
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
        let menu_sound = if previous_screen == Screen::Character {
            match code {
                "ArrowLeft" | "KeyA" => Some(194),
                "ArrowRight" | "KeyD" => Some(195),
                _ => None,
            }
        } else {
            None
        };
        let menu_sound = if code == "Enter"
            && matches!(
                (previous_screen, &self.state.screen),
                (Screen::Title, Screen::Profile)
                    | (Screen::Profile, Screen::Main)
                    | (Screen::Main, Screen::Character)
            ) {
            Some(8)
        } else {
            menu_sound
        };
        if let Some(id) = menu_sound {
            // Source forward confirmations and character-selection sounds.
            // Public input is consumed in the following released system phase.
            self.state.audio.pending.push(audio::SoundRequest {
                id,
                volume: 1.0,
                delay: 0,
                looping: false,
                pitch: 1.0,
            });
        }
        if previous_screen == Screen::Profile && self.state.screen == Screen::Main {
            // SetMenu plays the forward confirmation before CharacterReset.
            self.state
                .pending_menu_initializations
                .push(menu_lifecycle::Pending {
                    after_audio_requests: self.state.audio.pending.len(),
                    initialization: menu_lifecycle::Initialization::ResetCharacterSeed,
                });
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
        // Level.UpdateVisibility586940 retains descriptor flags. Fresh Isaac
        // reveals adjacent rooms, but a secret endpoint requires an actually
        // blown-open connection even when the current room is the secret.
        self.state.room_display_flags[self.state.current_room] |= 5;
        let current_type = self.state.floor.descriptors[self.state.current_room]
            .template
            .room_type;
        for door in &self.state.room.appearance.doors {
            let target = door.target_room as usize;
            let target_type = self.state.floor.descriptors[target].template.room_type;
            let secret = matches!(current_type, 7 | 8) || matches!(target_type, 7 | 8);
            if secret && !door.blown_open {
                continue;
            }
            self.state.room_display_flags[target] |=
                if secret || matches!(target_type, 2 | 6 | 12 | 13 | 18 | 19 | 20 | 21) {
                    3
                } else {
                    5
                };
        }
        self.state.minimap.cells = self
            .state
            .room_display_flags
            .iter()
            .enumerate()
            .filter(|(_, flags)| **flags & 1 != 0)
            .map(|(index, _)| {
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
        self.state.stage_effects.add_tear_shadows(&self.state.tears);
        self.state
            .stage_effects
            .add_pickup_shadows(&self.state.room.pickups);
        self.state
            .stage_effects
            .add_explosion_lights(&self.state.explosions);
    }
    fn start_run(&mut self) -> Result<(), String> {
        // Character 59f470 passes the existing menu seed code to the System
        // request; Game 54eb20 decodes that code into its startup stream. This
        // also applies when no public seed dialog was entered: the real
        // Character initializer owns the generated seed, not reset entropy.
        self.state.run_seed_value = self.state.boot_lifecycle.menu_run_seed;
        self.state.run_seed = seed::encode(self.state.run_seed_value)?;
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
        let mut run_seeds = seed::RunSeeds::new(self.state.run_seed_value)?;
        self.state.item_pools = Some(items::ItemPools::new(
            run_seeds.item_pool_seed(),
            &self.state.profiles[usize::from(self.state.profile - 1)].achievements,
        )?);
        self.state.player_items = Some(player_items::PlayerItems::new(
            run_seeds.initialized_fields[12],
        )?);
        self.state.run_seeds = Some(run_seeds);
        self.state.system_clock.align_run_start()?;
        self.state.screen = Screen::Playing;
        self.state.paused = false;
        self.state.run_tick = 0;
        self.state.game_update_count = 0;
        self.state.tears.clear();
        self.state.impacts.clear();
        self.state.next_tear = 1;
        self.state.enemy_projectiles.clear();
        self.state.next_enemy_projectile = 1;
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
        // Source 58bea7 constructs every descriptor's padded grid seed pairs.
        // floor::generate_basement owns the separate primary xorshift advance.
        self.boot_level_grid = self.state.boot_lifecycle.initialize_level_grid();
        self.state.floor = floor::generate_basement(seed::stage1_seed(self.state.run_seed_value))?;
        self.state.current_room = self.state.floor.starting_descriptor;
        self.state.visited_rooms = BTreeSet::from([self.state.current_room]);
        self.state.room_display_flags = vec![0; self.state.floor.descriptors.len()];
        self.state.revealed_secret_connections.clear();
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
        // Source Level generation loads the start room, then immediately calls
        // RoomMusic. This runtime's Level contract is fresh normal Basement.
        if let Some(id) = music::stage_track(self.state.stage, false) {
            self.state.music.room(id);
        }
        self.refresh_minimap();
        Ok(())
    }
    fn held(&self, code: &str) -> bool {
        self.state.pressed.contains(code)
    }
    /// The native list admits Player first, then supported selectors in their
    /// authored order. Persist that order with the room rather than regrouping
    /// NPCs and pickups when rebuilding the shared actor scheduler.
    pub fn admit_room_pickups(
        &mut self,
        selectors: &[room_spawns::SpawnSelector],
    ) -> Result<(), String> {
        let mut pickups = Vec::new();
        let mut npcs = Vec::new();
        let mut order = Vec::new();
        let mut next = self.state.next_pickup;
        let room_type = self
            .state
            .room
            .descriptor
            .as_ref()
            .map_or(1, |descriptor| descriptor.template.room_type);
        for selector in selectors {
            let id = u32::try_from(npcs.len()).map_err(|_| "NPC identity exhausted")?;
            if let Some(npc) = npcs::Npc::from_selector(id, selector)? {
                order.push(actors::ActorRef {
                    kind: actors::ActorKind::Npc,
                    id,
                });
                npcs.push(npc);
            } else if selector.entity_type == 5
                && selector.variant == 100
                && !matches!(room_type, 2 | 3 | 14 | 22)
            {
                // Free natural pedestals use the run's actual ItemPool owner.
                // Shop/devil pricing and challenge overrides remain separate.
                if let Some(mut pickup) = Pickup::from_collectible_selector(
                    next,
                    selector,
                    self.state
                        .item_pools
                        .as_mut()
                        .ok_or("missing run item pools")?,
                    room_type,
                    &mut self.state.boot_lifecycle.shared,
                )? {
                    pickup.birth_game_update = self.state.game_update_count;
                    order.push(actors::ActorRef {
                        kind: actors::ActorKind::Pickup,
                        id: next,
                    });
                    next = next
                        .checked_add(1)
                        .ok_or("pickup identity counter exhausted")?;
                    pickups.push(pickup);
                }
            } else if room_type == 1 || matches!((room_type, selector.variant), (4, 20) | (7, 30)) {
                // Direct treasure-room coins and secret-room regular keys
                // keep free pickup Init/Collect rules (no shop placeholder).
                // Other special contexts retain typed selectors until their
                // prices, pools and replacements are admitted.
                if let Some(mut pickup) = Pickup::from_selector(next, selector, true)? {
                    pickup.birth_game_update = self.state.game_update_count;
                    order.push(actors::ActorRef {
                        kind: actors::ActorKind::Pickup,
                        id: next,
                    });
                    next = next
                        .checked_add(1)
                        .ok_or("pickup identity counter exhausted")?;
                    pickups.push(pickup);
                }
            }
        }
        self.state.room.pickups = pickups;
        self.state.room.npcs = npcs;
        self.state.room.authored_actor_order = order;
        self.state.next_pickup = next;
        self.rebuild_room_actor_schedule()?;
        self.refresh_stage_effects();
        Ok(())
    }
    /// Persisted Room clones retain supported actor handles in authored order.
    pub fn rebuild_room_actor_schedule(&mut self) -> Result<(), String> {
        self.state.actor_scheduler = actors::Scheduler::default();
        self.state.actor_scheduler.admit(actors::ActorRef {
            kind: actors::ActorKind::Player,
            id: 0,
        })?;
        for actor in self.state.room.authored_actor_order.iter().copied() {
            let admitted = match actor.kind {
                actors::ActorKind::Npc => self
                    .state
                    .room
                    .npcs
                    .iter()
                    .any(|npc| npc.id == actor.id && !npc.removed),
                actors::ActorKind::Pickup => self
                    .state
                    .room
                    .pickups
                    .iter()
                    .any(|pickup| pickup.id == actor.id && !pickup.removed),
                _ => return Err("invalid authored room actor handle".into()),
            };
            if admitted {
                self.state.actor_scheduler.admit(actor)?;
            }
        }
        Ok(())
    }
    fn flush_menu_initializations(&mut self, audio_counter: u32) -> Result<(), String> {
        menu_lifecycle::validate(
            &self.state.pending_menu_initializations,
            self.state.audio.pending.len(),
        )?;
        let requests = std::mem::take(&mut self.state.audio.pending);
        let mut initializers = std::mem::take(&mut self.state.pending_menu_initializations)
            .into_iter()
            .peekable();
        for boundary in 0..=requests.len() {
            while initializers
                .peek()
                .is_some_and(|event| event.after_audio_requests == boundary)
            {
                match initializers
                    .next()
                    .expect("pending menu initializer")
                    .initialization
                {
                    menu_lifecycle::Initialization::LoadFreshProfile { .. } => {
                        self.boot_saved_grid = self.state.boot_lifecycle.clear_saved_game();
                    }
                    menu_lifecycle::Initialization::ResetCharacterSeed => {
                        self.state.boot_lifecycle.reset_character_seed();
                    }
                    menu_lifecycle::Initialization::CommitCharacterSeed { seed } => {
                        self.state.boot_lifecycle.commit_character_seed(seed)?;
                    }
                }
            }
            if let Some(request) = requests.get(boundary) {
                self.state.audio.admit(
                    &mut self.state.boot_lifecycle.shared,
                    audio_counter,
                    *request,
                );
            }
        }
        Ok(())
    }
    fn tick(&mut self) -> Result<(), String> {
        // Source 594a10 consumes the prior menu request before game dispatch.
        // Room construction and clock alignment happen here, after the f32
        // confirmation fade, rather than in the zero-tick DOM key callback.
        if self.state.run_start_request.system_pending {
            self.state.run_start_request.system_pending = false;
            self.start_run()?;
        }
        let audio_counter = self.state.system_clock.system_counter;
        let phase = self.state.system_clock.advance()?;
        if phase != phase::Phase::Skipped {
            self.state.music.flush_pending();
        }
        self.flush_menu_initializations(audio_counter)?;
        let bomb_down = self.held("KeyE");
        let bomb_triggered = bomb_down && !self.state.bomb_action_was_down;
        self.state.bomb_action_was_down = bomb_down;
        if self.state.screen == Screen::Title && phase == phase::Phase::Update {
            self.state.title_animation_frame = (self.state.title_animation_frame + 1) % 40;
        }
        self.state.menu_fade.advance();
        if phase != phase::Phase::Skipped && self.state.screen != Screen::Playing {
            let request = &mut self.state.run_start_request;
            if request.fading {
                // Source 59b180 updates +4 before Character59d080 tests >=1.
                // Forty f32 additions remain below one; the forty-first wins.
                request.fade_alpha += 0.025_f32;
                if request.fade_alpha >= 1.0_f32 {
                    request.system_pending = true;
                }
            }
            if request.input_pending {
                request.input_pending = false;
                request.fading = true;
                self.state.music.crossfade(95);
            }
        }
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
            if phase != phase::Phase::Skipped {
                self.state.music.advance();
            }
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
                doors::begin_room_admission(&mut self.state);
                // Transition state1 explicitly dispatches Player::Update at
                // 5ddf87 before loading, then the admitted Room::Update at
                // 5de048 dispatches it once more. Inbound phases stay frozen.
                combat::advance_player_immunity(&mut self.state);
                combat::advance_player_immunity(&mut self.state);
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
                    // The admitted Room::Update follows authored mixed order.
                    for actor in self.state.room.authored_actor_order.clone() {
                        match actor.kind {
                            actors::ActorKind::Npc => {
                                self.advance_npc(actor.id, phase)?;
                            }
                            actors::ActorKind::Pickup => {
                                self.advance_pickup(actor.id, phase)?;
                            }
                            _ => return Err("invalid authored room actor handle".into()),
                        }
                    }
                    self.state.room.pickups.retain(|pickup| !pickup.removed);
                    let pickups = &self.state.room.pickups;
                    self.state.actor_scheduler.retain(|actor| {
                        actor.kind != actors::ActorKind::Pickup
                            || pickups.iter().any(|pickup| pickup.id == actor.id)
                    });
                    npcs::admit_room_combat(&mut self.state);
                }
            }
            self.refresh_minimap();
            return Ok(());
        }
        doors::advance(&mut self.state, phase);
        let explosion_fence = self.state.explosions.len();
        grid_hazards::advance(&mut self.state, phase)?;
        let existing = self.state.actor_scheduler.full_phase();
        if phase == phase::Phase::Update {
            self.state.game_update_count = self
                .state
                .game_update_count
                .checked_add(1)
                .ok_or("game update counter exhausted")?;
            rock_blast::advance_markers(&mut self.state);
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
        // Source manager454df0 finishes its admission-fenced actor updates
        // before4545d0 admits entity contacts. Pickups can become idle in this
        // full phase and collect against the player's newly moved position.
        if phase == phase::Phase::Update {
            npcs::player_contacts(&mut self.state)?;
            npcs::nonplayer_contacts(&mut self.state)?;
            enemy_projectiles::player_contacts(&mut self.state)?;
            // Actual full manager contact pass uses typed ground coordinates.
            let tears: Vec<_> = self
                .state
                .tears
                .iter()
                .filter(|tear| tear.motion.lifetime == projectiles::Lifetime::Alive)
                .map(|tear| (tear.id, tear.motion, tear.collision_radius))
                .collect();
            for (id, mut motion, radius) in tears {
                if npcs::tear_hit(&mut self.state, &mut motion, radius, id, 3.5)?.is_some() {
                    if let Some(tear) = self.state.tears.iter_mut().find(|tear| tear.id == id) {
                        motion.lifetime = projectiles::Lifetime::Dying;
                        tear.motion = motion;
                        tear.project();
                    }
                }
            }
            for index in 0..self.state.room.pickups.len() {
                if self.state.room.pickups[index].variant == 100 {
                    if !player_items::collect(&mut self.state, index)? {
                        player_items::empty_altar_contact(&mut self.state, index)?;
                    }
                    continue;
                }
                let pickup = &mut self.state.room.pickups[index];
                if pickup.touches(&self.state.player)
                    && pickup.try_collect(&mut self.state.inventory)
                {
                    // Collect504480 routes key ->504720 and bomb ->5046d0
                    // after inventory mutation, before the System increment.
                    let sound = match pickup.variant {
                        30 => Some((58, 0.85)),
                        40 => Some((201, 1.0)),
                        _ => None,
                    };
                    if let Some((id, volume)) = sound {
                        self.state.audio.admit(
                            &mut self.state.boot_lifecycle.shared,
                            self.state.system_clock.system_counter - 1,
                            audio::SoundRequest {
                                id,
                                volume,
                                delay: 2,
                                looping: false,
                                pitch: 1.0,
                            },
                        );
                    }
                }
            }
        }
        for impact in &mut self.state.impacts {
            impact.age += 1;
        }
        self.state.impacts.retain(|impact| impact.age < 12);
        let mut surviving = Vec::new();
        for tear in std::mem::take(&mut self.state.tears) {
            if tear.motion.lifetime == projectiles::Lifetime::Removed {
                self.state.impacts.push(Impact {
                    id: tear.id,
                    x: f64::from(tear.motion.position.x) * motion::WORLD_SCALE + 32.0,
                    y: f64::from(tear.motion.position.y) * motion::WORLD_SCALE - 46.0,
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
        self.state.room.npcs.retain(|npc| !npc.removed);
        self.state
            .enemy_projectiles
            .retain(|projectile| !projectile.removed);
        self.refresh_minimap();
        if phase.updates_nonplayers() {
            npcs::advance_room_clear(&mut self.state);
            let dx = i32::from(self.held("KeyD")) - i32::from(self.held("KeyA"));
            let dy = i32::from(self.held("KeyS")) - i32::from(self.held("KeyW"));
            doors::check_unlock(&mut self.state, dx, dy);
            traversal::check_departure(&mut self.state);
        }
        let surviving = &self.state.tears;
        let pickups = &self.state.room.pickups;
        self.state.actor_scheduler.retain(|actor| match actor.kind {
            actors::ActorKind::Tear => surviving.iter().any(|tear| tear.id == actor.id),
            actors::ActorKind::Pickup => pickups.iter().any(|pickup| pickup.id == actor.id),
            actors::ActorKind::Npc => self.state.room.npcs.iter().any(|npc| npc.id == actor.id),
            actors::ActorKind::Projectile => self
                .state
                .enemy_projectiles
                .iter()
                .any(|projectile| projectile.id == actor.id),
            _ => true,
        });
        if phase != phase::Phase::Skipped {
            self.state.music.advance();
        }
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
                let grid = if phase == phase::Phase::Update && !newborn {
                    Some(
                        self.state
                            .room
                            .grid
                            .clone()
                            .map(grid::Grid::new)
                            .transpose()?
                            .unwrap_or_else(|| traversal::doorway_grid(&self.state.room)),
                    )
                } else {
                    None
                };
                if let Some(tear) = self.state.tears.iter_mut().find(|tear| tear.id == actor.id) {
                    if newborn {
                        tear.motion.present_birth(phase)?;
                    } else {
                        tear.motion.advance(phase, tear.grid_contact)?;
                        if phase == phase::Phase::Update {
                            tear.age = tear.age.checked_add(1).ok_or("tear age exhausted")?;
                            tear.grid_contact = grid
                                .as_ref()
                                .expect("full tear grid query")
                                .contacts_regular_tear(tear.motion.position);
                        }
                    }
                    tear.project();
                }
            }
            actors::ActorKind::Bomb => bombs::advance_actor(&mut self.state, phase, actor.id)?,
            actors::ActorKind::Pickup => {
                self.advance_pickup(actor.id, phase)?;
            }
            actors::ActorKind::Npc => {
                self.advance_npc(actor.id, phase)?;
            }
            actors::ActorKind::Projectile => {
                enemy_projectiles::advance_actor(&mut self.state, phase, actor.id)?
            }
            _ => return Err("actor kind has no admitted runtime yet".into()),
        }
        Ok(())
    }
    fn advance_pickup(&mut self, id: u32, phase: phase::Phase) -> Result<(), String> {
        let Some(index) = self
            .state
            .room
            .pickups
            .iter()
            .position(|pickup| pickup.id == id)
        else {
            return Ok(());
        };
        if phase.updates_nonplayers() {
            if let Some(variant) = self.state.room.pickups[index].bomb_conversion_variant() {
                let position = self.state.room.pickups[index].position;
                // Update50757d/507cc0 calls the real shared-MT seed wrapper,
                // Game::Spawn4/variant at the PU position, then PU::Remove.
                let seed = self.state.boot_lifecycle.shared.next_u32();
                bombs::admit_conversion(&mut self.state, variant, seed, position)?;
                self.state.room.pickups[index].removed = true;
                return Ok(());
            }
        }
        self.state.room.pickups[index].advance(phase, self.state.game_update_count);
        Ok(())
    }
    fn advance_npc(&mut self, id: u32, phase: phase::Phase) -> Result<(), String> {
        let grid = grid::Grid::new(self.state.room.grid.clone().expect("admitted NPC has grid"))?;
        let player = bombs::player_world(&self.state);
        let Some(index) = self.state.room.npcs.iter().position(|npc| npc.id == id) else {
            return Ok(());
        };
        if self.state.room.npcs[index].kind != 244 {
            return self.state.room.npcs[index].advance(
                phase,
                self.state.game_update_count,
                &mut self.state.boot_lifecycle.shared,
                player,
                &grid,
            );
        }
        let mut occupied = vec![player];
        occupied.extend(
            self.state
                .room
                .npcs
                .iter()
                .filter(|npc| !npc.removed)
                .map(|npc| npc.position),
        );
        occupied.extend(
            self.state
                .room
                .pickups
                .iter()
                .filter(|pickup| !pickup.removed)
                .map(|pickup| pickup.position),
        );
        occupied.extend(self.state.bombs.iter().map(|bomb| bomb.motion.position));
        let player_velocity = bombs::player_velocity(&self.state);
        let shot = roundworm::advance(
            &mut self.state.room.npcs[index],
            phase,
            &mut self.state.boot_lifecycle.shared,
            &mut roundworm::World {
                player,
                player_velocity,
                occupied: &occupied,
                grid: &grid,
                audio: &mut self.state.audio,
                system_counter: self.state.system_clock.system_counter.wrapping_sub(1),
            },
        )?;
        if let Some(shot) = shot {
            let id = self.state.next_enemy_projectile;
            self.state.next_enemy_projectile = id
                .checked_add(1)
                .ok_or("enemy projectile identity exhausted")?;
            self.state
                .enemy_projectiles
                .push(enemy_projectiles::EnemyProjectile::birth(
                    id,
                    self.state.room.npcs[index].id,
                    shot,
                ));
            self.state.actor_scheduler.admit(actors::ActorRef {
                kind: actors::ActorKind::Projectile,
                id,
            })?;
        }
        Ok(())
    }
    fn update_player(&mut self, phase: phase::Phase, bomb_triggered: bool) -> Result<(), String> {
        player_items::advance_presentation(&mut self.state, phase);
        let dx = i32::from(self.held("KeyD")) - i32::from(self.held("KeyA"));
        let dy = i32::from(self.held("KeyS")) - i32::from(self.held("KeyW"));
        combat::advance_player_immunity(&mut self.state);
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
            if self.state.fire_cooldown < 0
                && !self
                    .state
                    .player_items
                    .as_ref()
                    .is_some_and(|items| items.pickup_animation_guard)
            {
                let (dx, dy) = match direction {
                    Direction::Left => (-1.0, 0.0),
                    Direction::Right => (1.0, 0.0),
                    Direction::Up => (0.0, -1.0),
                    Direction::Down => (0.0, 1.0),
                };
                let aim = motion::WorldVector { x: dx, y: dy };
                let inherited = motion::WorldVector {
                    x: (inherited_velocity.0 / motion::WORLD_SCALE) as f32,
                    y: (inherited_velocity.1 / motion::WORLD_SCALE) as f32,
                };
                let velocity = projectiles::stored_velocity(aim, inherited)?;
                self.state.player.eye = self.state.player.eye.flipped();
                let eye_uniform = self.state.boot_lifecycle.shared.uniform();
                let origin = projectiles::eye_origin(
                    bombs::player_world(&self.state),
                    velocity,
                    self.state.player.eye,
                    eye_uniform,
                )?;
                let spawn_seed = self.state.boot_lifecycle.shared.next_u32();
                // Init computes a falling speed that FireTear overwrites. Its
                // shared draw is still required before sound admission.
                let _initializer_falling_speed =
                    (self.state.boot_lifecycle.shared.uniform() - 0.5) * 0.2;
                self.state.audio.admit(
                    &mut self.state.boot_lifecycle.shared,
                    self.state.system_clock.system_counter - 1,
                    audio::SoundRequest {
                        id: 153,
                        volume: 1.0,
                        delay: 2,
                        looping: false,
                        pitch: 1.0,
                    },
                );
                let final_uniform = self.state.boot_lifecycle.shared.uniform();
                let mut tear = Tear {
                    id: self.state.next_tear,
                    x: 0.0,
                    y: 0.0,
                    vx: 0.0,
                    vy: 0.0,
                    age: 0,
                    spawn_seed,
                    collision_radius: projectiles::collision_radius(3.5, 8.0)?,
                    grid_contact: false,
                    motion: projectiles::TearMotion::spawn(
                        origin,
                        velocity,
                        projectiles::Parameters::FRESH_ISAAC,
                        final_uniform,
                    )?,
                };
                tear.project();
                self.state.tears.push(tear);
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
        player_items::finish_player_update(&mut self.state, phase)?;
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
