//! A fresh, narrow foundation driven by observed base-Rebirth menu/run captures.
//! World positions use logical pixels in a 480 × 270 viewport; public stepping
//! follows full/intermediate source phases at nominal 60 presentations per second.
//! Sprite offsets and stage effects are explicit state separate from ground positions.
//! Public seed encoding is supported; original room RNG routing and simulation
//! ticks remain unverified. Scene descriptors are ordinary generated state.
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
pub mod acquisition_history;
mod active_charges;
pub mod active_items;
pub mod actors;
pub mod ambience;
pub mod attack_color;
pub mod attack_flags;
mod attack_grid;
pub mod attack_runtime;
pub mod audio;
pub mod blood_effects;
pub mod boil;
pub mod bombs;
pub mod boss_death;
mod brimstone;
pub mod camera;
pub mod challenges;
pub mod champions;
pub mod chests;
pub mod chub;
pub mod combat;
mod content;
mod controlled_brimstone;
mod controlled_fetus;
mod controlled_knife;
mod controlled_knife_satellites;
pub mod controlled_tear;
mod controlled_technology;
mod creep_effects;
pub mod critters;
pub mod deaths_head;
pub mod dingle;
pub mod dips;
pub mod display_settings;
mod doors;
mod dr_fetus;
pub mod duke;
mod effects;
pub mod enemy_projectiles;
mod entities;
mod epic_fetus;
mod experimental_treatment;
mod explosion_effects;
pub mod game_shake;
pub mod spider_webs;
pub mod sprite_color;
pub mod squirt;
mod surprise;
pub mod tear_deaths;
mod tear_factory;
mod technology;
mod technology2;
pub use entities::{implemented_npc_kinds, supports_npc_selector};
mod contact_candidates;
mod controlled_knife_homing;
pub mod controls_menu;
pub mod envy;
pub mod familiars;
pub mod fatty;
pub mod fireplaces;
pub mod fistula;
pub mod floor;
pub mod fly_effects;
mod fmath;
pub mod fullscreen;
pub mod gapers;
pub mod gemini;
pub mod gib_counts;
pub mod globin;
pub mod gluttony;
pub mod greed;
pub mod grid;
mod grid_hazards;
pub mod grid_records;
pub mod gurdy;
pub mod gurdy_jr;
pub mod gushers;
mod health_damage;
pub mod hoppers;
mod horfs;
pub mod information_animation;
mod inner_eye;
pub mod input;
mod item_death_effects;
mod item_pipeline;
mod item_queries;
pub mod items;
pub mod keepers;
pub mod keyboard;
mod knife_path;
pub mod knives;
pub mod larry;
pub mod laser_line;
mod laser_npc_runtime;
mod laser_player_runtime;
mod laser_runtime;
pub mod lasers;
pub mod layout;
pub mod level2_flies;
pub mod level_progress;
pub mod levels;
pub mod living_restart;
pub mod long_legs;
pub mod maggots;
pub mod map_presentation;
pub mod mega_fatty;
pub mod mega_maw;
pub mod menu_lifecycle;
pub mod menu_presentation;
pub mod monstro;
mod monstros_lung;
pub mod motion;
pub mod mt;
pub mod mulligans;
pub mod music;
pub mod nightmares;
pub mod npcs;
pub mod options_menu;
pub mod ordinary_placement;
pub mod pathfinder;
pub mod peep;
pub mod persistent;
pub mod persistent_effects;
pub mod phase;
pub mod pickup_world_grants;
pub mod pickups;
pub mod pill_effect_history;
mod plain_npc;
pub mod player_animation;
pub mod player_costume118;
pub mod player_costume229;
mod player_costume_registration;
pub mod player_death;
mod player_footprints;
pub mod player_items;
pub mod player_save;
pub mod player_stats;
pub mod pockets;
mod polyphemus_contact;
pub mod poop_effects;
pub mod projectiles;
pub mod public_actions;
pub mod rock_blast;
mod room;
pub mod room_assets;
pub mod room_costs;
mod room_lights;
pub mod room_spawns;
pub mod room_templates;
pub mod roundworm;
pub mod run_admission;
pub mod run_quit;
pub mod save_host;
pub mod save_profiles;
pub mod saved_game;
pub mod seed;
pub mod shop;
pub mod skinny;
pub mod slots;
mod source_angles;
pub mod stage_shooting_guard;
pub mod stage_transition;
mod target_eligibility;
pub mod tnt;
pub mod traversal;
pub mod unlock_notifications;
pub mod wall_creep;
pub mod wall_hugger;
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
const MAX_BOOT_RESOURCE: usize = 512 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Screen {
    Title,
    Profile,
    Main,
    Options,
    Controls,
    Keyboard,
    Character,
    Challenge,
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
    ///None is legacy/unowned Entity source history, never a neutral default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_source: Option<PlayerEntitySource>,
    ///Actual Player+3074 final starting-grant callback completion.
    pub initialized_3074: bool,
    ///Actual Player+3075 constructor byte, then Challenge Config+98 override.
    pub can_shoot: bool,
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
    /// Actual Player+BD0; fresh Init and TriggerNewStage own its false writes.
    /// Missing legacy histories remain unknown rather than an invented key grant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub golden_key: Option<bool>,
    pub facing: Direction,
    pub body_facing: Direction,
    pub walk_frame: u8,
    pub head_frame: u8,
    pub shooting: bool,
    pub shooting_lock: u32,
    /// Actual Player+7d4 input gate, distinct from Entity+31d visibility.
    pub controls_enabled: bool,
    /// Actual signed counters at 3068/306C. Unadmitted history remains absent.
    #[serde(default)]
    pub transformation_counts: Option<[i32; 2]>,
    /// Actual Player D68 and Entity 340 cache outputs. These are independent
    /// of Entity 344, which controls entity pair collisions.
    #[serde(default)]
    pub flying: Option<bool>,
    #[serde(default)]
    pub grid_collision_class: Option<u32>,
    /// Entity+408/+470/+490 slowing state written by spider webs.
    #[serde(default, skip_serializing_if = "spider_webs::Slowing::is_constructed")]
    pub slowing: spider_webs::Slowing,
    pub eye: projectiles::Eye,
    /// Actual Player+c84, independent of manager tear handles.
    pub shots_fired: u32,
    pub render_offset: Point,
}
///Actual Entity10/14/1C/20 provenance owned by the supported Player Init.
///It is separate from inventory, current character type and copied P/V.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayerEntitySource {
    pub entity_type: u32,
    pub variant: u32,
    pub spawner_type: u32,
    pub spawner_variant: u32,
}
impl PlayerEntitySource {
    ///5BF4F5..505 passes type1/variant0;519480→441EE8/EF clears1C/20.
    const fn fresh_isaac() -> Self {
        Self {
            entity_type: 1,
            variant: 0,
            spawner_type: 0,
            spawner_variant: 0,
        }
    }
}
impl Player {
    /// Player::TryUseKey5087F0 tests BD0 before BCC and never spends a golden key.
    pub(crate) fn try_use_key(&self, inventory: &mut Inventory) -> bool {
        match self.golden_key {
            Some(true) => true,
            Some(false) if inventory.keys > 0 => {
                inventory.keys -= 1;
                true
            }
            _ => false,
        }
    }

    fn starting() -> Self {
        Self {
            entity_source: None,
            initialized_3074: false,
            can_shoot: true,
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
            golden_key: None,
            facing: Direction::Down,
            body_facing: Direction::Down,
            walk_frame: 0,
            head_frame: 0,
            shooting: false,
            shooting_lock: 0,
            controls_enabled: true,
            transformation_counts: None,
            flying: None,
            grid_collision_class: None,
            slowing: spider_webs::Slowing::default(),
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
    ///Actual BaseInit441D24/441D34 birth42C, owned at fresh factory Init.
    ///Unknown/deserialized histories keep None; first observation is no owner.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_game_update: Option<u32>,
    /// Source-owned builder variant copied to Entity+14; None is unqualified.
    pub source_variant: Option<u32>,
    /// Actual Tear+838, copied from Player+c84 before its increment.
    pub shot_ordinal: Option<u32>,
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
    /// 542f70: this tear attempted its known grid-death Hurt branch before
    /// that callback's ground integration. The later poof/removal callback
    /// does not repeat it; unknown source_variant remains unknown.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub grid_death: bool,
    pub motion: projectiles::TearMotion,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub controlled: Option<controlled_tear::ControlledTear>,
    ///Actual808/818 owner for the selected born ballistic bit9 family.
    ///Old restore and untracked flag families retain unknown ownership.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persistent_contacts: Option<polyphemus_contact::TearHitRegistry>,
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
    ///Actual Persistent+D4 array, initialized by5bc520 for a fresh profile.
    pub counters: Vec<i32>,
    pub collected_history: persistent::CollectedHistory,
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
    /// HUD5732C0's second authored line. Empty stage/miniboss subtitles
    /// retain the existing serialized shape and presentation.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub subtitle: String,
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
            subtitle: String::new(),
            stage: true,
        }
    }

    ///505D7F..505DE9 takes the current collectible config's name and
    ///description before the Pickup pose and deferred AddCollectible queue.
    pub(crate) fn collectible(config: &items::CollectibleDefinition) -> Self {
        Self::pickup(config.name, config.description)
    }

    pub(crate) fn pickup(name: &str, description: &str) -> Self {
        Self {
            age: 0,
            active: true,
            title: name.into(),
            subtitle: description.into(),
            stage: false,
        }
    }

    ///54F920 -> HUD572600 -> Sprite40F9F0 advances the shared streak
    ///before the Game pause/transition guards. One Full is one authored
    ///frame; age keeps its existing half-frame presentation unit.
    fn advance_full(&mut self) {
        if self.active {
            self.age += 2;
            self.active = self.age < 140;
        }
    }
}
/// HUD boss bar: SetFill 5732a0 at HUD+61894 (-1 hidden) and its flash
/// countdown at +61898; the manager's non-shrinking boss max HP is +ec.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BossBar {
    #[serde(with = "nan_as_null")]
    pub fill: f32,
    pub flash: i32,
    pub maximum: f32,
}
/// JSON has no NaN: serde_json writes it as null, so read null back as NaN.
mod nan_as_null {
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(value: &f32, serializer: S) -> Result<S::Ok, S::Error> {
        if value.is_nan() {
            serializer.serialize_none()
        } else {
            serializer.serialize_f32(*value)
        }
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
        Ok(Option::<f32>::deserialize(deserializer)?.unwrap_or(f32::NAN))
    }
}
/// The 0/0 fill is NaN, and any two NaN fills are the same bar, so state
/// equality stays reflexive. NaN sign and payload are not compared: native and
/// wasm32 may produce different NaN bits for 0/0, and JSON keeps neither.
impl PartialEq for BossBar {
    fn eq(&self, other: &Self) -> bool {
        (self.fill == other.fill || (self.fill.is_nan() && other.fill.is_nan()))
            && self.flash == other.flash
            && self.maximum == other.maximum
    }
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
    ///594690 chooses saved54ECE0 rather than NewRun54EB20 at the next System dispatch.
    #[serde(default)]
    pub resume_saved: bool,
    ///594620 actual challenge argument; None is the existing New Run route.
    pub challenge_id: Option<u32>,
    ///593470 retains the chosen loading texture independently of later seed changes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loading_image_index: Option<u8>,
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
    pub stat_frames: [Option<u8>; 6],
}
impl PauseMenu {
    fn hidden() -> Self {
        Self {
            phase: PausePhase::Hidden,
            age: 0,
            selection: 1,
            stat_frames: [Some(2), Some(1), Some(2), Some(2), Some(2), Some(1)],
        }
    }
    ///PauseOpen5bb560 samples the actual cached Player fields once. It does
    ///not evaluate caches or infer missing scalar history from inventory.
    fn open(&mut self, stats: &player_stats::PlayerStats, max_fire_delay: i32) {
        self.phase = PausePhase::Appearing;
        self.age = 0;
        self.selection = 1;
        //5bb6ce..732: retain the source's individual f32 operations and order.
        let range_shot_speed = (stats.shot_speed - 1.0_f32) / 0.15_f32 + 1.0_f32;
        let range_falling_speed =
            (stats.tear_falling_speed + stats.tear_falling_speed) + range_shot_speed;
        let range = (-stats.tear_height / 20.0_f32) + range_falling_speed;
        self.stat_frames = [
            Some(pause_stat_frame(stats.move_speed + stats.move_speed)),
            Some(pause_stat_frame(15.0_f32 / max_fire_delay as f32)),
            Some(pause_stat_frame(stats.damage / 1.5_f32)),
            Some(pause_stat_frame(range)),
            Some(pause_stat_frame(stats.shot_speed + stats.shot_speed)),
            stats.luck.map(pause_stat_frame),
        ];
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

///5bb602/64d/694/736/779/7b8 CVTTSS2SI, then signed clamp1..7.
///An overflowing/NaN input becomes INT_MIN, unlike Rust's saturating cast.
fn pause_stat_frame(value: f32) -> u8 {
    let integer =
        if value.is_finite() && (-2_147_483_648.0_f32..2_147_483_648.0_f32).contains(&value) {
            value as i32
        } else {
            i32::MIN
        };
    integer.clamp(1, 7) as u8
}
///A cache of Game's immutable Challenge catalogue owner, refilled from that
///owner after a deserialization: every value compares equal.
#[derive(Clone, Debug, Default)]
pub(crate) struct CatalogCache(pub(crate) Option<std::sync::Arc<challenges::Catalog>>);

impl PartialEq for CatalogCache {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct State {
    pub schema: u32,
    ///A completed Save/Clear requests origin-local IO once; it is not Game age.
    #[serde(default)]
    pub save_revision: u64,
    #[serde(default)]
    pub save_readmission_pending: bool,
    pub save_write_pending: bool,
    #[serde(default)]
    pub saved_load_valid: Option<bool>,
    pub tick_rate: u32,
    pub tick: u32,
    pub menu_animation_tick: u32,
    pub title_animation_frame: u32,
    pub menu_fade: MenuFade,
    pub menu_presentation: menu_presentation::MenuPresentation,
    pub run_start_request: RunStartRequest,
    #[serde(default)]
    pub run_quit: run_quit::RunQuit,
    #[serde(default)]
    pub living_restart: living_restart::LivingRestart,
    #[serde(default)]
    pub persistent_runtime: Option<persistent::Runtime>,
    #[serde(default)]
    pub unlock_notifications: Option<unlock_notifications::Runtime>,
    #[serde(default)]
    pub unlock_popups: Option<bool>,
    #[serde(default)]
    pub unlock_notifications_loading: bool,
    #[serde(default)]
    pub platform_achievement_backend: Option<persistent_effects::PlatformBackend>,
    #[serde(default)]
    pub platform_achievement_attempts: Vec<persistent_effects::PlatformAttempt>,
    ///Ephemeral borrowable cache of Game's actual immutable resource owner.
    ///It is rebuilt from that owner before the next command, so it is not
    ///state: it neither serializes nor takes part in State equality.
    #[serde(skip)]
    persistent_catalog: CatalogCache,
    ///The System counter the current callback interval began at; a command
    ///between Ticks reads the last one, so it serializes with the state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    persistent_callback_pre_counter: Option<u32>,
    pub system_clock: phase::Clock,
    pub run_admission: run_admission::RunAdmission,
    pub run_tick: u32,
    /// Equals native Game+ce9f4; read it through `native_game_frame`.
    pub game_update_count: u32,
    pub screen: Screen,
    pub profile: u8,
    pub main_selection: u8,
    #[serde(default)]
    pub options_menu: options_menu::OptionsMenu,
    #[serde(default)]
    pub controls_menu: controls_menu::ControlsMenu,
    #[serde(default)]
    pub keyboard: keyboard::Keyboard,
    #[serde(default)]
    pub input_configs: keyboard::InputConfigs,
    #[serde(default)]
    pub map_settings: map_presentation::MapPreferences,
    pub gamma_settings: display_settings::GammaPreferences,
    #[serde(default)]
    pub minimap_presentation: map_presentation::MapPresentation,
    #[serde(default)]
    pub fullscreen: fullscreen::FullscreenState,
    pub challenge_selection: Option<u32>,
    ///Real Config achievement requirements queried for the fresh menu rows.
    pub challenge_unlocked: Vec<bool>,
    pub challenge_id: u32,
    pub challenge_run_parameters: Option<challenges::RunParameters>,
    pub character_selection: u8,
    pub character: String,
    pub difficulty: String,
    pub seed_text: String,
    pub seed_valid: bool,
    pub run_seed: String,
    pub initial_seed: u32,
    pub boot_epoch_seed: u32,
    pub boot_lifecycle: mt::BootState,
    /// Actual Game 10EAD8 word, separate from floor flags and run r8/r9.
    /// Unknown legacy or restore history remains absent.
    #[serde(default)]
    pub source_cache_run_flags: Option<u64>,
    /// Actual normal-variant5120EE/514AD8 challenge minimum predicate.
    /// Unknown old Game history stays absent; ID0 skips the Config9C read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_shot_speed_minimum: Option<bool>,
    ///Actual normal-variant513D60/596170 Config9A maxdamage predicate.
    ///An absent legacy Game owner never means a false predicate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_damage_maximum: Option<bool>,
    ///Actual normal-variant512787/596170 Config9B minfirerate predicate.
    ///Missing legacy Game ownership remains unknown independently of9A/9C.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_fire_delay_minimum: Option<bool>,
    pub saved_run_summary: Option<saved_game::SavedRunSummary>,
    pub audio: audio::AudioState,
    #[serde(default)]
    pub pending_menu_initializations: Vec<menu_lifecycle::Pending>,
    pub music: music::MusicState,
    pub run_seed_value: u32,
    pub run_seeds: Option<seed::RunSeeds>,
    /// Actual Game+8BA0, independent from counts and RunSeeds. Absent legacy
    /// history stays unknown until a reached constructor or Stop clears it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acquisition_history: Option<acquisition_history::AcquisitionHistory>,
    pub item_pools: Option<items::ItemPools>,
    #[serde(
        default,
        serialize_with = "pill_effect_history::serialize_player_items",
        deserialize_with = "pill_effect_history::deserialize_player_items"
    )]
    pub player_items: Option<player_items::PlayerItems>,
    pub player_animation: player_animation::PlayerAnimation,
    pub public_actions: public_actions::PublicActions,
    #[serde(default)]
    pub player_stats: player_stats::PlayerStats,
    #[serde(default)]
    pub weapon_cache: weapon_plan::WeaponCache,
    #[serde(default)]
    pub weapon_admission: weapon_plan::WeaponAdmission,
    pub stage_shooting_guard: stage_shooting_guard::StageShootingGuard,
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
    ///Independent Game+CE9FC/CEA00 history; legacy restore stays unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_shake: Option<game_shake::GameShake>,
    pub floor: floor::Floor,
    pub current_room: usize,
    ///Actual supported Level previous/entry stores; legacy or unowned writers stay absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub saved_room_entry: Option<save_profiles::RoomEntry>,
    pub visited_rooms: BTreeSet<usize>,
    pub room_display_flags: Vec<u32>,
    pub revealed_secret_connections: BTreeSet<[usize; 2]>,
    pub persisted_rooms: BTreeMap<usize, Room>,
    pub room_transition: Option<traversal::RoomTransition>,
    #[serde(default)]
    pub stage_transition: stage_transition::StageTransition,
    /// Room ambience loops 5c6980 (insect swarm and fire).
    #[serde(default)]
    pub ambience: ambience::Ambience,
    /// The player's world position while ChangeRoom runs the entered room's
    /// Room.Init, before the entry door placement: Room.Init's critter
    /// updates measure the player there.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub departure_position: Option<motion::WorldVector>,
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
    /// Tears in the last contact pass's tear partition (4545d0), read by
    /// tiny flies at live positions in the next actor pass.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tear_partition: Vec<u32>,
    ///Previous source4545D0 query membership; a missing restore has no owner.
    ///Explicit continuation preserves known membership; missing legacy data
    ///stays unknown until its actual rebuild, never a default empty query.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) knife_query_partition: Option<contact_candidates::QuerySnapshot>,
    #[serde(default)]
    pub lasers: Vec<lasers::Laser>,
    #[serde(default)]
    pub knives: Vec<knives::Knife>,
    #[serde(default)]
    pub attack_runtime: attack_runtime::AttackRuntime,
    #[serde(default)]
    pub next_laser: u32,
    pub familiars: Vec<familiars::Familiar>,
    pub next_familiar: u32,
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

impl State {
    /// Native Game+ce9f4 as full callbacks read it. Game54fa82 increments it
    /// before the PlayerManager/Room callbacks and this port increments
    /// `game_update_count` at the same point, so the two are equal: a fresh
    /// run's first gameplay update reads the same value in both games once the
    /// reset clock carries the declared boot elapsed time (#438). An older
    /// `+1` here compensated for that harness clock offset; never re-add one.
    pub(crate) fn native_game_frame(&self) -> u32 {
        self.game_update_count
    }
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
    #[serde(default)]
    pub menu_readmission: Option<save_profiles::MenuReadmission>,
    /// The declared host capability decides whether a readmission waits for
    /// the host's document, so a resumed Game keeps it.
    #[serde(default)]
    save_io_enabled: bool,
    #[serde(skip)]
    save_write_request: Option<save_profiles::SaveWriteRequest>,
    #[serde(skip)]
    command_origin_screen: Option<Screen>,
    #[serde(skip)]
    living_write_resume: Option<living_restart::WriteResume>,
    #[serde(skip)]
    living_start_context: Option<living_restart::Context>,
    ///An attempted host revision cannot be rolled back/retried when a later
    ///port continuation reaches an unsupported Source callback.
    #[serde(default)]
    save_continuation_failure: Option<String>,
    ///Malformed/older unknown own documents are preserved, not zero-migrated.
    #[serde(default)]
    profile_document_rejected: bool,
    #[serde(default, with = "save_profiles::shared")]
    pub saved_profiles: Option<std::sync::Arc<save_profiles::Profiles>>,
    pub player_manager_secondary_rng: saved_game::PlayerManagerSecondaryRng,
    pub challenge_catalog: challenges::Catalog,
    /// Process-lifetime GetRoomByIdx static; later runs reuse it.
    #[serde(default)]
    pub room_lookup_dummy_constructed: bool,
    ///Immutable authored context belongs to the complete Game continuation,
    ///outside per-frame State snapshots. Selector and admission share it.
    #[serde(default, with = "room_assets::shared_catalogue")]
    special_room_assets: Option<std::sync::Arc<Vec<room_assets::RoomAsset>>>,
}

impl State {
    /// Clears what each command publishes afresh for the host: sound and
    /// music events, SFX volume callbacks, settings and input-configuration
    /// saves, and unavailable-action reports. A batched Tick publishes every
    /// iteration's.
    pub fn begin_command(&mut self) {
        self.audio.begin_command();
        self.options_menu.begin_command();
        self.controls_menu.begin_command();
        self.fullscreen.begin_command();
        self.music.begin_command();
    }
}

impl Game {
    pub fn new(initial_seed: u32) -> Result<Self, String> {
        Self::new_with_boot_epoch(initial_seed, 0)
    }
    /// See `RunAdmission::set_boot_elapsed`; valid only before any command.
    pub fn set_boot_elapsed(
        &mut self,
        milliseconds: u32,
        sub_millisecond_nanoseconds: u32,
    ) -> Result<(), String> {
        if self.state.tick != 0 {
            return Err("boot elapsed time must be declared before any command".into());
        }
        self.state.run_admission.set_boot_elapsed(
            self.state.boot_epoch_seed,
            milliseconds,
            sub_millisecond_nanoseconds,
        )
    }
    pub fn new_with_boot_epoch(initial_seed: u32, boot_epoch_seed: u32) -> Result<Self, String> {
        Self::new_with_challenges(
            initial_seed,
            boot_epoch_seed,
            challenges::Catalog::authored()?,
            None,
            None,
            None,
            keyboard::InputConfigs::default(),
            None,
            None,
        )
    }
    pub fn new_with_resources(
        initial_seed: u32,
        boot_epoch_seed: u32,
        resources: challenges::ResourceInputs,
    ) -> Result<Self, String> {
        Self::new_with_boot_inputs(initial_seed, boot_epoch_seed, Some(resources), None)
    }
    /// File-derived public settings are admitted before the first Title Music.
    /// Native replay uses the same owner as the browser's additive reset ABI.
    pub fn new_with_boot_inputs(
        initial_seed: u32,
        boot_epoch_seed: u32,
        resources: Option<challenges::ResourceInputs>,
        preferences: Option<options_menu::AudioPreferences>,
    ) -> Result<Self, String> {
        Self::new_with_boot_preferences(initial_seed, boot_epoch_seed, resources, preferences, None)
    }
    /// Additive display preference ingress; legacy audio-only callers are unchanged.
    pub fn new_with_boot_preferences(
        initial_seed: u32,
        boot_epoch_seed: u32,
        resources: Option<challenges::ResourceInputs>,
        preferences: Option<options_menu::AudioPreferences>,
        fullscreen: Option<fullscreen::FullscreenPreferences>,
    ) -> Result<Self, String> {
        Self::new_with_input_configs(
            initial_seed,
            boot_epoch_seed,
            resources,
            preferences,
            fullscreen,
            None,
        )
    }
    ///Global file-derived Keyboard configuration is immutable boot input,
    ///admitted before the actual input owner is constructed.
    pub fn new_with_input_configs(
        initial_seed: u32,
        boot_epoch_seed: u32,
        resources: Option<challenges::ResourceInputs>,
        preferences: Option<options_menu::AudioPreferences>,
        fullscreen: Option<fullscreen::FullscreenPreferences>,
        input_configs: Option<keyboard::InputConfigs>,
    ) -> Result<Self, String> {
        Self::new_with_input_configs_and_map_preferences(
            initial_seed,
            boot_epoch_seed,
            resources,
            preferences,
            fullscreen,
            input_configs,
            None,
        )
    }
    /// Additive MapOpacity ingress; all legacy callers retain source .3.
    pub fn new_with_map_preferences(
        initial_seed: u32,
        boot_epoch_seed: u32,
        resources: Option<challenges::ResourceInputs>,
        preferences: Option<options_menu::AudioPreferences>,
        fullscreen: Option<fullscreen::FullscreenPreferences>,
        map: Option<map_presentation::MapPreferences>,
    ) -> Result<Self, String> {
        Self::new_with_input_configs_and_map_preferences(
            initial_seed,
            boot_epoch_seed,
            resources,
            preferences,
            fullscreen,
            None,
            map,
        )
    }
    /// Independent immutable Keyboard and Map owners share one constructor.
    pub fn new_with_input_configs_and_map_preferences(
        initial_seed: u32,
        boot_epoch_seed: u32,
        resources: Option<challenges::ResourceInputs>,
        preferences: Option<options_menu::AudioPreferences>,
        fullscreen: Option<fullscreen::FullscreenPreferences>,
        input_configs: Option<keyboard::InputConfigs>,
        map: Option<map_presentation::MapPreferences>,
    ) -> Result<Self, String> {
        Self::new_with_settings_preferences(
            initial_seed,
            boot_epoch_seed,
            resources,
            preferences,
            fullscreen,
            input_configs,
            map,
            None,
        )
    }
    /// Additive global display input; legacy constructors retain Gamma1.
    #[expect(
        clippy::too_many_arguments,
        reason = "each argument is a separate declared boot input: seeds, resources, audio, fullscreen, input, map and Gamma preferences"
    )]
    pub fn new_with_settings_preferences(
        initial_seed: u32,
        boot_epoch_seed: u32,
        resources: Option<challenges::ResourceInputs>,
        preferences: Option<options_menu::AudioPreferences>,
        fullscreen: Option<fullscreen::FullscreenPreferences>,
        input_configs: Option<keyboard::InputConfigs>,
        map: Option<map_presentation::MapPreferences>,
        gamma: Option<display_settings::GammaPreferences>,
    ) -> Result<Self, String> {
        let gamma = gamma
            .map(display_settings::GammaPreferences::validate)
            .transpose()?;
        let map = map
            .map(map_presentation::MapPreferences::validate)
            .transpose()?;
        let input_configs = input_configs
            .map(keyboard::InputConfigs::validate)
            .transpose()?
            .unwrap_or_default();
        let preferences = preferences
            .map(options_menu::AudioPreferences::validate)
            .transpose()?;
        let (catalog, special) = if let Some(resources) = resources {
            let catalog = resources.catalog()?;
            let special = resources
                .special_rooms
                .as_deref()
                .map(room_assets::decode)
                .transpose()?
                .map(std::sync::Arc::new);
            (catalog, special)
        } else {
            (challenges::Catalog::authored()?, None)
        };
        Self::new_with_challenges(
            initial_seed,
            boot_epoch_seed,
            catalog,
            special,
            preferences,
            fullscreen,
            input_configs,
            map,
            gamma,
        )
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "each argument is a separate boot input: seeds, challenge catalog, special-room assets and the saved audio, fullscreen, input and map preferences"
    )]
    fn new_with_challenges(
        initial_seed: u32,
        boot_epoch_seed: u32,
        challenge_catalog: challenges::Catalog,
        special_room_assets: Option<std::sync::Arc<Vec<room_assets::RoomAsset>>>,
        preferences: Option<options_menu::AudioPreferences>,
        fullscreen: Option<fullscreen::FullscreenPreferences>,
        input_configs: keyboard::InputConfigs,
        map: Option<map_presentation::MapPreferences>,
        gamma: Option<display_settings::GammaPreferences>,
    ) -> Result<Self, String> {
        let run_seed = seed::encode(initial_seed)?;
        let (boot_lifecycle, saved_game, boot_level_grid) =
            mt::BootLifecycle::new(boot_epoch_seed).into_parts();
        let player = Player::starting();
        let floor = floor::generate_basement_with_special(
            seed::stage1_seed(initial_seed),
            special_room_assets.as_deref().map(Vec::as_slice),
        )?;
        let current_room = floor.starting_descriptor;
        let room = Room::from_floor_descriptor_with_special(
            &floor,
            current_room,
            special_room_assets.as_deref().map(Vec::as_slice),
        )?;
        let camera = Some(camera::Camera::new(&room, None));
        let stage_effects = StageEffects::basement(&player, &room.appearance.fx, 0);
        //Settings5B9930 runs before MenuInit's first61 request. Apply to the
        //real owners while no samples are loaded, before Music initialization.
        let mut audio = audio::AudioState::default();
        if let Some(values) = preferences {
            audio.set_volume(values.sfx_volume)?;
        }
        let music = preferences.map_or_else(music::MusicState::default, |values| {
            music::MusicState::with_volume(values.music_volume)
        });
        let mut game = Self {
            state: State {
                schema: 1,
                save_revision: 0,
                save_readmission_pending: false,
                save_write_pending: false,
                saved_load_valid: Some(false),
                tick_rate: TICK_RATE,
                tick: 0,
                menu_animation_tick: 0,
                title_animation_frame: 0,
                menu_fade: MenuFade::first_title(),
                menu_presentation: menu_presentation::MenuPresentation::new(
                    menu_presentation::Page::Title,
                    1,
                    character_catalog().len(),
                ),
                run_start_request: RunStartRequest::default(),
                run_quit: run_quit::RunQuit::default(),
                living_restart: living_restart::LivingRestart::source_reset(),
                persistent_runtime: Some(persistent::Runtime::fresh()),
                unlock_notifications: Some(unlock_notifications::Runtime::fresh()?),
                //Settings constructor5B8D03: System2DF88+29=Popups2DFB1.
                unlock_popups: Some(true),
                //411D5A initializes manager1408; living594710 does not reload it.
                unlock_notifications_loading: false,
                platform_achievement_backend: Some(
                    persistent_effects::PlatformBackend::Unavailable,
                ),
                platform_achievement_attempts: Vec::new(),
                persistent_catalog: CatalogCache(Some(std::sync::Arc::new(
                    challenge_catalog.clone(),
                ))),
                persistent_callback_pre_counter: None,
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
                options_menu: options_menu::OptionsMenu::default(),
                controls_menu: controls_menu::ControlsMenu::default(),
                keyboard: input_configs.keyboard(),
                input_configs,
                map_settings: map.unwrap_or_default(),
                gamma_settings: gamma.unwrap_or_default(),
                minimap_presentation: map_presentation::MapPresentation::default(),
                fullscreen: fullscreen::FullscreenState::from_preferences(fullscreen)?,
                challenge_selection: None,
                challenge_unlocked: Vec::new(),
                challenge_id: 0,
                challenge_run_parameters: None,
                character_selection: 0,
                character: "Isaac".into(),
                difficulty: "normal".into(),
                seed_text: String::new(),
                seed_valid: false,
                run_seed,
                initial_seed,
                boot_epoch_seed,
                boot_lifecycle,
                source_cache_run_flags: Some(0), // Game constructor 554034.
                source_shot_speed_minimum: None,
                source_damage_maximum: None,
                source_fire_delay_minimum: None,
                saved_run_summary: None,
                audio,
                pending_menu_initializations: Vec::new(),
                music,
                run_seed_value: initial_seed,
                run_seeds: None,
                // Game553DB5 ->576AC0 owns the initially empty history.
                acquisition_history: Some(acquisition_history::AcquisitionHistory::default()),
                item_pools: None,
                player_items: None,
                player_animation: player_animation::PlayerAnimation::default(),
                public_actions: public_actions::PublicActions::default(),
                player_stats: player_stats::PlayerStats::default(),
                weapon_cache: weapon_plan::WeaponCache::fresh_player(),
                weapon_admission: weapon_plan::WeaponAdmission::default(),
                stage_shooting_guard: stage_shooting_guard::StageShootingGuard::fresh_game(),
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
                        counters: vec![0; 95],
                        collected_history: persistent::CollectedHistory::source_reset(),
                    })
                    .collect(),
                character_catalog: character_catalog(),
                paused: false,
                pressed: BTreeSet::new(),
                player,
                player_death: player_death::PlayerDeath::default(),
                room,
                camera,
                game_shake: Some(game_shake::GameShake::fresh()),
                room_display_flags: vec![0; floor.descriptors.len()],
                floor,
                current_room,
                saved_room_entry: None,
                visited_rooms: BTreeSet::from([current_room]),
                revealed_secret_connections: BTreeSet::new(),
                persisted_rooms: BTreeMap::new(),
                room_transition: None,
                stage_transition: stage_transition::StageTransition::default(),
                ambience: ambience::Ambience::default(),
                departure_position: None,
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
                tear_partition: Vec::new(),
                knife_query_partition: Some(contact_candidates::QuerySnapshot::empty()),
                lasers: Vec::new(),
                knives: Vec::new(),
                attack_runtime: attack_runtime::AttackRuntime::default(),
                next_laser: 1,
                familiars: Vec::new(),
                next_familiar: 1,
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
            menu_readmission: None,
            save_io_enabled: false,
            save_write_request: None,
            command_origin_screen: None,
            living_write_resume: None,
            living_start_context: None,
            save_continuation_failure: None,
            profile_document_rejected: false,
            saved_profiles: None,
            player_manager_secondary_rng: saved_game::PlayerManagerSecondaryRng::default(),
            challenge_catalog,
            room_lookup_dummy_constructed: false,
            special_room_assets,
        };
        game.refresh_minimap();
        Ok(game)
    }
    /// Invalid commands leave all state, including held keys, unchanged.
    pub fn command(&mut self, command: Command) -> Result<(), String> {
        if let Some(error) = &self.save_continuation_failure {
            return Err(error.clone());
        }
        if self.menu_readmission.is_some() || self.save_write_request.is_some() {
            return Err("complete actual own save IO before another public command".into());
        }
        let mut next = self.clone();
        next.command_origin_screen = Some(self.state.screen);
        next.apply(command)?;
        next.finish_public_command();
        *self = next;
        Ok(())
    }

    ///The public command postlude runs once after all actual IO boundaries,
    ///not at an intermediate return from a batched Tick.
    fn finish_public_command(&mut self) {
        if self.menu_readmission.is_some() || self.save_write_request.is_some() {
            return;
        }
        if self
            .command_origin_screen
            .take()
            .is_some_and(|screen| screen != self.state.screen)
        {
            self.state.menu_animation_tick = 0;
        }
        self.refresh_stage_effects();
    }

    fn apply(&mut self, command: Command) -> Result<(), String> {
        self.state.begin_command();
        self.state.platform_achievement_attempts.clear();
        if self.state.persistent_catalog.0.is_none() {
            self.state.persistent_catalog =
                CatalogCache(Some(std::sync::Arc::new(self.challenge_catalog.clone())));
        }
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
                self.state.audio.set_volume(sfx_volume)?;
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
                self.advance_public_ticks(count, timestamp_millis)?;
            }
        }
        Ok(())
    }
    fn advance_public_ticks(
        &mut self,
        count: u32,
        timestamp_millis: Option<u64>,
    ) -> Result<(), String> {
        if let Some(request) = &mut self.save_write_request {
            //A just-resumed System iteration may itself have requested a
            //write. Preserve every remaining tick until that attempt completes.
            if request.remaining_ticks.is_some() {
                return Err("save write already owns a tick continuation".into());
            }
            request.remaining_ticks = Some(count);
            return Ok(());
        }
        for offset in 0..count {
            let presented = self.state.system_clock.next_phase() != phase::Phase::Skipped;
            let room_before = self.state.current_room;
            self.tick(timestamp_millis)?;
            if let Some(resume) = &mut self.living_write_resume {
                if self.save_write_request.is_none() || resume.interval.is_some() {
                    return Err("living Restart has an invalid unfinished interval owner".into());
                }
                resume.interval = Some(living_restart::Interval {
                    remaining_ticks: count - offset,
                    timestamp_millis,
                    presented,
                    room_before,
                });
                return Ok(());
            }
            if let Some(request) = &mut self.menu_readmission {
                //This interval's clock input was consumed, but its System
                //iteration has not advanced. Resolve IO before any next tick.
                request.remaining_ticks = count - offset;
                request.timestamp_millis = timestamp_millis;
                return Ok(());
            }
            if presented && self.state.screen == Screen::Playing {
                room_lights::present(&mut self.state);
                if self.state.current_room != room_before {
                    room_lights::present(&mut self.state);
                }
            }
            if let Some(request) = &mut self.save_write_request {
                //This System iteration completed. Its actual immutable write
                //must be attempted before any further requested iteration.
                //Timestamped public Tick requires count1, so no timestamp is
                //reused by the remaining batch.
                request.remaining_ticks = Some(count - offset - 1);
                return Ok(());
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
        if self.state.screen == Screen::Playing && self.state.run_quit.active() {
            return Ok(());
        }
        //Legacy Title/Profile/Main/Character/Seed/Pause navigation stays in
        //this existing physical-key seam in this bounded leaf. Gameplay and
        //Menu9/10 use the actual keyboard owner; reconfiguring movement must
        //not silently rewrite these physical public menu commands. Full source
        //replacement of legacy WASD menu aliases remains a separate frontier.
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
                "ArrowLeft" | "KeyA" => {
                    self.state.profile = (self.state.profile + 1) % 3 + 1;
                    self.state.menu_presentation.save.select(self.state.profile);
                }
                "ArrowRight" | "KeyD" => {
                    self.state.profile = self.state.profile % 3 + 1;
                    self.state.menu_presentation.save.select(self.state.profile);
                }
                _ => {}
            },
            Screen::Main => match code {
                "Escape" => self.state.screen = Screen::Profile,
                "ArrowUp" | "KeyW" => {
                    self.state.main_selection = (self.state.main_selection + 4) % 5;
                    if self.state.main_selection == 1 && self.saved_run.is_none() {
                        self.state.main_selection = 0;
                    }
                }
                "ArrowDown" | "KeyS" => {
                    self.state.main_selection = (self.state.main_selection + 1) % 5;
                    if self.state.main_selection == 1 && self.saved_run.is_none() {
                        self.state.main_selection = 2;
                    }
                }
                "Enter" if self.state.main_selection == 1 => {
                    if self.state.saved_load_valid != Some(true)
                        || self.state.save_readmission_pending
                    {
                        return Err("Continue requires actual successful own profile load".into());
                    }
                    self.require_profile_document()?;
                    let record = self.current_profile_save()?;
                    record.admit_initial_room()?;
                    //Main5A47B0 decodes the retained saved string for593470;
                    //the later loader independently invokes the Seeds setter.
                    self.state
                        .boot_lifecycle
                        .commit_character_seed(record.context.run_seed_value)?;
                    self.state.run_start_request.resume_saved = true;
                    self.state.run_start_request.challenge_id = None;
                    self.state.run_start_request.input_pending = true;
                }
                "Enter" if self.state.main_selection == 0 => self.state.screen = Screen::Character,
                "Enter" if self.state.main_selection == 2 => {
                    //59cef0 initializes selectedID1/counter0 when Menu5 opens.
                    self.state.challenge_selection = Some(1);
                    self.state.challenge_unlocked = (1..=20)
                        .map(|id| {
                            self.challenge_catalog.get(id).map(|definition| {
                                definition.is_unlocked(
                                    &self.state.profiles[usize::from(self.state.profile - 1)]
                                        .achievements,
                                )
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    self.state.screen = Screen::Challenge;
                }
                "Enter" if self.state.main_selection == 4 => self.state.screen = Screen::Options,
                "Enter" => return Err("this menu action is outside the supported runtime".into()),
                _ => {}
            },
            // Options owns the released System poll, including held repeat.
            Screen::Options | Screen::Controls | Screen::Keyboard => {}
            Screen::Challenge => match code {
                "Escape" => self.state.screen = Screen::Main,
                "ArrowUp" | "ArrowDown" | "KeyW" | "KeyS" => {
                    let direction = if matches!(code, "ArrowUp" | "KeyW") {
                        -1
                    } else {
                        1
                    };
                    self.state.challenge_selection = Some(
                        self.challenge_catalog.adjacent_unlocked(
                            self.state
                                .challenge_selection
                                .ok_or("Challenge menu is uninitialized")?,
                            direction,
                            &self.state.profiles[usize::from(self.state.profile - 1)].achievements,
                        )?,
                    );
                }
                "Enter" => {
                    let id = self
                        .state
                        .challenge_selection
                        .ok_or("Challenge menu is uninitialized")?;
                    let definition = self.challenge_catalog.get(id)?;
                    if !definition.is_unlocked(
                        &self.state.profiles[usize::from(self.state.profile - 1)].achievements,
                    ) {
                        return Err("selected Challenge is locked".into());
                    }
                    definition.validate_initial_scope()?;
                    self.require_profile_document()?;
                    self.state.run_start_request.challenge_id = Some(id);
                    self.state.run_start_request.input_pending = true;
                }
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
                    self.require_profile_document()?;
                    self.state.run_start_request.challenge_id = None;
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
                        self.state
                            .pause_menu
                            .open(&self.state.player_stats, self.state.max_fire_delay);
                    } else if self.state.pause_menu.phase != PausePhase::Disappearing {
                        self.state.pause_menu.phase = PausePhase::Disappearing;
                        self.state.pause_menu.age = 0;
                    }
                } else if self.state.paused {
                    if code == "Enter"
                        && self.state.pause_menu.selection == 2
                        && self.state.pause_menu.phase != PausePhase::Disappearing
                        && self.state.room_transition.is_none()
                        && self.state.run_admission.initial_fade <= 0.0
                    {
                        if self.state.run_quit.save_enabled.is_none() {
                            return Err("Pause Quit requires its source save-enable history".into());
                        }
                        self.state.run_quit.input_pending = true;
                    }
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
                    | (Screen::Main, Screen::Challenge)
                    | (Screen::Main, Screen::Options)
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
    fn advance_minimap_full(&mut self) {
        if self.state.minimap_presentation.advance_full(
            self.state.public_actions.held[13],
            self.state.map_settings.opacity,
        ) {
            //5B1240 requests585500 on exactly the tenth held Game Full.
            self.state.run_introduction = RunIntroduction::stage(true);
        }
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
        self.start_run_until_initial_save()?;
        self.finish_start_run_after_save()
    }

    fn require_profile_document(&self) -> Result<(), String> {
        if self.profile_document_rejected {
            return Err(
                "unknown rejected own-profile document must be recovered or explicitly reset"
                    .into(),
            );
        }
        Ok(())
    }

    fn start_run_until_initial_save(&mut self) -> Result<(), String> {
        self.require_profile_document()?;
        if self.living_start_context.is_none() {
            //A new System request replaces the prior run's ownership. An
            //unqualified Challenge/hard/selector must never reuse its context.
            self.state.living_restart.context = None;
            self.state.living_restart.pending = None;
        }
        if self.living_start_context.is_none()
            && self.state.run_start_request.challenge_id.is_none()
            && self.state.screen == Screen::Character
            && self.state.character == "Isaac"
            && self.state.difficulty == "normal"
        {
            self.state.living_restart.context = Some(living_restart::Context::normal_isaac(
                self.state.boot_lifecycle.menu_seed_text.clone(),
                self.state.seeded_run,
            )?);
        }
        // Character 59f470 passes the existing menu seed code to the System
        // request; Game 54eb20 decodes that code into its startup stream. This
        // also applies when no public seed dialog was entered: the real
        // Character initializer owns the generated seed, not reset entropy.
        let challenge = self
            .state
            .run_start_request
            .challenge_id
            .map(|id| self.challenge_catalog.get(id).cloned())
            .transpose()?;
        //A newly selected Game key invalidates the previous predicate even
        //if validation rejects this start before the new run is initialized.
        self.state.source_damage_maximum = None;
        self.state.source_fire_delay_minimum = None;
        self.state.challenge_id = challenge.as_ref().map_or(0, |definition| definition.id);
        self.state.challenge_run_parameters = challenge
            .as_ref()
            .map(|_| challenges::RunParameters::CHALLENGE_MENU);
        if let Some(definition) = &challenge {
            definition.validate_initial_scope()?;
            self.state.character = "Isaac".into();
            self.state.difficulty = "normal".into();
        }
        self.state.run_seed_value = if let Some(context) = &self.living_start_context {
            seed::decode(&context.seed_code)?
        } else {
            self.state.boot_lifecycle.menu_run_seed
        };
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
        if let Some(context) = &self.living_start_context {
            context.validate()?;
            run_seeds.admit_menu_seed_effect(&context.seed_code)?;
        } else if self.state.screen == Screen::Character {
            //59EE70 commits the actual typed code;59F617 forwards this same
            //menu string through594620/54EB20 to the5E1470 seed setter.
            //Fresh generated codes use that menu owner after5E1170 as well.
            run_seeds.admit_menu_seed_effect(&self.state.boot_lifecycle.menu_seed_text)?;
        } else if self.state.screen == Screen::Challenge && challenge.is_some() {
            //Fresh59D080 uses5E1170's9B formatter for its actual setter.
            //Restart and legacy paths retain their separate input owners.
            run_seeds.admit_formatted_seed_effect()?;
        }
        //54EBFD copies sourceR9 before the new ItemPools/Player/Level owners.
        let source_r9 = challenge.as_ref().map_or(self.state.seeded_run, |_| {
            challenges::RunParameters::CHALLENGE_MENU.source_r9
        });
        self.state
            .persistent_runtime
            .as_mut()
            .ok_or("Persistent run inhibit owner is unknown")?
            .set_run_inhibit(source_r9);
        self.state.item_pools = Some(items::ItemPools::new(
            run_seeds.item_pool_seed(),
            &self.state.profiles[usize::from(self.state.profile - 1)].achievements,
        )?);
        self.state.player_items = Some(player_items::PlayerItems::new(
            run_seeds.initialized_fields[12],
        )?);
        self.state.run_seeds = Some(run_seeds);
        // NewRun54eb20 begins every run on the first stage, whether it comes
        // from the GameOver Restart or the menus after a descent, and outside
        // any stage transition the last run was in.
        self.state.stage = 1;
        self.state.stage_transition = stage_transition::StageTransition::default();
        if self.living_start_context.is_none() {
            //Existing independently qualified menu/death timing stays separate.
            self.state.system_clock.align_run_start()?;
        }
        self.state.living_restart.held_polls = Some(0);
        self.state.screen = Screen::Playing;
        self.state.paused = false;
        self.state.run_tick = 0;
        self.state.game_update_count = 0;
        self.state.minimap_presentation = map_presentation::MapPresentation::default();
        mt::note_update(0);
        //54F471/54F635 clear duration, without writing the independent output.
        self.state
            .game_shake
            .get_or_insert_with(Default::default)
            .reset_duration();
        self.state.tears.clear();
        self.state.next_tear = 1;
        self.state.enemy_projectiles.clear();
        self.state.next_enemy_projectile = 1;
        self.state.fire_particles.clear();
        self.state.next_effect = 1;
        self.state.sprite_effects.clear();
        laser_runtime::reset_room(&mut self.state)?;
        self.state.knives.clear();
        self.state.attack_runtime = attack_runtime::AttackRuntime::fresh_player();
        self.state.next_laser = 1;
        self.state.familiars.clear();
        self.state.next_familiar = 1;
        //Fresh Player51980B initializes C74 before any firing callback.
        self.state.fire_cooldown = 0;
        self.state.max_fire_delay = 10;
        self.state.player_stats = player_stats::PlayerStats::default();
        // Fresh reset 54E6A3 initializes the actual Game run-flag word.
        self.state.source_cache_run_flags = Some(0);
        //54EB55 stores the actual challenge key;596170 selects its Config.
        //The branch for key0 bypasses the9C cap. Other admitted definitions
        //own their decoded59805A minshotspeed value before Player callbacks.
        self.state.source_shot_speed_minimum = Some(
            challenge
                .as_ref()
                .is_some_and(|definition| definition.id != 0 && definition.min_shot_speed),
        );
        //54EB55/513D60: key0 bypasses Config9A; a real challenge uses its
        //own decoded maxdamage flag, independently of ShotSpeed's Config9C.
        self.state.source_damage_maximum = Some(
            challenge
                .as_ref()
                .is_some_and(|definition| definition.id != 0 && definition.max_damage),
        );
        //512787/515434: key0 bypasses Config9B; otherwise the selected
        //definition owns minfirerate independently of maxdamage/minshotspeed.
        self.state.source_fire_delay_minimum = Some(
            challenge
                .as_ref()
                .is_some_and(|definition| definition.id != 0 && definition.min_fire_rate),
        );
        self.state.weapon_cache = weapon_plan::WeaponCache::fresh_player();
        self.state.weapon_admission = weapon_plan::WeaponAdmission::fresh_isaac();
        //Fresh Player5198D0 initializes BB0 independently of later decrements.
        self.state.head_timer = 0;
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
        self.state.bomb_action_was_down = self.held(8);
        //54FC2B starts the initial stage streak only after the opaque fade.
        self.state.run_introduction = RunIntroduction::stage(false);
        self.state.boss_bar = BossBar::default();
        self.state.pause_menu = PauseMenu::hidden();
        self.state.run_quit = run_quit::RunQuit::fresh();
        self.state.player = Player::starting();
        //Actual fresh Manager.Init→Player.Init; title/old restores stay None.
        self.state.player.entity_source = Some(PlayerEntitySource::fresh_isaac());
        // Player.Init5194B4 clears BD0 before starting resources are assigned.
        self.state.player.golden_key = Some(false);
        // 519767/519771 initialize the two transformation history counters.
        self.state.player.transformation_counts = Some([0, 0]);
        // 51985C initializes the actual D64 Luck output.
        self.state.player_stats.luck = Some(0.0);
        // 51987A initializes D68 to false. Base/config 446A90 independently
        // initializes the authored Player's 40-point grid collision class to 5.
        self.state.player.flying = Some(false);
        self.state.player.grid_collision_class = Some(5);
        if let Some(definition) = &challenge {
            challenges::configure_player(&mut self.state, definition)?;
        }
        self.state.player_death = player_death::PlayerDeath::default();
        self.state.player_animation = player_animation::PlayerAnimation::default();
        // Source 58bea7 constructs every descriptor's padded grid seed pairs.
        // floor::generate_basement owns the separate primary xorshift advance.
        self.state.level_progress.begin_level();
        self.boot_level_grid = self.state.boot_lifecycle.initialize_level_grid();
        // Level generation's first GetRoomByIdx (58a13f) constructs its
        // static dummy descriptor once per process, after the grid init.
        if !self.room_lookup_dummy_constructed {
            mt::DescriptorSeeds::construct_lookup_dummy(&mut self.state.boot_lifecycle.shared);
            self.room_lookup_dummy_constructed = true;
        }
        self.state.floor = if let Some(definition) = &challenge {
            //The real curse/local-RNG owner is passed at this Level boundary,
            //before room placement; its sampler cannot be skipped by filter127.
            floor::generate_challenge_basement(
                seed::stage1_seed(self.state.run_seed_value),
                definition,
                &self.state.profiles[usize::from(self.state.profile - 1)],
                item_pipeline::ItemInputs::new(
                    self.state
                        .player_items
                        .as_ref()
                        .ok_or("Level requires actual Player item owner")?,
                )?
                .positive_owned_count(260),
                self.special_room_assets.as_deref().map(Vec::as_slice),
            )?
        } else {
            floor::generate_basement_with_special(
                seed::stage1_seed(self.state.run_seed_value),
                self.special_room_assets.as_deref().map(Vec::as_slice),
            )?
        };
        //Neither fresh Level nor a reused manager owns empty query buckets.
        self.state.knife_query_partition = None;
        self.state.current_room = self.state.floor.starting_descriptor;
        self.state.saved_room_entry = Some(save_profiles::RoomEntry::initial(
            self.state.floor.descriptors[self.state.current_room].grid_index,
        )?);
        self.state.visited_rooms = BTreeSet::from([self.state.current_room]);
        self.state.room_display_flags = vec![0; self.state.floor.descriptors.len()];
        self.state.revealed_secret_connections.clear();
        self.state.persisted_rooms.clear();
        self.state.room_transition = None;
        self.state.departure_position = None;
        self.state.room = Room::from_floor_descriptor_with_special(
            &self.state.floor,
            self.state.current_room,
            self.special_room_assets.as_deref().map(Vec::as_slice),
        )?;
        // Fresh587f44→58821c clears descriptor+3d; placement preserves it.
        // Room.Init5ced20 stamps159c before its grid/entity factories.
        self.state.room.initialization_game_update = Some(self.state.game_update_count);
        self.state.room.source_tear_slowdown_2494 = Some(0);
        self.state.room.combat_clear = false;
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
        // Room.Init's backdrop setup loads the FX rays before its grid records.
        // The port builds only ordinary Basement (stage type 0).
        ambience::reset_room(&mut self.state);
        room::load_fx_rays(
            &mut self.state.room,
            &mut self.state.boot_lifecycle.shared,
            self.state.stage,
            0,
        );
        // admit_room_pickups runs Room.Init's selector pass (grid objects and
        // actors together), the boundary, pit links and the critters.
        let selectors = self
            .state
            .room
            .spawns
            .as_ref()
            .ok_or("starting descriptor has no authored actor admission")?
            .selectors
            .clone();
        let requested_entities = selectors
            .iter()
            .filter(|selector| selector.category == room_spawns::SpawnCategory::Entity)
            .count();
        // 5cf5c6 constructs/registers actors before loaded5cf614 and Save.
        // This startup caller has no virtual18 or doorway manager update.
        // Room.Init's ambient critters follow inside admission; Game+ce9f4 reads0.
        let counter = self.state.native_game_frame();
        // The run's starting room sees the player already at its start.
        let viewer = bombs::player_world(&self.state);
        let suppressed_entities = self.admit_room_pickups(&selectors, counter, true, viewer)?;
        let requested_entities = requested_entities
            .checked_sub(suppressed_entities)
            .ok_or("suppressed authored entity count exceeds selected inputs")?;
        if self.state.room.authored_actor_order.len() != requested_entities {
            return Err(format!(
                "starting authored entity frontier is incomplete: admitted {} of {requested_entities}",
                self.state.room.authored_actor_order.len()
            ));
        }
        //5BF220 reaches510550 after actual Room.Init. Fresh3074 is
        //still false here, independently of the later starting grants.
        item_queries::note_room_placement(&mut self.state);
        npcs::initialize_startup_clear(&mut self.state);
        // Source Level generation loads the start room, then immediately calls
        // RoomMusic. This runtime's Level contract is fresh normal Basement.
        if let Some(id) = music::stage_track(self.state.stage, false) {
            self.state.music.room(id);
        }
        self.save_supported_run()?;
        Ok(())
    }

    fn finish_start_run_after_save(&mut self) -> Result<(), String> {
        let challenge = if self.state.challenge_id == 0 {
            None
        } else {
            Some(self.challenge_catalog.get(self.state.challenge_id)?.clone())
        };
        if let Some(definition) = &challenge {
            challenges::initialize_player(&mut self.state, definition)?;
        } else {
            //52e340/final3074 store also runs on the ordinary Challenge0 route.
            player_stats::evaluate_cache(&mut self.state)?;
            self.state.player.initialized_3074 = true;
        }
        // Fresh54eb20 invokes54eaa0 after Level.Init/Save and the primary
        // manager initialization; room factories see the prior control byte.
        self.state
            .run_admission
            .initialize_fresh(&mut self.state.player)?;
        self.refresh_minimap();
        Ok(())
    }

    fn current_profile_save(&self) -> Result<save_profiles::RunRecord, String> {
        self.saved_profiles
            .as_ref()
            .and_then(|profiles| profiles.profiles.get(usize::from(self.state.profile - 1)))
            .and_then(|profile| profile.saved.clone())
            .ok_or_else(|| "Continue requires this profile's actual retained save".into())
    }

    ///54ECE0 loads saved persistent stores into freshly constructed owners.
    ///No Game/State snapshot is installed and no NewRun/starting grant runs.
    fn continue_run(&mut self) -> Result<(), String> {
        self.require_profile_document()?;
        let record = self.current_profile_save()?;
        record.admit_initial_room()?;
        let run = &record.run;
        let context = &record.context;
        // Saved+EF80 owns the formatted string; the internal codec stays compact.
        // Validation already joins either legacy8 or formatted9 to this number.
        let run_seed = seed::encode(context.run_seed_value)?;
        if run.run_seed != run_seed && run.run_seed != seed::formatted(context.run_seed_value)? {
            return Err("saved seed text requires canonical legacy or formatted code".into());
        }
        let mut run_seeds = seed::RunSeeds::new(context.run_seed_value)?;
        run_seeds.admit_menu_seed_effect(&run_seed)?;
        //54EE90 initializes ItemPool from SavedED94, then581CF0 restores
        //its actual saved tables. It does not request NewRun's14th seed draw.
        let constructed_pools = items::ItemPools::new(
            run.item_pools.seed,
            &self.state.profiles[usize::from(self.state.profile - 1)].achievements,
        )?;
        //54EEA7 passes actual Seeds+44/GameA1D4 as constructor seed;
        //Saved889C is the separately admitted Isaac player type0.
        let mut player_items = player_items::PlayerItems::new(run_seeds.initialized_fields[12])?;
        run.player_items
            .restore_after_constructor(&mut player_items)?;
        player_items.manager_rng = context
            .player_manager_rng
            .ok_or("saved manager RNG is unknown")?;
        self.state.run_seeds = Some(run_seeds);
        self.state.run_seed_value = context.run_seed_value;
        self.state.run_seed = run_seed;
        self.state.seeded_run = run.seeded_run;
        self.state.achievement_eligibility = !run.seeded_run;
        //54EBFD precedes the saved-loader branch as well as fresh construction.
        //Saved Player/Room callbacks must see the actual requested R9 already.
        self.state
            .persistent_runtime
            .as_mut()
            .ok_or("Persistent Continue inhibit owner is unknown")?
            .set_run_inhibit(run.seeded_run);
        self.state.character = context.character.clone();
        self.state.difficulty = context.difficulty.clone();
        self.state.challenge_id = context.challenge_id;
        self.state.challenge_run_parameters = None;
        self.state.source_shot_speed_minimum = Some(false);
        self.state.source_damage_maximum = Some(false);
        self.state.source_fire_delay_minimum = Some(false);
        self.state.acquisition_history = run.acquisition_history.clone();
        self.state.item_pools = Some(run.item_pools.restore_after_init(&constructed_pools)?);
        self.state.player_items = Some(player_items);
        self.player_manager_secondary_rng.seed = self.saved_game.player_rng_seed;
        self.state.player = Player::starting();
        self.state.player.entity_source = Some(PlayerEntitySource::fresh_isaac());
        //5116B0 restores health/inventory stores; movement, clips, death,
        //input and sprite clocks retain their actual new-Player owners.
        //511937..958 zeroes all four words before the normal-Isaac
        //Max508540→Red508230→savedmask→Soul5083B0 helper sequence.
        self.state.player.hp = 0;
        self.state.player.max_hp = 0;
        self.state.player.soul_hp = 0;
        self.state.player.black_heart_mask = 0;
        player_items::add_max_hearts(&mut self.state.player, i32::from(run.player.max_hp));
        player_items::add_red_hearts(&mut self.state.player, i32::from(run.player.hp));
        self.state.player.black_heart_mask = run.player.black_heart_mask;
        player_items::add_soul_hearts(&mut self.state.player, run.player.soul_hp);
        self.state.player.eternal_hearts = run.player.eternal_hearts;
        self.state.player.golden_key = run.player.golden_key;
        self.state.player.transformation_counts = run.player.transformation_counts;
        self.state.inventory = run.inventory.clone();
        if let Some(history) = self
            .state
            .player_items
            .as_mut()
            .and_then(|owner| owner.query_history.as_mut())
        {
            history.damage_health_known = true;
        }
        self.state.player_stats = player_stats::PlayerStats::default();
        self.state.player_stats.luck = Some(0.0);
        self.state.player.flying = Some(false);
        self.state.player.grid_collision_class = Some(5);
        self.state.weapon_cache = weapon_plan::WeaponCache::fresh_player();
        self.state.weapon_admission = weapon_plan::WeaponAdmission::fresh_isaac();
        self.state.actor_scheduler = actors::Scheduler::default();
        self.state.actor_scheduler.admit(actors::ActorRef {
            kind: actors::ActorKind::Player,
            id: 0,
        })?;
        self.state.tears.clear();
        self.state.next_tear = 1;
        self.state.enemy_projectiles.clear();
        self.state.next_enemy_projectile = 1;
        self.state.fire_particles.clear();
        self.state.sprite_effects.clear();
        self.state.next_effect = 1;
        laser_runtime::reset_room(&mut self.state)?;
        self.state.knives.clear();
        self.state.familiars.clear();
        self.state.next_familiar = 1;
        self.state.attack_runtime = attack_runtime::AttackRuntime::fresh_player();
        self.state.next_laser = 1;
        self.state.bombs.clear();
        self.state.explosions.clear();
        self.state.next_bomb = 1;
        self.state.next_pickup = 1;
        self.state.fire_cooldown = -1;
        self.state.max_fire_delay = 10;
        self.state.head_timer = -1;
        self.state.damage_cooldown = 0;
        self.state.bomb_placement_cooldown = bombs::BombPlacementCooldown::default();
        self.state.bomb_action_was_down = self.held(8);
        self.state.player_death = player_death::PlayerDeath::default();
        self.state.player_animation = player_animation::PlayerAnimation::default();
        self.state.run_introduction = RunIntroduction::stage(false);
        self.state.boss_bar = BossBar::default();
        self.state.pause_menu = PauseMenu::hidden();
        self.state.run_quit = run_quit::RunQuit::fresh();
        self.state.stage_transition = stage_transition::StageTransition::default();
        self.state.room_transition = None;
        self.state.departure_position = None;
        self.state.knife_query_partition = None;
        self.state.stage = run.stage;
        self.state.stage_seed = context.stage_seed;
        self.state.floor = run.floor.clone();
        self.state.current_room = run.current_room;
        self.state.saved_room_entry = context.level_entry.clone();
        self.state.visited_rooms = context.visited_rooms.clone();
        self.state.room_display_flags = context.room_display_flags.clone();
        self.state.revealed_secret_connections = context.revealed_secret_connections.clone();
        self.state.persisted_rooms = context.persisted_rooms.clone();
        self.state.level_progress = context.level_progress.clone();
        self.state.game_update_count = run.game_update_count;
        mt::note_update(run.game_update_count);
        self.state.run_tick = 0;
        self.state.screen = Screen::Playing;
        self.state.paused = false;
        //58C9F2 resets135 real descriptors before58CB69 copies saved records.
        //The discarded initialization still consumes its actual shared words.
        let _cleared = self.state.boot_lifecycle.initialize_level_grid();
        self.boot_level_grid = self.saved_game.descriptor_seeds.clone();
        self.state
            .boot_lifecycle
            .update_level_grid_samples(&self.boot_level_grid);
        if !self.room_lookup_dummy_constructed {
            mt::DescriptorSeeds::construct_lookup_dummy(&mut self.state.boot_lifecycle.shared);
            self.room_lookup_dummy_constructed = true;
        }
        self.state.room = Room::from_floor_descriptor_with_special(
            &self.state.floor,
            self.state.current_room,
            self.special_room_assets.as_deref().map(Vec::as_slice),
        )?;
        self.state.room.initialization_game_update = Some(run.game_update_count);
        self.state.room.source_tear_slowdown_2494 = Some(0);
        self.state.room.combat_clear = run.room.combat_clear;
        ambience::reset_room(&mut self.state);
        //5CEF3E→5C5B30 reloads backdrop FX on every Room.Init, including
        //visited descriptors. The distinct5CF618 gate skips first-visit critters.
        room::load_fx_rays(
            &mut self.state.room,
            &mut self.state.boot_lifecycle.shared,
            self.state.stage,
            0,
        );
        grid_records::restore_records(
            &mut self.state.room,
            &mut self.state.boot_lifecycle.shared,
            &run.grid_records,
            run.game_update_count,
        )?;
        let selectors = self
            .state
            .room
            .spawns
            .as_ref()
            .ok_or("saved Room has no selectors")?
            .selectors
            .clone();
        let viewer = bombs::player_world(&self.state);
        self.admit_room_pickups_owned(
            &selectors,
            run.game_update_count,
            true,
            viewer,
            Some(
                context
                    .ambient_records
                    .as_deref()
                    .ok_or("Continue saved ambient ownership is unknown")?,
            ),
        )?;
        //The Room.Init placement call sees constructor3074=false and returns.
        item_queries::note_room_placement(&mut self.state);
        if let Some(id) = music::stage_track(self.state.stage, false) {
            self.state.music.room(id);
        }
        //515F20 requests the complete cache after saved stores, then3074=true.
        self.state
            .player_items
            .as_mut()
            .expect("loaded Player")
            .unevaluated_cache_flags = u32::MAX;
        player_stats::evaluate_cache(&mut self.state)?;
        if self
            .state
            .player_items
            .as_ref()
            .expect("loaded Player")
            .unevaluated_cache_flags
            != 0
        {
            return Err("Continue post-load cache has an unresolved actual callback".into());
        }
        self.state.player.initialized_3074 = true;
        self.place_continued_player()?;
        item_queries::note_room_placement(&mut self.state);
        self.state.camera = Some(camera::Camera::new(&self.state.room, None));
        //54EF4E→584200 reaches the same actual Room.Update callback without
        //advancing System or Game counters.54EF5E then starts the initial fade.
        self.advance_room_runtime(phase::Phase::Update, false)?;
        self.state
            .run_admission
            .initialize_saved(&mut self.state.player)?;
        //The actual loaded queue replaces any previous live request; its
        //unsupported contexts remain absent rather than borrowing that run.
        self.state.living_restart = living_restart::LivingRestart::source_reset();
        if self.state.character == "Isaac"
            && self.state.difficulty == "normal"
            && self.state.challenge_id == 0
        {
            self.state.living_restart.context = Some(living_restart::Context::normal_isaac(
                self.state.run_seed.clone(),
                self.state.seeded_run,
            )?);
            self.state.living_restart.held_polls = Some(0);
        }
        self.state.run_start_request.resume_saved = false;
        //594BE4 joins594AD3 AFTER the saved loader's immediate callback;
        //the actual odd System pre-counter advances once at594BF0.
        self.state.system_clock.align_run_start()?;
        self.refresh_minimap();
        Ok(())
    }

    fn place_continued_player(&mut self) -> Result<(), String> {
        let entry = self
            .state
            .saved_room_entry
            .as_ref()
            .ok_or("saved Level entry is unknown")?;
        let mut slot = entry.slot;
        if slot == -1 {
            //510AC8 scans actual bound Door pointers, not every possible slot.
            //The first wins without RNG; each later wins unless RandomInt7=0.
            for index in 0..8 {
                if !self
                    .state
                    .room
                    .appearance
                    .doors
                    .iter()
                    .any(|door| usize::from(door.slot) == index)
                {
                    continue;
                }
                if slot == -1 || self.state.boot_lifecycle.shared.bounded(7) != 0 {
                    slot = index as i32;
                }
            }
            if slot == -1 {
                slot = self.state.boot_lifecycle.shared.bounded(8) as i32;
            }
        }
        let world = if entry.previous_room == -4
            && self.state.current_room < self.state.floor.descriptors.len()
            && (entry.position.x != 0.0 || entry.position.y != 0.0)
        {
            entry.position
        } else {
            let door = self
                .state
                .room
                .appearance
                .doors
                .iter()
                .find(|door| i32::from(door.slot) == slot)
                .ok_or("Continue placement has no actual bound Door geometry")?;
            let offset = match slot % 4 {
                0 => motion::WorldVector { x: 40.0, y: 0.0 },
                1 => motion::WorldVector { x: 0.0, y: 40.0 },
                2 => motion::WorldVector { x: -40.0, y: 0.0 },
                _ => motion::WorldVector { x: 0.0, y: -40.0 },
            };
            door.world_position + offset
        };
        self.state.player.x = f64::from(world.x) * 0.65 + 32.0;
        self.state.player.y = f64::from(world.y) * 0.65 - 46.0;
        self.state.player.vx = 0.0;
        self.state.player.vy = 0.0;
        Ok(())
    }

    /// ChangeRoom5509eb saves the room being left (5182c3 -> 58d921 ->
    /// Room.Save5ce2d0), then the next Room.Init loads its backdrop FX rays.
    /// Selector, door and wall constructors follow in fresh admission.
    fn change_room_shared_draws(&mut self, room_before: usize) -> Result<(), String> {
        //Other Level entry stores require the actual doorway/save owner;
        //a previously known initial value must not survive room travel.
        self.state.saved_room_entry = None;
        let left = self
            .state
            .persisted_rooms
            .get(&room_before)
            .ok_or("room change requires the persisted room it left")?;
        // The transition first renders the room being left (its light pass).
        room_lights::present_room(left, &mut self.state.boot_lifecycle.shared);
        saved_game::save_grid_cells(
            &mut self.boot_level_grid,
            &mut self.state.boot_lifecycle.shared,
            left,
            room_before,
        )?;
        // Backdrop::pre_render_walls runs inside 5c5b30 before the FX rays.
        self.state.room.apply_boss_backdrop(self.state.stage)?;
        for (position, scale) in room::basement_boss_blood(&self.state.room, self.state.stage)? {
            let seed = self.state.boot_lifecycle.shared.next_u32();
            let id =
                familiars::admit_blood_splat(&mut self.state, seed, position, [1.0; 4], [0.0; 3])?;
            familiars::set_blood_scale(&mut self.state, id, scale)?;
            familiars::advance_blood_initial(&mut self.state, id)?;
        }
        // The port builds only ordinary Basement (stage type 0).
        ambience::reset_room(&mut self.state);
        room::load_fx_rays(
            &mut self.state.room,
            &mut self.state.boot_lifecycle.shared,
            self.state.stage,
            0,
        );
        Ok(())
    }

    /// Level.Init calls System.Save at 58c411 once the starting room is loaded.
    /// Game.Save clears the destination, updates Room grid records, then copies
    /// ordinary Level rows and PlayerManager state in that source order.
    fn save_supported_run(&mut self) -> Result<(), String> {
        self.saved_game = self.state.boot_lifecycle.clear_saved_game();
        self.state.saved_load_valid = Some(false);
        // Game.Save 54f33b copies the actual modeled ItemPool before Room.Save.
        // Retain its owned vectors/RNGs and trinket availability/count;
        //581ED0 does not copy current-profile trinket eligibility or sample.
        let item_pools = self
            .state
            .item_pools
            .as_ref()
            .ok_or("run save requires initialized item pools")?
            .copy_for_save()?;
        let grid_save = saved_game::save_grid_cells(
            &mut self.boot_level_grid,
            &mut self.state.boot_lifecycle.shared,
            &self.state.room,
            self.state.current_room,
        )?;
        //Room.Save5CE418 captures actor records before PlayerManager.Save
        //can flush later grant callbacks. This is not a later live-state clone.
        let regular_records = pickups::capture_regular_records(&self.state)?;
        let ambient_records = critters::capture_saved_records(&self.state);
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
        let mut saved = saved_game::SavedRun::copy_from(
            &self.state,
            item_pools,
            grid_save.records,
            player_items,
        )?;
        saved.room.saved_regular_pickups = regular_records;
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
        self.retain_profile_save(ambient_records)?;
        Ok(())
    }

    ///The caller has completed54F230's actual save copy. File transport
    ///does not invent another Save, consume RNG or serialize the live Game.
    fn retain_profile_save(
        &mut self,
        ambient_records: Option<Vec<critters::SavedAmbientRecord>>,
    ) -> Result<(), String> {
        let record = save_profiles::RunRecord {
            run: self
                .saved_run
                .clone()
                .ok_or("profile save requires the completed run copy")?,
            seeds: self.saved_game.clone(),
            summary: self
                .state
                .saved_run_summary
                .clone()
                .ok_or("profile save is missing its completed summary")?,
            context: save_profiles::RunContext::capture(&self.state, ambient_records),
        };
        // A room holding more actors than the record format admits refuses
        // this save: the profile keeps its previous record
        // and no write is requested, so the Tick and the run continue.
        if !record.rooms_within_supported_counts() {
            return Ok(());
        }
        record.validate()?;
        let retained = self.saved_profiles.get_or_insert_with(|| {
            std::sync::Arc::new(save_profiles::Profiles::fresh(&self.state.profiles))
        });
        let profiles = std::sync::Arc::make_mut(retained);
        let slot = usize::from(self.state.profile - 1);
        profiles.profiles[slot] = save_profiles::ProfileRecord {
            profile: self.state.profiles[slot].clone(),
            saved: Some(record),
        };
        self.state.save_revision = self
            .state
            .save_revision
            .checked_add(1)
            .ok_or("save revision exhausted")?;
        if let Some(runtime) = &mut self.state.persistent_runtime {
            runtime.mark_run_checksum_write();
            runtime.take_dirty_flush()?;
        }
        self.request_save_write()
    }

    ///Profile load follows the actual destination Clear. The file replaces
    ///only its serialized stores; the unlisted ordinary rows remain cleared.
    fn load_profile_save(&mut self, profile: u8) -> Result<(), String> {
        if !(1..=3).contains(&profile) {
            return Err("profile load requires a real slot".into());
        }
        if let Some(record) = self
            .saved_profiles
            .as_ref()
            .and_then(|profiles| profiles.profiles[usize::from(profile - 1)].saved.as_ref())
        {
            self.saved_game
                .load_serialized_stores(&record.seeds, record.run.floor.descriptors.len())?;
        }
        self.install_loaded_profile_metadata(profile)
    }

    ///Called only after the actual selected load has completed. This also
    ///serves fresh-menu callbacks without copying their seed stores twice.
    fn install_loaded_profile_metadata(&mut self, profile: u8) -> Result<(), String> {
        self.state.saved_load_valid = Some(false);
        if !(1..=3).contains(&profile) {
            return Err("profile load requires a real slot".into());
        }
        let Some(record) = self
            .saved_profiles
            .as_ref()
            .and_then(|profiles| profiles.profiles[usize::from(profile - 1)].saved.as_ref())
        else {
            return Ok(());
        };
        self.saved_run = Some(record.run.clone());
        self.state.saved_run_summary = Some(record.summary.clone());
        self.state.saved_load_valid = Some(true);
        self.state
            .boot_lifecycle
            .update_saved_game_samples(&self.saved_game);
        Ok(())
    }

    ///Application boot input contains only this browser's versioned profile
    ///document. It cannot replace gameplay or the running System/clock owner.
    pub fn import_save_data(&mut self, bytes: &[u8]) -> Result<(), String> {
        if self.state.screen != Screen::Title || self.state.run_start_request.fading {
            return Err("saved profiles can only load into a fresh Title Game".into());
        }
        let profiles = match save_profiles::Profiles::from_bytes(bytes) {
            Ok(profiles) => profiles,
            Err(error) => {
                self.profile_document_rejected = true;
                return Err(error);
            }
        };
        self.profile_document_rejected = false;
        self.state.profiles = profiles
            .profiles
            .iter()
            .map(|record| record.profile.clone())
            .collect();
        self.saved_profiles = Some(std::sync::Arc::new(profiles));
        Ok(())
    }

    /// A rejected own document remains quarantined until a successful import.
    /// This reports the actual transport owner, not a guessed save capability.
    pub fn rejected_profile_document(&self) -> bool {
        self.profile_document_rejected
    }

    pub fn export_save_data(&self) -> Result<Vec<u8>, String> {
        if let Some(error) = &self.save_continuation_failure {
            return Err(error.clone());
        }
        if let Some(request) = &self.save_write_request {
            return request.packet.export();
        }
        self.serialize_profile_data()?.export()
    }

    fn serialize_profile_data(&self) -> Result<save_profiles::SaveWritePacket, String> {
        if self.profile_document_rejected {
            return Err("preserve rejected own-profile bytes; unknown legacy owners cannot be saved as zero".into());
        }
        let fresh;
        let profiles = match self.saved_profiles.as_deref() {
            Some(profiles) => profiles,
            None => {
                fresh = save_profiles::Profiles::fresh(&self.state.profiles);
                &fresh
            }
        };
        profiles.write_packet()
    }

    ///A real application storage owner declares this capability before any
    ///public run. Default standalone/native replay has no host IO dependency.
    pub fn enable_save_io(&mut self, protocol: u32) -> Result<(), String> {
        if protocol != 1 {
            return Err("unsupported own Save IO protocol".into());
        }
        if self.state.screen != Screen::Title
            || self.state.save_revision != 0
            || self.menu_readmission.is_some()
            || self.save_write_request.is_some()
        {
            return Err("own save IO must be declared at fresh Title".into());
        }
        self.save_io_enabled = true;
        Ok(())
    }

    fn request_save_write(&mut self) -> Result<(), String> {
        if !self.save_io_enabled {
            return Ok(());
        }
        if self.save_write_request.is_some() {
            //Never collapse distinct callbacks into the newest revision. An
            //unowned same-iteration second write is refused atomically.
            return Err("complete the actual Save/Clear write before another callback".into());
        }
        let packet = self.serialize_profile_data()?;
        self.save_write_request = Some(save_profiles::SaveWriteRequest {
            protocol: 1,
            revision: self.state.save_revision,
            packet,
            remaining_ticks: None,
        });
        self.state.save_write_pending = true;
        Ok(())
    }

    ///Host success/failure belongs to actual IO evidence, not loaded validity.
    ///Both outcomes resume; a later real read decides whether older bytes load.
    pub fn complete_save_write(
        &mut self,
        revision: u64,
        _write_succeeded: bool,
    ) -> Result<(), String> {
        if let Some(error) = &self.save_continuation_failure {
            return Err(error.clone());
        }
        let mut next = self.clone();
        let request = next
            .save_write_request
            .take()
            .ok_or("no pending save write")?;
        if !next.save_io_enabled || request.protocol != 1 || request.revision != revision {
            return Err("stale save write acknowledgement".into());
        }
        next.state.save_write_pending = false;
        let resumed = (|| -> Result<(), String> {
            if let Some(resume) = next.living_write_resume.take() {
                let interval = resume
                    .interval
                    .ok_or("living Restart acknowledgement has no consumed interval")?;
                let completed = if let living_restart::ResumeStage::AfterCharacterProfileFlush {
                    phase,
                    pre_counter,
                    bomb_triggered,
                } = resume.stage
                {
                    next.complete_character_request()?;
                    next.tick_after_character_request(phase, pre_counter, bomb_triggered)?;
                    true
                } else {
                    next.resume_living_restart(resume.stage)?;
                    if let Some(next_resume) = &mut next.living_write_resume {
                        next_resume.interval = Some(interval);
                        false
                    } else {
                        //The public clock was consumed before the first IO callback.
                        //Only now finish this same System interval; never call tick().
                        next.tick_after_menu_requests()?;
                        true
                    }
                };
                if completed {
                    if interval.presented && next.state.screen == Screen::Playing {
                        room_lights::present(&mut next.state);
                        if next.state.current_room != interval.room_before {
                            room_lights::present(&mut next.state);
                        }
                    }
                    if let Some(request) = &mut next.save_write_request {
                        request.remaining_ticks = Some(interval.remaining_ticks - 1);
                    } else {
                        next.advance_public_ticks(interval.remaining_ticks - 1, None)?;
                    }
                }
            } else if let Some(remaining) = request.remaining_ticks {
                next.advance_public_ticks(remaining, None)?;
            }
            Ok(())
        })();
        if let Err(error) = resumed {
            //The exact revision was already attempted by the host. Retain
            //that consumed acknowledgement and the completed Source prefix,
            //make the real continuation error terminal, and never retry it.
            next.save_continuation_failure = Some(error.clone());
            *self = next;
            return Err(error);
        }
        next.finish_public_command();
        *self = next;
        Ok(())
    }

    ///Retain the same invalidation after the caller executes594910's actual
    ///55CE40→55BD80 destination Clear. This transport adds no shared draws.
    fn invalidate_profile_run(&mut self) -> Result<(), String> {
        self.saved_run = None;
        self.state.saved_load_valid = Some(false);
        self.state.saved_run_summary = None;
        self.state.run_quit.save_enabled = Some(false);
        let retained = self.saved_profiles.get_or_insert_with(|| {
            std::sync::Arc::new(save_profiles::Profiles::fresh(&self.state.profiles))
        });
        let profiles = std::sync::Arc::make_mut(retained);
        let slot = usize::from(self.state.profile - 1);
        profiles.profiles[slot].saved = None;
        profiles.profiles[slot].profile = self.state.profiles[slot].clone();
        self.state
            .persistent_runtime
            .as_mut()
            .ok_or("Persistent Clear checksum owner is unknown")?
            .mark_run_checksum_write();
        self.state.save_revision = self
            .state
            .save_revision
            .checked_add(1)
            .ok_or("save revision exhausted")?;
        self.request_save_write()
    }
    ///This first Restart slice owns only actual ordinary normal Isaac.
    ///Unsupported requests retain the existing public capability failure.
    fn living_restart_admitted(&self) -> bool {
        !self.state.player.dead
            && self.state.character == "Isaac"
            && self.state.difficulty == "normal"
            && self.state.challenge_id == 0
            && self
                .state
                .living_restart
                .context
                .as_ref()
                .is_some_and(|context| context.validate().is_ok())
    }

    fn held(&self, action: usize) -> bool {
        self.state.keyboard.held(action, &self.state.pressed)
    }
    /// The supported free regular callback is shared by ordinary full
    /// contacts and the actual Room.Update admission tail.
    fn collect_regular_pickup_at(&mut self, index: usize) -> Result<(), String> {
        if shop::player_contact(&mut self.state, index)? {
            return Ok(());
        }
        if self.state.room.pickups[index].variant == 90 {
            return pickups::collect_battery(&mut self.state, index);
        }
        if self.state.room.pickups[index].variant == 350 {
            return Err("loose350 contact is outside the admitted grant-only slice".into());
        }
        if matches!(self.state.room.pickups[index].variant, 70 | 300) {
            pockets::player_contact(&mut self.state, index)?;
            return Ok(());
        }
        if !pickups::player_can_collect(&self.state) {
            return Ok(());
        }
        //The existing free Coin operator has not qualified held/Kind2
        //effects. Refuse that reached context before granting inventory.
        if self.state.room.pickups[index].variant == 20
            && self.state.room.pickups[index].touches(&self.state.player)
            && self.state.player_items.as_ref().is_none_or(|owner| {
                owner.trinket_slots != [0, 0] || !owner.queue_excludes_trinkets()
            })
        {
            return Err("free Coin with held/queued trinkets is not admitted".into());
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
                pickup.try_collect(&mut self.state.inventory, &mut self.state.player)
            })
        {
            pickups::collect_sound(&mut self.state, index);
        }
        Ok(())
    }

    /// The ChangeRoom Room.Update5d12a0 tail updates the Player first at the
    /// unchanged counter, already placed at the entry door with zero velocity.
    /// Its only shared draws are the 50c000 footprint, which reads the velocity
    /// after this update's input (531c8f), so it fires only while a movement
    /// key is still held. A native probe of 50c000 shows (0, -0.528) with Up
    /// held and (0, 0) once released.
    fn admitted_update_footprint(&mut self, phase: phase::Phase) -> Result<(), String> {
        self.state.departure_position = None;
        if phase != phase::Phase::Update {
            return Ok(());
        }
        let (dx, dy) = if self.state.player.controls_enabled {
            (
                i32::from(self.held(1)) - i32::from(self.held(0)),
                i32::from(self.held(3)) - i32::from(self.held(2)),
            )
        } else {
            (0, 0)
        };
        let velocity = motion::advance_world(
            bombs::player_world(&self.state),
            bombs::player_velocity(&self.state),
            dx,
            dy,
            self.state.player_stats.move_speed,
        )?
        .1;
        player_footprints::advance(&mut self.state, velocity)
    }
    /// The native list admits Player first, then supported selectors in their
    /// authored order. Persist that order with the room rather than regrouping
    /// NPCs and pickups when rebuilding the shared actor scheduler.
    /// `critter_counter` is Game+ce9f4 while this Room.Init runs and `loader`
    /// marks the run loader's starting room.
    /// `critter_viewer` is where Room.Init's critter updates see the player.
    pub fn admit_room_pickups(
        &mut self,
        selectors: &[room_spawns::SpawnSelector],
        critter_counter: u32,
        loader: bool,
        critter_viewer: motion::WorldVector,
    ) -> Result<usize, String> {
        self.admit_room_pickups_owned(selectors, critter_counter, loader, critter_viewer, None)
    }

    fn admit_room_pickups_owned(
        &mut self,
        selectors: &[room_spawns::SpawnSelector],
        critter_counter: u32,
        loader: bool,
        critter_viewer: motion::WorldVector,
        saved_ambient: Option<&[critters::SavedAmbientRecord]>,
    ) -> Result<usize, String> {
        let saved_room = saved_ambient.is_some();
        //453EC0 finished its ordinary-list reset and callbacks.
        ordinary_placement::begin_fresh_room(&mut self.state);
        let mut pickups = Vec::new();
        let mut npcs = Vec::new();
        let mut room_slots = Vec::new();
        let mut order = Vec::new();
        let mut next = self.state.next_pickup;
        // The source descriptor map survives Room.Init. Only a source-owned
        // fresh empty map is admitted here; saved payload replay is separate.
        let room_index = self.state.current_room;
        let descriptor = self
            .state
            .floor
            .descriptors
            .get(room_index)
            .ok_or("persistent spawn map requires current Level descriptor")?;
        let mut persistent_cells = descriptor
            .persistent_spawn_cells
            .clone()
            .ok_or("persistent spawn map history is unknown")?;
        let cells = (usize::from(descriptor.template.width) + 2)
            * (usize::from(descriptor.template.height) + 2);
        if [&persistent_cells.present, &persistent_cells.unknown]
            .into_iter()
            .any(|keys| {
                keys.windows(2).any(|pair| pair[0] >= pair[1])
                    || keys.iter().any(|&cell| usize::from(cell) >= cells)
            })
            || persistent_cells
                .unknown
                .iter()
                .any(|cell| persistent_cells.present.binary_search(cell).is_ok())
            || self.state.room.descriptor.as_ref().is_none_or(|room| {
                room.index != room_index
                    || room.persistent_spawn_cells.as_ref() != Some(&persistent_cells)
            })
        {
            return Err("persistent spawn map owner is inconsistent".into());
        }
        if !persistent_cells.present.is_empty() || !persistent_cells.unknown.is_empty() {
            return Err("persistent spawn map saved actor replay is unadmitted".into());
        }
        let mut suppressed_entities = 0usize;
        let room_type = self
            .state
            .room
            .descriptor
            .as_ref()
            .map_or(1, |descriptor| descriptor.template.room_type);
        for selector in selectors {
            // 5cf230/238 and5cf25e already consumed each authored RNG11
            // draw/selection. 5cf285..2cc checks the map before entering
            // SpawnEntity, including grid factories and their local Init.
            if persistent_cells
                .present
                .binary_search(&selector.grid_index)
                .is_ok()
            {
                if selector.category == room_spawns::SpawnCategory::Entity {
                    suppressed_entities += 1;
                }
                continue;
            }
            if persistent_cells
                .unknown
                .binary_search(&selector.grid_index)
                .is_ok()
            {
                return Err("persistent spawn cell depends on an unadmitted factory".into());
            }
            let before_entities = order.len();
            // Room.Init5cecc0 constructs grid objects in the same selector pass.
            grid_records::admit_selector(
                &mut self.state.room,
                &mut self.state.boot_lifecycle.shared,
                selector,
                self.state.game_update_count,
            )?;
            ordinary_placement::remember_room_frontier(&mut self.state);
            let id = u32::try_from(npcs.len()).map_err(|_| "NPC identity exhausted")?;
            let context = entities::SpawnContext {
                stage: self.state.stage,
                game_update_count: self.state.game_update_count,
            };
            if let Some(mut npc) = entities::spawn(id, selector, &context)? {
                //Registered AI is not a proof of its producer footprint.
                ordinary_placement::invalidate(
                    &mut self.state,
                    ordinary_placement::Frontier::ActorEffects {
                        actor: actors::ActorRef {
                            kind: actors::ActorKind::Npc,
                            id,
                        },
                    },
                );
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
            } else if selector.entity_type == 6 {
                // Slot Init and its spawnConfig rewrite draw no shared MT and
                // roll no champion; the idle actor spawns nothing.
                let id = u32::try_from(room_slots.len()).map_err(|_| "Slot identity exhausted")?;
                if let Some(slot) = slots::Slot::from_selector(id, selector, room_type)? {
                    order.push(slots::actor(id));
                    room_slots.push(slot);
                }
            } else if room_type == 2 && selector.entity_type == 5 && selector.variant == 150 {
                if let Some(mut pickup) = shop::from_selector(&mut self.state, next, selector)? {
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
                let pickup = if selector.entity_type == 5 && matches!(selector.variant, 70 | 300) {
                    Pickup::from_pocket_selector(next, selector, true, &self.state)?
                } else if selector.entity_type == 5 && selector.variant == 60 {
                    chests::from_locked_selector(next, selector, true, &self.state)?
                } else if selector.entity_type == 5 && selector.variant == 50 {
                    chests::from_source_selector(next, selector, true, &self.state)?
                } else {
                    Pickup::from_source_selector(next, selector, true, &self.state)?
                };
                if let Some(mut pickup) = pickup {
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
            if selector.category == room_spawns::SpawnCategory::Entity
                && order.len() == before_entities
            {
                ordinary_placement::invalidate(
                    &mut self.state,
                    ordinary_placement::Frontier::Factory {
                        entity_type: u32::from(selector.entity_type),
                        variant: u32::from(selector.variant),
                        subtype: u32::from(selector.subtype),
                    },
                );
            }
            // SpawnEntity5ce7a8..7d3 calls5c57c0 with the selected type,
            // variant, edx=0,cl=0 after successful Game.Spawn. For this
            // authored entity domain1..999, exactly4..6,17,33 persist.
            // Grid records never insert a key; an occupied key still skips
            // a later grid group above. This is not coordinate deduplication.
            if selector.category == room_spawns::SpawnCategory::Entity
                && matches!(selector.entity_type, 4..=6 | 17 | 33)
            {
                if order.len() == before_entities {
                    // An authored 5.350 trinket's Init/Get is outside the
                    // admitted grant-only slice. Its room stays traversable
                    // like any unsupported factory; item queries record the
                    // unrepresented spawn instead of halting the run.
                    if selector.entity_type == 5 && selector.variant == 350 {
                        crate::item_queries::note_unrepresented_trinket_spawn(&mut self.state);
                    }
                    // Preserve unrelated traversal when its existing factory
                    // is unsupported. Only a later lookup of this cell needs
                    // the missing success/key owner and must stop there.
                    let insertion = persistent_cells
                        .unknown
                        .binary_search(&selector.grid_index)
                        .expect_err("cell was checked before its factory");
                    persistent_cells
                        .unknown
                        .insert(insertion, selector.grid_index);
                } else {
                    let insertion = persistent_cells
                        .present
                        .binary_search(&selector.grid_index)
                        .expect_err("cell was checked before its factory");
                    persistent_cells
                        .present
                        .insert(insertion, selector.grid_index);
                }
                self.state.floor.descriptors[room_index].persistent_spawn_cells =
                    Some(persistent_cells.clone());
                self.state
                    .room
                    .descriptor
                    .as_mut()
                    .expect("checked current descriptor")
                    .persistent_spawn_cells = Some(persistent_cells.clone());
            }
        }
        // Doors, walls and shape0 ambient decoration follow the selectors.
        // Other shapes and backdrops keep their boundary draws unadmitted.
        if self
            .state
            .room
            .descriptor
            .as_ref()
            .is_some_and(|descriptor| {
                grid_records::shape0_ambient(descriptor.asset_stage)
                    && descriptor.template.shape_index() == 0
                    && descriptor.template.room_type != 16
            })
        {
            if saved_room {
                grid_records::admit_restored_boundary(
                    &mut self.state.room,
                    &mut self.state.boot_lifecycle.shared,
                    self.state.game_update_count,
                )?;
            } else {
                grid_records::admit_boundary(
                    &mut self.state.room,
                    &mut self.state.boot_lifecycle.shared,
                    self.state.game_update_count,
                    !loader,
                )?;
            }
        }
        // Room.Init5cf550 then links pits over every grid entity in order.
        if let Some(grid) = self.state.room.grid.as_mut() {
            grid::link_pits(grid, &mut self.state.boot_lifecycle.shared);
        }
        self.state.room.pickups = pickups;
        self.state.room.npcs = npcs;
        self.state.room.slots = room_slots;
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
        if let Some(records) = saved_ambient {
            //5CF585 restores ordinary records before descriptor3C first-visit
            //gating; the old ambient producer must never run on Continue.
            critters::restore_saved_records(&mut self.state, records)?;
        }
        self.state.room.complete_initialization();
        // Room.Init 5cf628: the first visit to a boss room slams its
        // portcullis (SFX 190, 5d0687) after the grid records; 5d0690 then
        // rejoins 5cf64f ahead of the ambient critters.
        if !loader && room_type == 5 {
            self.state.audio.admit(
                &mut self.state.boot_lifecycle.shared,
                self.state.system_clock.system_counter.wrapping_sub(1),
                audio::SoundRequest {
                    id: 190,
                    volume: 1.0,
                    delay: 2,
                    looping: false,
                    pitch: 1.0,
                },
            );
        }
        // First-visit ambient critters close Room.Init5cf64f inside this full
        // update, after the selector entities took their manager sequences.
        //5CF618 sends an already visited descriptor directly to5CFB36.
        //Its saved Room never repeats first-visit ambient producers.
        if !saved_room {
            critters::spawn(&mut self.state, critter_counter, loader, critter_viewer)?;
        }
        self.state.room.boss_count = self.state.room.manager_counts.boss_total().max(0) as u32;
        self.refresh_stage_effects();
        Ok(suppressed_entities)
    }
    /// Render keys (virtual+50) of the ordinary actors, then the manager's
    /// render-array insertion and, on a full update, its 452760 sort.
    /// Entity 441bd0 (Player, NPC, Familiar, Tear, Bomb, Pickup, Projectile):
    /// floor(f32(y * 100 + 10030 + 0.5)) plus the integer Entity+460 addend,
    /// compared unsigned. Entity_Player::Init 519350 sets that addend to 5
    /// (519485) after Entity::Init clears it, so Isaac draws over an NPC
    /// standing up to 0.05 below him; Fireplace AI 49a530 writes -1000
    /// (49a681/49a886/49affe). Effect 4586c0 adds +7f8 to y, or keys
    /// (Entity+8 & 0x3ff) + 100 when Init set +7d8; Knife 472620 and Laser
    /// 475f70 add +7e4 to y and take no addend. `departure` is the player's
    /// world y before a room change: the new room's actors are inserted
    /// against that key, without sorting.
    fn sort_render_array(&mut self, departure: Option<f32>, passes: bool) {
        let key = |y: f32, offset: i32| {
            ((((y * 100.0) + 10030.0) + 0.5).floor() as i64 as u32).wrapping_add(offset as u32)
        };
        let state = &self.state;
        let scheduler = &state.actor_scheduler;
        // Player x/y are screen units; the key reads world Entity+44c.
        let player = key(departure.unwrap_or(bombs::player_world(state).y), 5);
        let keys: Vec<(actors::ActorRef, u32)> = scheduler
            .full_phase()
            .into_iter()
            .filter_map(|actor| {
                let id = actor.id;
                let value = match actor.kind {
                    actors::ActorKind::Player => Some(player),
                    actors::ActorKind::Npc => {
                        state.room.npcs.iter().find(|npc| npc.id == id).map(|npc| {
                            let offset = npc
                                .fireplace
                                .as_ref()
                                .map_or(0, |fire| fire.render_order_offset);
                            key(npc.position.y, offset)
                        })
                    }
                    actors::ActorKind::Familiar => state
                        .familiars
                        .iter()
                        .find(|familiar| familiar.id == id)
                        .map(|familiar| key(familiar.position.y, 0)),
                    actors::ActorKind::Tear => state
                        .tears
                        .iter()
                        .find(|tear| tear.id == id)
                        .map(|tear| key(tear.motion.position.y, 0)),
                    actors::ActorKind::Bomb => state
                        .bombs
                        .iter()
                        .find(|bomb| bomb.id == id)
                        .map(|bomb| key(bomb.motion.position.y, 0)),
                    actors::ActorKind::Pickup => state
                        .room
                        .pickups
                        .iter()
                        .find(|pickup| pickup.id == id)
                        .map(|pickup| key(pickup.position.y, 0)),
                    // Entity_Slot keeps the base 441bd0 key at +448 y.
                    actors::ActorKind::Slot => state
                        .room
                        .slots
                        .iter()
                        .find(|slot| slot.id == id)
                        .map(|slot| key(slot.position.y, 0)),
                    actors::ActorKind::Projectile => state
                        .enemy_projectiles
                        .iter()
                        .find(|projectile| projectile.id == id)
                        .map(|projectile| key(projectile.position.y, 0)),
                    actors::ActorKind::Effect => {
                        if let Some(effect) =
                            state.sprite_effects.iter().find(|effect| effect.id == id)
                        {
                            if effect.raw_7d8 {
                                scheduler.sequence_id(actor).map(|sequence| {
                                    ((sequence & 0x3ff) + 100)
                                        .wrapping_add(effect.render_order_offset as u32)
                                })
                            } else {
                                Some(key(
                                    effect.position.y + effect.raw_7f8,
                                    effect.render_order_offset,
                                ))
                            }
                        } else if let Some(ember) =
                            state.fire_particles.iter().find(|ember| ember.id == id)
                        {
                            Some(key(ember.position.y, 0))
                        } else {
                            state
                                .lasers
                                .iter()
                                .filter_map(|laser| laser.impact.as_ref())
                                .find(|impact| impact.id == id)
                                .map(|impact| key(impact.position.y + impact.depth, 0))
                        }
                    }
                    actors::ActorKind::Knife => state
                        .knives
                        .iter()
                        .find(|knife| knife.id == id)
                        .map(|knife| key(knife.position.y + knife.raw_7e4, 0)),
                    actors::ActorKind::Laser => state
                        .lasers
                        .iter()
                        .find(|laser| laser.id == id)
                        .map(|laser| key(laser.position.y + laser.render_sort_offset, 0)),
                };
                value.map(|value| (actor, value))
            })
            .collect();
        self.state.actor_scheduler.sort_render(&keys, passes);
    }
    /// Persisted Room clones retain supported actor handles in authored order.
    pub fn rebuild_room_actor_schedule(&mut self) -> Result<(), String> {
        // Room.Init's backdrop pass (5c5b30) spawns the Basement boss blood
        // before the authored entities, so Room5c6940 numbers those flags8
        // stains first: a Larry Jr. room's nine segments take Entity+8 54..62
        // after its 54 stains. They are the only background effects holding
        // an identity since the room's schedule was last cleared.
        let mut backdrop: Vec<(u32, familiars::SpriteEffect)> = self
            .state
            .sprite_effects
            .iter()
            .filter(|effect| effect.flags & 8 != 0)
            .filter_map(|effect| {
                self.state
                    .actor_scheduler
                    .sequence_id(actors::ActorRef {
                        kind: actors::ActorKind::Effect,
                        id: effect.id,
                    })
                    .map(|sequence| (sequence, effect.clone()))
            })
            .collect();
        backdrop.sort_by_key(|(sequence, _)| *sequence);
        attack_runtime::reset_room_schedule(&mut self.state);
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
            attack_runtime::reset_room_schedule(&mut self.state);
        }
        for (_, effect) in backdrop {
            self.state
                .actor_scheduler
                .admit_background(actors::ActorRef {
                    kind: actors::ActorKind::Effect,
                    id: effect.id,
                })?;
            self.state.sprite_effects.push(effect);
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
                actors::ActorKind::Slot => {
                    self.state.room.slots.iter().any(|slot| slot.id == actor.id)
                }
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
                    menu_lifecycle::Initialization::LoadFreshProfile { profile } => {
                        self.state.persistent_runtime = Some(persistent::Runtime::fresh());
                        self.saved_game = self.state.boot_lifecycle.clear_saved_game();
                        self.saved_run = None;
                        self.state.saved_run_summary = None;
                        self.load_profile_save(profile)?;
                        self.state.main_selection = if self.state.saved_load_valid == Some(true) {
                            1
                        } else {
                            0
                        };
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
    ///594CB0 ->5949A0 ->54F3F0. The supported in-memory save happens
    ///before Stop resets modeled Game fields. Binary encoders55B7B0/55B890,
    ///audio virtual5E1D00 and manager Uninit callbacks remain separate owners.
    fn request_menu_readmission(&mut self) -> Result<(), String> {
        if self.menu_readmission.is_some() {
            return Err("duplicate menu readmission request".into());
        }
        //59A51A precedes profile reads. No new System interval runs while
        //the application performs the actual synchronous own-store read.
        self.state.music.crossfade(61);
        if !self.save_io_enabled {
            //No nonexistent application callback may stall ordinary replay.
            //This backend has no own stored document, not an inferred success.
            return self.reconstruct_menu_from_profiles(self.state.profile, None);
        }
        self.state.save_readmission_pending = true;
        self.state.saved_load_valid = Some(false);
        self.menu_readmission = Some(save_profiles::MenuReadmission {
            protocol: 1,
            request_revision: self.state.save_revision,
            profile: self.state.profile,
            remaining_ticks: 0,
            timestamp_millis: None,
        });
        Ok(())
    }

    ///Revision matches the outstanding request only. Stored profiles may
    ///legitimately be older after a failed write; never require their revision.
    pub fn complete_menu_readmission(
        &mut self,
        revision: u64,
        bytes: Option<&[u8]>,
    ) -> Result<bool, String> {
        if self.save_write_request.is_some() {
            return Err("attempt actual save write before menu readmission".into());
        }
        let mut next = self.clone();
        let request = next
            .menu_readmission
            .take()
            .ok_or("no pending menu readmission")?;
        if request.protocol != 1
            || request.request_revision != revision
            || request.remaining_ticks == 0
        {
            return Err("stale or incomplete menu readmission response".into());
        }
        let (profiles, rejected) = match bytes.map(save_profiles::Profiles::from_bytes).transpose()
        {
            Ok(profiles) => (profiles, false),
            Err(error) => {
                //An absent document owns a fresh constructor. Rejected own
                //legacy bytes do not authorize manufacturing missing stats.
                return Err(format!(
                    "preserve rejected own-profile readmission: {error}"
                ));
            }
        };
        next.reconstruct_menu_from_profiles(request.profile, profiles)?;
        //Resume this exact interrupted System iteration: its public clock
        //was already advanced. Remaining requested ticks keep their timestamps.
        next.tick_after_menu_requests()?;
        next.advance_public_ticks(request.remaining_ticks - 1, request.timestamp_millis)?;
        next.finish_public_command();
        *self = next;
        Ok(!rejected)
    }

    ///One source-producing fresh-menu callback sequence. No-host replay
    ///has no observed durable file and therefore cannot enable Continue.
    fn reconstruct_menu_from_profiles(
        &mut self,
        profile: u8,
        profiles: Option<save_profiles::Profiles>,
    ) -> Result<(), String> {
        //594DC0's actual profile reset/load clears transient dirty/inhibit.
        self.state.persistent_runtime = Some(persistent::Runtime::fresh());
        self.saved_profiles = profiles.map(std::sync::Arc::new);
        if let Some(profiles) = self.saved_profiles.as_deref() {
            self.state.profiles = profiles
                .profiles
                .iter()
                .map(|r| r.profile.clone())
                .collect();
        } else {
            self.state.profiles = (1..=3)
                .map(|slot| Profile {
                    slot,
                    unlocked_characters: vec!["Isaac".into()],
                    selected_character: "Isaac".into(),
                    achievements: vec![false; 179],
                    counters: vec![0; 95],
                    collected_history: persistent::CollectedHistory::source_reset(),
                })
                .collect();
        }
        self.state.profile = profile;
        self.state.menu_presentation.reinitialize();
        self.state.title_animation_frame = 0;
        //Native SaveMenu.Init performs all four Clear/load calls. The
        //validated own bytes issue the callback, never the pending Save DTO.
        let profiles = self.saved_profiles.clone();
        self.saved_game =
            self.state
                .boot_lifecycle
                .initialize_menu_saves(profile, |slot, destination| {
                    if let Some(record) = profiles
                        .as_ref()
                        .and_then(|p| {
                            slot.checked_sub(1)
                                .and_then(|s| p.profiles.get(usize::from(s)))
                        })
                        .and_then(|p| p.saved.as_ref())
                    {
                        destination.load_serialized_stores(
                            &record.seeds,
                            record.run.floor.descriptors.len(),
                        )?;
                    }
                    Ok(())
                })?;
        self.saved_run = None;
        self.state.saved_run_summary = None;
        self.install_loaded_profile_metadata(profile)?;
        self.state.main_selection = if self.state.saved_load_valid == Some(true) {
            1
        } else {
            0
        };
        //Main follows Save; Character then resets its public entry fields,
        //takes its one real shared sample, and Secrets takes179 bounded calls.
        self.state.seed_text.clear();
        self.state.seed_valid = false;
        self.state.boot_lifecycle.reset_character_seed();
        self.load_menu_character();
        self.state.boot_lifecycle.reset_secret_poses();
        self.state.options_menu.reinitialize();
        self.state.controls_menu.reinitialize();
        self.state.menu_fade = MenuFade {
            active: true,
            alpha: 1.0,
            color: [0.0, 0.0, 0.0, 1.0],
        };
        self.state.screen = Screen::Main;
        self.state.save_readmission_pending = false;
        Ok(())
    }

    /// Stop54F590 ->584140 ->5CDE50 reconstructs the Room.
    /// Backdrop423FA0 owns one raw seed before fresh menu/new-run admission.
    fn reconstruct_stopped_room_backdrop(&mut self) {
        let boot = &mut self.state.boot_lifecycle;
        boot.backdrop_seed = boot.shared.next_u32();
    }

    fn finish_pause_quit(&mut self) -> Result<(), String> {
        match self.state.run_quit.save_enabled {
            Some(true) if !self.state.player.dead => {
                self.state.music.reset_pitch();
                self.save_supported_run()?;
                self.state.run_quit.save_outcome = Some(run_quit::SaveOutcome::SupportedRunCopied);
            }
            Some(false) => {
                self.state.music.reset_pitch();
                //594910 is called again by Stop when the save-enable byte is false.
                self.saved_game = self.state.boot_lifecycle.clear_saved_game();
                self.saved_run = None;
                self.state.saved_run_summary = None;
                self.invalidate_profile_run()?;
                self.state.run_quit.save_outcome = Some(run_quit::SaveOutcome::DeadRunCleared);
            }
            _ => return Err("Pause Quit has no admitted live/dead save disposition".into()),
        }
        self.state.information_animation.clear();
        self.state.player_death = player_death::PlayerDeath::default();
        self.state.stage_transition = stage_transition::StageTransition::default();
        self.state.room_transition = None;
        self.state.boss_bar = BossBar::default();
        self.state.run_introduction.active = false;
        self.state.game_update_count = 0;
        //Stop54F530 resets the real hold even when the destination is Main.
        self.state.living_restart.held_polls = Some(0);
        self.state.minimap_presentation = map_presentation::MapPresentation::default();
        if let Some(shake) = &mut self.state.game_shake {
            shake.reset_duration();
        }
        self.state.source_cache_run_flags = Some(0);
        // Stop54F54B follows Save; its copied penalty stage stays intact.
        self.state.level_progress.last_penalty_stage = Some(0);
        self.state
            .persistent_runtime
            .as_mut()
            .ok_or("Persistent Stop inhibit owner is unknown")?
            .stop_clear_inhibit();
        // Stop54F571 ->576B70 follows the actual Save copy above.
        self.state.acquisition_history = Some(acquisition_history::AcquisitionHistory::default());
        self.reconstruct_stopped_room_backdrop();
        self.state.run_admission.initial_fade = 0.0;
        self.state.run_quit.fade_alpha = 0.0;
        self.state.run_quit.system_pending = false;
        self.state.run_start_request = RunStartRequest::default();
        self.state.paused = false;
        self.state.pause_menu = PauseMenu::hidden();
        self.request_menu_readmission()?;
        Ok(())
    }

    ///A profile-only callback preserves the retained old run and writes the
    ///actual live Profile. It owns no Save/Clear RNG and no public tick.
    fn flush_living_dirty_profile(&mut self) -> Result<bool, String> {
        let runtime = self
            .state
            .persistent_runtime
            .as_mut()
            .ok_or("Persistent dirty owner is unknown")?;
        if !runtime.take_dirty_flush()? {
            return Ok(false);
        }
        let slot = usize::from(self.state.profile - 1);
        let retained = self.saved_profiles.get_or_insert_with(|| {
            std::sync::Arc::new(save_profiles::Profiles::fresh(&self.state.profiles))
        });
        std::sync::Arc::make_mut(retained).profiles[slot].profile =
            self.state.profiles[slot].clone();
        self.state.save_revision = self
            .state
            .save_revision
            .checked_add(1)
            .ok_or("save revision exhausted")?;
        self.request_save_write()?;
        Ok(true)
    }

    fn wait_living_write(&mut self, stage: living_restart::ResumeStage) -> bool {
        if self.save_write_request.is_some() {
            self.living_write_resume = Some(living_restart::WriteResume {
                stage,
                interval: None,
            });
            true
        } else {
            false
        }
    }

    fn begin_living_restart_dispatch(&mut self) -> Result<(), String> {
        self.state
            .living_restart
            .pending
            .as_ref()
            .ok_or("living Restart request disappeared")?
            .validate()?;
        //5E1D00 queries loaded/playing/looping on the actual output backend.
        self.state.audio.stop_looping()?;
        self.state.music.reset_pitch();
        self.flush_living_dirty_profile()?;
        if self.wait_living_write(living_restart::ResumeStage::AfterProfileFlush) {
            return Ok(());
        }
        self.resume_living_restart(living_restart::ResumeStage::AfterProfileFlush)
    }

    fn resume_living_restart(&mut self, stage: living_restart::ResumeStage) -> Result<(), String> {
        use living_restart::ResumeStage;
        if stage == ResumeStage::AfterProfileFlush {
            //Stop(false) deletes the old run, then performs actual120961 Clear.
            self.saved_game = self.state.boot_lifecycle.clear_saved_game();
            self.invalidate_profile_run()?;
            if self.wait_living_write(ResumeStage::AfterClear) {
                return Ok(());
            }
            return self.resume_living_restart(ResumeStage::AfterClear);
        }
        if stage == ResumeStage::AfterClear {
            //Stop54F54B clears Game+10ECDC before NewRun's initial Save.
            self.state.level_progress.last_penalty_stage = Some(0);
            self.state
                .persistent_runtime
                .as_mut()
                .ok_or("Persistent Stop owner is unknown")?
                .stop_clear_inhibit();
            self.state.information_animation.clear();
            self.state.acquisition_history =
                Some(acquisition_history::AcquisitionHistory::default());
            self.state.player_death = player_death::PlayerDeath::default();
            self.state.stage_transition = stage_transition::StageTransition::default();
            self.state.room_transition = None;
            self.state.paused = false;
            self.state.pause_menu = PauseMenu::hidden();
            self.state.game_update_count = 0;
            self.state.living_restart.held_polls = Some(0);
            self.reconstruct_stopped_room_backdrop();
            let context = self
                .state
                .living_restart
                .pending
                .clone()
                .ok_or("living Restart queued context disappeared")?;
            context.validate()?;
            self.state.character = "Isaac".into();
            self.state.difficulty = "normal".into();
            self.state.seeded_run = context.source_r9;
            self.state.achievement_eligibility = !context.source_r9;
            self.state.run_start_request = RunStartRequest::default();
            self.living_start_context = Some(context);
            self.start_run_until_initial_save()?;
            self.state.run_quit.save_enabled = Some(false);
            if self.wait_living_write(ResumeStage::AfterInitialSave) {
                return Ok(());
            }
            return self.resume_living_restart(ResumeStage::AfterInitialSave);
        }
        self.finish_start_run_after_save()?;
        //594AD3 normalizes only after actual NewRun/initial Save/grants.
        self.state.system_clock.align_run_start()?;
        self.living_start_context = None;
        self.state.living_restart.pending = None;
        self.state.run_quit.save_enabled = Some(true);
        Ok(())
    }

    fn tick(&mut self, timestamp_millis: Option<u64>) -> Result<(), String> {
        self.state.persistent_callback_pre_counter = Some(self.state.system_clock.system_counter);
        self.state.run_admission.clock.advance(timestamp_millis)?;
        if self.state.system_clock.next_phase() != phase::Phase::Skipped
            && self.state.living_restart.pending.is_some()
        {
            self.begin_living_restart_dispatch()?;
            if self.living_write_resume.is_some() {
                return Ok(());
            }
        }
        // Source 594a10 consumes the prior menu request before game dispatch.
        // Room construction and clock alignment happen here, after the f32
        // confirmation fade, rather than in the zero-tick DOM key callback.
        if let Some(request) = self.state.player_death.system_pending.take() {
            //5949a0 sees old Systemstate2 before either deferred dispatch,
            // including2->2 Restart, and54f588 clears the shared Info owner.
            // NewRun54eb20 itself has no clear; delay remains unchanged.
            // The same Stop54F571 clears history for both deferred requests.
            // NewRun itself never reconstructs unknown history from counts.
            // Both deferred destinations pass Stop54F54B before dispatch.
            self.state.level_progress.last_penalty_stage = Some(0);
            self.state.acquisition_history =
                Some(acquisition_history::AcquisitionHistory::default());
            self.state.information_animation.clear();
            self.reconstruct_stopped_room_backdrop();
            match request {
                player_death::Request::Restart => {
                    if !self.state.seeded_run {
                        let seed = self.state.boot_lifecycle.shared.next_u32().max(1);
                        self.state.boot_lifecycle.commit_character_seed(seed)?;
                    }
                    self.start_run()?;
                }
                player_death::Request::Exit => {
                    self.state.player_death = player_death::PlayerDeath::default();
                    self.state.run_start_request = RunStartRequest::default();
                    self.state.paused = false;
                    self.state.pause_menu = PauseMenu::hidden();
                    self.request_menu_readmission()?;
                }
            }
        }
        if self.menu_readmission.is_some() {
            return Ok(());
        }
        if self.state.run_start_request.system_pending {
            self.state.run_start_request.system_pending = false;
            if self.state.run_start_request.resume_saved {
                self.continue_run()?;
            } else {
                self.start_run()?;
            }
        }
        //595110 consumes menu requests on the full System path (including
        //enabled interpolation), never on the disabled odd/skipped path.
        if self.state.run_quit.system_pending
            && self.state.system_clock.next_phase() != phase::Phase::Skipped
        {
            self.finish_pause_quit()?;
            return if self.menu_readmission.is_some() {
                Ok(())
            } else {
                self.tick_after_menu_requests()
            };
        }
        self.tick_after_menu_requests()
    }

    fn tick_after_menu_requests(&mut self) -> Result<(), String> {
        let audio_counter = self.state.system_clock.system_counter;
        self.state.persistent_callback_pre_counter = Some(audio_counter);
        let phase = self.state.system_clock.advance()?;
        if phase != phase::Phase::Skipped && self.state.nightmares.apply_request(self.state.stage) {
            self.state.music.room(96);
        }
        if phase != phase::Phase::Skipped {
            self.state
                .public_actions
                .advance(&self.state.pressed, &self.state.keyboard);
            let notification_dispatch = !self.state.unlock_notifications_loading
                && self
                    .state
                    .unlock_notifications
                    .as_ref()
                    .is_some_and(|runtime| runtime.active());
            if notification_dispatch {
                //The branch was sampled before Update. Finishing a popup does
                //not fall through to Game in this same System interval.
                self.state
                    .unlock_notifications
                    .as_mut()
                    .expect("sampled active owner")
                    .advance(
                        audio_counter,
                        self.state.unlock_notifications_loading,
                        &mut self.state.audio,
                        &mut self.state.boot_lifecycle.shared,
                    )?;
                self.state.tick += 1;
                self.state.menu_animation_tick += 1;
                self.state.music.flush_pending();
                self.state.music.advance();
                return Ok(());
            }
            if self.state.screen == Screen::Playing
                && !self.state.nightmares.system_active
                && self.living_restart_admitted()
            {
                if self.state.living_restart.poll(
                    self.state.public_actions.held[16],
                    self.state.weapon_admission.source_player_type,
                )? {
                    if self.profile_document_rejected {
                        return Err("living Restart requires known own-profile stores".into());
                    }
                    persistent_effects::reset_counter21(&mut self.state, &self.challenge_catalog)?;
                    self.state
                        .living_restart
                        .request_after_counter(&mut self.state.boot_lifecycle.shared)?;
                    self.state.run_quit.save_enabled = Some(false);
                    //Do not short-circuit the old Game's current presentation.
                }
            }
            if self.state.screen == Screen::Playing && self.state.run_quit.input_pending {
                self.state.run_quit.input_pending = false;
                if self.state.paused
                    && self.state.pause_menu.phase != PausePhase::Disappearing
                    && self.state.pause_menu.selection == 2
                    && self.state.player_death.game_over.is_none()
                    && self.state.room_transition.is_none()
                    && self.state.run_admission.initial_fade <= 0.0
                {
                    if self.state.player.dead {
                        //5BC1DD invalidates the saved run before requesting the fade.
                        self.saved_game = self.state.boot_lifecycle.clear_saved_game();
                        self.saved_run = None;
                        self.state.saved_run_summary = None;
                        self.invalidate_profile_run()?;
                    }
                    self.state.run_quit.begin();
                }
            }
            //Legacy snapshots without held-record ownership keep the old
            //conservative invalidation. Known records use5282B0's actual
            //controls/cooldown/main-clip/nonempty admission in pockets.
            if self.state.screen == Screen::Playing
                && self.state.public_actions.pocket_pressed
                && self
                    .state
                    .player_items
                    .as_ref()
                    .is_none_or(|items| items.held_pockets.is_none())
            {
                ordinary_placement::invalidate(
                    &mut self.state,
                    ordinary_placement::Frontier::PocketEffects,
                );
                self.state.attack_runtime.aim_rotation_c70 = None;
                if let Some(items) = &mut self.state.player_items {
                    //Untracked pill/card execution cannot retain speed history.
                    items.move_speed_inputs = None;
                }
            }
        }
        if phase != phase::Phase::Skipped {
            self.state.music.flush_pending();
        }
        self.flush_menu_initializations(audio_counter)?;
        let bomb_down = self.held(8);
        let bomb_triggered =
            phase != phase::Phase::Skipped && self.state.public_actions.triggered[8];
        self.state.bomb_action_was_down = bomb_down;
        // MenuManager+38 before this iteration's input picks the page update, so
        // the update that reads Start still advances the title clip and a
        // return to Title starts advancing it one iteration later.
        let title_page = self.state.menu_presentation.page == menu_presentation::Page::Title;
        if self.state.screen != Screen::Playing && title_page && phase == phase::Phase::Update {
            self.state.title_animation_frame = (self.state.title_animation_frame + 1) % 40;
        }
        self.state.menu_fade.advance();
        let menu_input = menu_presentation::Input {
            even: phase == phase::Phase::Update,
            key_held: !self.state.pressed.is_empty(),
            character: usize::from(self.state.character_selection),
            seed: self.state.screen == Screen::Seed,
            challenge: self.state.challenge_selection.unwrap_or(0),
            hard: self.state.difficulty == "hard",
        };
        //59B454 samples the dispatch page once. Main's opening Enter must
        //not also confirm Controls in the newly selected Options page.
        let options_callback = phase != phase::Phase::Skipped
            && self.state.screen == Screen::Options
            && self.state.menu_presentation.page == menu_presentation::Page::Options;
        let controls_callback = phase != phase::Phase::Skipped
            && matches!(
                (self.state.screen, self.state.menu_presentation.page),
                (Screen::Controls, menu_presentation::Page::Controls)
                    | (Screen::Keyboard, menu_presentation::Page::Keyboard)
            );
        self.state
            .menu_presentation
            .advance(self.state.screen, menu_input);
        if options_callback {
            let previous_selection = self.state.options_menu.selection;
            let action = self.state.options_menu.advance(
                &self.state.public_actions,
                &self.state.pressed,
                &mut self.state.audio,
                &mut self.state.music,
                &mut self.state.map_settings,
                &mut self.state.gamma_settings,
                &mut self.state.boot_lifecycle.shared,
                audio_counter,
            )?;
            if self.state.options_menu.selection != previous_selection {
                self.state
                    .menu_presentation
                    .set_options_scroll(self.state.options_menu.selection);
            }
            match action {
                options_menu::Action::None => {}
                options_menu::Action::Controls => {
                    self.state.screen = Screen::Controls;
                    self.state
                        .menu_presentation
                        .reconcile_page(self.state.screen, menu_input);
                    self.state.audio.admit(
                        &mut self.state.boot_lifecycle.shared,
                        audio_counter,
                        audio::SoundRequest {
                            id: 8,
                            volume: 1.0,
                            delay: 0,
                            looping: false,
                            pitch: 1.0,
                        },
                    );
                }
                options_menu::Action::Fullscreen => {
                    self.state.fullscreen.toggle()?;
                }
                options_menu::Action::SaveAndBack => {
                    self.state.options_menu.request_save(
                        &self.state.audio,
                        &self.state.music,
                        &self.state.fullscreen,
                        &self.state.map_settings,
                        &self.state.gamma_settings,
                    )?;
                    self.state.screen = Screen::Main;
                    //59CAC0 stores the new page now; next iteration's view
                    //uses Main. No second view/widget/animation advance.
                    self.state
                        .menu_presentation
                        .reconcile_page(self.state.screen, menu_input);
                    //5B8DB0 Save precedes SetMenu59CAC0's immediate SFX8.
                    self.state.audio.admit(
                        &mut self.state.boot_lifecycle.shared,
                        audio_counter,
                        audio::SoundRequest {
                            id: 8,
                            volume: 1.0,
                            delay: 0,
                            looping: false,
                            pitch: 1.0,
                        },
                    );
                }
                options_menu::Action::UnsupportedSetting(row) => {
                    self.state.options_menu.request_unavailable(row)?;
                }
            }
        }
        if controls_callback {
            let action = if self.state.screen == Screen::Controls {
                self.state
                    .controls_menu
                    .devices(&self.state.public_actions)?
            } else {
                self.state.controls_menu.configure(
                    &self.state.public_actions,
                    &mut self.state.keyboard,
                    &mut self.state.input_configs,
                )?
            };
            match action {
                controls_menu::Action::None => {}
                controls_menu::Action::Keyboard => self.state.screen = Screen::Keyboard,
                controls_menu::Action::Devices => self.state.screen = Screen::Controls,
                controls_menu::Action::Options => self.state.screen = Screen::Options,
                controls_menu::Action::Refused => {
                    self.state.audio.admit(
                        &mut self.state.boot_lifecycle.shared,
                        audio_counter,
                        audio::SoundRequest {
                            id: 187,
                            volume: 1.0,
                            delay: 2,
                            looping: false,
                            pitch: 1.0,
                        },
                    );
                }
            }
            if matches!(
                action,
                controls_menu::Action::Keyboard
                    | controls_menu::Action::Devices
                    | controls_menu::Action::Options
            ) {
                //SetMenu59CAC0 admits8 for every actual nonzero page change.
                self.state.audio.admit(
                    &mut self.state.boot_lifecycle.shared,
                    audio_counter,
                    audio::SoundRequest {
                        id: 8,
                        volume: 1.0,
                        delay: 0,
                        looping: false,
                        pitch: 1.0,
                    },
                );
            }
            self.state
                .menu_presentation
                .set_keyboard_scroll(self.state.controls_menu.keyboard_scroll());
            self.state
                .menu_presentation
                .reconcile_page(self.state.screen, menu_input);
        }
        if phase != phase::Phase::Skipped && self.state.screen == Screen::Playing {
            //No source callback has been admitted for these global actions.
            //Their real held/triggered owners remain observable; each actual
            //attempt emits an explicit capability failure, never a fake effect.
            if !self.living_restart_admitted() && self.state.public_actions.triggered[16] {
                self.state.controls_menu.unavailable_actions.push(16);
            }
            for action in [11, 12, 17, 18] {
                if self.state.public_actions.triggered[action] {
                    self.state.controls_menu.unavailable_actions.push(action);
                }
            }
        }
        if phase != phase::Phase::Skipped && self.state.screen != Screen::Playing {
            if self.state.run_start_request.fading {
                //59B180 precedes Character59F470's fade>=1 callback.
                self.state.run_start_request.fade_alpha += 0.025_f32;
                if self.state.run_start_request.fade_alpha >= 1.0_f32
                    && !self.state.run_start_request.system_pending
                {
                    if self.state.screen == Screen::Character
                        && !self.state.run_start_request.resume_saved
                    {
                        //59F579: selected type9/Eden decrements20, then Saved
                        //valid F448 independently controls the counter21 reset.
                        if self.state.character_selection == 9 {
                            persistent_effects::add_counter(&mut self.state, 20, -1)?;
                        }
                        match self.state.saved_load_valid {
                            Some(true) => persistent_effects::reset_counter21(
                                &mut self.state,
                                &self.challenge_catalog,
                            )?,
                            Some(false) => {}
                            None => {
                                return Err(
                                    "Character confirmation Saved validity owner is unknown".into(),
                                )
                            }
                        }
                        self.flush_living_dirty_profile()?;
                        if self.wait_living_write(
                            living_restart::ResumeStage::AfterCharacterProfileFlush {
                                phase,
                                pre_counter: audio_counter,
                                bomb_triggered,
                            },
                        ) {
                            return Ok(());
                        }
                    }
                    self.complete_character_request()?;
                }
            }
            let request = &mut self.state.run_start_request;
            if request.input_pending {
                request.input_pending = false;
                request.fading = true;
                self.state.music.crossfade(95);
                //59B14B..172 passes the current menu seed to593470 only once.
                //Its zero fallback stores the raw seed, without encoding a code.
                let boot = &mut self.state.boot_lifecycle;
                if boot.menu_run_seed == 0 {
                    boot.menu_run_seed = boot.shared.next_u32().max(1);
                }
                request.loading_image_index = Some((boot.menu_run_seed % 56 + 1) as u8);
                self.state.run_admission.confirm_menu_request();
            }
        }
        self.tick_after_character_request(phase, audio_counter, bomb_triggered)
    }

    fn complete_character_request(&mut self) -> Result<(), String> {
        if self.state.run_start_request.challenge_id.is_some() {
            //Existing59D080 Challenge request remains a distinct seed owner.
            self.state.seed_text.clear();
            self.state.seed_valid = false;
            let seed = self.state.boot_lifecycle.shared.next_u32().max(1);
            self.state.boot_lifecycle.commit_character_seed(seed)?;
        }
        self.state.run_start_request.system_pending = true;
        Ok(())
    }

    fn tick_after_character_request(
        &mut self,
        phase: phase::Phase,
        audio_counter: u32,
        bomb_triggered: bool,
    ) -> Result<(), String> {
        self.state.persistent_callback_pre_counter = Some(audio_counter);
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
        if phase == phase::Phase::Update && self.state.screen == Screen::Playing {
            attack_runtime::restore_fast_positions(&mut self.state);
            laser_runtime::restore_fast_positions(&mut self.state);
            bombs::restore_shot_fast_positions(&mut self.state);
            epic_fetus::restore_fast_positions(&mut self.state)?;
            explosion_effects::restore_fast_positions(&mut self.state)?;
            critters::restore_fast_positions(&mut self.state);
        }
        //54F920's shared HUD streak precedes54F938 and every later
        //GameFull popup/transition/pause guard. Fast has no HUD update.
        if phase == phase::Phase::Update && self.state.screen == Screen::Playing {
            self.state.run_introduction.advance_full();
            if let Some(shake) = &mut self.state.game_shake {
                shake.advance_full_clock();
            }
        }
        //54FA20/54FB20 precede Pause update.5BC194 stopped that sprite;
        //HUD above still advances, while actors/GameAge remain frozen.
        if self.state.screen == Screen::Playing && self.state.run_quit.active() {
            if phase == phase::Phase::Update {
                //54F9DA/54FB88 retain priority over the outgoing fade.
                if self.state.stage_transition.active() {
                    self.advance_stage_transition(phase)?;
                } else {
                    if self.state.information_animation.active() {
                        self.state.information_animation.advance_full()?;
                    }
                    if self.state.information_animation.state != 2 {
                        self.state.run_quit.advance_full();
                    }
                }
            }
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
        // Game::Update54f9da: an active StageTransition replaces the whole
        // full Game update, and fast550760 freezes the Room as well.
        if self.state.stage_transition.active() {
            if phase == phase::Phase::Update {
                self.advance_stage_transition(phase)?;
            }
            if phase != phase::Phase::Skipped {
                self.state.music.advance();
            }
            return Ok(());
        }
        let room_before = self.state.current_room;
        let departure_y = bombs::player_world(&self.state).y;
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
        if traversal::advance_transition(
            &mut self.state,
            phase,
            self.special_room_assets.as_deref().map(Vec::as_slice),
        )? {
            if phase == phase::Phase::Update {
                self.advance_minimap_full();
            }
            if boss_cleanup {
                // State3 calls Player.Update at5dde9e before enabling controls
                // and leaving the transition. The outer actor pass stays frozen.
                self.update_player(phase, false, false)?;
            }
            if room_before != self.state.current_room {
                self.change_room_shared_draws(room_before)?;
                self.state.camera = Some(camera::Camera::new(
                    &self.state.room,
                    self.state
                        .room_transition
                        .as_ref()
                        .map(|transition| transition.target_slot),
                ));
                doors::begin_room_admission(&mut self.state);
                if !self.state.knives.is_empty() {
                    self.state.weapon_admission.record_unadmitted(
                        self.state.weapon_cache.primary,
                        "ordinaryKnifeRoomAdmissionTail",
                    );
                }
                // Transition explicitly updates Player before loading; the
                // GameChangeRoom5509f2 ->584200 ->Room.Update5d12a0 tail
                // admits another update at unchanged Game counter.
                combat::advance_player_immunity(&mut self.state);
                combat::advance_player_immunity(&mut self.state);
                if restoring {
                    //Saved4/5/6 replay/retirement needs its own full owner.
                    ordinary_placement::invalidate(
                        &mut self.state,
                        ordinary_placement::Frontier::Restore,
                    );
                    let restored_pickups = pickups::readmit_regular_records(&mut self.state)?;
                    let restored_slots = slots::readmit_saved(&mut self.state);
                    item_queries::note_room_placement(&mut self.state);
                    self.rebuild_room_actor_schedule()?;
                    self.admitted_update_footprint(phase)?;
                    //5CF58E replays persistent+B8 records after ordinary5C5860.
                    crate::explosion_effects::replay_dynamic_craters(&mut self.state)?;
                    self.state.room.complete_initialization();
                    // The native Room.Update admission tail advances the new
                    // actors once at the unchanged Game clock. Other saved
                    // families keep their existing reconstruction owners.
                    for id in restored_pickups.iter().copied() {
                        self.advance_pickup(id, phase)?;
                    }
                    for id in restored_slots {
                        slots::advance_actor(&mut self.state, phase, id)?;
                    }
                    //The restored Room.Update has the same late manager paint
                    //and actual inactive Uninit fence before entity contacts.
                    familiars::paint_background_effects(&mut self.state)?;
                    explosion_effects::retire_full(&mut self.state);
                    pickups::retire_full(&mut self.state);
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
                            self.collect_regular_pickup_at(index)?;
                        }
                    }
                    slots::player_contacts(&mut self.state)?;
                    pickups::nonplayer_contacts(&mut self.state, candidates)?;
                    pickups::full_grid_contacts(&mut self.state)?;
                } else {
                    let selectors = self
                        .state
                        .room
                        .spawns
                        .as_ref()
                        .expect("first entered descriptor has authored admission")
                        .selectors
                        .clone();
                    // Room.Init reads the current Game+CE9F4 for its immediate
                    // critter Update, as does the admitted Room.Update below.
                    let counter = self.state.native_game_frame();
                    let viewer = self
                        .state
                        .departure_position
                        .take()
                        .unwrap_or_else(|| bombs::player_world(&self.state));
                    self.admit_room_pickups(&selectors, counter, false, viewer)?;
                    item_queries::note_room_placement(&mut self.state);
                    self.admitted_update_footprint(phase)?;
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
                            actors::ActorKind::Slot => {
                                slots::advance_actor(&mut self.state, phase, actor.id)?;
                            }
                            _ => return Err("invalid authored room actor handle".into()),
                        }
                        self.count_updated_room_actor(actor, &mut counted);
                    }
                    // The same Room::Update reaches the critters Room.Init just
                    // spawned, after the selector entities, at this counter.
                    let critters: Vec<u32> = self
                        .state
                        .sprite_effects
                        .iter()
                        .filter(|effect| effect.critter.is_some() && effect.active)
                        .map(|effect| effect.id)
                        .collect();
                    for id in critters {
                        critters::advance(&mut self.state, phase, id)?;
                    }
                    self.state.room.manager_counts.finish_full(counted);
                    // The admitting manager pass also sets the bar: a fresh
                    // boss is present but not yet summed, so the fill is 0/0.
                    self.update_boss_bar();
                    familiars::paint_background_effects(&mut self.state)?;
                    blood_effects::clear_retired_links(&mut self.state);
                    pickups::retire_full(&mut self.state);
                    pickups::full_grid_contacts(&mut self.state)?;
                    chests::full_grid_contacts(&mut self.state)?;
                    npcs::post_contact_grid(&mut self.state)?;
                    bombs::full_grid_contacts(&mut self.state)?;
                    npcs::admit_room_combat(&mut self.state);
                    self.show_miniboss_streak();
                    // ChangeRoom 5dd858 then selects the room's music (5c4bf0).
                    // A first-visit treasure room (type 4, Room+1 set by
                    // Room.Init when descriptor+0x3c is clear) draws its jingle
                    // 87 + Random(4) at 5c4fff. The jingle is not played yet.
                    if self
                        .state
                        .room
                        .descriptor
                        .as_ref()
                        .is_some_and(|descriptor| descriptor.template.room_type == 4)
                    {
                        let _jingle = 87 + self.state.boot_lifecycle.shared.bounded(4);
                    }
                }
                // The admitted Room.Update also ends with 5c6980; the active
                // transition makes Game550760 report paused.
                if phase == phase::Phase::Update {
                    ambience::update(&mut self.state, true);
                }
                // ChangeRoom5dd260 restores Player+7d4=false after the
                // admitted Room.Update, before the incoming leg or Scene.
                self.state.player.controls_enabled = false;
                if !restoring {
                    traversal::begin_boss_scene(&mut self.state);
                }
                // Room.Init inserts the new room's actors into the render
                // array while Isaac still holds his departure position; the
                // admitting update does not reorder it (later full updates do).
                self.sort_render_array(Some(departure_y), false);
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
        if phase == phase::Phase::Update {
            let initial_fade_was_positive = self.state.run_admission.initial_fade > 0.0;
            if self
                .state
                .run_admission
                .blocks_full_update(&mut self.state.player)?
            {
                // Fast5507f0 does not take this guard. Music belongs to System and
                // continues while Game's full update and age are held.
                self.state.music.advance();
                return Ok(());
            }
            if initial_fade_was_positive {
                //54FC2B ->585500 restarts the existing HUD streak after this
                //Full's earlier HUD update, before GameAge and actor callbacks.
                //The positive-to-zero edge is one-shot;593660 input reset is
                //still represented by completion_callbacks_pending, not cleared.
                self.state.run_introduction = RunIntroduction::stage(true);
            }
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
            self.advance_minimap_full();
            // Source54fa82 increments before PlayerManager/Room callbacks.
            self.state.game_update_count = self
                .state
                .game_update_count
                .checked_add(1)
                .ok_or("game update counter exhausted")?;
            mt::note_update(self.state.game_update_count);
        }
        self.advance_room_runtime(phase, bomb_triggered)?;
        if phase != phase::Phase::Skipped {
            self.state.music.advance();
        }
        self.advance_camera(phase);
        Ok(())
    }
    ///The existing Room5D12A0 runtime body, shared by the normal Game
    ///dispatch and54EF4E's actual post-Continue callback. System time, Game
    ///age, music advancement and the initial fade remain caller-owned.
    fn advance_room_runtime(
        &mut self,
        phase: phase::Phase,
        bomb_triggered: bool,
    ) -> Result<(), String> {
        doors::advance(&mut self.state, phase)?;
        // Room grid pass5d1378: GridEntity_TrapDoor::Update before entities.
        stage_transition::advance_trapdoor(&mut self.state, phase)?;
        tnt::advance(&mut self.state, phase)?;
        let explosion_fence = self.state.explosions.len();
        grid_hazards::advance(&mut self.state, phase)?;
        spider_webs::advance(&mut self.state, phase)?;
        ordinary_placement::observe_room_boundary(&mut self.state);
        if phase == phase::Phase::Update {
            //5D1384 grid callbacks precede5D145A's manager454DF0 fence.
            //A bomb rock's Effect1 therefore gets its first real Full here;
            //a child born inside a Bomb actor callback remains fenced out.
            rock_blast::advance_bomb_rocks(&mut self.state)?;
            rock_blast::advance_markers(&mut self.state);
        }
        let existing = self.state.actor_scheduler.full_phase();
        let mut player_updated = false;
        if phase == phase::Phase::Update {
            self.refresh_room_boss_count();
            if let Some(counter) = &mut self.state.room.source_tear_slowdown_2494 {
                if *counter > 0 {
                    *counter -= 1;
                }
            }
            let mut counted = actors::FullCounts::default();
            for actor in existing.iter().copied() {
                // Every actual Player callback defers its full grid query
                // until the manager has completed all entity contacts.
                if actor.kind == actors::ActorKind::Player {
                    player_updated = true;
                }
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
            attack_runtime::retire_full(&mut self.state);
            laser_runtime::retire_full(&mut self.state)?;
            bombs::retire_shots_full(&mut self.state);
            epic_fetus::retire_full(&mut self.state);
            explosion_effects::retire_full(&mut self.state);
            controlled_tear::retire_full(&mut self.state);
            pickups::retire_full(&mut self.state);
            //454DF0: after fenced actors, inactive31E Uninit/registered
            //unlink and stable manager compaction precede455292->4545D0.
            //Retain the prior snapshot throughout those actor callbacks;
            //only the real rebuild below replaces it for the next phase.
            controlled_knife_homing::unlink_removed_targets(&mut self.state);
            npcs::unlink_removed(&mut self.state.room.npcs);
            self.state.room.npcs.retain(|npc| !npc.removed);
            self.state.actor_scheduler.retain(|actor| {
                actor.kind != actors::ActorKind::Npc
                    || self.state.room.npcs.iter().any(|npc| npc.id == actor.id)
            });
            controlled_knife_homing::rebuild_partition(&mut self.state);
            attack_runtime::contacts(&mut self.state)?;
            let pickup_contacts = pickups::contact_candidates(&self.state)?;
            //4549D1 completes the NPC/Bomb partition's self-pairs before
            //454A0A queries that partition against the actual Players.
            npcs::nonplayer_contacts(&mut self.state)?;
            bombs::bomb_contacts(&mut self.state)?;
            bombs::player_contacts(&mut self.state)?;
            npcs::player_contacts(&mut self.state)?;
            enemy_projectiles::player_contacts(&mut self.state)?;
            // Actual full manager contact pass uses typed ground coordinates.
            self.state.tear_partition = self
                .state
                .tears
                .iter()
                .filter(|tear| tear.motion.lifetime == projectiles::Lifetime::Alive)
                .map(|tear| tear.id)
                .collect();
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
            let partitioned_contacts = polyphemus_contact::contacts_all(&mut self.state)?;
            for (id, mut motion, radius, damage) in tears {
                if self.state.tears.iter().any(|tear| {
                    tear.id == id
                        && tear.controlled.is_none()
                        && (partitioned_contacts == polyphemus_contact::ContactPass::Handled
                            || (partitioned_contacts
                                == polyphemus_contact::ContactPass::Unadmitted
                                && tear.attack_flags.0 == 0x200))
                }) {
                    continue;
                }
                if self
                    .state
                    .tears
                    .iter()
                    .any(|tear| tear.id == id && tear.controlled.is_some())
                {
                    controlled_tear::contacts_actor(&mut self.state, id)?;
                    continue;
                }
                if npcs::tear_hit(&mut self.state, &mut motion, radius, id, damage)?.is_some() {
                    if let Some(tear) = self.state.tears.iter_mut().find(|tear| tear.id == id) {
                        motion.lifetime = projectiles::Lifetime::Dying;
                        tear.motion = motion;
                        tear.project();
                    }
                }
            }
            for index in 0..self.state.room.pickups.len() {
                if shop::player_contact(&mut self.state, index)? {
                    continue;
                }
                if self.state.room.pickups[index].variant == 100 {
                    if player_items::collect(&mut self.state, index)?
                        == player_items::CollectibleContact::Refused
                    {
                        player_items::altar_contact(&mut self.state, index)?;
                    }
                    continue;
                }
                if self.state.room.pickups[index].chest.is_some() {
                    chests::player_contact(&mut self.state, index)?;
                    continue;
                }
                self.collect_regular_pickup_at(index)?;
            }
            slots::player_contacts(&mut self.state)?;
            pickups::nonplayer_contacts(&mut self.state, pickup_contacts)?;
            if player_updated {
                // Final manager454b1d..454b2f visits the retained Player
                // first, including one killed during the contact pass.
                motion::resolve_player_grid(&mut self.state.player, &self.state.room);
            }
            pickups::full_grid_contacts(&mut self.state)?;
            chests::full_grid_contacts(&mut self.state)?;
            npcs::post_contact_grid(&mut self.state)?;
            bombs::full_grid_contacts(&mut self.state)?;
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
        self.state
            .tears
            .retain(|tear| tear.motion.lifetime != projectiles::Lifetime::Removed);
        bombs::advance_explosions(&mut self.state, phase, explosion_fence);
        bombs::retire_removed(&mut self.state);
        //Pickup contacts run after the Full manager sweep. A body removed
        //there stays listed through this presentation and the intervening
        //non-Full phase; the next actual Full pre-contact sweep retires it.
        controlled_knife_homing::unlink_removed_targets(&mut self.state);
        npcs::unlink_removed(&mut self.state.room.npcs);
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
            npcs::advance_room_clear(&mut self.state)?;
            let dx = i32::from(self.held(1)) - i32::from(self.held(0));
            let dy = i32::from(self.held(3)) - i32::from(self.held(2));
            doors::check_unlock(&mut self.state, dx, dy);
            traversal::check_departure(
                &mut self.state,
                self.special_room_assets.as_deref().map(Vec::as_slice),
            );
            // Room.Update ends with the ambience loops 5c6980 (5d1866).
            let paused =
                self.state.room_transition.is_some() || self.state.stage_transition.active();
            ambience::update(&mut self.state, paused);
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
            actors::ActorKind::Knife => self.state.knives.iter().any(|knife| knife.id == actor.id),
            actors::ActorKind::Laser => self.state.lasers.iter().any(|laser| laser.id == actor.id),
            _ => true,
        });
        if phase == phase::Phase::Update {
            // Manager Update454df0 ends with the render sort 452760 (45529a).
            self.sort_render_array(None, true);
        }
        Ok(())
    }
    /// StageTransition::Update5e38b0 and the Game callbacks it dispatches.
    fn advance_stage_transition(&mut self, phase: phase::Phase) -> Result<(), String> {
        match stage_transition::advance(&mut self.state) {
            stage_transition::Step::UpdatePlayer => {
                self.update_player(phase, false, false)?;
                if self.state.stage_transition.state == 4
                    && self.state.information_animation.active()
                {
                    // 5e3902 advances the information manager after the Player.
                    self.state.information_animation.advance_full()?;
                }
                stage_transition::after_player_update(&mut self.state);
            }
            stage_transition::Step::NextStage => {
                // SetNextStage585620 -> SetStage585600 -> 5b8340, then
                // ShowNightmareScene595580(sameStage=false).
                let next = stage_transition::next_stage(self.state.stage)?;
                self.state.stage = next;
                self.state.nightmares.select_normal_basement(next)?;
                self.state.nightmares.request(false);
            }
            stage_transition::Step::AdmitLevel => self.admit_next_level(phase)?,
            stage_transition::Step::Done => {}
        }
        Ok(())
    }

    /// StageTransition state2 completion5e398d..5e3a2c for animation0: Appear,
    /// Player Update and visibility, then Level::Init58be30 for the retained
    /// run and the first Level update584200. The no-damage/boss streak
    /// counters10eaf0/10eaf4 belong to persistent achievements, not admitted.
    fn admit_next_level(&mut self, phase: phase::Phase) -> Result<(), String> {
        player_animation::force_play(&mut self.state, player_animation::Clip::Appear);
        self.update_player(phase, false, false)?;
        self.state.player.visible = true;
        let stage = self.state.stage;
        let request = stage_transition::floor_request(&self.state, stage)?;
        // Level::Init58be30 clears the per-level owners and reinitializes the
        // descriptor grid seeds from the shared stream (587ec0).
        self.state.level_progress.begin_level();
        self.boot_level_grid = self.state.boot_lifecycle.initialize_level_grid();
        self.state.floor = floor::generate_normal_basement(request)?;
        self.state.stage_seed = request.stage_seed;
        //Neither fresh Level nor a reused manager owns empty query buckets.
        self.state.knife_query_partition = None;
        self.state.current_room = self.state.floor.starting_descriptor;
        self.state.saved_room_entry = Some(save_profiles::RoomEntry::initial(
            self.state.floor.descriptors[self.state.current_room].grid_index,
        )?);
        self.state.visited_rooms = BTreeSet::from([self.state.current_room]);
        self.state.room_display_flags = vec![0; self.state.floor.descriptors.len()];
        self.state.revealed_secret_connections.clear();
        self.state.persisted_rooms.clear();
        self.state.room_transition = None;
        self.state.room = Room::from_starting_floor(&self.state.floor);
        self.state.actor_scheduler.clear_room();
        self.state.tears.clear();
        self.state.enemy_projectiles.clear();
        self.state.fire_particles.clear();
        self.state.sprite_effects.clear();
        self.state.lasers.clear();
        self.state.bombs.clear();
        self.state.explosions.clear();
        ambience::reset_room(&mut self.state);
        // Room.Init loads current FX layers before the fresh level grid records.
        room::load_fx_rays(
            &mut self.state.room,
            &mut self.state.boot_lifecycle.shared,
            self.state.stage,
            0,
        );
        grid_records::admit_fresh(
            &mut self.state.room,
            &mut self.state.boot_lifecycle.shared,
            self.state.game_update_count,
        )?;
        item_queries::note_room_placement(&mut self.state);
        // PlayerManager 5bf220 -> Player 510550 after Room.Init: a new level
        // has no enter door (Game+8198 is -1), so 510ac8..510b0e picks one.
        // The first door in slot order is taken; each later door draws
        // Random(7) and replaces it unless that is 0. A doorless room draws
        // Random(8). TriggerNewStage below then centres the player.
        let mut door_slots: Vec<u8> = self
            .state
            .room
            .appearance
            .doors
            .iter()
            .map(|door| door.slot)
            .collect();
        door_slots.sort_unstable();
        door_slots.dedup();
        if door_slots.is_empty() {
            let _door = self.state.boot_lifecycle.shared.bounded(8);
        }
        for _ in door_slots.iter().skip(1) {
            let _replace = self.state.boot_lifecycle.shared.bounded(7) != 0;
        }
        // Player::TriggerNewStage52a260 (58c3a3): a later stage places the
        // player at the room centre and converts one Eternal heart into a
        // container through GiantBook13. Map/compass item branches remain
        // with their item owners.
        // TriggerNewStage52A3F3 clears BD0; a room change does not.
        self.state.player.golden_key = Some(false);
        let centre = motion::WorldVector {
            x: ((self.state.room.left + self.state.room.right) / 2.0) as f32,
            y: ((self.state.room.top + self.state.room.bottom) / 2.0) as f32,
        };
        self.state.player.x = f64::from(centre.x);
        self.state.player.y = f64::from(centre.y);
        if self.state.player.eternal_hearts > 0 {
            self.state.player.eternal_hearts = 0;
            self.state.information_animation.request_eternal();
            player_items::add_max_hearts(&mut self.state.player, 2);
            player_items::add_red_hearts(&mut self.state.player, 2);
        }
        if let Some(id) = music::stage_track(stage, false) {
            self.state.music.room(id);
        }
        self.save_supported_run()?;
        self.state.camera = Some(camera::Camera::new(&self.state.room, None));
        // Level::Update584200 -> Room.Update: the player entity is updated again.
        self.update_player(phase, false, false)?;
        self.refresh_minimap();
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
            x: f32::from(u8::from(self.held(5))) - f32::from(u8::from(self.held(4))),
            y: f32::from(u8::from(self.held(7))) - f32::from(u8::from(self.held(6))),
        };
        let known_constructor = self.state.camera.is_some();
        let camera = self
            .state
            .camera
            .get_or_insert_with(|| camera::Camera::new(&self.state.room, None));
        if !known_constructor {
            //A missing restored Camera is not an observed Room constructor.
            camera.shake_history_known = None;
        }
        camera.advance_source_shake(
            &mut self.state.game_shake,
            self.state.game_update_count,
            &mut self.state.boot_lifecycle.shared,
            phase,
        );
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
            actors::ActorKind::Player => self.update_player_with_grid(
                phase,
                bomb_triggered,
                true,
                phase == phase::Phase::Update,
            )?,
            actors::ActorKind::Familiar => {
                familiars::advance_actor(&mut self.state, phase, actor.id)?
            }
            actors::ActorKind::Tear => {
                if self
                    .state
                    .tears
                    .iter()
                    .any(|tear| tear.id == actor.id && tear.controlled.is_some())
                {
                    return controlled_tear::advance(&mut self.state, phase, actor.id);
                }
                // 542F70..5430A5 consumes the prior Full grid contact and
                // calls Hurt before Base447190 integrates this tear. Poof
                // and removal remain at the next Full dead-entry callback.
                let mut grid_death = false;
                if phase == phase::Phase::Update && !newborn {
                    let hit = self
                        .state
                        .tears
                        .iter()
                        .find(|tear| tear.id == actor.id)
                        .filter(|tear| {
                            tear.motion.lifetime == projectiles::Lifetime::Alive
                                && tear.grid_contact
                                && tear.source_variant.is_some_and(|variant| variant != 9)
                                && tear.attack_flags.0 & (0x8_0000 | 0x2_0000_0000) == 0
                                && tear.motion.height > -60.0
                                && tear.motion.height <= -5.0
                        })
                        .map(|tear| {
                            (
                                tear.motion.position,
                                1 + i32::from(tear.collision_damage > 10.0),
                            )
                        });
                    if let Some((position, damage)) = hit {
                        tear_grid_hurt(&mut self.state, position, damage)?;
                        grid_death = true;
                    }
                }
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
                let mut death = None;
                if let Some(tear) = self.state.tears.iter_mut().find(|tear| tear.id == actor.id) {
                    if newborn {
                        tear.motion.present_birth(phase)?;
                    } else {
                        let dying = tear.motion.lifetime == projectiles::Lifetime::Dying;
                        let lifetime = tear.motion.advance(phase, tear.grid_contact)?;
                        if lifetime == projectiles::Lifetime::Removed && dying {
                            death = Some(tear_deaths::Death {
                                position: tear.motion.position,
                                height: tear.motion.height,
                                scale: tear.scale,
                            });
                        } else if !dying && lifetime == projectiles::Lifetime::Dying {
                            tear.grid_death = grid_death;
                        }
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
                if let Some(death) = death {
                    tear_deaths::emit(&mut self.state, death)?;
                }
            }
            actors::ActorKind::Bomb => bombs::advance_actor(&mut self.state, phase, actor.id)?,
            actors::ActorKind::Pickup => {
                self.advance_pickup(actor.id, phase)?;
            }
            actors::ActorKind::Slot => slots::advance_actor(&mut self.state, phase, actor.id)?,
            actors::ActorKind::Npc => {
                controlled_tear::before_npc_update(&mut self.state, phase, actor.id);
                self.advance_npc(actor.id, phase)?;
            }
            actors::ActorKind::Projectile => {
                enemy_projectiles::advance_actor(&mut self.state, phase, actor.id)?
            }
            actors::ActorKind::Knife => {
                let primary = self.state.weapon_cache.primary.numeric();
                attack_runtime::advance_scheduled_actor(&mut self.state, phase, actor.id, primary)?;
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
            subtitle: String::new(),
            stage: false,
        };
    }

    /// EntityManager update 454df0 sums HP over boss actors that are not
    /// final-flagged (0x100000) and are past their second frame, against
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
            if npc.removed || !npc.boss || final_flag {
                continue;
            }
            any = true;
            // Only actors past their first two frames contribute HP; before
            // that the original divides 0 by 0 and the bar draws nothing.
            if npc.frame_age > 1 {
                hp += npc.hp;
                maximum += npc.max_hp;
            }
        }
        let fill = if any {
            bar.maximum = bar.maximum.max(maximum);
            hp / bar.maximum
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
        let room_type = self
            .state
            .room
            .descriptor
            .as_ref()
            .map_or(1, |descriptor| descriptor.template.room_type);
        let tear_probe = fly_effects::tear_partition(&self.state);
        let knife_probe: Vec<_> = self
            .state
            .knives
            .iter()
            .map(|knife| (knife.position, knife.collision_radius))
            .collect();
        let game_counter = self.state.native_game_frame();
        let mut births = Vec::new();
        let (before, rest) = self.state.room.npcs.split_at_mut(index);
        let (npc, after) = rest.split_first_mut().expect("indexed NPC");
        (family.advance)(
            npc,
            phase,
            &mut self.state.boot_lifecycle.shared,
            &mut entities::NpcWorld {
                // Game increments before ordinary Room callbacks. Admission
                // and synchronous child callbacks retain the current owner.
                game_counter,
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
                item_death_pickups: item_death_effects::PickupAdmission {
                    game_update_count: self.state.game_update_count,
                    next_pickup: &mut self.state.next_pickup,
                    pickups: &mut self.state.room.pickups,
                    actor_order: &mut self.state.room.authored_actor_order,
                },
                effects: familiars::NpcEffectAdmission {
                    game_update_count: self.state.game_update_count,
                    next_effect: &mut self.state.next_effect,
                    effects: &mut self.state.sprite_effects,
                    scheduler: &mut self.state.actor_scheduler,
                },
                room_type,
                peers: entities::Peers { before, after },
                tear_probe: &tear_probe,
                knife_probe: &knife_probe,
            },
            &mut births,
        )?;
        for birth in births {
            self.admit_npc_birth(index, birth)?;
        }
        Ok(())
    }
    fn admit_envy_child(
        &mut self,
        index: usize,
        position: motion::WorldVector,
        variant: u32,
        seed: u32,
    ) -> Result<(), String> {
        self.admit_npc_birth(
            index,
            entities::Birth::Npc(envy::child_selector(position, variant, seed)),
        )?;
        let id = {
            let child = self.state.room.npcs.last_mut().expect("admitted NPC birth");
            child.variant = variant;
            child.no_appear = true;
            child.id
        };
        self.advance_npc(id, phase::Phase::Update)
    }
    /// Mulligan EXPLODE 4e8200: the blast (damage 40, radius 60, source the
    /// Mulligan), the effect and burst draws, Kill's non-boss burst, then
    /// the dead branch. The post-AI dead handling removes it this update.
    fn mulligan_explode(&mut self, index: usize) -> Result<(), String> {
        let (id, kind, position) = {
            let npc = &self.state.room.npcs[index];
            (npc.id, npc.kind, npc.position)
        };
        bombs::explode(
            &mut self.state,
            position,
            60.0,
            40.0,
            combat::DamageSource {
                id,
                entity_type: kind,
            },
        )?;
        let mut births = Vec::new();
        mulligans::explode_shots(position, &mut self.state.boot_lifecycle.shared, &mut births);
        // Kill 4464c0: the non-boss burst 445c60 at once.
        self.state.audio.admit(
            &mut self.state.boot_lifecycle.shared,
            self.state.system_clock.system_counter.wrapping_sub(1),
            audio::SoundRequest {
                id: 30,
                volume: 0.8,
                delay: 2,
                looping: false,
                pitch: 1.0,
            },
        );
        let seed = self.state.boot_lifecycle.shared.next_u32();
        let npc = &mut self.state.room.npcs[index];
        npc.dead = true;
        npc.death_effect_seed = Some(seed);
        births.push(entities::Birth::Blood(seed));
        mulligans::death(npc, &mut self.state.boot_lifecycle.shared, &mut births);
        for birth in births {
            self.admit_npc_birth(index, birth)?;
        }
        self.state.room.npcs[index].removed = true;
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
            entities::Birth::Projectile(birth) => {
                let spawner_type = self.state.room.npcs[index].kind;
                self.admit_enemy_projectile(|id| enemy_projectiles::EnemyProjectile {
                    spawner_type,
                    ..enemy_projectiles::EnemyProjectile::birth_with_parameters(id, parent, birth)
                })
            }
            entities::Birth::Shot(shot) => self.admit_enemy_projectile(|id| {
                enemy_projectiles::EnemyProjectile::birth(id, parent, shot)
            }),
            entities::Birth::CreepHit => creep_effects::hurt_player(&mut self.state),
            entities::Birth::Fart {
                position,
                radius,
                source,
            } => {
                fatty::fart(&mut self.state, position, radius, source);
                Ok(())
            }
            entities::Birth::VariantProjectile(birth, variant) => {
                let spawner_type = self.state.room.npcs[index].kind;
                self.admit_enemy_projectile(|id| enemy_projectiles::EnemyProjectile {
                    variant,
                    spawner_type,
                    ..enemy_projectiles::EnemyProjectile::birth_with_parameters(id, parent, birth)
                })
            }
            entities::Birth::FlaggedProjectile(birth, variant, params) => {
                let spawner_type = self.state.room.npcs[index].kind;
                let params = params.admitted()?;
                self.admit_enemy_projectile(|id| {
                    enemy_projectiles::EnemyProjectile {
                        variant,
                        spawner_type,
                        ..enemy_projectiles::EnemyProjectile::birth_with_parameters(
                            id, parent, birth,
                        )
                    }
                    .with_flags(params)
                })
            }
            entities::Birth::Hop { selector, target } => {
                self.admit_npc_birth(index, entities::Birth::Npc(selector))?;
                let hopper = self.state.room.npcs.last_mut().expect("admitted NPC birth");
                hopper.target = target;
                hopper.state = 4;
                hopper
                    .hopper
                    .as_mut()
                    .ok_or("Mega Maw summon requires a Hopper")?
                    .counter = 5;
                plain_npc::play(hopper, "Hop", true);
                hopper.animation_frame = 5;
                hopper.animation_clock = 5.0;
                Ok(())
            }
            entities::Birth::PlayerHurt => {
                combat::try_damage_player(&mut self.state, 1, 60, 0).map(|_| ())
            }
            entities::Birth::Shake(duration) => {
                self.state
                    .game_shake
                    .get_or_insert_with(Default::default)
                    .overwrite_duration(duration);
                Ok(())
            }
            entities::Birth::Suck(center) => {
                mega_fatty::suck(&mut self.state, center);
                Ok(())
            }
            entities::Birth::UnportedBlueSpider => {
                ordinary_placement::invalidate(
                    &mut self.state,
                    ordinary_placement::Frontier::Factory {
                        entity_type: 3,
                        variant: 73,
                        subtype: 0,
                    },
                );
                Ok(())
            }
            entities::Birth::FlyExplosion {
                seed,
                position,
                color,
            } => {
                familiars::NpcEffectAdmission {
                    game_update_count: self.state.game_update_count,
                    next_effect: &mut self.state.next_effect,
                    effects: &mut self.state.sprite_effects,
                    scheduler: &mut self.state.actor_scheduler,
                }
                .admit_fly_explosion(seed, position, color.tint, color.offset)?;
                Ok(())
            }
            entities::Birth::ExplosionEffect { seed, position } => {
                explosion_effects::admit_explosion(
                    &mut self.state,
                    seed,
                    position,
                    crate::attack_color::AttackColor::WHITE,
                )
                .map(|_| ())
            }
            entities::Birth::GridHurt { position, damage } => {
                let descriptor = self
                    .state
                    .room
                    .grid
                    .clone()
                    .ok_or("grid Hurt requires a room grid")?;
                let Some(cell) = grid::Grid::new(descriptor)?.index(position) else {
                    return Ok(());
                };
                let outcome = attack_grid::callback_at_index(&mut self.state, cell, damage)?;
                outcome.record_frontier(&mut self.state, "npcGridHurt");
                Ok(())
            }
            entities::Birth::Bomb {
                variant,
                seed,
                position,
            } => bombs::admit_conversion(&mut self.state, variant, seed, position),
            entities::Birth::MulliganExplode => self.mulligan_explode(index),
            entities::Birth::Spawned(spawned) => {
                self.admit_npc_birth(index, entities::Birth::Npc(spawned.selector))?;
                let npc = self.state.room.npcs.last_mut().expect("admitted NPC birth");
                npc.velocity = spawned.velocity;
                npc.no_appear = !spawned.appear;
                npc.spawner_type = spawned.spawner_type;
                npc.boss_spawned = spawned.boss_spawned;
                if spawned.scale != 1.0 {
                    npc.sprite_scale = Some(spawned.scale);
                }
                npc.max_hp *= spawned.max_health_scale;
                npc.hp *= spawned.health_scale;
                npc.hp += spawned.health_delta;
                Ok(())
            }
            entities::Birth::EnvyChild {
                position,
                variant,
                seed,
            } => self.admit_envy_child(index, position, variant, seed),
            entities::Birth::EnvyThird {
                position,
                variant,
                seed,
                level,
            } => {
                let roll = self.state.boot_lifecycle.shared.bounded(2);
                if level < roll + 1 {
                    self.admit_envy_child(index, position, variant, seed)?;
                }
                Ok(())
            }
            entities::Birth::Child(selector) => {
                self.admit_npc_birth(index, entities::Birth::Npc(selector))?;
                let id = {
                    let child = self.state.room.npcs.last_mut().expect("admitted NPC birth");
                    child.parent = Some(parent);
                    child.id
                };
                self.state.room.npcs[index].child = Some(id);
                self.advance_npc(id, phase::Phase::Update)
            }
            entities::Birth::Parented(selector) => {
                self.admit_npc_birth(index, entities::Birth::Npc(selector))?;
                let child = self.state.room.npcs.last_mut().expect("admitted NPC birth");
                child.parent = Some(parent);
                Ok(())
            }
            entities::Birth::Attached(selector) => {
                self.admit_npc_birth(index, entities::Birth::Npc(selector))?;
                let piece = self.state.room.npcs.last_mut().expect("admitted NPC birth");
                piece.parent = Some(parent);
                piece.entity_collision_class = 0;
                piece.visible = false;
                piece.no_appear = true;
                Ok(())
            }
            entities::Birth::FlungNpc(selector, velocity) => {
                self.admit_npc_birth(index, entities::Birth::Npc(selector))?;
                let npc = self.state.room.npcs.last_mut().expect("admitted NPC birth");
                npc.velocity = velocity;
                npc.no_appear = true;
                Ok(())
            }
            entities::Birth::Npc(selector) => {
                // Retired NPCs leave the room list, so its length can name a
                // live handle; take one past every id still held anywhere.
                let id = self
                    .state
                    .room
                    .npcs
                    .iter()
                    .map(|npc| npc.id.checked_add(1))
                    .chain([self
                        .state
                        .actor_scheduler
                        .next_free_id(actors::ActorKind::Npc)])
                    .try_fold(0_u32, |next, id| id.map(|id| next.max(id)))
                    .ok_or("NPC identity exhausted")?;
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
        // Standalone transition callbacks retain their existing grid path.
        self.update_player_with_grid(phase, bomb_triggered, controls_enabled, false)
    }
    fn update_player_with_grid(
        &mut self,
        phase: phase::Phase,
        bomb_triggered: bool,
        controls_enabled: bool,
        defer_full_grid: bool,
    ) -> Result<(), String> {
        if phase == phase::Phase::Update {
            self.state.player_animation.advance_full();
            if matches!(
                self.state.player_animation.current,
                Some(
                    player_animation::Clip::UseItem
                        | player_animation::Clip::Hit
                        | player_animation::Clip::Sad
                        | player_animation::Clip::Happy
                        | player_animation::Clip::Trapdoor
                        | player_animation::Clip::Appear
                )
            ) && self.state.player_animation.is_finished(None)
            {
                // Current-main completion returns ordinary presentation; an
                // interrupted Pickup never resumes its obsolete main clock.
                // The original shows WalkDown frame0 once the stage
                // transition's Trapdoor and Appear clips end.
                self.state.player_animation.current = None;
            }
        }
        let released_shoot = !(self.held(6) || self.held(7) || self.held(4) || self.held(5));
        let bomb_callback_owned = !bomb_triggered
            || !controls_enabled
            || !self.state.player.controls_enabled
            || self.state.player.input_cooldown > 0
            || self.state.inventory.bombs == 0
            || self.state.bomb_placement_cooldown.remaining_updates > 0;
        if let Some(frontier) = ordinary_placement::player_update_frontier(
            &self.state,
            phase == phase::Phase::Update,
            !released_shoot,
            bomb_callback_owned,
        ) {
            ordinary_placement::invalidate(&mut self.state, frontier);
        }
        // Base main-sprite time advances before the Player death/shooting caller.
        player_items::advance_presentation(&mut self.state, phase);
        player_costume229::begin_full(&mut self.state, phase);
        spider_webs::begin_player_update(&mut self.state, phase)?;
        let had_game_over = self.state.player_death.game_over.is_some();
        if player_death::advance(&mut self.state, phase)? {
            if !had_game_over && self.state.player_death.game_over.is_some() {
                //558900's GameOver Init reaches594910 at558A79.55CE40
                //removes its own file then tail-calls the actual55BD80 Clear.
                self.saved_game = self.state.boot_lifecycle.clear_saved_game();
                self.invalidate_profile_run()?;
            }
            if phase == phase::Phase::Interpolate {
                // RoomFast5cd975 dispatches Player, then443d00(false).
                motion::resolve_player_grid(&mut self.state.player, &self.state.room);
            }
            return Ok(());
        }
        let controls_enabled = controls_enabled
            && self.state.player.controls_enabled
            && self.state.player.input_cooldown <= 0;
        attack_runtime::update_movement_direction(&mut self.state, controls_enabled)?;
        //52f370 positive7dc returns before its both-phase519fc0 callbacks.
        //53004c otherwise precedes later full/body work, independent of aim.
        let technology2_selected = technology2::selected(&self.state);
        let technology2_callback = technology2_selected
            && self
                .state
                .player_items
                .as_ref()
                .is_some_and(|owner| owner.active_state_d80 == 0)
            && self.state.player.shooting_lock == 0
            && !self.state.stage_shooting_guard.active();
        //The secondary callback has its own current readset and runs before
        //primary kind1/cache admission. Its retained beam does not reread C88.
        let technology2_step = if technology2_callback {
            technology2::prepare_callback(&mut self.state, phase, controls_enabled)?
        } else {
            None
        };
        let shoot_callback_admitted =
            (controls_enabled
                && self.state.player.can_shoot
                && self.state.player.shooting_lock == 0
                && !self.state.player_animation.guard
                && self.state.player_items.as_ref().is_some_and(|items| {
                    !items.pickup_animation_guard && items.active_state_d80 == 0
                })
                && !self.state.stage_shooting_guard.active()
                && self.state.player.controls_enabled
                && self.state.player.input_cooldown <= 0)
                || technology2_step.is_some();
        let mut source_weapon_clock = technology2_step.is_some();
        let mut regular_plan = None;
        let mut shooting_aim_db4 = technology2_step.as_ref().and_then(|step| step.db4_write);
        let regular_dispatch = if shoot_callback_admitted {
            match weapon_plan::resolve_current_weapon(&self.state)? {
                weapon_plan::LiveWeaponResolution::Unadmitted(channel) => {
                    //A known regular-shot cache refusal cannot write C18.
                    //Other unresolved source channels may skip a persistent
                    //controller/cleanup; current pointers never heal history.
                    if channel != "regularTearCache" {
                        self.state.weapon_admission.c18_controller_history_known = false;
                    }
                    if technology2_selected {
                        technology2::primary_frontier(&mut self.state, channel);
                    } else {
                        self.state
                            .weapon_admission
                            .record_unadmitted(self.state.weapon_cache.primary, channel);
                    }
                    false
                }
                weapon_plan::LiveWeaponResolution::Resolved(plan) => {
                    let buttons = if let Some(step) = &technology2_step {
                        step.buttons
                    } else {
                        attack_runtime::ShootButtons::read(&self.state)
                    };
                    let callback_aim =
                        if plan.controller == weapon_plan::ControllerChannel::OrdinaryKnife {
                            buttons.ordinary_knife_aim()
                        } else {
                            buttons.aim()
                        };
                    let inner_eye = inner_eye::selected(&self.state, &plan);
                    let ordinary = inner_eye
                        || technology2_selected
                        || matches!(
                            plan.controller,
                            weapon_plan::ControllerChannel::OrdinaryKnife
                                | weapon_plan::ControllerChannel::ChargedBrimstone
                                | weapon_plan::ControllerChannel::OrdinaryTechnology
                                | weapon_plan::ControllerChannel::OrdinaryFetus
                                | weapon_plan::ControllerChannel::MonstroLung
                        );
                    let controlled =
                        plan.controller == weapon_plan::ControllerChannel::ControlledKnife;
                    let controlled_tear =
                        plan.controller == weapon_plan::ControllerChannel::ControlledTear;
                    let controlled_brim =
                        plan.controller == weapon_plan::ControllerChannel::ControlledBrimstone;
                    let controlled_technology =
                        plan.controller == weapon_plan::ControllerChannel::ControlledTechnology;
                    let controlled_fetus =
                        plan.controller == weapon_plan::ControllerChannel::ControlledFetus;
                    let epic_fetus = plan.controller == weapon_plan::ControllerChannel::EpicFetus;
                    let controlled_target =
                        controlled_brim || controlled_technology || controlled_fetus || epic_fetus;
                    let aim_step = if let Some(step) = &technology2_step {
                        Some(attack_runtime::ShootingAimStep {
                            callback_aim: step.aim,
                            db4_write: step.db4_write,
                        })
                    } else if ordinary || controlled || controlled_tear || controlled_target {
                        attack_runtime::ordinary_shooting_aim(
                            callback_aim,
                            buttons.held(),
                            self.state.attack_runtime.shooting_aim_db4,
                        )?
                    } else {
                        Some(attack_runtime::ShootingAimStep {
                            callback_aim,
                            db4_write: (callback_aim.x != 0.0 || callback_aim.y != 0.0)
                                .then_some(callback_aim),
                        })
                    };
                    if let Some(mut aim_step) = aim_step {
                        // 52F6B7 normalizes r13 before the callback and DB4 store.
                        if (controlled
                            || inner_eye
                            || controlled_tear
                            || controlled_target
                            || plan.controller == weapon_plan::ControllerChannel::MonstroLung)
                            && knives::squared_length(aim_step.callback_aim) > 1.0
                        {
                            aim_step.callback_aim = knives::normalized(aim_step.callback_aim);
                            if aim_step.db4_write.is_some() {
                                aim_step.db4_write = Some(aim_step.callback_aim);
                            }
                        }
                        shooting_aim_db4 = aim_step.db4_write;
                        //52F5B8 updates DA0 before519FC0 for regular shots too.
                        //The existing ordinary group already owns this write.
                        if plan.controller == weapon_plan::ControllerChannel::RegularShots
                            && !ordinary
                        {
                            attack_runtime::ordinary_head_direction(
                                &mut self.state,
                                aim_step.callback_aim,
                            );
                        }
                        let head_frame_request = if self.state.head_timer > 8 { 2 } else { 0 };
                        if !technology2_selected
                            && (ordinary || controlled || controlled_tear || controlled_target)
                        {
                            source_weapon_clock = true;
                            if controlled {
                                attack_runtime::controlled_head_direction(
                                    &mut self.state,
                                    aim_step.callback_aim,
                                    buttons.held(),
                                )?;
                            } else if controlled_tear {
                                controlled_tear::head_direction(
                                    &mut self.state,
                                    aim_step.callback_aim,
                                    buttons.held(),
                                )?;
                            } else if controlled_target {
                                controlled_brimstone::head_direction(
                                    &mut self.state,
                                    aim_step.callback_aim,
                                    buttons.held(),
                                )?;
                            } else {
                                attack_runtime::ordinary_head_direction(
                                    &mut self.state,
                                    aim_step.callback_aim,
                                );
                            }
                            if phase == phase::Phase::Update {
                                if self.state.fire_cooldown >= 0 {
                                    self.state.fire_cooldown -= 1;
                                }
                                if self.state.head_timer >= 0 {
                                    self.state.head_timer -= 1;
                                }
                            }
                        }
                        regular_plan = plan.regular;
                        let outcome = if technology2_selected
                            && (!self.state.player.can_shoot
                                || self.state.player_animation.guard
                                || self
                                    .state
                                    .player_items
                                    .as_ref()
                                    .is_some_and(|items| items.pickup_animation_guard))
                        {
                            attack_runtime::DispatchOutcome::Handled
                        } else {
                            attack_runtime::dispatch_player(
                                &mut self.state,
                                &plan,
                                buttons,
                                phase,
                                aim_step.callback_aim,
                            )?
                        };
                        if outcome == attack_runtime::DispatchOutcome::Unadmitted
                            && plan.controller.writes_c18()
                        {
                            self.state.weapon_admission.c18_controller_history_known = false;
                        }
                        //519FC0 returns before the released idle D9C -> DA0 effect.
                        if controlled {
                            attack_runtime::controlled_idle_head(&mut self.state, buttons.held());
                        } else if controlled_tear {
                            controlled_tear::idle_head(&mut self.state, buttons.held());
                        } else if controlled_target {
                            attack_runtime::controlled_idle_head_for(
                                &mut self.state,
                                buttons.held(),
                                if controlled_fetus {
                                    "controlledFetusMovementHistory"
                                } else if epic_fetus {
                                    "epicFetusMovementHistory"
                                } else if controlled_technology {
                                    "controlledTechnologyMovementHistory"
                                } else {
                                    "controlledBrimstoneMovementHistory"
                                },
                            );
                        }
                        if plan.controller == weapon_plan::ControllerChannel::OrdinaryTechnology {
                            //530A19 released ordinary idle uses the same canonical D9C/DA0.
                            attack_runtime::ordinary_brimstone_idle_head(
                                &mut self.state,
                                buttons.held(),
                            );
                        }
                        if inner_eye
                            || matches!(
                                plan.controller,
                                weapon_plan::ControllerChannel::OrdinaryFetus
                                    | weapon_plan::ControllerChannel::MonstroLung
                            )
                        {
                            //5306E0 excludes onlyC08=2; these released callers read D9C.
                            attack_runtime::controlled_idle_head_for(
                                &mut self.state,
                                buttons.held(),
                                if inner_eye {
                                    "innerEyeMovementHistory"
                                } else if plan.controller
                                    == weapon_plan::ControllerChannel::MonstroLung
                                {
                                    "monstroLungMovementHistory"
                                } else {
                                    "drFetusMovementHistory"
                                },
                            );
                        }
                        if plan.controller == weapon_plan::ControllerChannel::ChargedBrimstone
                            || controlled_brim
                        {
                            if !controlled_brim {
                                attack_runtime::ordinary_brimstone_idle_head(
                                    &mut self.state,
                                    buttons.held(),
                                );
                            }
                            player_costume118::sync_head(&mut self.state, head_frame_request)?;
                        }
                        if plan.primary == weapon_plan::PrimaryWeaponC08::MonstrosLung {
                            player_costume229::sync_head(&mut self.state, head_frame_request)?;
                        }
                        outcome == attack_runtime::DispatchOutcome::RegularTear
                    } else {
                        if plan.controller.writes_c18() {
                            self.state.weapon_admission.c18_controller_history_known = false;
                        }
                        self.state
                            .weapon_admission
                            .record_unadmitted(plan.primary, "ordinaryShootingAimHistory");
                        false
                    }
                }
            }
        } else {
            false
        };

        if let Some(step) = &technology2_step {
            //The current idle/DB4 continuation survives primary admission failure.
            technology2::idle_head(&mut self.state, step.buttons.held());
        }

        let dx = if controls_enabled {
            i32::from(self.held(1)) - i32::from(self.held(0))
        } else {
            0
        };
        let dy = if controls_enabled {
            i32::from(self.held(3)) - i32::from(self.held(2))
        } else {
            0
        };
        combat::advance_player_immunity(&mut self.state);
        let bomb_admitted = self.state.bomb_placement_cooldown.advance(phase);
        // Shooting inherits the velocity entering this phase, before input acceleration.
        let inherited_velocity = (self.state.player.vx, self.state.player.vy);
        // Player531c8f reads the velocity after input but before this
        // update's grid collision, which the engine applies afterward.
        let footprint_velocity = motion::advance_world(
            bombs::player_world(&self.state),
            bombs::player_velocity(&self.state),
            dx,
            dy,
            self.state.player_stats.move_speed,
        )?
        .1;
        if defer_full_grid {
            // Actions, later actors and contacts see provisional full-pass
            // vectors; manager454b2f owns the subsequent grid response.
            motion::advance_without_grid(
                &mut self.state.player,
                dx,
                dy,
                self.state.player_stats.move_speed,
            )?;
        } else {
            motion::advance(
                &mut self.state.player,
                &self.state.room,
                dx,
                dy,
                self.state.player_stats.move_speed,
            );
        }
        spider_webs::finish_player_update(&mut self.state);
        if controls_enabled && bomb_triggered && bomb_admitted {
            bombs::place(&mut self.state, phase)?;
        }
        active_charges::before_advance(
            &mut self.state,
            controls_enabled,
            phase == phase::Phase::Update,
        );
        active_items::advance(&mut self.state, controls_enabled)?;
        pockets::use_pocket(&mut self.state, controls_enabled)?;
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
        } else if self.held(4) {
            Some(Direction::Left)
        } else if self.held(5) {
            Some(Direction::Right)
        } else if self.held(6) {
            Some(Direction::Up)
        } else if self.held(7) {
            Some(Direction::Down)
        } else {
            None
        };
        let shooting_locked = self.state.player.shooting_lock > 0;
        self.state.player.shooting = aim.is_some() && !shooting_locked;
        if phase == phase::Phase::Update && !shooting_locked {
            self.state.player.head_frame = if self.state.head_timer > 8 { 2 } else { 0 };
            if !source_weapon_clock && self.state.fire_cooldown >= 0 {
                self.state.fire_cooldown -= 1;
            }
            if !source_weapon_clock && self.state.head_timer >= 0 {
                self.state.head_timer -= 1;
            }
        }
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
                let plan = regular_plan
                    .ok_or("regular dispatch requires its actual immutable birth plan")?;
                let velocity = projectiles::stored_velocity(aim, inherited, plan.shot_speed)?;
                self.state.player.eye = self.state.player.eye.flipped();
                let eye_uniform = self.state.boot_lifecycle.shared.uniform();
                let origin = projectiles::eye_origin(
                    bombs::player_world(&self.state),
                    velocity,
                    self.state.player.eye,
                    eye_uniform,
                )?;
                let origin = if self
                    .state
                    .player_items
                    .as_ref()
                    .and_then(|owner| item_pipeline::ItemInputs::new(owner).ok())
                    .is_some_and(|inputs| inputs.admits_polyphemus_caller_context())
                {
                    //52068A consumes the eye draw first;520703..734 then
                    //copies current Player P for this one real169 shot.
                    bombs::player_world(&self.state)
                } else {
                    origin
                };
                let source_variant = if plan.flags == self.state.weapon_cache.flags_c98 {
                    self.state
                        .player_items
                        .as_ref()
                        .and_then(|owner| item_pipeline::ItemInputs::new(owner).ok())
                        .and_then(|inputs| {
                            inputs.neutral_regular_tear_variant(
                                self.state.weapon_admission.source_player_type,
                                self.state.weapon_cache.primary,
                                plan.flags,
                                item_queries::AppearanceQueries::new(
                                    self.state.player_items.as_ref()?,
                                    self.state.weapon_cache.primary,
                                    self.state.player.entity_source,
                                    None,
                                ),
                            )
                        })
                } else {
                    None
                };
                tear_factory::fire(&mut self.state, plan, source_variant, origin, velocity)?;
                self.state.fire_cooldown = plan.fire_delay;
                self.state.head_timer = plan.head_timer;
            }
        }
        //52F700/5306E0 follows the real kind1 callback and timer continuation.
        //With no held shooting input, BB0<0 selects the actual D9C idle head.
        if regular_dispatch {
            attack_runtime::controlled_idle_head_for(
                &mut self.state,
                aim.is_some(),
                "regularTearMovementHistory",
            );
        }
        //52F738/744 gates the postcallback DB4 store by nonzero processed input.
        //Release and canceled held input retain the actual prior DB4.
        if let Some(aim) = shooting_aim_db4 {
            self.state.attack_runtime.shooting_aim_db4 = Some(aim);
        }
        if phase == phase::Phase::Update {
            player_footprints::advance(&mut self.state, footprint_velocity)?;
        }
        //531E44 reaches deferred CF8 evaluation only on the full path,
        //before531E6F copied-item animation and the later queued grant.
        item_queries::reach_late_temporary_evaluation(&mut self.state, phase)?;
        player_items::advance_item_sprite(&mut self.state, phase);
        player_costume118::advance_full(&mut self.state, phase);
        player_costume229::advance_full(&mut self.state, phase);
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
    /// The browser's per-tick state: `json` with background stains elided.
    pub fn frame_json(&self) -> String {
        familiars::with_background_stains_elided(|| self.json())
    }
    /// The current room's background stains from index `from`, with the room
    /// identity (index and Room.Init stamp) that scopes the append-only list.
    pub fn background_stains_since_json(&self, from: usize) -> String {
        let room = &self.state.room;
        let stains = room.background_stains.get(from..).unwrap_or(&[]);
        #[derive(Serialize)]
        struct StainDelta<'a> {
            room: usize,
            initialization: Option<u32>,
            total: usize,
            stains: &'a [familiars::BackgroundStain],
        }
        serde_json::to_string(&StainDelta {
            room: self.state.current_room,
            initialization: room.initialization_game_update,
            total: room.background_stains.len(),
            stains,
        })
        .expect("background stains contain finite values")
    }
}

thread_local! {
    static GAME: RefCell<Option<Game>> = const { RefCell::new(None) };
    static REQUEST: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static BOOT_RESOURCE: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static SAVE_INPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static SNAPSHOT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static ERROR: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static PUBLIC_KEYS_JSON: Vec<u8> =
        serde_json::to_vec(input::PUBLIC_KEYS.as_slice()).expect("key codes serialize");
}
///Additive: a controlled harness's declared boot elapsed time, set right after
///reset and before any command.
#[no_mangle]
pub extern "C" fn set_boot_elapsed(milliseconds: u32, sub_millisecond_nanoseconds: u32) -> u32 {
    let result = GAME.with(|state| match state.borrow_mut().as_mut() {
        Some(game) => game.set_boot_elapsed(milliseconds, sub_millisecond_nanoseconds),
        None => Err("boot elapsed time requires a reset game".into()),
    });
    match result {
        Ok(()) => {
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
/// Additive public preference ingress, separate from authored resources.
/// Loading settings after reset would erase the real firstTitle Music events.
#[no_mangle]
pub extern "C" fn reset_with_audio_preferences(
    initial_seed: u32,
    boot_epoch_seed: u32,
    sfx_volume: f32,
    music_volume: f32,
) -> u32 {
    reset_with_audio_inputs(
        initial_seed,
        boot_epoch_seed,
        None,
        sfx_volume,
        music_volume,
    )
}
#[no_mangle]
pub extern "C" fn reset_with_boot_resources_and_audio_preferences(
    initial_seed: u32,
    boot_epoch_seed: u32,
    sfx_volume: f32,
    music_volume: f32,
) -> u32 {
    let resources = BOOT_RESOURCE.with(|buffer| {
        serde_json::from_slice::<challenges::ResourceInputs>(&buffer.borrow())
            .map_err(|error| error.to_string())
    });
    match resources {
        Ok(resources) => reset_with_audio_inputs(
            initial_seed,
            boot_epoch_seed,
            Some(resources),
            sfx_volume,
            music_volume,
        ),
        Err(message) => {
            ERROR.with(|error| *error.borrow_mut() = message.into_bytes());
            1
        }
    }
}
fn reset_with_audio_inputs(
    initial_seed: u32,
    boot_epoch_seed: u32,
    resources: Option<challenges::ResourceInputs>,
    sfx_volume: f32,
    music_volume: f32,
) -> u32 {
    let preferences = options_menu::AudioPreferences {
        sfx_volume,
        music_volume,
    };
    match Game::new_with_boot_inputs(initial_seed, boot_epoch_seed, resources, Some(preferences)) {
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
/// Display preferences are input values, never supplied window acknowledgements.
#[no_mangle]
pub extern "C" fn reset_with_fullscreen_preferences(
    initial_seed: u32,
    boot_epoch_seed: u32,
    fullscreen: u32,
    has_audio: u32,
    sfx_volume: f32,
    music_volume: f32,
) -> u32 {
    reset_with_fullscreen_inputs(
        initial_seed,
        boot_epoch_seed,
        None,
        fullscreen,
        has_audio,
        sfx_volume,
        music_volume,
    )
}
#[no_mangle]
pub extern "C" fn reset_with_boot_resources_and_fullscreen_preferences(
    initial_seed: u32,
    boot_epoch_seed: u32,
    fullscreen: u32,
    has_audio: u32,
    sfx_volume: f32,
    music_volume: f32,
) -> u32 {
    let resources = BOOT_RESOURCE.with(|buffer| {
        serde_json::from_slice::<challenges::ResourceInputs>(&buffer.borrow())
            .map_err(|error| error.to_string())
    });
    match resources {
        Ok(resources) => reset_with_fullscreen_inputs(
            initial_seed,
            boot_epoch_seed,
            Some(resources),
            fullscreen,
            has_audio,
            sfx_volume,
            music_volume,
        ),
        Err(message) => {
            ERROR.with(|error| *error.borrow_mut() = message.into_bytes());
            1
        }
    }
}
fn reset_with_fullscreen_inputs(
    initial_seed: u32,
    boot_epoch_seed: u32,
    resources: Option<challenges::ResourceInputs>,
    fullscreen: u32,
    has_audio: u32,
    sfx_volume: f32,
    music_volume: f32,
) -> u32 {
    let result = if fullscreen > 1 || has_audio > 1 {
        Err("Initial fullscreen/audio presence flags must be exactly zero or one".into())
    } else {
        let audio = (has_audio == 1).then_some(options_menu::AudioPreferences {
            sfx_volume,
            music_volume,
        });
        Game::new_with_boot_preferences(
            initial_seed,
            boot_epoch_seed,
            resources,
            audio,
            Some(fullscreen::FullscreenPreferences {
                fullscreen: fullscreen == 1,
            }),
        )
    };
    match result {
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
///Additive inputconfigs.dat boot ingress. REQUEST carries only declared
///startup inputs, never a live binding/state setter. Legacy reset ABIs remain.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InputConfigBoot {
    #[serde(default)]
    with_resources: bool,
    #[serde(default)]
    audio_preferences: Option<options_menu::AudioPreferences>,
    #[serde(default)]
    fullscreen_preferences: Option<fullscreen::FullscreenPreferences>,
    #[serde(default)]
    map_preferences: Option<map_presentation::MapPreferences>,
    input_configs: keyboard::InputConfigs,
}
#[no_mangle]
pub extern "C" fn reset_with_input_configs(initial_seed: u32, boot_epoch_seed: u32) -> u32 {
    let result = REQUEST.with(|request| {
        let boot: InputConfigBoot =
            serde_json::from_slice(&request.borrow()).map_err(|error| error.to_string())?;
        let resources = if boot.with_resources {
            Some(BOOT_RESOURCE.with(|buffer| {
                serde_json::from_slice::<challenges::ResourceInputs>(&buffer.borrow())
                    .map_err(|error| error.to_string())
            })?)
        } else {
            None
        };
        Game::new_with_input_configs_and_map_preferences(
            initial_seed,
            boot_epoch_seed,
            resources,
            boot.audio_preferences,
            boot.fullscreen_preferences,
            Some(boot.input_configs),
            boot.map_preferences,
        )
    });
    match result {
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
/// Explicit global preference inputs; no mutable runtime state is accepted.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SettingsBoot {
    #[serde(default)]
    with_resources: bool,
    #[serde(default)]
    audio_preferences: Option<options_menu::AudioPreferences>,
    #[serde(default)]
    fullscreen_preferences: Option<fullscreen::FullscreenPreferences>,
    #[serde(default)]
    input_configs: Option<keyboard::InputConfigs>,
    #[serde(default)]
    map_preferences: Option<map_presentation::MapPreferences>,
    gamma_preferences: display_settings::GammaPreferences,
}
#[no_mangle]
pub extern "C" fn reset_with_settings_preferences(initial_seed: u32, boot_epoch_seed: u32) -> u32 {
    let result = REQUEST.with(|request| {
        let boot: SettingsBoot =
            serde_json::from_slice(&request.borrow()).map_err(|error| error.to_string())?;
        let resources = if boot.with_resources {
            Some(BOOT_RESOURCE.with(|buffer| {
                serde_json::from_slice::<challenges::ResourceInputs>(&buffer.borrow())
                    .map_err(|error| error.to_string())
            })?)
        } else {
            None
        };
        Game::new_with_settings_preferences(
            initial_seed,
            boot_epoch_seed,
            resources,
            boot.audio_preferences,
            boot.fullscreen_preferences,
            boot.input_configs,
            boot.map_preferences,
            Some(boot.gamma_preferences),
        )
    });
    match result {
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
/// All preference values are declared inputs; no platform acknowledgements.
#[no_mangle]
pub extern "C" fn reset_with_map_preferences(
    initial_seed: u32,
    boot_epoch_seed: u32,
    fullscreen: u32,
    has_audio: u32,
    sfx_volume: f32,
    music_volume: f32,
    map_opacity: f32,
) -> u32 {
    reset_with_map_inputs(
        initial_seed,
        boot_epoch_seed,
        None,
        fullscreen,
        has_audio,
        sfx_volume,
        music_volume,
        map_opacity,
    )
}
#[no_mangle]
pub extern "C" fn reset_with_boot_resources_and_map_preferences(
    initial_seed: u32,
    boot_epoch_seed: u32,
    fullscreen: u32,
    has_audio: u32,
    sfx_volume: f32,
    music_volume: f32,
    map_opacity: f32,
) -> u32 {
    let resources = BOOT_RESOURCE.with(|buffer| {
        serde_json::from_slice::<challenges::ResourceInputs>(&buffer.borrow())
            .map_err(|error| error.to_string())
    });
    match resources {
        Ok(resources) => reset_with_map_inputs(
            initial_seed,
            boot_epoch_seed,
            Some(resources),
            fullscreen,
            has_audio,
            sfx_volume,
            music_volume,
            map_opacity,
        ),
        Err(message) => {
            ERROR.with(|error| *error.borrow_mut() = message.into_bytes());
            1
        }
    }
}
#[expect(
    clippy::too_many_arguments,
    reason = "forwards the scalar wasm ABI arguments of the exported reset_with_*map_preferences entry points, which cannot take a struct"
)]
fn reset_with_map_inputs(
    initial_seed: u32,
    boot_epoch_seed: u32,
    resources: Option<challenges::ResourceInputs>,
    fullscreen: u32,
    has_audio: u32,
    sfx_volume: f32,
    music_volume: f32,
    map_opacity: f32,
) -> u32 {
    // The original additive map ABI declares its fullscreen value explicitly.
    reset_with_map_input_presence(
        initial_seed,
        boot_epoch_seed,
        resources,
        1,
        fullscreen,
        has_audio,
        sfx_volume,
        music_volume,
        map_opacity,
    )
}
/// A map preference does not declare another owner's startup request.
#[no_mangle]
pub extern "C" fn reset_with_map_preferences_v2(
    initial_seed: u32,
    boot_epoch_seed: u32,
    has_fullscreen: u32,
    fullscreen: u32,
    has_audio: u32,
    sfx_volume: f32,
    music_volume: f32,
    map_opacity: f32,
) -> u32 {
    reset_with_map_input_presence(
        initial_seed,
        boot_epoch_seed,
        None,
        has_fullscreen,
        fullscreen,
        has_audio,
        sfx_volume,
        music_volume,
        map_opacity,
    )
}
#[no_mangle]
pub extern "C" fn reset_with_boot_resources_and_map_preferences_v2(
    initial_seed: u32,
    boot_epoch_seed: u32,
    has_fullscreen: u32,
    fullscreen: u32,
    has_audio: u32,
    sfx_volume: f32,
    music_volume: f32,
    map_opacity: f32,
) -> u32 {
    let resources = BOOT_RESOURCE.with(|buffer| {
        serde_json::from_slice::<challenges::ResourceInputs>(&buffer.borrow())
            .map_err(|error| error.to_string())
    });
    match resources {
        Ok(resources) => reset_with_map_input_presence(
            initial_seed,
            boot_epoch_seed,
            Some(resources),
            has_fullscreen,
            fullscreen,
            has_audio,
            sfx_volume,
            music_volume,
            map_opacity,
        ),
        Err(message) => {
            ERROR.with(|error| *error.borrow_mut() = message.into_bytes());
            1
        }
    }
}
#[expect(
    clippy::too_many_arguments,
    reason = "forwards the scalar wasm ABI arguments of the exported reset_with_*map_preferences_v2 entry points, which cannot take a struct"
)]
fn reset_with_map_input_presence(
    initial_seed: u32,
    boot_epoch_seed: u32,
    resources: Option<challenges::ResourceInputs>,
    has_fullscreen: u32,
    fullscreen: u32,
    has_audio: u32,
    sfx_volume: f32,
    music_volume: f32,
    map_opacity: f32,
) -> u32 {
    let result = if has_fullscreen > 1 || fullscreen > 1 || has_audio > 1 {
        Err("Initial fullscreen/audio presence flags must be exactly zero or one".into())
    } else {
        Game::new_with_map_preferences(
            initial_seed,
            boot_epoch_seed,
            resources,
            (has_audio == 1).then_some(options_menu::AudioPreferences {
                sfx_volume,
                music_volume,
            }),
            (has_fullscreen == 1).then_some(fullscreen::FullscreenPreferences {
                fullscreen: fullscreen == 1,
            }),
            Some(map_presentation::MapPreferences {
                opacity: map_opacity,
            }),
        )
    };
    match result {
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
///Existing ABI2 resource reset reads the64KiB REQUEST filled by request_ptr.
#[no_mangle]
pub extern "C" fn reset_with_resources(initial_seed: u32, boot_epoch_seed: u32) -> u32 {
    REQUEST.with(|request| reset_with_resource_buffer(initial_seed, boot_epoch_seed, request))
}
///Additive boot-resource reset pairs only with boot_resource_ptr's512KiB buffer.
#[no_mangle]
pub extern "C" fn reset_with_boot_resources(initial_seed: u32, boot_epoch_seed: u32) -> u32 {
    BOOT_RESOURCE
        .with(|resource| reset_with_resource_buffer(initial_seed, boot_epoch_seed, resource))
}
fn reset_with_resource_buffer(
    initial_seed: u32,
    boot_epoch_seed: u32,
    buffer: &RefCell<Vec<u8>>,
) -> u32 {
    let resources = {
        let input = buffer.borrow();
        serde_json::from_slice::<challenges::ResourceInputs>(&input)
            .map_err(|error| error.to_string())
    };
    let result = resources
        .and_then(|resources| Game::new_with_resources(initial_seed, boot_epoch_seed, resources));
    match result {
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

///Separate save transport; public command and authored-resource bounds remain
///unchanged. Import is accepted only by the fresh Title profile owner.
#[no_mangle]
pub extern "C" fn save_input_ptr(length: usize) -> *mut u8 {
    if length > save_profiles::MAX_BYTES {
        return std::ptr::null_mut();
    }
    SAVE_INPUT.with(|input| {
        let mut bytes = input.borrow_mut();
        bytes.resize(length, 0);
        bytes.as_mut_ptr()
    })
}

#[no_mangle]
pub extern "C" fn load_save_data() -> u32 {
    let result = SAVE_INPUT.with(|input| {
        GAME.with(|game| {
            game.borrow_mut()
                .as_mut()
                .ok_or("engine has not been reset".to_owned())?
                .import_save_data(&input.borrow())
        })
    });
    match result {
        Ok(()) => {
            ERROR.with(|error| error.borrow_mut().clear());
            0
        }
        Err(message) => {
            ERROR.with(|error| *error.borrow_mut() = message.into_bytes());
            1
        }
    }
}

/// Read the actual quarantine owner without changing State or the last error.
#[no_mangle]
pub extern "C" fn rejected_profile_document() -> u32 {
    GAME.with(|game| {
        game.borrow()
            .as_ref()
            .map_or(0, |game| u32::from(game.rejected_profile_document()))
    })
}

#[no_mangle]
pub extern "C" fn enable_saved_game_io(protocol: u32) -> u32 {
    let result = GAME.with(|game| {
        game.borrow_mut()
            .as_mut()
            .ok_or("engine has not reset".to_string())?
            .enable_save_io(protocol)
    });
    match result {
        Ok(()) => {
            ERROR.with(|e| e.borrow_mut().clear());
            0
        }
        Err(message) => {
            ERROR.with(|e| *e.borrow_mut() = message.into_bytes());
            1
        }
    }
}

#[no_mangle]
pub extern "C" fn complete_saved_game_write(low: u32, high: u32, succeeded: u32) -> u32 {
    let result = GAME.with(|game| {
        if succeeded > 1 {
            return Err("invalid actual write result".to_string());
        }
        let mut game = game.borrow_mut();
        let game = game.as_mut().ok_or("engine has not reset".to_string())?;
        game.complete_save_write(u64::from(low) | (u64::from(high) << 32), succeeded == 1)?;
        Ok(())
    });
    match result {
        Ok(()) => {
            ERROR.with(|e| e.borrow_mut().clear());
            0
        }
        Err(message) => {
            ERROR.with(|e| *e.borrow_mut() = message.into_bytes());
            1
        }
    }
}

#[no_mangle]
pub extern "C" fn complete_save_menu_readmission(
    revision_low: u32,
    revision_high: u32,
    loaded: u32,
) -> u32 {
    let revision = u64::from(revision_low) | (u64::from(revision_high) << 32);
    let result = if loaded > 1 {
        Err("invalid menu IO outcome".to_owned())
    } else {
        SAVE_INPUT.with(|input| {
            let bytes = input.borrow();
            GAME.with(|game| {
                game.borrow_mut()
                    .as_mut()
                    .ok_or("engine has not been reset".to_owned())?
                    .complete_menu_readmission(
                        revision,
                        if loaded == 1 {
                            Some(bytes.as_slice())
                        } else {
                            None
                        },
                    )
            })
        })
    };
    match result {
        Ok(true) => {
            ERROR.with(|e| e.borrow_mut().clear());
            0
        }
        Ok(false) => {
            ERROR.with(|e| *e.borrow_mut() = b"actual own saved document was rejected".to_vec());
            2
        }
        Err(message) => {
            ERROR.with(|e| *e.borrow_mut() = message.into_bytes());
            1
        }
    }
}

#[no_mangle]
pub extern "C" fn save_data_ptr() -> *const u8 {
    let result = GAME.with(|game| {
        game.borrow()
            .as_ref()
            .ok_or("engine has not been reset".to_owned())?
            .export_save_data()
    });
    match result {
        Ok(bytes) => {
            ERROR.with(|error| error.borrow_mut().clear());
            SNAPSHOT.with(|output| {
                *output.borrow_mut() = bytes;
                output.borrow().as_ptr()
            })
        }
        Err(message) => {
            ERROR.with(|error| *error.borrow_mut() = message.into_bytes());
            SNAPSHOT.with(|output| output.borrow_mut().clear());
            std::ptr::null()
        }
    }
}
///Separate bounded buffer for complete authored resources. Ordinary Command
///requests retain their64KiB cap; resource bytes only enter a fresh Game reset.
#[no_mangle]
pub extern "C" fn boot_resource_ptr(length: usize) -> *mut u8 {
    if length > MAX_BOOT_RESOURCE {
        return std::ptr::null_mut();
    }
    BOOT_RESOURCE.with(|resource| {
        let mut bytes = resource.borrow_mut();
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
fn store_snapshot(text: impl FnOnce(&Game) -> String) -> *const u8 {
    SNAPSHOT.with(|snapshot| {
        *snapshot.borrow_mut() = GAME.with(|game| {
            game.borrow()
                .as_ref()
                .map(text)
                .unwrap_or_else(|| "null".into())
                .into_bytes()
        });
        snapshot.borrow().as_ptr()
    })
}
/// Per-tick browser state without background stains; length via `snapshot_len`.
#[no_mangle]
pub extern "C" fn frame_snapshot_ptr() -> *const u8 {
    store_snapshot(Game::frame_json)
}
/// Current-room background stains from `from` onward; length via `snapshot_len`.
#[no_mangle]
pub extern "C" fn background_stains_ptr(from: u32) -> *const u8 {
    store_snapshot(|game| game.background_stains_since_json(from as usize))
}
#[no_mangle]
pub extern "C" fn error_ptr() -> *const u8 {
    ERROR.with(|error| error.borrow().as_ptr())
}
#[no_mangle]
pub extern "C" fn error_len() -> usize {
    ERROR.with(|error| error.borrow().len())
}
/// Diagnostic shared-MT draw log for the e2e `--mt-draws` comparator.
/// Nonzero enables (and clears) it; zero disables it. Off by default.
#[no_mangle]
pub extern "C" fn mt_draws_enable(enabled: u32) {
    mt::set_draw_log(enabled != 0);
}
/// JSON array of the draws logged since the previous call; empty when off.
#[no_mangle]
pub extern "C" fn mt_draws_ptr() -> *const u8 {
    SNAPSHOT.with(|snapshot| {
        *snapshot.borrow_mut() =
            serde_json::to_vec(&mt::take_draws()).expect("draw log serializes");
        snapshot.borrow().as_ptr()
    })
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

/// 5430a5: the alive callback marks a grid death and immediately hurts the
/// cell under its pre-integration position or the four adjacent probes.
/// Poof and removal belong to the next Full callback's dead-entry branch.
fn tear_grid_hurt(
    state: &mut State,
    position: motion::WorldVector,
    damage: i32,
) -> Result<(), String> {
    let Some(descriptor) = state.room.grid.clone() else {
        return Ok(());
    };
    let grid = grid::Grid::new(descriptor)?;
    for (x, y) in [
        (0.0, 0.0),
        (10.0, 0.0),
        (0.0, 10.0),
        (-10.0, 0.0),
        (0.0, -10.0),
    ] {
        let Some(cell) = grid.index(position + motion::WorldVector { x, y }) else {
            continue;
        };
        let outcome = attack_grid::callback_at_index(state, cell, damage)?;
        outcome.record_frontier(state, "tearGridHurt");
        if outcome.taken() == Some(true) {
            break;
        }
    }
    Ok(())
}
