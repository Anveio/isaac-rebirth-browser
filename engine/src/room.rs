//! Recovered Basement backdrop assembly with provisional run-to-room routing.
//!
//! The original uses decoration xorshift index 35, a repeated/shuffled floor
//! catalogue, and conditional wall-variant draws. Independent original GL
//! cache submissions verify the complete geometry and patch sequence when the
//! room decoration seed is supplied. The level generator that derives that
//! seed from the run remains unresolved; so do the FX stream assignments.
use crate::effects::{Rect, Viewport};
use crate::seed::stage1_seed;
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
    pub projectile_bounds: ProjectileBounds,
    pub appearance: RoomAppearance,
}

/// Projected tear limits retained separately from verified player clearance.
/// Native tear-ground collision and its entity radius remain unresolved.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectileBounds {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
    pub evidence: String,
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
    pub x: f64,
    pub y: f64,
    pub target_room: u32,
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

/// Build the provisional starting-room template for any unsigned seed.
///
/// Native public GL submissions assemble the floor from four overlapping
/// authored patches and the outer walls from forty-four quads. The recovered
/// backdrop generator is verified given the room decoration seed. Substituting
/// the Stage 1 seed for that seed remains provisional.
/// Floor interiors have six valid patches; Basement walls use three of the
/// four authored wall patches. Atlas row three
/// contains checkerboard placeholders outside its floor interiors.
/// Collision planes and authored door anchors do not depend on the seed.
pub fn generate_room(seed: u32) -> Room {
    let mut room = Room::from_decoration_seed(stage1_seed(seed));
    room.appearance.generation =
        "recovered-basement-backdrop;provisional-stage-to-room-seed-and-fx-route".into();
    room
}

impl Room {
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
            left: left_plane + radius,
            right: right_plane - radius,
            top: top_plane + radius,
            bottom: bottom_plane - radius,
            projectile_bounds: ProjectileBounds {
                left: 70.0,
                right: 410.0,
                top: 48.0,
                bottom: 222.0,
                evidence: "projected-provisional-tear-limits;typed-ground-collision-unrecovered".into(),
            },
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
            doors: vec![
                RoomDoor {
                    side: "left".into(),
                    state: "Opened".into(),
                    x: 62.0,
                    y: 135.0,
                    target_room: 1,
                },
                RoomDoor {
                    side: "right".into(),
                    state: "Opened".into(),
                    x: 418.0,
                    y: 135.0,
                    target_room: 2,
                },
            ],
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn supplied_decoration_seeds_match_independent_original_gl_cache_submissions() {
        // Original fresh-profile N4Z8 GWV3 and P01M XJW0 runs. The descriptor
        // observer identified decoration seeds through each actual spawn log;
        // public GL cache UVs supplied these expected patch choices. These are
        // regression evidence only, never run-to-room routing lookup entries.
        for (decoration_seed, floors, walls) in [
            (
                4_271_600_350,
                [3, 2, 4, 4],
                [
                    0, 1, 0, 0, 2, 0, 0, 0, 0, 0, 1, 0, 2, 0, 0, 0, 1, 0, 0, 2, 0, 0, 0, 2, 0, 0,
                    1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0,
                ],
            ),
            (
                1_985_222_680,
                [0, 3, 0, 0],
                [
                    0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 2, 0, 0, 2, 1, 0, 2, 0, 0, 0,
                    0, 0, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0,
                ],
            ),
        ] {
            let appearance = Room::from_decoration_seed(decoration_seed).appearance;
            assert_eq!(
                appearance
                    .floor_quads
                    .iter()
                    .map(|quad| quad.source_patch_index)
                    .collect::<Vec<_>>(),
                floors
            );
            assert_eq!(
                appearance
                    .wall_quads
                    .iter()
                    .map(|quad| quad.source_patch_index)
                    .collect::<Vec<_>>(),
                walls
            );
        }
    }

    #[test]
    fn independently_observed_wall_limits_match_authored_circle_clearance() {
        // Separate native left/right/up/down contacts were world X=70/570,
        // Y=150/410. The independently observed projection is 0.65×world
        // plus (32,-46), and the XML player radius is 10 world units.
        for seed in crate::test_seeds::samples(
            "independently_observed_wall_limits_match_authored_circle_clearance",
            16,
        ) {
            let room = generate_room(seed);
            assert_eq!(room.left, 70.0 * motion::WORLD_SCALE + 32.0);
            assert_eq!(room.right, 570.0 * motion::WORLD_SCALE + 32.0);
            assert_eq!(room.top, 150.0 * motion::WORLD_SCALE - 46.0);
            assert_eq!(room.bottom, 410.0 * motion::WORLD_SCALE - 46.0);
            assert_eq!(content::PLAYER_COLLISION_RADIUS_WORLD, 10.0);
            // Correcting player clearance must not silently alter the
            // separately provisional tear collision/height contract.
            assert_eq!(
                (
                    room.projectile_bounds.left,
                    room.projectile_bounds.right,
                    room.projectile_bounds.top,
                    room.projectile_bounds.bottom
                ),
                (70.0, 410.0, 48.0, 222.0)
            );
        }
    }

    #[test]
    fn seed_replay_reproduces_the_complete_descriptor() {
        for seed in crate::test_seeds::samples("seed_replay_reproduces_the_complete_descriptor", 32)
        {
            let first = generate_room(seed);
            let _unrelated = generate_room(seed.wrapping_add(1));
            assert_eq!(first, generate_room(seed));
        }
    }

    #[test]
    fn seed_variation_changes_generated_patch_choices() {
        let seeds =
            crate::test_seeds::samples("seed_variation_changes_generated_patch_choices", 256);
        let maps: BTreeSet<_> = seeds
            .into_iter()
            .map(|seed| {
                let room = generate_room(seed);
                room.appearance
                    .floor_quads
                    .into_iter()
                    .chain(room.appearance.wall_quads)
                    .map(|quad| quad.source_patch_index)
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(maps.len(), 256);
    }

    #[test]
    fn all_seed_boundaries_produce_renderable_domains() {
        for seed in [0, 1, 0x7fff_ffff, 0x8000_0000, u32::MAX]
            .into_iter()
            .chain(crate::test_seeds::samples(
                "all_seed_boundaries_produce_renderable_domains",
                32,
            ))
        {
            let room = generate_room(seed);
            let appearance = &room.appearance;
            assert_eq!(appearance.floor_quads.len(), 4);
            assert_eq!(appearance.wall_quads.len(), 44);
            assert!(appearance
                .floor_quads
                .iter()
                .all(|quad| quad.source_patch_index < 6));
            assert!(appearance
                .wall_quads
                .iter()
                .all(|quad| quad.source_patch_index < 3));
            assert_eq!(
                appearance.floor_quads[2].source_patch_index,
                appearance.floor_quads[3].source_patch_index
            );
            for quad in appearance.floor_quads.iter().chain(&appearance.wall_quads) {
                assert!(
                    quad.source_rect.x >= 0.0
                        && quad.source_rect.x + quad.source_rect.width <= 234.0
                );
                assert!(
                    quad.source_rect.y >= 0.0
                        && quad.source_rect.y + quad.source_rect.height <= 156.0
                );
                assert!(
                    quad.dest_rect.x >= 0.0 && quad.dest_rect.x + quad.dest_rect.width <= 442.0
                );
                assert!(
                    quad.dest_rect.y >= 0.0 && quad.dest_rect.y + quad.dest_rect.height <= 286.0
                );
            }
            assert_eq!(appearance.columns, ROOM_COLUMNS);
            assert_eq!(appearance.rows, ROOM_ROWS);
            assert_eq!(appearance.tile_size * f64::from(ROOM_COLUMNS), 442.0);
            assert_eq!(appearance.tile_size * f64::from(ROOM_ROWS), 286.0);
            assert_eq!(appearance.origin, RoomOrigin { x: 19.0, y: -8.0 });
            assert_eq!(
                (room.left, room.right, room.top, room.bottom),
                (77.5, 402.5, 51.5, 220.5)
            );
            assert!(appearance.doors.iter().all(|door| {
                matches!(door.side.as_str(), "left" | "right")
                    && door.state == "Opened"
                    && door.x.is_finite()
                    && door.y.is_finite()
            }));
            assert!((1..=5).any(|variant| {
                appearance.fx.overlay == format!("gfx/overlays/basement/1x1_overlay_{variant}.png")
            }));
            assert!((0.0..=1.0).contains(&appearance.fx.overlay_opacity));
            assert_eq!(
                appearance.generation,
                "recovered-basement-backdrop;provisional-stage-to-room-seed-and-fx-route"
            );
        }
    }

    #[test]
    fn authored_floor_overlap_and_wall_mirroring_leave_no_geometry_gaps() {
        let room = generate_room(crate::test_seeds::one(
            "authored_floor_overlap_and_wall_mirroring_leave_no_geometry_gaps",
        ));
        let floor = &room.appearance.floor_quads;
        assert_eq!(
            floor[0].dest_rect.x + floor[0].dest_rect.width - floor[1].dest_rect.x,
            26.0
        );
        assert_eq!(
            floor[0].dest_rect.y + floor[0].dest_rect.height - floor[2].dest_rect.y,
            26.0
        );
        assert_eq!(
            floor
                .iter()
                .map(|quad| (quad.flip_x, quad.flip_y))
                .collect::<Vec<_>>(),
            vec![(false, false), (true, false), (false, true), (true, true)]
        );
        // Coverage of the floor/wall cache follows authored rectangles, not
        // independently guessed per-cell variants. Every logical pixel belongs
        // to at least one submitted quad before the tutorial/doors are drawn.
        for y in 0..286 {
            for x in 0..442 {
                assert!(
                    room.appearance
                        .floor_quads
                        .iter()
                        .chain(&room.appearance.wall_quads)
                        .any(|quad| {
                            let rect = &quad.dest_rect;
                            f64::from(x) >= rect.x
                                && f64::from(x) < rect.x + rect.width
                                && f64::from(y) >= rect.y
                                && f64::from(y) < rect.y + rect.height
                        }),
                    "uncovered cache pixel {x},{y}"
                );
            }
        }
    }

    #[test]
    fn renderer_contract_uses_camel_case_and_roundtrips() {
        let room = generate_room(crate::test_seeds::one(
            "renderer_contract_uses_camel_case_and_roundtrips",
        ));
        let value = serde_json::to_value(&room).unwrap();
        assert_eq!(value["appearance"]["tileSize"], 26.0);
        assert_eq!(value["appearance"]["doors"][0]["targetRoom"], 1);
        assert_eq!(
            value["appearance"]["fx"]["actorTint"],
            serde_json::json!([255, 255, 255])
        );
        assert!(value["appearance"]["fx"]["overlayOpacity"].is_number());
        assert_eq!(serde_json::from_value::<Room>(value).unwrap(), room);
    }
}
