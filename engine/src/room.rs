//! General room appearance with a provisional decoration stream assignment.
//!
//! Authored texture geometry is fixed; floor and wall patch choices and the
//! overlay are selected from the supplied seed using the startup xorshift.
//! Its use for decoration, draw order, and variant selection are unverified
//! hypotheses, not a claim of Rebirth room-generation parity.
//! Public startup stream source:
//! https://gist.github.com/bladecoding/5fcc1356bfb0cf26555b0ade7c4fedca
use crate::effects::{Rect, Viewport};
use crate::seed::{stage1_seed, startup_next};
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
        (0, "plain", 8, "starting-room"),
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

fn choose_wall(rng: &mut ProvisionalRoomRng, catalog: &[WallVariant]) -> u8 {
    let mut choice = rng.next() % catalog.iter().map(|variant| variant.weight).sum::<u32>();
    for variant in catalog {
        if choice < variant.weight {
            return variant.source_patch_index;
        }
        choice -= variant.weight;
    }
    unreachable!("nonempty wall catalog exhausts its total weight")
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

/// Reuse the public startup generator after its two Stage 1 initialization
/// draws. Routing subsequent draws to decoration is provisional. Zero is the
/// generator's ordinary absorbing state and receives no special handling.
struct ProvisionalRoomRng {
    state: u32,
}

impl ProvisionalRoomRng {
    fn new(seed: u32) -> Self {
        Self {
            state: stage1_seed(seed),
        }
    }

    fn next(&mut self) -> u32 {
        self.state = startup_next(self.state);
        self.state
    }
}

/// Build the provisional starting-room template for any unsigned seed.
///
/// Native public GL submissions assemble the floor from four overlapping
/// authored patches and the outer walls from forty-four quads. Bottom floor
/// halves share a patch choice in all independently observed runs. Choosing
/// those patches from the startup RNG is still an unverified hypothesis.
/// Floor interiors have six valid patches; walls have four. Atlas row three
/// contains checkerboard placeholders outside its floor interiors.
/// Collision planes and authored door anchors do not depend on the seed.
pub fn generate_room(seed: u32) -> Room {
    let mut rng = ProvisionalRoomRng::new(seed);
    let upper_left = (rng.next() % 6) as u8;
    let upper_right = (rng.next() % 6) as u8;
    let lower = (rng.next() % 6) as u8;
    // backdrops.xml declares Basement wallvariants=3, floorvariants=6.
    // Four native captures show predominantly plain walls, occasional
    // rough/cloth. 8:1:1 is an empirical distribution hypothesis, not a recovered
    // original generator. All RNG routing must continue failing parity checks.
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
    let mut wall_quads = vec![
        quad(
            0,
            rect(0.0, 0.0, 52.0, 52.0),
            rect(0.0, 0.0, 52.0, 52.0),
            false,
            false,
        ),
        quad(
            0,
            rect(0.0, 0.0, 52.0, 52.0),
            rect(390.0, 0.0, 52.0, 52.0),
            true,
            false,
        ),
        quad(
            0,
            rect(0.0, 0.0, 52.0, 52.0),
            rect(0.0, 234.0, 52.0, 52.0),
            false,
            true,
        ),
        quad(
            0,
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
                choose_wall(&mut rng, &wall_variant_catalog),
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
                choose_wall(&mut rng, &wall_variant_catalog),
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

    Room {
        left: 70.0,
        right: 410.0,
        top: 48.0,
        bottom: 222.0,
        appearance: RoomAppearance {
            backdrop: "gfx/backdrop/01_basement.png".into(),
            columns: ROOM_COLUMNS,
            rows: ROOM_ROWS,
            tile_size: ROOM_TILE_SIZE,
            origin: RoomOrigin { x: 19.0, y: -8.0 },
            backdrop_patch_size: Viewport {
                width: 234.0,
                height: 156.0,
            },
            backdrop_patch_columns: 2,
            floor_quads,
            wall_quads,
            wall_variant_catalog,
            wall_generation_evidence:
                "authored-basement-palette;provisional-empirical-weights-and-original-rng-route"
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
            generation: "public-start-rng-provisional-decoration-route".into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

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
                .all(|quad| quad.source_patch_index < 4));
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
                (70.0, 410.0, 48.0, 222.0)
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
                "public-start-rng-provisional-decoration-route"
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
