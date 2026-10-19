//! Source-derived normal Basement starting-room backdrop and door routing.
//!
//! The original uses decoration xorshift index 35, a repeated/shuffled floor
//! catalogue, and conditional wall-variant draws. Independent original GL
//! cache submissions verify the complete geometry and patch sequence when the
//! room decoration seed is supplied. The level generator that derives that
//! seed from the run follows the fresh-profile floor generator. FX stream
//! assignments remain provisional.
use crate::effects::{Rect, Viewport};
use crate::{content, motion, Player};
use serde::{Deserialize, Serialize};

pub const ROOM_TILE_SIZE: f64 = 26.0;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Room {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
    pub appearance: RoomAppearance,
    pub descriptor: Option<crate::floor::Descriptor>,
    /// Room+159c is stamped by Room.Init before grid and entity factories.
    #[serde(default)]
    pub initialization_game_update: Option<u32>,
    /// Room+0 becomes true after the admitted factory or restore tail.
    #[serde(default)]
    pub loaded: bool,
    /// Actual Room+2494. Missing restore or unclosed writer history is unknown.
    #[serde(default)]
    pub source_tear_slowdown_2494: Option<i32>,
    pub pickups: Vec<crate::pickups::Pickup>,
    #[serde(default)]
    pub saved_regular_pickups: Vec<crate::pickups::SavedRegularPickup>,
    /// Descriptor+B8 records survive crater paint/Remove and replay on reentry.
    /// Legacy or missing restore history remains unknown until its producer.
    #[serde(default)]
    pub dynamic_crater_records: Option<Vec<crate::explosion_effects::DynamicCraterRecord>>,
    pub npcs: Vec<crate::npcs::Npc>,
    /// Entity_Slot machines and beggars admitted from the authored layout.
    #[serde(default)]
    pub slots: Vec<crate::slots::Slot>,
    /// Append-only within a room. The per-frame WASM snapshot elides them
    /// (see `frame_snapshot_ptr`); the browser reads new ones incrementally.
    #[serde(
        default,
        serialize_with = "crate::familiars::serialize_background_stains"
    )]
    pub background_stains: Vec<crate::familiars::BackgroundStain>,
    pub authored_actor_order: Vec<crate::actors::ActorRef>,
    #[serde(default)]
    pub manager_counts: crate::actors::ManagerCounts,
    ///Source ordinary4/5/6 membership completeness, not a current-roster test.
    ///Legacy restores cannot inherit the fresh Room reset capability.
    #[serde(default)]
    pub ordinary_placement: Option<crate::ordinary_placement::Coverage>,
    pub boss_count: u32,
    pub boss_presentation_index: u32,
    pub combat_clear: bool,
    pub combat_clear_countdown: u32,
    pub combat_shut_door_slots: Vec<u8>,
    /// Game update on which combat played the doors' Close animation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combat_shut_update: Option<u32>,
    /// Frame of the Close clip that cracked bedroom doors (variant3) play
    /// from every Door::Init (563b70) until its last frame.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cracked_close_frame: Option<u32>,
    pub spawns: Option<crate::room_spawns::RoomSpawns>,
    pub grid: Option<crate::grid::GridDescriptor>,
    /// The actual signed Room+e98 owner; absent until its source Init is admitted.
    #[serde(default)]
    pub path_costs: Option<crate::room_costs::RoomCosts>,
    /// Rock+348 countdowns of type5 bomb rocks a blast moved to state3.
    #[serde(default)]
    pub bomb_rock_fuses: Vec<crate::rock_blast::BombRockFuse>,
    /// Grid17 boss-room exit spawned by the clear award; its grid cell holds
    /// the identity, this owns the TrapDoor state and sprite clock.
    #[serde(default)]
    pub trapdoor: Option<crate::stage_transition::Trapdoor>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoomAppearance {
    pub backdrop: String,
    /// Room+1ae0, the backdrops.xml id selected by 5c5b30.
    #[serde(default = "basement_backdrop_id")]
    pub backdrop_id: u32,
    pub columns: u8,
    pub rows: u8,
    pub tile_size: f64,
    pub origin: RoomOrigin,
    pub backdrop_patch_size: Viewport,
    pub backdrop_patch_columns: u8,
    pub floor_quads: Vec<RoomQuad>,
    pub wall_quads: Vec<RoomQuad>,
    pub wall_variant_catalog: Vec<WallVariant>,
    pub wall_generation_evidence: String,
    pub tutorial: bool,
    pub doors: Vec<RoomDoor>,
    pub fx: RoomFx,
    pub template: String,
    pub generation: String,
    /// The selected fxRays group (FX+0x38) with its rays' load phases.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fx_rays: Option<FxRayGroup>,
}

/// One authored fxlayers.xml rayGroup (0x40-byte group at FX+0x20).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FxRayGroup {
    /// Index among the stage-matched groups, as stored at FX+0x38.
    pub index: u32,
    pub x_min: f32,
    pub x_max: f32,
    pub top_parallax: f32,
    pub bottom_parallax: f32,
    pub ray_length: f32,
    pub ray_spacing: f32,
    pub perspective: f32,
    pub rays: Vec<FxRay>,
}

/// One 0x20-byte ray: opacity +0, animationSpeed +4, phase +8, texture +0x10.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FxRay {
    pub opacity: f32,
    pub animation_speed: f32,
    /// Load phase uniform()*pi; render 547890 adds animationSpeed per call.
    pub phase: f32,
    pub texture: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoomOrigin {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoomQuad {
    pub source_patch_index: u8,
    /// Source coordinates within one authored 234 × 156 atlas patch.
    pub source_rect: Rect,
    /// Destination coordinates relative to the room's origin.
    pub dest_rect: Rect,
    pub flip_x: bool,
    pub flip_y: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WallVariant {
    pub source_patch_index: u8,
    pub name: String,
    pub weight: u32,
    pub eligibility: String,
}

fn basement_backdrop_id() -> u32 {
    BASEMENT.id
}

fn starting_wall_catalog() -> Vec<WallVariant> {
    [
        (0, "plain", 7, "starting-room"),
        (1, "rough", 1, "starting-room"),
        (2, "cloth", 1, "starting-room"),
        (3, "blood", 0, "basement-boss-room"),
    ]
    .into_iter()
    .map(
        |(source_patch_index, name, weight, eligibility)| WallVariant {
            source_patch_index,
            name: name.into(),
            weight,
            eligibility: eligibility.into(),
        },
    )
    .collect()
}

/// A backdrops.xml row a room's backdrop setup selects. Every admitted id
/// is a 468x468 sheet of six 234x156 patches; pre_render_floor/walls pick a
/// floor set (Random(floors)) of `floor_variants` patches and a wall set
/// (Random(walls)) of `wall_variants` patches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Backdrop {
    pub id: u32,
    pub gfx: &'static str,
    pub walls: u32,
    pub wall_variants: u32,
    pub floors: u32,
    pub floor_variants: u32,
}

const BASEMENT: Backdrop = Backdrop {
    id: 1,
    gfx: "gfx/backdrop/01_basement.png",
    walls: 1,
    wall_variants: 3,
    floors: 1,
    floor_variants: 6,
};
const BACKDROPS: [Backdrop; 11] = [
    BASEMENT,
    Backdrop {
        id: 3,
        gfx: "gfx/backdrop/03_caves.png",
        walls: 2,
        wall_variants: 3,
        floors: 2,
        floor_variants: 3,
    },
    Backdrop {
        id: 5,
        gfx: "gfx/backdrop/05_depths.png",
        walls: 1,
        wall_variants: 3,
        floors: 1,
        floor_variants: 6,
    },
    Backdrop {
        id: 9,
        gfx: "gfx/backdrop/09_sheol.png",
        walls: 1,
        wall_variants: 3,
        floors: 1,
        floor_variants: 6,
    },
    Backdrop {
        id: 14,
        gfx: "gfx/backdrop/0a_library.png",
        walls: 1,
        wall_variants: 3,
        floors: 1,
        floor_variants: 6,
    },
    Backdrop {
        id: 15,
        gfx: "gfx/backdrop/0b_shop.png",
        walls: 1,
        wall_variants: 2,
        floors: 1,
        floor_variants: 6,
    },
    Backdrop {
        id: 16,
        gfx: "gfx/backdrop/0c_isaacsroom.png",
        walls: 1,
        wall_variants: 2,
        floors: 1,
        floor_variants: 6,
    },
    Backdrop {
        id: 17,
        gfx: "gfx/backdrop/0d_barrenroom.png",
        walls: 1,
        wall_variants: 2,
        floors: 1,
        floor_variants: 6,
    },
    Backdrop {
        id: 18,
        gfx: "gfx/backdrop/0f_secretroom.png",
        walls: 1,
        wall_variants: 3,
        floors: 1,
        floor_variants: 6,
    },
    Backdrop {
        id: 19,
        gfx: "gfx/backdrop/0e_diceroom.png",
        walls: 1,
        wall_variants: 4,
        floors: 1,
        floor_variants: 6,
    },
    Backdrop {
        id: 20,
        gfx: "gfx/backdrop/0e_arcade.png",
        walls: 1,
        wall_variants: 4,
        floors: 1,
        floor_variants: 6,
    },
];

/// Room backdrop setup 5c5b30: types 0/1/4/5/6 take the stage backdrop
/// (5d49b0; Basement for the admitted stages 1 and 2), the others their
/// jump-table entry at 0x67a630. Returns the selected id (Room+1ae0) and the
/// generation the port admits for it. Ids with a different layout or path
/// (0 no backdrop, 10 Cathedral, 12 Chest, 13 Mega Satan, 21 error) keep the
/// Basement generation until admitted.
pub(crate) fn backdrop(room_type: u32, variant: u32, stage: u32) -> (u32, Option<Backdrop>) {
    let id = match room_type {
        5 if variant == 5000 => 13,
        // 5d49b0: an odd stage keeps its number, an even one drops by one,
        // and the alternate type (locked on a fresh profile) adds one.
        0 | 1 | 4 | 5 | 6 => {
            match crate::levels::stage_chapter(stage).and_then(|chapter| chapter.backdrop) {
                Some(id) => id,
                None => return (0, None),
            }
        }
        2 => 15,
        3 => 21,
        7 => 18,
        9 => 20,
        10 | 11 | 14 | 17 | 22 => 9,
        12 => 14,
        13 => 5,
        15 => 10,
        18 => 16,
        19 => 17,
        20 => 12,
        21 => 19,
        _ => 0,
    };
    (
        id,
        BACKDROPS.iter().find(|backdrop| backdrop.id == id).copied(),
    )
}

/// Wall generation 424440's variant parameters: chance Random(chance) == 0
/// then Random(variants). A shop backdrop (15) in a shop room reads the shop
/// level Room+2158; a fresh profile owns none of the System+c7..ca store
/// upgrades, so the level is 1: one variant at chance 3.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WallParameters {
    chance: u32,
    variants: u32,
    /// The selected wall set's first patch, set*wallvariants.
    first: u32,
}

impl WallParameters {
    fn for_room(backdrop: Backdrop, room_type: u32) -> Self {
        if backdrop.id == 15 && room_type == 2 {
            return Self {
                chance: 3,
                variants: 1,
                first: 0,
            };
        }
        Self {
            chance: 3,
            variants: backdrop.wall_variants,
            first: 0,
        }
    }

    fn in_set(self, set: u32) -> Self {
        Self {
            first: set * self.variants,
            ..self
        }
    }

    /// The patch distribution the conditional draws produce, in units of
    /// 1/(chance*variants), for the appearance's evidence catalogue.
    fn catalog(self, backdrop: Backdrop) -> Vec<WallVariant> {
        if backdrop == BASEMENT && self.variants == 3 {
            return starting_wall_catalog();
        }
        (0..self.variants)
            .map(|index| WallVariant {
                source_patch_index: index as u8,
                name: format!("variant{index}"),
                weight: if index == 0 {
                    (self.chance - 1) * self.variants + 1
                } else {
                    1
                },
                eligibility: format!("backdrop{}", backdrop.id),
            })
            .collect()
    }
}

fn choose_wall(rng: &mut DecorationRng, parameters: WallParameters) -> u8 {
    // A variant draw is conditional. Flattening the resulting distribution
    // to one weighted draw changes all subsequent stream state.
    let variant = if rng.next().is_multiple_of(parameters.chance) {
        rng.next() % parameters.variants
    } else {
        0
    };
    (parameters.first + variant) as u8
}

/// pre_render_floor 428930..428a4a: Random(floors) picks a floor set; its
/// `floor_variants` patches (jump table 0x665e00) double until more than 31,
/// then shuffle. Basement's single set draws Random(1) all the same.
fn floor_patches(rng: &mut DecorationRng, backdrop: Backdrop) -> Vec<u8> {
    let first = rng.next() % backdrop.floors * backdrop.floor_variants;
    let mut catalogue: Vec<u8> = (first..first + backdrop.floor_variants)
        .map(|patch| patch as u8)
        .collect();
    while catalogue.len() < 32 {
        catalogue.extend_from_within(..);
    }
    for index in (1..catalogue.len()).rev() {
        let other = rng.next() as usize % (index + 1);
        catalogue.swap(index, other);
    }
    catalogue
}

/// The grid factory owns the canonical native allocation bound. Atlas caps
/// additionally require the recovered centre/edge strips to have an interior.
/// Smaller future shapes need their own source contour, never unsigned wrap.
fn validate_backdrop_dimensions(width: u8, height: u8) -> Result<(), String> {
    crate::grid::validate_room_dimensions(u16::from(width) + 2, u16::from(height) + 2)?;
    if width <= 7 || height <= 4 {
        return Err("unadmitted narrow backdrop atlas geometry".into());
    }
    Ok(())
}

/// Backdrop::pre_render_floor428020 uses the actual padded grid dimensions.
/// Its centre patches precede the right strip, bottom strip and final corner.
/// The final corner reuses the last bottom-strip patch without another draw.
fn tile_floor(rng: &mut DecorationRng, backdrop: Backdrop, width: u8, height: u8) -> Vec<RoomQuad> {
    let catalogue = floor_patches(rng, backdrop);
    // Room+14/+18 include one boundary cell on each side. The render cache
    // has one additional tile of wall depth on each side (42A1DE..42A201).
    let right = i32::from(width) - 7;
    let bottom = i32::from(height) - 4;
    let offsets = |end: i32, step: i32| {
        let mut result = Vec::new();
        let mut at = 0;
        while at < end {
            result.push(at);
            at += step;
            // 4286F6/4290C5 retain the authored seam at the first overlap.
            if at == step && end != step + 1 {
                at += 1;
            }
        }
        result
    };
    let columns = offsets(right, 6);
    let rows = offsets(bottom, 3);
    let mut quads = Vec::new();
    let mut patch = 0;
    let mut append = |column: i32, row: i32, flip_x: bool, flip_y: bool| {
        let selected = catalogue[patch];
        patch += 1;
        quads.push(quad(
            selected,
            rect(52.0, 52.0, 182.0, 104.0),
            rect(
                52.0 + f64::from(column) * 26.0,
                52.0 + f64::from(row) * 26.0,
                182.0,
                104.0,
            ),
            flip_x,
            flip_y,
        ));
    };
    for &row in &rows {
        for &column in &columns {
            append(column, row, column != 0, row != 0);
        }
    }
    // 429DF0 emits the right edge after all centre patches.
    for &row in &rows {
        append(right, row, true, row != 0);
    }
    // 4290E9 emits the bottom edge, then428727 reuses its last choice.
    for &column in &columns {
        append(column, bottom, column != 0, true);
    }
    quads.push(quad(
        catalogue[patch - 1],
        rect(52.0, 52.0, 182.0, 104.0),
        rect(
            52.0 + f64::from(right) * 26.0,
            52.0 + f64::from(bottom) * 26.0,
            182.0,
            104.0,
        ),
        true,
        true,
    ));
    quads
}

fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}

fn quad(
    source_patch_index: u8,
    source_rect: Rect,
    dest_rect: Rect,
    flip_x: bool,
    flip_y: bool,
) -> RoomQuad {
    RoomQuad {
        source_patch_index,
        source_rect,
        dest_rect,
        flip_x,
        flip_y,
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoomDoor {
    pub side: String,
    pub state: String,
    pub asset: String,
    pub spritesheet: Option<String>,
    pub x: f64,
    pub y: f64,
    pub target_room: u32,
    pub slot: u8,
    pub world_position: motion::WorldVector,
    pub blown_open: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoomFx {
    pub ambient: [u8; 3],
    pub shading: String,
    pub overlay: String,
    pub overlay_origin: RoomOrigin,
    pub overlay_anchor: RoomOrigin,
    pub overlay_parallax: RoomOrigin,
    pub overlay_motion_evidence: String,
    pub overlay_opacity: f64,
    pub actor_tint: [u8; 3],
}

/// Original Backdrop::SetSeed uses public shift-index 35 (5, 9, 7).
/// Zero remains the mathematical absorbing state for invariant testing.
struct DecorationRng {
    state: u32,
}

impl DecorationRng {
    fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u32 {
        self.state ^= self.state >> 5;
        self.state ^= self.state << 9;
        self.state ^= self.state >> 7;
        self.state
    }

    /// RNG::RandomFloat 5c2db0: the next word times 2.3283064e-10 in f32.
    fn float(&mut self) -> f32 {
        self.next() as f32 * 2.328_306_4e-10
    }
}

/// Wall tiling after Random(walls): the four corners, then the top/bottom
/// strips by column and the side strips by row, each tile drawing its own
/// variant. A boss backdrop overrides the variant but keeps every draw.
fn tile_walls(
    rng: &mut DecorationRng,
    walls: WallParameters,
    variant: Option<u8>,
    width: u8,
    height: u8,
) -> Vec<RoomQuad> {
    let wall = |rng: &mut DecorationRng| {
        let drawn = choose_wall(rng, walls);
        variant.unwrap_or(drawn)
    };
    let right = f64::from(width + 2) * ROOM_TILE_SIZE;
    let bottom = f64::from(height + 2) * ROOM_TILE_SIZE;
    let mut wall_quads = vec![
        quad(
            wall(rng),
            rect(0.0, 0.0, 52.0, 52.0),
            rect(0.0, 0.0, 52.0, 52.0),
            false,
            false,
        ),
        quad(
            wall(rng),
            rect(0.0, 0.0, 52.0, 52.0),
            rect(right, 0.0, 52.0, 52.0),
            true,
            false,
        ),
        quad(
            wall(rng),
            rect(0.0, 0.0, 52.0, 52.0),
            rect(0.0, bottom, 52.0, 52.0),
            false,
            true,
        ),
        quad(
            wall(rng),
            rect(0.0, 0.0, 52.0, 52.0),
            rect(right, bottom, 52.0, 52.0),
            true,
            true,
        ),
    ];
    for column in 0..width {
        let x = 52.0 + f64::from(column) * 26.0;
        // 424BF9 left seven cells,4256D3 alternating middle,425C53 right
        // six cells. Each cell keeps its own conditional variant draws.
        let (source_column, flip_x) = if column < 7 {
            (column + 2, false)
        } else if column < width - 6 {
            (7 + column % 2, false)
        } else {
            (width - column + 1, true)
        };
        let source_x = f64::from(source_column) * 26.0;
        for bottom in [false, true] {
            wall_quads.push(quad(
                wall(rng),
                rect(source_x, 0.0, 26.0, 52.0),
                rect(
                    x,
                    if bottom {
                        f64::from(height + 2) * 26.0
                    } else {
                        0.0
                    },
                    26.0,
                    52.0,
                ),
                flip_x,
                bottom,
            ));
        }
    }
    for row in 0..height {
        let y = 52.0 + f64::from(row) * 26.0;
        // 425F60 top four cells,4266F4 alternating middle,424ED0 bottom
        // three cells. This is atlas tiling, never stretched wall pixels.
        let (source_row, flip_y) = if row < 4 {
            (row + 2, false)
        } else if row < height - 3 {
            (4 + row % 2, false)
        } else {
            (height - row + 1, true)
        };
        let source_y = f64::from(source_row) * 26.0;
        for right in [false, true] {
            wall_quads.push(quad(
                wall(rng),
                rect(0.0, source_y, 52.0, 26.0),
                rect(
                    if right {
                        f64::from(width + 2) * 26.0
                    } else {
                        0.0
                    },
                    y,
                    52.0,
                    26.0,
                ),
                right,
                flip_y,
            ));
        }
    }
    wall_quads
}

/// Backdrop::pre_render_walls 424440 on the Basement backdrop (1) in a
/// boss room (type 5, not RoomData 5000): after pre_render_floor's patches
/// and Random(walls), each grid cell inside the room shrunk by 20 that wins
/// a local Int(2) gets a BloodSplat 1000.7 scaled local.Float·2.5 + 0.5.
/// Returns the splat cells and scales in grid order; the caller draws each
/// splat's MT.next seed and immediate Update in turn (4270ac).
pub(crate) fn basement_boss_blood(
    room: &Room,
    stage: u32,
) -> Result<Vec<(motion::WorldVector, f32)>, String> {
    let Some(descriptor) = &room.descriptor else {
        return Ok(Vec::new());
    };
    let template = &descriptor.template;
    if !boss_blood_backdrop(stage) || template.room_type != 5 || template.variant == 5000 {
        return Ok(Vec::new());
    }
    let grid = crate::grid::Grid::new(
        room.grid
            .clone()
            .ok_or("boss-room blood scatter requires the room grid")?,
    )?;
    let mut rng = DecorationRng::new(descriptor.decoration_seed);
    floor_patches(&mut rng, BASEMENT);
    rng.next();
    boss_blood_scatter(&mut rng, &grid)
}

/// The stage chapter draws the bloody boss backdrop.
fn boss_blood_backdrop(stage: u32) -> bool {
    crate::levels::stage_chapter(stage).is_some_and(|chapter| chapter.boss_blood_backdrop)
}

/// The boss scatter's local draws: one Int(2) per grid cell in grid order,
/// plus Float for each winning cell inside the room shrunk by 20. Shared by
/// the splat admission and the wall tiling that follows it in the stream.
fn boss_blood_scatter(
    rng: &mut DecorationRng,
    grid: &crate::grid::Grid,
) -> Result<Vec<(motion::WorldVector, f32)>, String> {
    let mut splats = Vec::new();
    for index in 0..grid.descriptor().cells.len() {
        if rng.next().is_multiple_of(2) {
            continue;
        }
        let position = grid
            .position(index)
            .ok_or("boss-room blood scatter cell outside its grid")?;
        if grid.clamp_room(position, 20.0) != position {
            continue;
        }
        let scale = rng.float() * 2.5 + 0.5;
        splats.push((position, scale));
    }
    Ok(splats)
}

impl Room {
    /// pre_render_walls 424440 in a Basement boss room: after the scatter the
    /// boss branch replaces Random(walls) with wallvariants (3), so every wall
    /// tile shows the bloody boss row while still drawing its Int(3)/Int(1).
    /// The tiling continues the decoration stream after the scatter's draws.
    pub(crate) fn apply_boss_backdrop(&mut self, stage: u32) -> Result<(), String> {
        let Some(descriptor) = &self.descriptor else {
            return Ok(());
        };
        let template = &descriptor.template;
        if !boss_blood_backdrop(stage) || template.room_type != 5 || template.variant == 5000 {
            return Ok(());
        }
        validate_backdrop_dimensions(template.width, template.height)?;
        let grid = crate::grid::Grid::new(
            self.grid
                .clone()
                .ok_or("boss-room backdrop requires the room grid")?,
        )?;
        let mut rng = DecorationRng::new(descriptor.decoration_seed);
        floor_patches(&mut rng, BASEMENT);
        rng.next();
        boss_blood_scatter(&mut rng, &grid)?;
        // The boss branch sets wallvariants to 1, so each tile draws Int(3)
        // then Int(1); the shown patch is the boss row (variant 3).
        self.appearance.wall_quads = tile_walls(
            &mut rng,
            WallParameters {
                chance: 3,
                variants: 1,
                first: 0,
            },
            Some(3),
            template.width,
            template.height,
        );
        Ok(())
    }
}

/// Room backdrop setup 5c5b30 reloads fxlayers.xml (5c5be3 -> 5489b0) on
/// every Room init. Each authored `fxRay`, in every ray group whatever its
/// stages, draws one phase uniform()*pi at 54a16d. Crawlspace rooms (type 16)
/// and backdrop 13 (boss RoomData 5000) clear the layers instead.
///
/// 54a5b6 then seeds a fresh RNG with Descriptor+48 (shift35, no warm-up),
/// stores FX+0x38 = RandomInt(matched groups) and, in an ordinary room
/// (type 1), clears it when RandomInt(stage < 3 ? 3 : 2) != 0 unless the
/// stage is 9 with a nonzero stage type.
pub fn load_fx_rays(room: &mut Room, shared: &mut crate::mt::Mt19937, stage: u32, stage_type: u32) {
    room.appearance.fx_rays = None;
    let Some(descriptor) = &room.descriptor else {
        return;
    };
    let template = &descriptor.template;
    if template.room_type == 16 || (template.room_type == 5 && template.variant == 5000) {
        return;
    }
    let xml = include_str!("../../references/isaac/rebirth/resources/fxlayers.xml");
    let rays = &xml[xml.find("<fxRays>").unwrap_or(0)..];
    let mut matched = Vec::new();
    for group in rays.split("<rayGroup ").skip(1) {
        let head = &group[..group.find('>').unwrap_or(group.len())];
        let number = |name: &str| {
            attribute(head, name)
                .and_then(|value| value.parse::<f32>().ok())
                .unwrap_or(0.0)
        };
        let mut parsed = FxRayGroup {
            index: 0,
            x_min: number("xMin"),
            x_max: number("xMax"),
            top_parallax: number("topParallax"),
            bottom_parallax: number("bottomParallax"),
            ray_length: number("rayLength"),
            ray_spacing: number("raySpacing"),
            perspective: number("perspective"),
            rays: Vec::new(),
        };
        let body = &group[..group.find("</rayGroup>").unwrap_or(group.len())];
        for ray in body.split("<fxRay ").skip(1) {
            let number = |name: &str| {
                attribute(ray, name)
                    .and_then(|value| value.parse::<f32>().ok())
                    .unwrap_or(0.0)
            };
            parsed.rays.push(FxRay {
                opacity: number("opacity"),
                animation_speed: number("animationSpeed"),
                phase: (f64::from(shared.uniform()) * std::f64::consts::PI) as f32,
                texture: format!("gfx/{}", attribute(ray, "path").unwrap_or_default()),
            });
        }
        if attribute(head, "stages").is_some_and(|stages| stages_match(stages, stage, stage_type)) {
            matched.push(parsed);
        }
    }
    if matched.is_empty() {
        return;
    }
    let mut random = crate::items::Rng::new(descriptor.decoration_seed, [5, 9, 7]);
    let index = random.bounded(matched.len() as u32);
    if template.room_type == 1
        && random.bounded(if stage < 3 { 3 } else { 2 }) != 0
        && !(stage == 9 && stage_type != 0)
    {
        return;
    }
    let mut group = matched.swap_remove(index as usize);
    group.index = index;
    room.appearance.fx_rays = Some(group);
}

fn attribute<'a>(element: &'a str, name: &str) -> Option<&'a str> {
    let start = element.find(&format!("{name}=\""))? + name.len() + 2;
    let end = element[start..].find('"')?;
    Some(&element[start..start + end])
}

/// 5489b0 stage tokens: "Na" needs stage type 1, a bare N type 0 (a trailing
/// "b" is stripped), and N covers stages 2N-1 and 2N.
fn stages_match(stages: &str, stage: u32, stage_type: u32) -> bool {
    stages.split(',').any(|token| {
        let token = token.trim().trim_end_matches('b');
        let (number, wanted_type) = match token.strip_suffix('a') {
            Some(number) => (number, 1),
            None => (token, 0),
        };
        number
            .parse::<u32>()
            .is_ok_and(|n| (stage == 2 * n || stage + 1 == 2 * n) && stage_type == wanted_type)
    })
}

impl Room {
    /// Death66/variant0,481090 Attack02 frame22 writes4822DF after its
    /// ordered Player effects. Unknown prior history is not invented as160.
    pub fn apply_death_tear_slowdown(&mut self) {
        self.source_tear_slowdown_2494 = self.source_tear_slowdown_2494.map(|prior| prior.max(160));
    }

    /// 5cf614/5cff3f store loaded after factories or saved actor admission.
    pub(crate) fn complete_initialization(&mut self) {
        self.loaded = true;
    }

    pub fn from_starting_floor(floor: &crate::floor::Floor) -> Self {
        Self::from_floor_descriptor(floor, floor.starting_descriptor)
    }

    /// Admit the authored configuration once. Mutable fields on Room persist
    /// with its descriptor when the player leaves and later returns.
    pub fn from_floor_descriptor(floor: &crate::floor::Floor, index: usize) -> Self {
        Self::from_floor_descriptor_with_special(floor, index, None)
            .expect("generated descriptor identifies original authored STB")
    }

    pub(crate) fn from_floor_descriptor_with_special(
        floor: &crate::floor::Floor,
        index: usize,
        special: Option<&[crate::room_assets::RoomAsset]>,
    ) -> Result<Self, String> {
        Self::from_floor_descriptor_with_layout(floor, index, special, None)
    }

    /// `layout` replaces the descriptor's authored template, as load_room
    /// 586c90 does for a surprise-miniboss room; the descriptor keeps its
    /// own type, seeds and backdrop.
    pub(crate) fn from_floor_descriptor_with_layout(
        floor: &crate::floor::Floor,
        index: usize,
        special: Option<&[crate::room_assets::RoomAsset]>,
        layout: Option<crate::room_assets::RoomAsset>,
    ) -> Result<Self, String> {
        let descriptor = floor
            .descriptors
            .get(index)
            .ok_or("Room descriptor index outside Floor")?;
        let template = &descriptor.template;
        let (backdrop_id, admitted) = backdrop(template.room_type, template.variant, floor.stage);
        let mut room = Self::from_backdrop(
            descriptor.decoration_seed,
            admitted.unwrap_or(BASEMENT),
            template.room_type,
            template.width,
            template.height,
            floor.stage,
        )?;
        room.appearance.backdrop_id = backdrop_id;
        room.descriptor = Some(descriptor.clone());
        // Room load5cedf0..5ceec7 uses GridPosition(gridWidth+1)-(20,20)
        // for minimum(60,140), then adds authored dimensions*40. Project the
        // radius-inset native planes into the authoritative player space.
        let radius = content::PLAYER_COLLISION_RADIUS_WORLD;
        room.left = (60.0 + radius) * motion::WORLD_SCALE + motion::WORLD_ORIGIN_X;
        room.right = (60.0 + f64::from(descriptor.template.width) * 40.0 - radius)
            * motion::WORLD_SCALE
            + motion::WORLD_ORIGIN_X;
        room.top = (140.0 + radius) * motion::WORLD_SCALE + motion::WORLD_ORIGIN_Y;
        room.bottom = (140.0 + f64::from(descriptor.template.height) * 40.0 - radius)
            * motion::WORLD_SCALE
            + motion::WORLD_ORIGIN_Y;
        room.appearance.doors.clear();
        // Start Room's authored boundary door grid positions, padded by one,
        // use RoomGetGridPosition (40+40*x,120+40*y), .65 projection and
        // common sprite Y−1. Normal door setup adds the source 15px inset.
        let authored = match layout {
            Some(layout) => layout,
            None => crate::room_assets::for_descriptor_with_special(descriptor, special)?,
        };
        room.spawns = Some(crate::room_spawns::select(
            &authored,
            descriptor.spawn_seed,
        )?);
        // Room.Init 4288b5..4293b3 loads backdrop/controls.png only on stage 1
        // (Game+0) in the starting room (Game+8188 == Game+8148); later
        // floors' starting rooms share the template but draw a plain floor.
        room.appearance.tutorial = floor.stage == 1 && index == floor.starting_descriptor;
        room.appearance.template = format!(
            "{}.{}({})",
            descriptor.template.room_type, descriptor.template.variant, descriptor.template.name
        );
        for (slot, target) in descriptor.neighbors.iter().enumerate() {
            let Some(target) = target else {
                continue;
            };
            let destination = &floor.descriptors[*target];
            let secret = matches!(descriptor.template.room_type, 7 | 8)
                || matches!(destination.template.room_type, 7 | 8);
            let (side, inset_x, inset_y) = match slot % 4 {
                0 => ("left", 15.0, 0.0),
                1 => ("up", 0.0, 15.0),
                2 => ("right", -15.0, 0.0),
                3 => ("down", 0.0, -15.0),
                _ => unreachable!(),
            };
            let door = authored
                .doors
                .iter()
                .find(|door| {
                    door.enabled && crate::room_assets::door_slot(door.x, door.y) == Ok(slot as u8)
                })
                .expect("required starting door is authored");
            let column = f64::from(door.x + 1);
            let row = f64::from(door.y + 1);
            room.appearance.doors.push(RoomDoor {
                side: side.into(),
                state: if secret {
                    "Hidden"
                } else if matches!(destination.template.room_type, 2 | 12 | 21) {
                    "KeyClosed"
                } else if matches!(destination.template.room_type, 18 | 19) {
                    // Cracked variant3: the main door holds its closed pose
                    // under the door_18 overlay until bombs break it.
                    "Closed"
                } else {
                    "Opened"
                }
                .into(),
                asset: if secret {
                    "gfx/grid/door_08_holeinwall.anm2"
                } else if descriptor.template.room_type == 10
                    || destination.template.room_type == 10
                {
                    "gfx/grid/door_04_selfsacrificeroomdoor.anm2"
                } else if descriptor.template.room_type == 4 || destination.template.room_type == 4
                {
                    "gfx/grid/door_02_treasureroomdoor.anm2"
                } else {
                    "gfx/grid/door_01_normaldoor.anm2"
                }
                .into(),
                // Door.Init562b48 uses the current room's backdrop palette,
                // not the neighbor type. Library palettes14/15 replace every
                // sheet of the ordinary door with the authored library PNG.
                spritesheet: (descriptor.template.room_type == 12
                    && destination.template.room_type == 1)
                    .then(|| "gfx/grid/door_13_librarydoor.png".into()),
                x: (40.0 + 40.0 * column) * motion::WORLD_SCALE + 32.0 + inset_x,
                y: (120.0 + 40.0 * row) * motion::WORLD_SCALE - 46.0 - 1.0 + inset_y,
                target_room: *target as u32,
                slot: slot as u8,
                world_position: motion::WorldVector {
                    x: (40.0 + 40.0 * column) as f32,
                    y: (120.0 + 40.0 * row) as f32,
                },
                blown_open: false,
            });
        }
        let doors = room
            .appearance
            .doors
            .iter()
            .map(|door| crate::grid::DoorCollision {
                slot: door.slot,
                open: door.state == "Opened",
                hidden: door.state == "Hidden",
                double_lock: floor.descriptors[door.target_room as usize]
                    .template
                    .room_type
                    == 21,
                cracked: matches!(
                    floor.descriptors[door.target_room as usize]
                        .template
                        .room_type,
                    18 | 19
                ),
                // A freshly constructed room is uncleared (descriptor+3d);
                // hidden secret connections already took variant6.
                barred: descriptor.template.room_type == 6 || descriptor.surprise_miniboss,
            })
            .collect::<Vec<_>>();
        room.grid = Some(crate::grid::for_room(
            &authored,
            room.spawns.as_ref().expect("authored groups were selected"),
            descriptor.decoration_seed,
            &doors,
        )?);
        room.cracked_close_frame = doors
            .iter()
            .any(|door| door.cracked && !door.hidden)
            .then_some(0);
        room.path_costs = room
            .grid
            .as_ref()
            .map(crate::room_costs::RoomCosts::for_supported_grid)
            .transpose()?;
        room.appearance.generation = format!(
            "source-fresh-{};descriptor-decoration12;authored-spawn11;provisional-fx-route",
            match admitted {
                Some(backdrop) => format!("backdrop{}-floor", backdrop.id),
                None => format!("unadmitted-backdrop{backdrop_id}-basement-floor"),
            }
        );
        Ok(room)
    }

    /// Room.Init5cecc0 clears its transient0x700-byte cost array on every
    /// admission. Saved room geometry then runs recovered Grid.Init stores;
    /// destruction markers themselves are never restored from the old room.
    pub(crate) fn initialize_path_costs(&mut self) {
        self.path_costs = self.grid.as_ref().map(|grid| {
            crate::room_costs::RoomCosts::for_supported_grid(grid)
                .expect("admitted geometry fits source Room.Init path costs")
        });
    }

    /// Assemble the starting-room backdrop from its independently supplied room
    /// decoration seed. This is also the original-observation verification entry.
    /// The seed is not a run seed; this method does not generate floor topology.
    pub fn from_decoration_seed(decoration_seed: u32) -> Self {
        Self::from_backdrop(decoration_seed, BASEMENT, 1, 13, 7, 1)
            .expect("starting-room dimensions fit the source backdrop")
    }

    /// Backdrop::Init42A170: floor then walls on the same decoration RNG35.
    /// Authored dimensions own art extent; the camera viewport stays480x270.
    fn from_backdrop(
        decoration_seed: u32,
        backdrop: Backdrop,
        room_type: u32,
        width: u8,
        height: u8,
        stage: u32,
    ) -> Result<Self, String> {
        validate_backdrop_dimensions(width, height)?;
        let walls = WallParameters::for_room(backdrop, room_type);
        let mut rng = DecorationRng::new(decoration_seed);
        let floor_quads = tile_floor(&mut rng, backdrop, width, height);
        let wall_variant_catalog = walls.catalog(backdrop);
        // 424532 Random(walls) picks the wall set; a tile's patch is its
        // drawn variant plus set*wallvariants (4247b9, 425487). Basement's
        // single set draws Random(1) all the same.
        let set = rng.next() % backdrop.walls;
        let wall_quads = tile_walls(&mut rng, walls.in_set(set), None, width, height);
        // fxlayers.xml (parsed per room load, 5489b0) seeds a fresh RNG per
        // fx entry from the decoration seed, shift index 35 (5494df): it does
        // not continue the backdrop/wall stream. RandomInt(5) picks the 1x1
        // overlay in authored order [2, 3, 4, 5, 1] (54972d); two
        // RandomFloat draws (5498f8, 549926; RandomFloat 5c2db0 is next *
        // 2.3283064e-10 in f32) give the layer's x and y offsets as
        // min + RandomFloat * (max - min). The overlay group follows the
        // Level's chapter (fxlayers `stage`): Basement fx2 spans 0..100,
        // Caves fx5 -10..10; both author their 1x1 overlays as [2, 3, 4, 5, 1].
        let crate::levels::Overlay {
            folder: overlay_folder,
            low,
            high,
        } = crate::levels::stage_chapter(stage)
            .and_then(|chapter| chapter.overlay)
            .unwrap_or(crate::levels::BASEMENT_OVERLAY);
        let mut fx = DecorationRng::new(decoration_seed);
        let overlay_variant = [2, 3, 4, 5, 1][(fx.next() % 5) as usize];
        let mut offset = || f64::from(fx.next() as f32 * 2.328_306_4e-10_f32 * (high - low) + low);
        let overlay_origin = RoomOrigin {
            // Natural overlay size is 663×429; viewport-centered sampling starts here.
            x: (663.0 - 480.0) * 0.5 + offset(),
            y: (429.0 - 270.0) * 0.5 + offset(),
        };

        let origin = RoomOrigin { x: 19.0, y: -8.0 };
        let radius = content::PLAYER_COLLISION_RADIUS_WORLD * motion::WORLD_SCALE;
        let render_offset = Player::starting().render_offset;
        // The authored floor's interior defines the wall planes. Actor ground
        // and sprite ground differ by the render offset; the actor's circle
        // must remain one radius inside every plane. Native wall observations
        // independently verify all four resulting center limits.
        let left_plane = origin.x
            + floor_quads
                .iter()
                .map(|q| q.dest_rect.x)
                .fold(f64::INFINITY, f64::min)
            - render_offset.x;
        let right_plane = origin.x
            + floor_quads
                .iter()
                .map(|q| q.dest_rect.x + q.dest_rect.width)
                .fold(f64::NEG_INFINITY, f64::max)
            - render_offset.x;
        let top_plane = origin.y
            + floor_quads
                .iter()
                .map(|q| q.dest_rect.y)
                .fold(f64::INFINITY, f64::min)
            - render_offset.y;
        let bottom_plane = origin.y
            + floor_quads
                .iter()
                .map(|q| q.dest_rect.y + q.dest_rect.height)
                .fold(f64::NEG_INFINITY, f64::max)
            - render_offset.y;
        Ok(Room {
            descriptor: None,
            initialization_game_update: None,
            loaded: false,
            source_tear_slowdown_2494: None,
            pickups: Vec::new(),
            saved_regular_pickups: Vec::new(),
            dynamic_crater_records: Some(Vec::new()),
            npcs: Vec::new(),
            slots: Vec::new(),
            background_stains: Vec::new(),
            authored_actor_order: Vec::new(),
            manager_counts: crate::actors::ManagerCounts::default(),
            ordinary_placement: None,
            boss_count: 0,
            boss_presentation_index: 0,
            combat_clear: true,
            combat_clear_countdown: 0,
            combat_shut_door_slots: Vec::new(),
            combat_shut_update: None,
            cracked_close_frame: None,
            spawns: None,
            grid: None,
            path_costs: None,
            bomb_rock_fuses: Vec::new(),
            trapdoor: None,
            left: left_plane + radius,
            right: right_plane - radius,
            top: top_plane + radius,
            bottom: bottom_plane - radius,
        appearance: RoomAppearance {
            backdrop: backdrop.gfx.into(),
            backdrop_id: backdrop.id,
            columns: width + 4,
            rows: height + 4,
            tile_size: ROOM_TILE_SIZE,
                origin,
            backdrop_patch_size: Viewport {
                width: 234.0,
                height: 156.0,
            },
            backdrop_patch_columns: 2,
            floor_quads,
            wall_quads,
            wall_variant_catalog,
            wall_generation_evidence:
                "recovered-decoration35-conditional-variant-draws;heldout-original-gl-cache"
                    .into(),
            tutorial: true,
            doors: Vec::new(),
            fx: RoomFx {
                ambient: [255, 255, 255],
                // Room.Init5CEF16 selects the authored size-specific mask;
                // its natural dimensions equal this backdrop cache extent.
                shading: match (width, height) {
                    (13, 14) => "gfx/backdrop/shading_1x2.png",
                    (26, 7) => "gfx/backdrop/shading_2x1.png",
                    (26, 14) => "gfx/backdrop/shading_2x2.png",
                    _ => "gfx/backdrop/shading.png",
                }.into(),
                overlay: format!("gfx/overlays/{overlay_folder}/1x1_overlay_{overlay_variant}.png"),
                overlay_origin,
                // Absolute sprite render anchor, not initial-spawn displacement
                // or the physical ground position used by the radial light.
                overlay_anchor: RoomOrigin { x: 0.0, y: 0.0 },
                overlay_parallax: RoomOrigin { x: 0.025, y: 0.025 },
                overlay_motion_evidence:
                    "authored-fxlayers-id2;heldout-public-gl-absolute-world-sprite-parallax;no-time-animation"
                        .into(),
                overlay_opacity: 0.15,
                actor_tint: [255, 255, 255],
            },
            template: "starting-room".into(),
            generation: "recovered-basement-backdrop;supplied-room-decoration-seed;provisional-fx-route".into(),
            fx_rays: None,
        },
    })
    }
}
