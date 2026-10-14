//! A fresh, narrow foundation driven by observed base-Rebirth menu/run captures.
//! World positions use logical pixels in a 480 × 270 viewport; public stepping
//! follows full/intermediate source phases at nominal 60 presentations per second.
//! Sprite offsets and stage effects are explicit state separate from ground positions.
//! Public seed encoding is supported; original room RNG routing and simulation
//! ticks remain unverified. Scene descriptors are ordinary generated state.
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
pub mod active_items;
pub mod actors;
pub mod attack_color;
pub mod attack_flags;
pub mod audio;
pub mod blood_effects;
pub mod bombs;
pub mod camera;
pub mod champions;
pub mod chests;
pub mod combat;
mod content;
pub mod dingle;
pub mod dips;
mod doors;
mod effects;
pub mod enemy_projectiles;
mod entities;
pub use entities::supports_npc_selector;
pub mod familiars;
pub mod fireplaces;
pub mod floor;
mod fmath;
pub mod gib_counts;
pub mod gluttony;
pub mod grid;
mod grid_hazards;
pub mod grid_records;
mod health_damage;
mod horfs;
pub mod information_animation;
pub mod input;
mod item_pipeline;
pub mod items;
pub mod keepers;
pub mod lasers;
pub mod layout;
pub mod level2_flies;
pub mod level_progress;
pub mod menu_lifecycle;
pub mod monstro;
pub mod motion;
pub mod mt;
pub mod music;
pub mod nightmares;
pub mod npcs;
pub mod pathfinder;
pub mod phase;
pub mod pickups;
mod plain_npc;
pub mod player_animation;
pub mod player_death;
pub mod player_items;
pub mod player_save;
pub mod player_stats;
pub mod poop_effects;
pub mod projectiles;
pub mod public_actions;
pub mod rock_blast;
mod room;
pub mod room_assets;
pub mod room_costs;
pub mod room_spawns;
pub mod room_templates;
pub mod roundworm;
pub mod run_admission;
pub mod saved_game;
pub mod seed;
pub mod traversal;
mod weapon_plan;
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
    /// Entity+31d; distinct from the persistent Player+7d4 controls field.
    pub visible: bool,
    /// Source Player+7d8, separate from pickup collection delay.
    pub input_cooldown: i32,
    pub dead: bool,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub hp: u8,
    pub max_hp: u8,
    pub soul_hp: i32,
    pub black_heart_mask: u32,
    /// Actual normal-Isaac Player+bbc Eternal half-heart owner.
    pub eternal_hearts: i32,
    pub facing: Direction,
    pub body_facing: Direction,
    pub walk_frame: u8,
    pub head_frame: u8,
    pub shooting: bool,
    pub shooting_lock: u32,
    /// Actual Player+7d4 input gate, distinct from Entity+31d visibility.
    pub controls_enabled: bool,
    pub eye: projectiles::Eye,
    /// Actual Player+c84, independent of manager tear handles.
    pub shots_fired: u32,
    pub render_offset: Point,
}
impl Player {
    fn starting() -> Self {
        Self {
            visible: true,
            input_cooldown: 0,
            dead: false,
            x: 240.0,
            y: 214.0,
            vx: 0.0,
            vy: 0.0,
            hp: 6,
            max_hp: 6,
            soul_hp: 0,
            black_heart_mask: 0,
            eternal_hearts: 0,
            facing: Direction::Down,
            body_facing: Direction::Down,
            walk_frame: 0,
            head_frame: 0,
            shooting: false,
            shooting_lock: 0,
            controls_enabled: true,
            eye: projectiles::Eye::Positive,
            shots_fired: 0,
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
    /// Actual Tear+838, copied from Player+c84 before its increment.
    pub shot_ordinal: u32,
    /// Source setter5411e0 writes collision48c and base83c at birth.
    pub collision_damage: f32,
    pub base_damage: f32,
    ///Actual kind1 birth copies flags7F0 and permanent ColorMod3A0.
    pub attack_flags: attack_flags::AttackFlags,
    pub source_color: attack_color::AttackColor,
    pub scale: f32,
    pub animation_tier: u8,
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
    /// HUD::ShowItemText 5732c0 title on the shared ui_streak sprite.
    #[serde(default = "stage_title")]
    pub title: String,
    /// The stage name (Level::ShowName 585500) rather than a later banner
    /// such as the miniboss VS text, which restarts the same streak clock.
    #[serde(default = "stage_streak")]
    pub stage: bool,
}
fn stage_title() -> String {
    "BASEMENT I".into()
}
fn stage_streak() -> bool {
    true
}
impl RunIntroduction {
    fn stage(active: bool) -> Self {
        Self {
            age: 0,
            active,
            title: stage_title(),
            stage: true,
        }
    }
}
/// HUD boss bar: SetFill 5732a0 at HUD+61894 (-1 hidden) and its flash
/// countdown at +61898; the manager's non-shrinking boss max HP is +ec.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BossBar {
    pub fill: f32,
    pub flash: i32,
    pub maximum: f32,
}
impl Default for BossBar {
    fn default() -> Self {
        Self {
            fill: -1.0,
            flash: -1,
            maximum: 0.0,
        }
    }
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
    pub run_admission: run_admission::RunAdmission,
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
    pub saved_run_summary: Option<saved_game::SavedRunSummary>,
    pub audio: audio::AudioState,
    #[serde(default)]
    pub pending_menu_initializations: Vec<menu_lifecycle::Pending>,
    pub music: music::MusicState,
    pub run_seed_value: u32,
    pub run_seeds: Option<seed::RunSeeds>,
    pub item_pools: Option<items::ItemPools>,
    pub player_items: Option<player_items::PlayerItems>,
    pub player_animation: player_animation::PlayerAnimation,
    pub public_actions: public_actions::PublicActions,
    #[serde(default)]
    pub player_stats: player_stats::PlayerStats,
    #[serde(default)]
    pub weapon_cache: weapon_plan::WeaponCache,
    #[serde(default)]
    pub weapon_admission: weapon_plan::WeaponAdmission,
    pub information_animation: information_animation::InformationAnimation,
    pub stage: u32,
    pub stage_seed: u32,
    pub level_progress: level_progress::LevelProgress,
    pub nightmares: nightmares::Nightmares,
    pub seeded_run: bool,
    pub achievement_eligibility: bool,
    pub profiles: Vec<Profile>,
    pub character_catalog: Vec<CharacterDefinition>,
    pub paused: bool,
    pub pressed: BTreeSet<String>,
    pub player: Player,
    pub player_death: player_death::PlayerDeath,
    pub room: Room,
    #[serde(default)]
    pub camera: Option<camera::Camera>,
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
    pub fire_particles: Vec<fireplaces::Ember>,
    pub next_effect: u32,
    pub sprite_effects: Vec<familiars::SpriteEffect>,
    #[serde(default)]
    pub lasers: Vec<lasers::Laser>,
    #[serde(default)]
    pub next_laser: u32,
    pub familiars: Vec<familiars::Familiar>,
    pub next_familiar: u32,
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
    #[serde(default)]
    pub boss_bar: BossBar,
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
        #[serde(
            default,
            rename = "timestampMillis",
            skip_serializing_if = "Option::is_none"
        )]
        timestamp_millis: Option<u64>,
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
    pub saved_game: mt::SavedGameSeeds,
    pub boot_level_grid: mt::DescriptorSeeds,
    pub saved_run: Option<saved_game::SavedRun>,
    pub player_manager_secondary_rng: saved_game::PlayerManagerSecondaryRng,
}

impl Game {
    pub fn new(initial_seed: u32) -> Result<Self, String> {
        Self::new_with_boot_epoch(initial_seed, 0)
    }
    pub fn new_with_boot_epoch(initial_seed: u32, boot_epoch_seed: u32) -> Result<Self, String> {
        let run_seed = seed::encode(initial_seed)?;
        let (boot_lifecycle, saved_game, boot_level_grid) =
            mt::BootLifecycle::new(boot_epoch_seed).into_parts();
        let player = Player::starting();
        let floor = floor::generate_basement(seed::stage1_seed(initial_seed))?;
        let current_room = floor.starting_descriptor;
        let room = Room::from_starting_floor(&floor);
        let camera = Some(camera::Camera::new(&room, None));
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
                run_admission: run_admission::RunAdmission::new(boot_epoch_seed),
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
                saved_run_summary: None,
                audio: audio::AudioState::default(),
                pending_menu_initializations: Vec::new(),
                music: music::MusicState::default(),
                run_seed_value: initial_seed,
                run_seeds: None,
                item_pools: None,
                player_items: None,
                player_animation: player_animation::PlayerAnimation::default(),
                public_actions: public_actions::PublicActions::default(),
                player_stats: player_stats::PlayerStats::default(),
                weapon_cache: weapon_plan::WeaponCache::fresh_player(),
                weapon_admission: weapon_plan::WeaponAdmission::default(),
                information_animation: information_animation::InformationAnimation::default(),
                stage: 1,
                stage_seed: seed::stage1_seed(initial_seed),
                level_progress: level_progress::LevelProgress::default(),
                nightmares: nightmares::Nightmares::new()?,
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
                player_death: player_death::PlayerDeath::default(),
                room,
                camera,
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
                fire_particles: Vec::new(),
                next_effect: 1,
                sprite_effects: Vec::new(),
                lasers: Vec::new(),
                next_laser: 1,
                familiars: Vec::new(),
                next_familiar: 1,
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
                run_introduction: RunIntroduction::stage(false),
                boss_bar: BossBar::default(),
                pause_menu: PauseMenu::hidden(),
            },
            saved_game,
            boot_level_grid,
            saved_run: None,
            player_manager_secondary_rng: saved_game::PlayerManagerSecondaryRng::default(),
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
                if !input::is_public_key(&code) {
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
            Command::Tick {
                count,
                timestamp_millis,
            } => {
                if timestamp_millis
                    .is_some_and(|timestamp| timestamp > run_admission::MAX_SAFE_TIMESTAMP_MS)
                {
                    return Err("host timestamp exceeds the JavaScript safe integer range".into());
                }
                if timestamp_millis.is_some() && count != 1 {
                    return Err("a host timestamp requires exactly one system iteration".into());
                }
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
                    self.tick(timestamp_millis)?;
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
        if self.state.screen == Screen::Playing
            && self
                .state
                .room_transition
                .as_ref()
                .is_some_and(|transition| {
                    matches!(
                        transition.phase,
                        traversal::TransitionPhase::BossScene
                            | traversal::TransitionPhase::BossCleanup
                    )
                })
        {
            // Game54f760 returns while transition-active after its narrow
            // Scene key handler. Physical action14 mapping remains unaccepted.
            return Ok(());
        }
        if self.state.player_death.game_over.is_some()
            || self.state.nightmares.system_active
            || self.state.nightmares.request_pending
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
                    self.load_menu_character();
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
                    self.validate_run_character()?;
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
                if code == "Escape" && !self.state.paused && self.state.room_transition.is_some() {
                    // The original refuses to pause while a room transition is in progress.
                } else if code == "Escape"
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
    /// Reject unsupported requests before the deferred start flag can persist
    /// across failing ticks. Queued menu initializers may still change the seed,
    /// so Random requires a supported runtime for every possible choice.
    fn validate_run_character(&self) -> Result<(), String> {
        let unlocked =
            &self.state.profiles[usize::from(self.state.profile - 1)].unlocked_characters;
        let candidates = if self.state.character == "Random" {
            unlocked.as_slice()
        } else {
            std::slice::from_ref(&self.state.character)
        };
        if candidates.is_empty() {
            return Err("profile has no unlocked character".into());
        }
        for character in candidates {
            if !unlocked.contains(character) {
                return Err("selected character is locked in this profile".into());
            }
            if character != "Isaac" {
                return Err("selected character runtime is not implemented".into());
            }
        }
        Ok(())
    }
    /// The menus show the profile's selected character, including Random,
    /// not the character a run resolved it to.
    fn load_menu_character(&mut self) {
        self.state.character = self.state.profiles[usize::from(self.state.profile - 1)]
            .selected_character
            .clone();
        self.state.character_selection = self
            .state
            .character_catalog
            .iter()
            .position(|character| character.name == self.state.character)
            .unwrap_or(0) as u8;
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
        self.state
            .stage_effects
            .add_familiar_shadows(&self.state.familiars);
        self.state
            .stage_effects
            .add_npc_shadows(&self.state.room.npcs);
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
        self.state.fire_particles.clear();
        self.state.next_effect = 1;
        self.state.sprite_effects.clear();
        self.state.lasers.clear();
        self.state.next_laser = 1;
        self.state.familiars.clear();
        self.state.next_familiar = 1;
        self.state.fire_cooldown = -1;
        self.state.max_fire_delay = 10;
        self.state.player_stats = player_stats::PlayerStats::default();
        self.state.weapon_cache = weapon_plan::WeaponCache::fresh_player();
        self.state.weapon_admission = weapon_plan::WeaponAdmission::fresh_isaac();
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
        self.state.run_introduction = RunIntroduction::stage(true);
        self.state.boss_bar = BossBar::default();
        self.state.pause_menu = PauseMenu::hidden();
        self.state.player = Player::starting();
        self.state.player_death = player_death::PlayerDeath::default();
        self.state.player_animation = player_animation::PlayerAnimation::default();
        // Source 58bea7 constructs every descriptor's padded grid seed pairs.
        // floor::generate_basement owns the separate primary xorshift advance.
        self.state.level_progress.begin_level();
        self.boot_level_grid = self.state.boot_lifecycle.initialize_level_grid();
        self.state.floor = floor::generate_basement(seed::stage1_seed(self.state.run_seed_value))?;
        self.state.current_room = self.state.floor.starting_descriptor;
        self.state.visited_rooms = BTreeSet::from([self.state.current_room]);
        self.state.room_display_flags = vec![0; self.state.floor.descriptors.len()];
        self.state.revealed_secret_connections.clear();
        self.state.persisted_rooms.clear();
        self.state.room_transition = None;
        self.state.room = Room::from_starting_floor(&self.state.floor);
        self.state.camera = Some(camera::Camera::new(&self.state.room, None));
        // PlayerManager.Init precedes Level.Init -> Room.Init. Room reset
        // keeps its admitted Player pointer/ID and starts a new allocator.
        self.state.actor_scheduler.clear_room();
        self.state.inventory = Inventory {
            coins: 0,
            bombs: 1,
            keys: 0,
        };
        self.state.next_pickup = 1;
        self.state.stage_seed = seed::stage1_seed(self.state.run_seed_value);
        // Level585600 ->5b8340 records the selected ordinary stage resource.
        self.state.nightmares.select_normal_basement(1)?;
        grid_records::admit_fresh(
            &mut self.state.room,
            &mut self.state.boot_lifecycle.shared,
            self.state.game_update_count,
        )?;
        // Source Level generation loads the start room, then immediately calls
        // RoomMusic. This runtime's Level contract is fresh normal Basement.
        if let Some(id) = music::stage_track(self.state.stage, false) {
            self.state.music.room(id);
        }
        self.save_supported_run()?;
        // The save keeps Door.Init's state1 records; the cleared start room's
        // opened doors are state2 again by gameplay start in the original.
        if let Some(grid) = &mut self.state.room.grid {
            for door in &self.state.room.appearance.doors {
                if door.state != "Opened" {
                    continue;
                }
                let column = ((door.world_position.x - 40.0) / 40.0) as usize;
                let row = ((door.world_position.y - 120.0) / 40.0) as usize;
                if let Some(entity) = grid
                    .cells
                    .get_mut(column + row * usize::from(grid.width))
                    .and_then(|cell| cell.entity.as_mut())
                    .filter(|entity| entity.kind == 16)
                {
                    entity.state = 2;
                }
            }
        }
        // Fresh54eb20 invokes54eaa0 after Level.Init/Save and the primary
        // manager initialization; room factories see the prior control byte.
        self.state
            .run_admission
            .initialize_fresh(&mut self.state.player)?;
        self.refresh_minimap();
        Ok(())
    }

    /// Level.Init calls System.Save at 58c411 once the starting room is loaded.
    /// Game.Save clears the destination, updates Room grid records, then copies
    /// ordinary Level rows and PlayerManager state in that source order.
    fn save_supported_run(&mut self) -> Result<(), String> {
        self.saved_game = self.state.boot_lifecycle.clear_saved_game();
        // Game.Save 54f33b copies the actual modeled ItemPool before Room.Save.
        // Retain its owned vectors/RNGs; copying neither reseeds nor samples.
        let item_pools = self
            .state
            .item_pools
            .clone()
            .ok_or("run save requires initialized item pools")?;
        let grid_save = saved_game::save_grid_cells(
            &mut self.boot_level_grid,
            &mut self.state.boot_lifecycle.shared,
            &self.state.room,
            self.state.current_room,
        )?;
        // Constructor 587ec0 initializes unused grid indices to -1; 58ced0
        // excludes only -3 special records. Fresh normal Level copies all135.
        let rooms: Vec<_> = (0..mt::ROOM_DESCRIPTOR_CAPACITY).collect();
        self.saved_game
            .descriptor_seeds
            .copy_rooms_from(&self.boot_level_grid, &rooms)?;
        //54f373→5bf400→518350 follows the actual Level copy, flushes the
        //owned grant queue and applies the supported Player save boundary.
        let player_items = player_save::capture_at_manager_save(&mut self.state)?;
        self.saved_game.player_rng_seed = self.player_manager_secondary_rng.seed;
        let saved = saved_game::SavedRun::copy_from(
            &self.state,
            item_pools,
            grid_save.records,
            player_items,
        );
        self.state.saved_run_summary = Some(saved.summary(
            grid_save.empty,
            grid_save.occupied_pending,
            rooms.len(),
            &self.boot_level_grid,
            &self.saved_game.descriptor_seeds,
        )?);
        self.saved_run = Some(saved);
        self.state
            .boot_lifecycle
            .update_level_grid_samples(&self.boot_level_grid);
        self.state
            .boot_lifecycle
            .update_saved_game_samples(&self.saved_game);
        Ok(())
    }
    fn held(&self, code: &str) -> bool {
        self.state.pressed.contains(code)
    }
    /// The supported free regular callback is shared by ordinary full
    /// contacts and the actual Room.Update admission tail.
    fn collect_regular_pickup_at(&mut self, index: usize) {
        if !pickups::player_can_collect(&self.state) {
            return;
        }
        let red_multiplier = if self
            .state
            .player_items
            .as_ref()
            .is_some_and(|items| items.owned_counts[312] > 0)
        {
            2
        } else {
            1
        };
        let pickup = &mut self.state.room.pickups[index];
        if pickup.touches(&self.state.player)
            && (if pickup.variant == 10 {
                pickup.try_collect_heart(
                    &mut self.state.player,
                    red_multiplier,
                    &mut self.state.level_progress,
                )
            } else {
                pickup.try_collect(&mut self.state.inventory)
            })
        {
            // Collect504480 routes key ->504720 and bomb ->5046d0
            // after inventory mutation, before the System increment.
            let sound = match pickup.variant {
                10 => Some((
                    match pickup.subtype {
                        3 => 54,
                        4 => 266,
                        _ => 185,
                    },
                    1.0,
                )),
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
            let context = entities::SpawnContext {
                stage: self.state.stage,
                game_update_count: self.state.game_update_count,
            };
            if let Some(mut npc) = entities::spawn(id, selector, &context)? {
                champions::roll(
                    &mut npc,
                    &mut self.state.boot_lifecycle.shared,
                    &champions::Roll {
                        stage: self.state.stage,
                        player_items: self.state.player_items.as_ref(),
                    },
                )?;
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
        // Game551a10: the authored bossportraits.xml presentation of the
        // room's boss (Monstro20/0 is presentation1).
        self.state.room.boss_presentation_index = if room_type == 5 {
            self.state
                .room
                .npcs
                .iter()
                .find_map(|npc| content::boss_presentation(npc.kind, npc.variant))
                .unwrap_or(0)
        } else {
            0
        };
        self.state.room.authored_actor_order = order;
        self.state.next_pickup = next;
        self.rebuild_room_actor_schedule()?;
        self.state.room.boss_count = self.state.room.manager_counts.boss_total().max(0) as u32;
        self.refresh_stage_effects();
        Ok(())
    }
    /// Persisted Room clones retain supported actor handles in authored order.
    pub fn rebuild_room_actor_schedule(&mut self) -> Result<(), String> {
        self.state.actor_scheduler.clear_room();
        //453ec0 resets all four actual manager count words on Room replacement.
        self.state.room.manager_counts = actors::ManagerCounts::default();
        self.state.fire_particles.clear();
        self.state.sprite_effects.clear();
        let player = actors::ActorRef {
            kind: actors::ActorKind::Player,
            id: 0,
        };
        if self.state.actor_scheduler.sequence_id(player).is_none() {
            self.state.actor_scheduler.admit(player)?;
            self.state.actor_scheduler.clear_room();
        }
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
                if actor.kind == actors::ActorKind::Npc {
                    let npc = self
                        .state
                        .room
                        .npcs
                        .iter()
                        .find(|npc| npc.id == actor.id)
                        .expect("checked admitted NPC");
                    self.state
                        .room
                        .manager_counts
                        .admit(npc.can_shut_doors, npc.boss);
                }
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
                        self.saved_game = self.state.boot_lifecycle.clear_saved_game();
                        self.saved_run = None;
                        self.state.saved_run_summary = None;
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
    fn tick(&mut self, timestamp_millis: Option<u64>) -> Result<(), String> {
        self.state.run_admission.clock.advance(timestamp_millis)?;
        // Source 594a10 consumes the prior menu request before game dispatch.
        // Room construction and clock alignment happen here, after the f32
        // confirmation fade, rather than in the zero-tick DOM key callback.
        if let Some(request) = self.state.player_death.system_pending.take() {
            //5949a0 sees old Systemstate2 before either deferred dispatch,
            // including2->2 Restart, and54f588 clears the shared Info owner.
            // NewRun54eb20 itself has no clear; delay remains unchanged.
            self.state.information_animation.clear();
            match request {
                player_death::Request::Restart => {
                    if !self.state.seeded_run {
                        let seed = self.state.boot_lifecycle.shared.next_u32().max(1);
                        self.state.boot_lifecycle.commit_character_seed(seed)?;
                    }
                    self.state.stage = 1;
                    self.start_run()?;
                }
                player_death::Request::Exit => {
                    self.state.player_death = player_death::PlayerDeath::default();
                    self.state.run_start_request = RunStartRequest::default();
                    self.state.paused = false;
                    self.state.pause_menu = PauseMenu::hidden();
                    // Popup557650 queues menu3;59cac0 stores it in Menu+38,
                    // the same native Main family observed by the paired gate.
                    self.state.screen = Screen::Main;
                    // A Random run stored its resolved character; the menus
                    // return to the profile's own selection.
                    self.load_menu_character();
                    self.state.audio.pending.push(audio::SoundRequest {
                        id: 8,
                        volume: 1.0,
                        delay: 0,
                        looping: false,
                        pitch: 1.0,
                    });
                    self.state.music.crossfade(1);
                }
            }
        }
        if self.state.run_start_request.system_pending {
            self.state.run_start_request.system_pending = false;
            self.start_run()?;
        }
        let audio_counter = self.state.system_clock.system_counter;
        let phase = self.state.system_clock.advance()?;
        if phase != phase::Phase::Skipped && self.state.nightmares.apply_request(self.state.stage) {
            self.state.music.room(96);
        }
        if phase != phase::Phase::Skipped {
            self.state.public_actions.advance(&self.state.pressed);
        }
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
                self.state.run_admission.confirm_menu_request();
            }
        }
        self.state.tick += 1;
        self.state.menu_animation_tick += 1;
        let nightmare_dispatch = if phase == phase::Phase::Skipped {
            // Disabled interpolation skips System polling but keeps state4.
            self.state.nightmares.system_active
        } else {
            self.state
                .nightmares
                .advance_system(self.state.public_actions.menu_confirm)
        };
        if nightmare_dispatch {
            if phase != phase::Phase::Skipped {
                self.state.music.advance();
            }
            return Ok(());
        }
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
        if self.state.player_death.game_over.is_some() {
            if phase != phase::Phase::Skipped {
                player_death::poll_popup(&mut self.state);
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
        let boss_cleanup = phase == phase::Phase::Update
            && self
                .state
                .room_transition
                .as_ref()
                .is_some_and(|transition| {
                    transition.phase == traversal::TransitionPhase::BossCleanup
                });
        if phase == phase::Phase::Update
            && self
                .state
                .room_transition
                .as_ref()
                .is_some_and(|transition| transition.phase == traversal::TransitionPhase::Out)
        {
            pickups::save_regular_records(&mut self.state)?;
        }
        if traversal::advance_transition(&mut self.state, phase) {
            if boss_cleanup {
                // State3 calls Player.Update at5dde9e before enabling controls
                // and leaving the transition. The outer actor pass stays frozen.
                self.update_player(phase, false, false)?;
            }
            if room_before != self.state.current_room {
                self.state.camera = Some(camera::Camera::new(
                    &self.state.room,
                    self.state
                        .room_transition
                        .as_ref()
                        .map(|transition| transition.target_slot),
                ));
                doors::begin_room_admission(&mut self.state);
                // Transition explicitly updates Player before loading; the
                // GameChangeRoom5509f2 ->584200 ->Room.Update5d12a0 tail
                // admits another update at unchanged Game counter.
                combat::advance_player_immunity(&mut self.state);
                combat::advance_player_immunity(&mut self.state);
                if restoring {
                    let restored_pickups = pickups::readmit_regular_records(&mut self.state)?;
                    self.rebuild_room_actor_schedule()?;
                    // The native Room.Update admission tail advances the new
                    // actors once at the unchanged Game clock. Other saved
                    // families keep their existing reconstruction owners.
                    for id in restored_pickups.iter().copied() {
                        self.advance_pickup(id, phase)?;
                    }
                    let candidates = pickups::contact_candidates(&self.state)?;
                    //454a0a dispatches Player/PU callbacks before453340's
                    //PU self-pairs. Disabled transition controls do not veto
                    //the fresh living Player's ordinary pickup callback.
                    for id in restored_pickups {
                        if let Some(index) = self
                            .state
                            .room
                            .pickups
                            .iter()
                            .position(|pickup| pickup.id == id)
                        {
                            self.collect_regular_pickup_at(index);
                        }
                    }
                    pickups::nonplayer_contacts(&mut self.state, candidates)?;
                    pickups::full_grid_contacts(&mut self.state)?;
                    self.state.room.pickups.retain(|pickup| !pickup.removed);
                    let pickups = &self.state.room.pickups;
                    self.state.actor_scheduler.retain(|actor| {
                        actor.kind != actors::ActorKind::Pickup
                            || pickups.iter().any(|pickup| pickup.id == actor.id)
                    });
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
                    self.refresh_room_boss_count();
                    let mut counted = actors::FullCounts::default();
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
                        self.count_updated_room_actor(actor, &mut counted);
                    }
                    self.state.room.manager_counts.finish_full(counted);
                    familiars::paint_background_effects(&mut self.state)?;
                    blood_effects::clear_retired_links(&mut self.state);
                    pickups::full_grid_contacts(&mut self.state)?;
                    chests::full_grid_contacts(&mut self.state)?;
                    npcs::post_contact_grid(&mut self.state)?;
                    self.state.room.pickups.retain(|pickup| !pickup.removed);
                    let pickups = &self.state.room.pickups;
                    self.state.actor_scheduler.retain(|actor| {
                        actor.kind != actors::ActorKind::Pickup
                            || pickups.iter().any(|pickup| pickup.id == actor.id)
                    });
                    npcs::admit_room_combat(&mut self.state);
                    self.show_miniboss_streak();
                }
                // ChangeRoom5dd260 restores Player+7d4=false after the
                // admitted Room.Update, before the incoming leg or Scene.
                self.state.player.controls_enabled = false;
                if !restoring {
                    traversal::begin_boss_scene(&mut self.state);
                }
            }
            self.refresh_minimap();
            if self.state.room_transition.is_none() {
                // State3 dispatches its final Player callback at5dde9e with
                // controls disabled, then5ddecb enables the persistent field.
                self.state.player.controls_enabled = true;
                self.advance_camera(phase);
            }
            return Ok(());
        }
        if phase == phase::Phase::Update
            && self
                .state
                .run_admission
                .blocks_full_update(&mut self.state.player)?
        {
            // Fast5507f0 does not take this guard. Music belongs to System and
            // continues while Game's full update and age are held.
            self.state.music.advance();
            return Ok(());
        }

        // Game54fb88 invokes580330 after its active transition branches.
        // Full dispatch freezes only when the resulting manager state is2;
        // completion resumes ordinaryRoom on this same full. Fast550760
        // freezes any nonzero information state without advancing its ANM.
        if self.state.information_animation.active() {
            if phase == phase::Phase::Update {
                self.state.information_animation.advance_full()?;
            }
            if phase != phase::Phase::Update || self.state.information_animation.state == 2 {
                if phase != phase::Phase::Skipped {
                    self.state.music.advance();
                }
                return Ok(());
            }
        }

        if phase == phase::Phase::Update {
            // Source54fa82 increments before PlayerManager/Room callbacks.
            self.state.game_update_count = self
                .state
                .game_update_count
                .checked_add(1)
                .ok_or("game update counter exhausted")?;
        }
        doors::advance(&mut self.state, phase)?;
        let explosion_fence = self.state.explosions.len();
        grid_hazards::advance(&mut self.state, phase)?;
        let existing = self.state.actor_scheduler.full_phase();
        if phase == phase::Phase::Update {
            rock_blast::advance_markers(&mut self.state);
            self.refresh_room_boss_count();
            let mut counted = actors::FullCounts::default();
            for actor in existing.iter().copied() {
                self.advance_actor(actor, phase, false, bomb_triggered)?;
                self.count_updated_room_actor(actor, &mut counted);
            }
            self.state.room.manager_counts.finish_full(counted);
            self.update_boss_bar();
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
            // The independent render array paints flags8 at its current count,
            // including births from the admission-fenced actor callbacks.
            familiars::paint_background_effects(&mut self.state)?;
            blood_effects::clear_retired_links(&mut self.state);
            let pickup_contacts = pickups::contact_candidates(&self.state)?;
            npcs::player_contacts(&mut self.state)?;
            npcs::nonplayer_contacts(&mut self.state)?;
            enemy_projectiles::player_contacts(&mut self.state)?;
            // Actual full manager contact pass uses typed ground coordinates.
            let tears: Vec<_> = self
                .state
                .tears
                .iter()
                .filter(|tear| tear.motion.lifetime == projectiles::Lifetime::Alive)
                .map(|tear| {
                    (
                        tear.id,
                        tear.motion,
                        tear.collision_radius,
                        tear.collision_damage,
                    )
                })
                .collect();
            for (id, mut motion, radius, damage) in tears {
                if npcs::tear_hit(&mut self.state, &mut motion, radius, id, damage)?.is_some() {
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
                if self.state.room.pickups[index].chest.is_some() {
                    chests::player_contact(&mut self.state, index)?;
                    continue;
                }
                self.collect_regular_pickup_at(index);
            }
            pickups::nonplayer_contacts(&mut self.state, pickup_contacts)?;
            pickups::full_grid_contacts(&mut self.state)?;
            chests::full_grid_contacts(&mut self.state)?;
            npcs::post_contact_grid(&mut self.state)?;
        }
        if phase == phase::Phase::Update {
            familiars::grid_contacts(&mut self.state)?;
            if self
                .state
                .sprite_effects
                .iter()
                .any(|effect| effect.poop_effect.is_some() && effect.active && !effect.removed)
            {
                let grid = grid::Grid::new(
                    self.state
                        .room
                        .grid
                        .clone()
                        .ok_or("poop effect requires actual grid")?,
                )?;
                for effect in &mut self.state.sprite_effects {
                    if effect.poop_effect.is_some() {
                        poop_effects::full_grid_contact(effect, &grid)?;
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
        self.state.familiars.retain(|familiar| !familiar.removed);
        //454df0 retires actual31e inactive actors;31f alone is not removal.
        self.state.sprite_effects.retain(|effect| effect.active);
        self.state
            .enemy_projectiles
            .retain(|projectile| !projectile.removed);
        self.state
            .fire_particles
            .retain(|particle| !particle.removed);
        lasers::retire(&mut self.state);
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
            actors::ActorKind::Familiar => self
                .state
                .familiars
                .iter()
                .any(|familiar| familiar.id == actor.id),
            actors::ActorKind::Tear => surviving.iter().any(|tear| tear.id == actor.id),
            actors::ActorKind::Pickup => pickups.iter().any(|pickup| pickup.id == actor.id),
            actors::ActorKind::Npc => self.state.room.npcs.iter().any(|npc| npc.id == actor.id),
            actors::ActorKind::Projectile => self
                .state
                .enemy_projectiles
                .iter()
                .any(|projectile| projectile.id == actor.id),
            actors::ActorKind::Effect => {
                self.state
                    .fire_particles
                    .iter()
                    .any(|particle| particle.id == actor.id)
                    || self
                        .state
                        .sprite_effects
                        .iter()
                        .any(|effect| effect.id == actor.id)
                    || self.state.lasers.iter().any(|laser| {
                        laser
                            .impact
                            .as_ref()
                            .is_some_and(|impact| impact.id == actor.id)
                    })
            }
            actors::ActorKind::Laser => self.state.lasers.iter().any(|laser| laser.id == actor.id),
            _ => true,
        });
        if phase != phase::Phase::Skipped {
            self.state.music.advance();
        }
        self.advance_camera(phase);
        Ok(())
    }
    fn advance_camera(&mut self, phase: phase::Phase) {
        if (self.state.run_introduction.active && self.state.run_introduction.stage)
            || self.state.room_transition.is_some()
            || self.state.information_animation.active()
        {
            return;
        }
        let aim = motion::WorldVector {
            x: f32::from(u8::from(self.held("ArrowRight")))
                - f32::from(u8::from(self.held("ArrowLeft"))),
            y: f32::from(u8::from(self.held("ArrowDown")))
                - f32::from(u8::from(self.held("ArrowUp"))),
        };
        let camera = self
            .state
            .camera
            .get_or_insert_with(|| camera::Camera::new(&self.state.room, None));
        camera.advance(&self.state.room, &self.state.player, aim, phase);
    }
    fn advance_actor(
        &mut self,
        actor: actors::ActorRef,
        phase: phase::Phase,
        newborn: bool,
        bomb_triggered: bool,
    ) -> Result<(), String> {
        match actor.kind {
            actors::ActorKind::Player => self.update_player(phase, bomb_triggered, true)?,
            actors::ActorKind::Familiar => {
                familiars::advance_actor(&mut self.state, phase, actor.id)?
            }
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
            actors::ActorKind::Laser => lasers::advance_actor(&mut self.state, phase, actor.id)?,
            actors::ActorKind::Effect => {
                lasers::advance_impact(&mut self.state, phase, actor.id)?;
                familiars::advance_effect(&mut self.state, phase, actor.id)?;
                if let Some(particle) = self
                    .state
                    .fire_particles
                    .iter_mut()
                    .find(|particle| particle.id == actor.id)
                {
                    particle.advance(phase, &mut self.state.boot_lifecycle.shared)?;
                }
            }
        }
        Ok(())
    }
    /// Room5d1423/5d1454 refresh2490 before the full manager callbacks. The
    /// final boss callback can decrement it later in that same pass.
    fn refresh_room_boss_count(&mut self) {
        self.state.room.boss_count = self.state.room.manager_counts.boss_total().max(0) as u32;
    }

    ///454eb9 counts only callbacks whose active31e remains true. NPC70/78
    /// read103c/103d, with no HP/dead predicate; boss flags100000 exclude only
    /// the boss subtotal. Other currently admitted entity families have v70false.
    /// Room::Init 5cfb36: entering an uncleared miniboss room (type 6) with
    /// countable or boss actors shows "<player> VS <room name>" through the
    /// shared streak, without the stage paper.
    fn show_miniboss_streak(&mut self) {
        let Some(descriptor) = self.state.room.descriptor.as_ref() else {
            return;
        };
        if descriptor.template.room_type != 6
            || !self
                .state
                .room
                .npcs
                .iter()
                .any(|npc| !npc.removed && (npc.can_shut_doors || npc.boss))
        {
            return;
        }
        let player = self
            .state
            .profiles
            .get(usize::from(self.state.profile.saturating_sub(1)))
            .map_or("Isaac", |profile| profile.selected_character.as_str());
        self.state.run_introduction = RunIntroduction {
            age: 0,
            active: true,
            title: format!("{player} VS {}", descriptor.template.name),
            stage: false,
        };
    }

    /// EntityManager update 454df0 sums HP over boss actors that are not
    /// final-flagged (0x100000) and have aged past their spawn frame, against
    /// the larger of their max HP and the retained maximum. HUD::Update
    /// 572d48 counts the flash down; SetFill 5732a0 restarts it at 8 whenever
    /// the fill drops. No boss hides the bar and forgets the maximum.
    fn update_boss_bar(&mut self) {
        let bar = &mut self.state.boss_bar;
        if bar.flash >= 0 {
            bar.flash -= 1;
        }
        let (mut hp, mut maximum, mut any) = (0.0_f32, 0.0_f32, false);
        for npc in &self.state.room.npcs {
            let final_flag = npc
                .monstro
                .as_ref()
                .is_some_and(|owner| owner.entity_flags & 0x100000 != 0);
            if npc.removed || !npc.boss || final_flag || npc.frame_age == 0 {
                continue;
            }
            hp += npc.hp;
            maximum += npc.max_hp;
            any = true;
        }
        let fill = if any {
            bar.maximum = bar.maximum.max(maximum);
            if bar.maximum > 0.0 {
                hp / bar.maximum
            } else {
                -1.0
            }
        } else {
            bar.maximum = 0.0;
            -1.0
        };
        if fill < bar.fill {
            bar.flash = 8;
        }
        bar.fill = fill;
    }

    fn count_updated_room_actor(&self, actor: actors::ActorRef, counted: &mut actors::FullCounts) {
        if actor.kind != actors::ActorKind::Npc {
            return;
        }
        if let Some(npc) = self
            .state
            .room
            .npcs
            .iter()
            .find(|npc| npc.id == actor.id && !npc.removed)
        {
            let final_flag = npc
                .monstro
                .as_ref()
                .is_some_and(|owner| owner.entity_flags & 0x100000 != 0);
            counted.observe(npc.can_shut_doors, npc.boss && !final_flag);
        }
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
        if self.state.room.pickups[index].chest.is_some() {
            return chests::advance_actor(&mut self.state, phase, id);
        }
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
        pickups::advance_actor(&mut self.state, phase, id)
    }
    fn admit_npc_blood(&mut self, index: usize, seed: u32) -> Result<(), String> {
        let position = self.state.room.npcs[index].position;
        // Fresh13/18/214 have the ordinary white color. Modifiers and tinted
        // actor colors require their actual source owner before admission.
        let id = familiars::admit_blood_splat(&mut self.state, seed, position, [1.0; 4], [0.0; 3])?;
        familiars::advance_blood_initial(&mut self.state, id)
    }
    fn advance_npc(&mut self, id: u32, phase: phase::Phase) -> Result<(), String> {
        let grid = grid::Grid::new(self.state.room.grid.clone().expect("admitted NPC has grid"))?;
        let player = bombs::player_world(&self.state);
        let Some(index) = self.state.room.npcs.iter().position(|npc| npc.id == id) else {
            return Ok(());
        };
        let family = entities::family(self.state.room.npcs[index].kind)
            .ok_or("admitted NPC has no registered family")?;
        let mut occupied = Vec::new();
        if family.occupancy == entities::Occupancy::Actors {
            occupied.push(player);
            occupied.extend(
                self.state
                    .room
                    .npcs
                    .iter()
                    .filter(|npc| !npc.removed)
                    .map(|npc| npc.position),
            );
        }
        if family.occupancy != entities::Occupancy::None {
            occupied.extend(
                self.state
                    .room
                    .pickups
                    .iter()
                    .filter(|pickup| !pickup.removed)
                    .map(|pickup| pickup.position),
            );
            occupied.extend(self.state.bombs.iter().map(|bomb| bomb.motion.position));
        }
        let player_velocity = bombs::player_velocity(&self.state);
        let mut births = Vec::new();
        (family.advance)(
            &mut self.state.room.npcs[index],
            phase,
            &mut self.state.boot_lifecycle.shared,
            &mut entities::NpcWorld {
                // Native Game+ce9f4 is one more than game_update_count (the
                // run-loader origin); AI frame gates read the native value.
                game_counter: self.state.game_update_count.wrapping_add(1),
                player,
                player_velocity,
                grid: &grid,
                grid_descriptor: self.state.room.grid.as_mut(),
                path_costs: self.state.room.path_costs.as_mut(),
                occupied: &occupied,
                countable_total: self.state.room.manager_counts.countable_total(),
                audio: &mut self.state.audio,
                system_counter: self.state.system_clock.system_counter.wrapping_sub(1),
                player_items: self.state.player_items.as_ref(),
                effects: familiars::NpcEffectAdmission {
                    game_update_count: self.state.game_update_count,
                    next_effect: &mut self.state.next_effect,
                    effects: &mut self.state.sprite_effects,
                    scheduler: &mut self.state.actor_scheduler,
                },
            },
            &mut births,
        )?;
        for birth in births {
            self.admit_npc_birth(index, birth)?;
        }
        Ok(())
    }
    /// Admits one NPC callback product at the callback's own return point.
    fn admit_npc_birth(&mut self, index: usize, birth: entities::Birth) -> Result<(), String> {
        let parent = self.state.room.npcs[index].id;
        match birth {
            entities::Birth::Blood(seed) => self.admit_npc_blood(index, seed),
            entities::Birth::Laser(birth) => lasers::admit(&mut self.state, parent, birth),
            entities::Birth::Ember(birth) => {
                let id = self.state.next_effect;
                self.state.next_effect = id.checked_add(1).ok_or("Ember identity exhausted")?;
                self.state.fire_particles.push(fireplaces::Ember::birth(
                    id,
                    birth,
                    self.state.room.npcs[index].variant,
                    self.state.game_update_count,
                ));
                self.state.actor_scheduler.admit(actors::ActorRef {
                    kind: actors::ActorKind::Effect,
                    id,
                })
            }
            entities::Birth::Loot(birth) => {
                let subtype = fireplaces::loot_subtype(birth.variant, birth.spawn_seed)?;
                let id = self.state.next_pickup;
                let mut pickup = Pickup::spawn(id, birth.variant, subtype, birth.position, true)?;
                pickup.initialize_spawn_seed(birth.spawn_seed)?;
                pickup.birth_game_update = self.state.game_update_count;
                self.state.next_pickup = id.checked_add(1).ok_or("Fire loot identity exhausted")?;
                self.state.room.pickups.push(pickup);
                let actor = actors::ActorRef {
                    kind: actors::ActorKind::Pickup,
                    id,
                };
                self.state.room.authored_actor_order.push(actor);
                self.state.actor_scheduler.admit(actor)
            }
            entities::Birth::Projectile(birth) => self.admit_enemy_projectile(|id| {
                enemy_projectiles::EnemyProjectile::birth_with_parameters(id, parent, birth)
            }),
            entities::Birth::Shot(shot) => self.admit_enemy_projectile(|id| {
                enemy_projectiles::EnemyProjectile::birth(id, parent, shot)
            }),
            entities::Birth::Npc(selector) => {
                let id = u32::try_from(self.state.room.npcs.len())
                    .map_err(|_| "NPC identity exhausted")?;
                let context = entities::SpawnContext {
                    stage: self.state.stage,
                    game_update_count: self.state.game_update_count,
                };
                let npc = entities::spawn(id, &selector, &context)?
                    .ok_or("NPC birth requires a registered family")?;
                let (countable, boss) = (npc.can_shut_doors, npc.boss);
                self.state.room.npcs.push(npc);
                self.state.actor_scheduler.admit(actors::ActorRef {
                    kind: actors::ActorKind::Npc,
                    id,
                })?;
                self.state.room.manager_counts.admit(countable, boss);
                Ok(())
            }
        }
    }
    fn admit_enemy_projectile(
        &mut self,
        birth: impl FnOnce(u32) -> enemy_projectiles::EnemyProjectile,
    ) -> Result<(), String> {
        let id = self.state.next_enemy_projectile;
        self.state.next_enemy_projectile = id
            .checked_add(1)
            .ok_or("enemy projectile identity exhausted")?;
        self.state.enemy_projectiles.push(birth(id));
        self.state.actor_scheduler.admit(actors::ActorRef {
            kind: actors::ActorKind::Projectile,
            id,
        })
    }
    fn update_player(
        &mut self,
        phase: phase::Phase,
        bomb_triggered: bool,
        controls_enabled: bool,
    ) -> Result<(), String> {
        if phase == phase::Phase::Update {
            self.state.player_animation.advance_full();
            if matches!(
                self.state.player_animation.current,
                Some(player_animation::Clip::UseItem | player_animation::Clip::Hit)
            ) && self.state.player_animation.is_finished(None)
            {
                // Current-main completion returns ordinary presentation; an
                // interrupted Pickup never resumes its obsolete main clock.
                self.state.player_animation.current = None;
            }
        }
        if player_death::advance(&mut self.state, phase) {
            return Ok(());
        }
        let controls_enabled = controls_enabled
            && self.state.player.controls_enabled
            && self.state.player.input_cooldown <= 0;
        player_items::advance_presentation(&mut self.state, phase);
        let dx = if controls_enabled {
            i32::from(self.held("KeyD")) - i32::from(self.held("KeyA"))
        } else {
            0
        };
        let dy = if controls_enabled {
            i32::from(self.held("KeyS")) - i32::from(self.held("KeyW"))
        } else {
            0
        };
        combat::advance_player_immunity(&mut self.state);
        let bomb_admitted = self.state.bomb_placement_cooldown.advance(phase);
        // Shooting inherits the velocity entering this phase, before input acceleration.
        let inherited_velocity = (self.state.player.vx, self.state.player.vy);
        motion::advance(
            &mut self.state.player,
            &self.state.room,
            dx,
            dy,
            self.state.player_stats.move_speed,
        );
        if controls_enabled && bomb_triggered && bomb_admitted {
            bombs::place(&mut self.state, phase)?;
        }
        active_items::advance(&mut self.state, controls_enabled)?;
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
        // Both main presentation owners represent Player+bf4. Source530388
        // suppresses shooting input before head/aim work, while ordinary
        // movement and the full fire/head timer continuation still run.
        let animation_guard = self.state.player_animation.guard
            || self
                .state
                .player_items
                .as_ref()
                .is_some_and(|items| items.pickup_animation_guard);
        let aim = if !controls_enabled || animation_guard {
            None
        } else if self.held("ArrowLeft") {
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
        let shooting_locked = self.state.player.shooting_lock > 0;
        self.state.player.shooting = aim.is_some() && !shooting_locked;
        if phase == phase::Phase::Update && !shooting_locked {
            self.state.player.head_frame = if self.state.head_timer > 8 { 2 } else { 0 };
            if self.state.fire_cooldown >= 0 {
                self.state.fire_cooldown -= 1;
            }
            if self.state.head_timer >= 0 {
                self.state.head_timer -= 1;
            }
        }
        let regular_dispatch = if controls_enabled && !animation_guard && !shooting_locked {
            match weapon_plan::resolve_dispatch_admission(
                self.state.weapon_cache,
                self.state.weapon_admission.source_player_type,
                self.state.player_stats,
                self.state.weapon_admission.pending_primary_cleanup,
            )? {
                weapon_plan::DispatchAdmission::Regular => true,
                weapon_plan::DispatchAdmission::Unadmitted(channel) => {
                    self.state
                        .weapon_admission
                        .record_unadmitted(self.state.weapon_cache.primary, channel);
                    false
                }
            }
        } else {
            false
        };
        if let Some(direction) = aim {
            self.state.player.facing = direction;
            if regular_dispatch && !shooting_locked && self.state.fire_cooldown < 0 {
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
                let plan = weapon_plan::RegularTearPlan::resolve(
                    self.state.player_stats,
                    self.state.max_fire_delay,
                    self.state.weapon_cache,
                )?;
                let velocity = projectiles::stored_velocity(aim, inherited, plan.shot_speed)?;
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
                    shot_ordinal: self.state.player.shots_fired,
                    collision_damage: plan.damage,
                    base_damage: plan.damage,
                    attack_flags: plan.flags,
                    source_color: plan.color,
                    scale: plan.scale,
                    animation_tier: plan.animation_tier,
                    collision_radius: plan.collision_radius,
                    grid_contact: false,
                    motion: projectiles::TearMotion::spawn(
                        origin,
                        velocity,
                        plan.parameters,
                        final_uniform,
                    )?,
                };
                tear.project();
                self.state.tears.push(tear);
                self.state.actor_scheduler.admit(actors::ActorRef {
                    kind: actors::ActorKind::Tear,
                    id: self.state.next_tear,
                })?;
                self.state.player.shots_fired = self.state.player.shots_fired.wrapping_add(1);
                self.state.next_tear = self
                    .state
                    .next_tear
                    .checked_add(1)
                    .ok_or("tear IDs exhausted")?;
                self.state.fire_cooldown = plan.fire_delay;
                self.state.head_timer = plan.head_timer;
            }
        }
        player_items::advance_item_sprite(&mut self.state, phase);
        if self.state.player.input_cooldown > 0 {
            self.state.player.input_cooldown -= 1;
        }
        player_items::finish_player_update(&mut self.state, phase)?;
        // Player531e9a decrements7dc after the shooting callback in both phases.
        self.state.player.shooting_lock = self.state.player.shooting_lock.saturating_sub(1);
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
    static PUBLIC_KEYS_JSON: Vec<u8> =
        serde_json::to_vec(&input::PUBLIC_KEYS).expect("key codes serialize");
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
/// The public key surface as a JSON array of key codes; needs no reset.
#[no_mangle]
pub extern "C" fn public_keys_ptr() -> *const u8 {
    PUBLIC_KEYS_JSON.with(|keys| keys.as_ptr())
}
#[no_mangle]
pub extern "C" fn public_keys_len() -> usize {
    PUBLIC_KEYS_JSON.with(Vec::len)
}
