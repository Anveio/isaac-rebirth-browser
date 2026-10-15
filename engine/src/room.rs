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

pub const ROOM_COLUMNS: u8 = 17;
pub const ROOM_ROWS: u8 = 11;
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
    pub pickups: Vec<crate::pickups::Pickup>,
    #[serde(default)]
    pub saved_regular_pickups: Vec<crate::pickups::SavedRegularPickup>,
    pub npcs: Vec<crate::npcs::Npc>,
    #[serde(default)]
    pub background_stains: Vec<crate::familiars::BackgroundStain>,
    pub authored_actor_order: Vec<crate::actors::ActorRef>,
    #[serde(default)]
    pub manager_counts: crate::actors::ManagerCounts,
    pub boss_count: u32,
    pub boss_presentation_index: u32,
    pub combat_clear: bool,
    pub combat_clear_countdown: u32,
    pub combat_shut_door_slots: Vec<u8>,
    pub spawns: Option<crate::room_spawns::RoomSpawns>,
    pub grid: Option<crate::grid::GridDescriptor>,
    /// The actual signed Room+e98 owner; absent until its source Init is admitted.
    #[serde(default)]
    pub path_costs: Option<crate::room_costs::RoomCosts>,
    /// Rock+348 countdowns of type5 bomb rocks a blast moved to state3.
    #[serde(default)]
    pub bomb_rock_fuses: Vec<crate::rock_blast::BombRockFuse>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoomAppearance {
    pub backdrop: String,
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

fn starting_wall_catalog() -> Vec<WallVariant> {
    [
        (0, "plain", 7, "starting-room"),
        (1, "rough", 1, "starting-room"),
        (2, "cloth", 1, "starting-room"),
        (3, "blood", 0, "outside-basement-authored-palette"),
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

fn choose_wall(rng: &mut DecorationRng) -> u8 {
    // A variant draw is conditional. Flattening the resulting 7:1:1
    // distribution to one weighted draw changes all subsequent stream state.
    if rng.next().is_multiple_of(3) {
        (rng.next() % 3) as u8
    } else {
        0
    }
}

fn floor_patches(rng: &mut DecorationRng) -> [u8; 4] {
    // Random(floors) advances the original stream even with floors=1.
    rng.next();
    let mut catalogue: Vec<u8> = (0..6).collect();
    while catalogue.len() < 32 {
        catalogue.extend_from_within(..);
    }
    for index in (1..catalogue.len()).rev() {
        let other = rng.next() as usize % (index + 1);
        catalogue.swap(index, other);
    }
    // The final overlapping strip shares one choice across both lower halves.
    [catalogue[0], catalogue[1], catalogue[2], catalogue[2]]
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
}

impl Room {
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
        let descriptor = floor
            .descriptors
            .get(index)
            .ok_or("Room descriptor index outside Floor")?;
        let mut room = Self::from_decoration_seed(descriptor.decoration_seed);
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
        let authored = crate::room_assets::for_descriptor_with_special(descriptor, special)?;
        room.spawns = Some(crate::room_spawns::select(
            &authored,
            descriptor.spawn_seed,
        )?);
        room.appearance.tutorial = descriptor.asset_stage == 0
            && descriptor.template.room_type == 1
            && descriptor.template.variant == 2;
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
            })
            .collect::<Vec<_>>();
        room.grid = Some(crate::grid::for_room(
            &authored,
            room.spawns.as_ref().expect("authored groups were selected"),
            descriptor.decoration_seed,
            &doors,
        )?);
        room.path_costs = room
            .grid
            .as_ref()
            .map(crate::room_costs::RoomCosts::for_supported_grid)
            .transpose()?;
        room.appearance.generation =
            "source-fresh-normal-basement-floor;descriptor-decoration12;authored-spawn11;provisional-fx-route"
                .into();
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
        let mut rng = DecorationRng::new(decoration_seed);
        let [upper_left, upper_right, lower, _] = floor_patches(&mut rng);
        let wall_variant_catalog = starting_wall_catalog();
        let floor_quads = vec![
            quad(
                upper_left,
                rect(52.0, 52.0, 182.0, 104.0),
                rect(52.0, 52.0, 182.0, 104.0),
                false,
                false,
            ),
            quad(
                upper_right,
                rect(52.0, 52.0, 182.0, 104.0),
                rect(208.0, 52.0, 182.0, 104.0),
                true,
                false,
            ),
            quad(
                lower,
                rect(52.0, 52.0, 182.0, 104.0),
                rect(52.0, 130.0, 182.0, 104.0),
                false,
                true,
            ),
            quad(
                lower,
                rect(52.0, 52.0, 182.0, 104.0),
                rect(208.0, 130.0, 182.0, 104.0),
                true,
                true,
            ),
        ];
        // Random(walls) also advances the stream with Basement walls=1.
        rng.next();
        let mut wall_quads = vec![
            quad(
                choose_wall(&mut rng),
                rect(0.0, 0.0, 52.0, 52.0),
                rect(0.0, 0.0, 52.0, 52.0),
                false,
                false,
            ),
            quad(
                choose_wall(&mut rng),
                rect(0.0, 0.0, 52.0, 52.0),
                rect(390.0, 0.0, 52.0, 52.0),
                true,
                false,
            ),
            quad(
                choose_wall(&mut rng),
                rect(0.0, 0.0, 52.0, 52.0),
                rect(0.0, 234.0, 52.0, 52.0),
                false,
                true,
            ),
            quad(
                choose_wall(&mut rng),
                rect(0.0, 0.0, 52.0, 52.0),
                rect(390.0, 234.0, 52.0, 52.0),
                true,
                true,
            ),
        ];
        for column in 0..13 {
            let x = 52.0 + f64::from(column) * 26.0;
            let source_x = 52.0 + f64::from(column.min(12 - column)) * 26.0;
            for bottom in [false, true] {
                wall_quads.push(quad(
                    choose_wall(&mut rng),
                    rect(source_x, 0.0, 26.0, 52.0),
                    rect(x, if bottom { 234.0 } else { 0.0 }, 26.0, 52.0),
                    column > 6,
                    bottom,
                ));
            }
        }
        for row in 0..7 {
            let y = 52.0 + f64::from(row) * 26.0;
            let source_y = 52.0 + f64::from(row.min(6 - row)) * 26.0;
            for right in [false, true] {
                wall_quads.push(quad(
                    choose_wall(&mut rng),
                    rect(0.0, source_y, 52.0, 26.0),
                    rect(if right { 390.0 } else { 0.0 }, y, 52.0, 26.0),
                    right,
                    row > 3,
                ));
            }
        }
        // Authored fxlayers.xml fx id=2, roomsize=0 replacement order.
        let overlay_variant = [2, 3, 4, 5, 1][(rng.next() % 5) as usize];
        let overlay_origin = RoomOrigin {
            // fxlayers.xml supplies random offsets 0..100 on both axes. Natural
            // overlay size is 663×429, so viewport-centered sampling starts here.
            // The original stream and exact float conversion remain unverified.
            x: (663.0 - 480.0) * 0.5 + f64::from(rng.next()) / f64::from(u32::MAX) * 100.0,
            y: (429.0 - 270.0) * 0.5 + f64::from(rng.next()) / f64::from(u32::MAX) * 100.0,
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
        Room {
            descriptor: None,
            initialization_game_update: None,
            loaded: false,
            pickups: Vec::new(),
            saved_regular_pickups: Vec::new(),
            npcs: Vec::new(),
            background_stains: Vec::new(),
            authored_actor_order: Vec::new(),
            manager_counts: crate::actors::ManagerCounts::default(),
            boss_count: 0,
            boss_presentation_index: 0,
            combat_clear: true,
            combat_clear_countdown: 0,
            combat_shut_door_slots: Vec::new(),
            spawns: None,
            grid: None,
            path_costs: None,
            bomb_rock_fuses: Vec::new(),
            left: left_plane + radius,
            right: right_plane - radius,
            top: top_plane + radius,
            bottom: bottom_plane - radius,
        appearance: RoomAppearance {
            backdrop: "gfx/backdrop/01_basement.png".into(),
            columns: ROOM_COLUMNS,
            rows: ROOM_ROWS,
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
                shading: "gfx/backdrop/shading.png".into(),
                overlay: format!("gfx/overlays/basement/1x1_overlay_{overlay_variant}.png"),
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
        },
    }
    }
}
