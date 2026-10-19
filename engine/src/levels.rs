//! Level framework: every floor is data, not a bespoke code path.
//!
//! A floor is a node of the stage progression (`STAGES`, keyed by Level
//! stage) plus the stages.xml chapter its stage resolves to (`CHAPTERS`,
//! keyed by stages.xml id, which is also the room set and stage backdrop id).
//! Generation, room loading, music, backdrop, fx overlay, boss selection and
//! the trapdoor all read these tables, so importing a floor means filling in
//! its chapter's fields and adding its node; see docs/levels.md.
//!
//! A `None` field is a port gap, not a claim about the original: the consumer
//! keeps refusing (or keeps its existing Basement fallback) until the field's
//! evidence is imported.
use crate::room_assets::{self, RoomAsset};

/// One stages.xml record and the resources the original keys by its id.
pub struct Chapter {
    /// stages.xml id: the authored room set and the stage backdrop id.
    pub id: u32,
    pub name: &'static str,
    /// The decoded authored STB room set.
    pub rooms: Option<fn() -> Vec<RoomAsset>>,
    /// The room set's shape0 rooms admit the ambient grid producer (5cf45b).
    pub shape0_ambient: bool,
    /// backdrops.xml id that room types 0/1/4/5/6 take (5c5b30 via 5d49b0).
    pub backdrop: Option<u32>,
    /// fxlayers.xml 1x1 overlay group for this chapter.
    pub overlay: Option<Overlay>,
    /// music.xml track the Level stage plays.
    pub music: Option<u32>,
    /// 584240 boss variant draw chain for a fresh profile.
    pub bosses: Option<&'static [BossStep]>,
    /// pre_render_walls 424440's boss-room blood scatter and bloody wall row.
    pub boss_blood_backdrop: bool,
}

/// fxlayers.xml overlay folder and the RandomFloat offset range (min, max).
#[derive(Clone, Copy)]
pub struct Overlay {
    pub folder: &'static str,
    pub low: f32,
    pub high: f32,
}

/// One step of 584240's boss variant chain, in draw order. Odd floors choose
/// and save a family flag; even floors read it back instead of drawing.
#[derive(Clone, Copy)]
pub enum BossStep {
    /// Consume one primary word (local RNG creation).
    Next,
    /// Consume RandomInt(bound) for a pool that a fresh profile never selects.
    Draw(u32),
    /// With `chance`, RandomInt(chance) == 0 selects the family (None always
    /// selects). `when_set` is chosen when odd == the saved flag.
    Family {
        chance: Option<u32>,
        flag: u64,
        when_set: u32,
        otherwise: u32,
    },
    /// RandomInt(chance) == 0 selects `boss`, on odd floors only if `odd_only`.
    Single {
        chance: u32,
        odd_only: bool,
        boss: u32,
    },
}

/// A Level stage's node in the progression state machine.
pub struct StageNode {
    pub stage: u32,
    /// Level::ShowName585500 streak.
    pub name: &'static str,
    /// SetNextStage585620's target when the boss trapdoor is taken. The edge
    /// is admitted only when that target is itself a node.
    pub trapdoor: Option<u32>,
    /// The generated floor's recorded contract.
    pub contract: &'static str,
}

use BossStep::{Draw, Family, Next, Single};

/// Basement 584240 chain. Families: Dingle 5020 / Gurglings 5140, Monstro
/// 1010 / Larry Jr. 1020; Duke of Flies 2010 and 2050 on odd floors only.
const BASEMENT_BOSSES: &[BossStep] = &[
    Next,
    Family {
        chance: Some(4),
        flag: 0x100000,
        when_set: 5020,
        otherwise: 5140,
    },
    Draw(2),
    Draw(5),
    Draw(2),
    Draw(3),
    // 584488 R10: The Fallen needs a visited devil room (flag 0x40).
    Draw(10),
    Draw(3),
    Draw(4),
    Single {
        chance: 5,
        odd_only: true,
        boss: 2010,
    },
    Draw(2),
    Single {
        chance: 4,
        odd_only: true,
        boss: 2050,
    },
    Draw(2),
    Draw(2),
    Draw(2),
    Family {
        chance: None,
        flag: 16,
        when_set: 1010,
        otherwise: 1020,
    },
];

/// Caves (0x675ec8) shares the Basement draw chain, reuses its family flags
/// and adds Double Trouble's R10 at 5846d8 (a 2x1/1x2 room in 3700..3749).
/// Families: Mega Fatty 5050 / Mega Maw 5030, Gurdy 1040 / Chub 1030; Peep
/// 2020 and Gurdy Jr. 3280 on odd floors only.
const CAVES_BOSSES: &[BossStep] = &[
    Next,
    Family {
        chance: Some(4),
        flag: 0x100000,
        when_set: 5050,
        otherwise: 5030,
    },
    Draw(2),
    Draw(5),
    Single {
        chance: 10,
        odd_only: false,
        boss: 3700,
    },
    Draw(2),
    Draw(3),
    Draw(10),
    Draw(3),
    Draw(4),
    Single {
        chance: 5,
        odd_only: true,
        boss: 2020,
    },
    Draw(2),
    Single {
        chance: 4,
        odd_only: true,
        boss: 3280,
    },
    Draw(2),
    Draw(2),
    Draw(2),
    Family {
        chance: None,
        flag: 16,
        when_set: 1040,
        otherwise: 1030,
    },
];

pub const BASEMENT_OVERLAY: Overlay = Overlay {
    folder: "basement",
    low: 0.0,
    high: 100.0,
};

/// Chapters without any imported field are omitted.
pub const CHAPTERS: &[Chapter] = &[
    Chapter {
        id: 0,
        name: "Special Rooms",
        rooms: Some(room_assets::special),
        shape0_ambient: true,
        backdrop: None,
        overlay: None,
        music: None,
        bosses: None,
        boss_blood_backdrop: false,
    },
    Chapter {
        id: 1,
        name: "Basement",
        rooms: Some(room_assets::basement),
        shape0_ambient: true,
        backdrop: Some(1),
        overlay: Some(BASEMENT_OVERLAY),
        music: Some(1),
        bosses: Some(BASEMENT_BOSSES),
        boss_blood_backdrop: true,
    },
    Chapter {
        id: 2,
        name: "Cellar",
        rooms: None,
        shape0_ambient: false,
        backdrop: None,
        overlay: None,
        music: Some(4),
        bosses: None,
        boss_blood_backdrop: false,
    },
    Chapter {
        id: 3,
        name: "Caves",
        rooms: Some(room_assets::caves),
        shape0_ambient: true,
        backdrop: Some(3),
        overlay: Some(Overlay {
            folder: "caves",
            low: -10.0,
            high: 10.0,
        }),
        music: Some(2),
        bosses: Some(CAVES_BOSSES),
        boss_blood_backdrop: false,
    },
    Chapter {
        id: 4,
        name: "Catacombs",
        rooms: None,
        shape0_ambient: false,
        backdrop: None,
        overlay: None,
        music: Some(5),
        bosses: None,
        boss_blood_backdrop: false,
    },
    Chapter {
        id: 5,
        name: "Depths",
        rooms: Some(room_assets::depths),
        shape0_ambient: false,
        backdrop: None,
        overlay: None,
        music: Some(3),
        bosses: None,
        boss_blood_backdrop: false,
    },
    Chapter {
        id: 6,
        name: "Necropolis",
        rooms: None,
        shape0_ambient: false,
        backdrop: None,
        overlay: None,
        music: Some(6),
        bosses: None,
        boss_blood_backdrop: false,
    },
    Chapter {
        id: 7,
        name: "Womb",
        rooms: None,
        shape0_ambient: false,
        backdrop: None,
        overlay: None,
        music: Some(7),
        bosses: None,
        boss_blood_backdrop: false,
    },
    Chapter {
        id: 8,
        name: "Utero",
        rooms: None,
        shape0_ambient: false,
        backdrop: None,
        overlay: None,
        music: Some(7),
        bosses: None,
        boss_blood_backdrop: false,
    },
    Chapter {
        id: 9,
        name: "Sheol",
        rooms: None,
        shape0_ambient: false,
        backdrop: None,
        overlay: None,
        music: Some(11),
        bosses: None,
        boss_blood_backdrop: false,
    },
    Chapter {
        id: 10,
        name: "Cathedral",
        rooms: None,
        shape0_ambient: false,
        backdrop: None,
        overlay: None,
        music: Some(10),
        bosses: None,
        boss_blood_backdrop: false,
    },
    Chapter {
        id: 11,
        name: "Dark Room",
        rooms: None,
        shape0_ambient: false,
        backdrop: None,
        overlay: None,
        music: Some(12),
        bosses: None,
        boss_blood_backdrop: false,
    },
    Chapter {
        id: 12,
        name: "Chest",
        rooms: None,
        shape0_ambient: false,
        backdrop: None,
        overlay: None,
        music: Some(13),
        bosses: None,
        boss_blood_backdrop: false,
    },
];

/// The admitted floors. A stage absent here has no generator: new runs and
/// trapdoors never reach it.
pub const STAGES: &[StageNode] = &[
    StageNode {
        stage: 1,
        name: "BASEMENT I",
        trapdoor: Some(2),
        contract: crate::floor::CONTRACT,
    },
    StageNode {
        stage: 2,
        name: "BASEMENT II",
        trapdoor: Some(3),
        contract: "continuing-profile-normal-isaac-basement-II-without-labyrinth-or-equipped-modifiers;source-growth-fitting-secret-primary-routing-retry-continuity",
    },
    StageNode {
        stage: 3,
        name: "CAVES I",
        trapdoor: Some(4),
        contract: "continuing-profile-normal-isaac-caves-I-without-labyrinth-curse-devil-room-or-equipped-modifiers;source-growth-fitting-secret-primary-routing-retry-continuity",
    },
    StageNode {
        stage: 4,
        name: "CAVES II",
        trapdoor: Some(5),
        contract: "continuing-profile-normal-isaac-caves-II-without-curse-devil-room-or-equipped-modifiers;source-growth-fitting-secret-primary-routing-retry-continuity",
    },
];

/// 5d49b0: an odd stage keeps its number, an even one drops by one, and the
/// alternate stage type adds one. The port names no chapter for stage 10.
pub fn chapter_id(stage: u32, alternate: bool) -> Option<u32> {
    if !matches!(stage, 1..=9 | 11) {
        return None;
    }
    Some(stage - u32::from(stage.is_multiple_of(2)) + u32::from(alternate))
}

pub fn chapter(id: u32) -> Option<&'static Chapter> {
    CHAPTERS.iter().find(|chapter| chapter.id == id)
}

/// The ordinary (stage type 0) chapter of a Level stage.
pub fn stage_chapter(stage: u32) -> Option<&'static Chapter> {
    chapter_id(stage, false).and_then(chapter)
}

pub fn node(stage: u32) -> Option<&'static StageNode> {
    STAGES.iter().find(|node| node.stage == stage)
}

/// The admitted trapdoor target of `stage`.
pub fn trapdoor(stage: u32) -> Option<u32> {
    node(stage)
        .and_then(|node| node.trapdoor)
        .filter(|&next| node(next).is_some())
}
