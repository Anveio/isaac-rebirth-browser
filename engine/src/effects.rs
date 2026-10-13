//! Stage effects expressed as world state rather than renderer corrections.
//!
//! The two mask passes, source rectangles, and modulation values are observable
//! public OpenGL submissions shared by independently seeded Basement captures.
//! Original decoration RNG initialization remains unverified.
use crate::{content, room::RoomFx, Player};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Viewport {
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LightGradient {
    pub asset: String,
    pub crop: Rect,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Light {
    pub owner_id: u32,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub destination_rect: Rect,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShadowCaster {
    pub owner_id: u32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub asset: String,
    pub authored_size_percent: f64,
    pub source_size: Viewport,
    pub destination_rect: Rect,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShadowDefinition {
    pub entity_type: u32,
    pub variant: u32,
    pub size_percent: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShadowCatalog {
    pub asset: String,
    pub source_size: Viewport,
    pub definitions: Vec<ShadowDefinition>,
}

impl ShadowCatalog {
    fn authored() -> Self {
        Self {
            asset: "gfx/shadow.png".into(),
            source_size: Viewport {
                width: content::SHADOW_SOURCE_WIDTH,
                height: content::SHADOW_SOURCE_HEIGHT,
            },
            definitions: vec![
                ShadowDefinition {
                    entity_type: 1,
                    variant: 0,
                    size_percent: content::PLAYER_SHADOW_PERCENT,
                },
                ShadowDefinition {
                    entity_type: 2,
                    variant: 0,
                    size_percent: content::TEAR_SHADOW_PERCENT,
                },
            ],
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Rounding {
    HalfAwayFromZero,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Raster {
    pub pixel_size: f64,
    pub rounding: Rounding,
}

impl Raster {
    /// Round each submitted corner independently. Rounding an origin and size
    /// separately changes widths at fractional centers and negative half ties.
    pub fn rectangle(&self, rect: Rect) -> Rect {
        let snap = |value: f64| match self.rounding {
            Rounding::HalfAwayFromZero => (value / self.pixel_size).round() * self.pixel_size,
        };
        let x = snap(rect.x);
        let y = snap(rect.y);
        let right = snap(rect.x + rect.width);
        let bottom = snap(rect.y + rect.height);
        Rect {
            x,
            y,
            width: right - x,
            height: bottom - y,
        }
    }
}

impl ShadowCaster {
    pub fn authored(owner_id: u32, x: f64, y: f64, size_percent: f64, raster: &Raster) -> Self {
        let width = content::SHADOW_SOURCE_WIDTH * size_percent / 100.0;
        let height = content::SHADOW_SOURCE_HEIGHT * size_percent / 100.0;
        Self {
            owner_id,
            x,
            y,
            width,
            height,
            asset: "gfx/shadow.png".into(),
            authored_size_percent: size_percent,
            source_size: Viewport {
                width: content::SHADOW_SOURCE_WIDTH,
                height: content::SHADOW_SOURCE_HEIGHT,
            },
            destination_rect: raster.rectangle(Rect {
                x: x - width / 2.0,
                y: y - height / 2.0,
                width,
                height,
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum BlendFactor {
    One,
    Zero,
    SourceAlpha,
    OneMinusSourceAlpha,
    DestinationColor,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Blend {
    /// The native passes use the same factors for RGB and alpha.
    pub source: BlendFactor,
    pub destination: BlendFactor,
}

impl Blend {
    fn source_alpha() -> Self {
        Self {
            source: BlendFactor::SourceAlpha,
            destination: BlendFactor::OneMinusSourceAlpha,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Sampler {
    pub filter: String,
    pub wrap: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Overlay {
    pub asset: String,
    pub source_crop: Rect,
    pub offset: Point,
    #[serde(rename = "modulationRGBA")]
    pub modulation_rgba: [f64; 4],
    pub blend: Blend,
    pub simulation_tick: u32,
    pub motion_evidence: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StageEffects {
    pub viewport: Viewport,
    pub mask_raster: Raster,
    pub shadow_catalog: ShadowCatalog,
    pub ambient_color: [f64; 4],
    pub light_gradient: LightGradient,
    pub lights: Vec<Light>,
    pub shadow_casters: Vec<ShadowCaster>,
    pub ground_shadow_mask_blend: Blend,
    pub lighting_mask_blend: Blend,
    pub ground_shadow_composite_blend: Blend,
    pub lighting_composite_blend: Blend,
    pub sampler: Sampler,
    #[serde(rename = "groundShadowCompositeRGBA")]
    pub ground_shadow_composite_rgba: [f64; 4],
    #[serde(rename = "lightingCompositeRGBA")]
    pub lighting_composite_rgba: [f64; 4],
    pub actor_tint: [u8; 3],
    pub overlay: Overlay,
}

impl StageEffects {
    /// Light and sprite anchors remain separate in the current logical model.
    /// Observed raster geometry alone does not identify native physical units.
    pub fn basement(player: &Player, fx: &RoomFx, run_tick: u32) -> Self {
        let offset = Point {
            x: (player.x + player.render_offset.x - fx.overlay_anchor.x) * fx.overlay_parallax.x,
            y: (player.y + player.render_offset.y - fx.overlay_anchor.y) * fx.overlay_parallax.y,
        };
        let mask_raster = Raster {
            pixel_size: 1.0,
            rounding: Rounding::HalfAwayFromZero,
        };
        let light_rect = mask_raster.rectangle(Rect {
            x: player.x - 150.0,
            y: player.y - 150.0,
            width: 300.0,
            height: 300.0,
        });
        let shadow = ShadowCaster::authored(
            0,
            player.x + player.render_offset.x,
            player.y + player.render_offset.y,
            content::PLAYER_SHADOW_PERCENT,
            &mask_raster,
        );
        Self {
            viewport: Viewport {
                width: 480.0,
                height: 270.0,
            },
            mask_raster,
            shadow_catalog: ShadowCatalog::authored(),
            ambient_color: [0.1, 0.15, 0.25, 1.0],
            light_gradient: LightGradient {
                asset: "gfx/backdrop/light_gradient.png".into(),
                crop: Rect {
                    x: 100.0,
                    y: 0.0,
                    width: 300.0,
                    height: 300.0,
                },
            },
            lights: vec![Light {
                owner_id: 0,
                x: player.x,
                y: player.y,
                radius: 150.0,
                destination_rect: light_rect,
            }],
            shadow_casters: vec![shadow],
            ground_shadow_composite_rgba: [1.0, 1.0, 1.0, 0.24],
            lighting_composite_rgba: [0.3, 0.3, 0.3, 0.3],
            ground_shadow_mask_blend: Blend {
                source: BlendFactor::One,
                destination: BlendFactor::Zero,
            },
            lighting_mask_blend: Blend::source_alpha(),
            ground_shadow_composite_blend: Blend::source_alpha(),
            lighting_composite_blend: Blend {
                source: BlendFactor::DestinationColor,
                destination: BlendFactor::OneMinusSourceAlpha,
            },
            sampler: Sampler {
                filter: "nearest".into(),
                wrap: "clamp-to-edge".into(),
            },
            actor_tint: [255, 255, 255],
            overlay: Overlay {
                asset: fx.overlay.clone(),
                // Authored overlays are 663 × 429. Their natural logical
                // texture size is preserved by cropping a viewport, not scaling
                // the image to a room. The seed-dependent origin is provisional.
                source_crop: Rect {
                    x: fx.overlay_origin.x + offset.x,
                    y: fx.overlay_origin.y + offset.y,
                    width: 480.0,
                    height: 270.0,
                },
                offset,
                modulation_rgba: [1.0, 1.0, 1.0, 1.0],
                blend: Blend::source_alpha(),
                simulation_tick: run_tick,
                // Both axes agree with authored fxlayers.xml and independent
                // controlled native movement. No time animation is introduced.
                motion_evidence: fx.overlay_motion_evidence.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;

    #[test]
    fn logical_light_and_authored_shadow_follow_separate_render_anchors() {
        let game = Game::new(crate::test_seeds::one(
            "logical_light_and_authored_shadow_follow_separate_render_anchors",
        ))
        .unwrap();
        let mut player = game.state.player;
        let first = StageEffects::basement(&player, &game.state.room.appearance.fx, 0);
        assert_eq!(first.lights[0].y, 214.0);
        assert_eq!(first.shadow_casters[0].y, 213.0);
        player.x += 57.125;
        player.y -= 29.75;
        player.render_offset.x = -2.0;
        let moved = StageEffects::basement(&player, &game.state.room.appearance.fx, 23);
        assert_eq!(moved.lights[0].x - first.lights[0].x, 57.125);
        assert_eq!(moved.lights[0].y - first.lights[0].y, -29.75);
        assert_eq!(moved.shadow_casters[0].x - moved.lights[0].x, -2.0);
        assert_eq!(moved.shadow_casters[0].y - moved.lights[0].y, -1.0);
        assert_eq!(moved.light_gradient, first.light_gradient);
    }

    #[test]
    fn independent_signed_corner_rounding_matches_negative_half_light_bounds() {
        let raster = Raster {
            pixel_size: 1.0,
            rounding: Rounding::HalfAwayFromZero,
        };
        let bounds = raster.rectangle(Rect {
            x: -24.5,
            y: 7.0,
            width: 300.0,
            height: 300.0,
        });
        assert_eq!(
            bounds,
            Rect {
                x: -25.0,
                y: 7.0,
                width: 301.0,
                height: 300.0
            }
        );
        // The reflected primitive checks the other negative half boundary;
        // rounding an origin plus an unchanged width gives the wrong edge.
        let reflected = raster.rectangle(Rect {
            x: -275.5,
            y: -307.0,
            width: 300.0,
            height: 300.0,
        });
        assert_eq!(
            reflected,
            Rect {
                x: -276.0,
                y: -307.0,
                width: 301.0,
                height: 300.0
            }
        );
        let quarter = Raster {
            pixel_size: 0.25,
            rounding: Rounding::HalfAwayFromZero,
        };
        assert_eq!(
            quarter.rectangle(Rect {
                x: -0.125,
                y: 0.125,
                width: 1.0,
                height: 1.0
            }),
            Rect {
                x: -0.25,
                y: 0.25,
                width: 1.25,
                height: 1.0
            }
        );
    }

    #[test]
    fn authored_shadow_scale_and_subpixel_position_change_destination_width() {
        let raster = Raster {
            pixel_size: 1.0,
            rounding: Rounding::HalfAwayFromZero,
        };
        let centered =
            ShadowCaster::authored(0, 240.0, 213.0, content::PLAYER_SHADOW_PERCENT, &raster);
        assert_eq!(
            centered.destination_rect,
            Rect {
                x: 230.0,
                y: 209.0,
                width: 20.0,
                height: 8.0
            }
        );
        let half = ShadowCaster::authored(0, 320.5, 213.0, content::PLAYER_SHADOW_PERCENT, &raster);
        assert_eq!(
            half.destination_rect,
            Rect {
                x: 311.0,
                y: 209.0,
                width: 19.0,
                height: 8.0
            }
        );
        assert_eq!(half.width, centered.width);
        assert_eq!(half.height, centered.height);
        assert_ne!(half.width, half.destination_rect.width);
        let base_tear =
            ShadowCaster::authored(1, 332.0, 211.0, content::TEAR_SHADOW_PERCENT, &raster);
        assert_eq!(
            base_tear.destination_rect,
            Rect {
                x: 327.0,
                y: 209.0,
                width: 10.0,
                height: 4.0
            }
        );
        assert_eq!(base_tear.width, centered.width * 0.5);
        assert_eq!(base_tear.height, centered.height * 0.5);
    }

    #[test]
    fn natural_overlay_crop_and_tick_survive_state_roundtrip() {
        for seed in
            crate::test_seeds::samples("natural_overlay_crop_and_tick_survive_state_roundtrip", 32)
        {
            let game = Game::new(seed).unwrap();
            let effects =
                StageEffects::basement(&game.state.player, &game.state.room.appearance.fx, 72);
            let crop = &effects.overlay.source_crop;
            assert_eq!((crop.width, crop.height), (480.0, 270.0));
            assert!((91.5..=191.5).contains(&(crop.x - effects.overlay.offset.x)));
            assert!((79.5..=179.5).contains(&(crop.y - effects.overlay.offset.y)));
            assert_eq!(effects.overlay.simulation_tick, 72);
            assert_eq!(effects.actor_tint, [255; 3]);
            let json = serde_json::to_value(&effects).unwrap();
            assert_eq!(json["shadowCasters"][0]["ownerId"], 0);
            assert_eq!(
                json["lightingCompositeRGBA"],
                serde_json::json!([0.3, 0.3, 0.3, 0.3])
            );
            assert_eq!(
                serde_json::from_value::<StageEffects>(json).unwrap(),
                effects
            );
        }
    }

    #[test]
    fn overlay_parallax_uses_both_axes_and_is_independent_of_clock() {
        let game = Game::new(crate::test_seeds::one(
            "overlay_parallax_uses_both_axes_and_is_independent_of_clock",
        ))
        .unwrap();
        let mut player = game.state.player;
        let initial = StageEffects::basement(&player, &game.state.room.appearance.fx, 0);
        let later = StageEffects::basement(&player, &game.state.room.appearance.fx, 1000);
        assert_eq!(initial.overlay.source_crop, later.overlay.source_crop);
        player.x += 80.0;
        let moved = StageEffects::basement(&player, &game.state.room.appearance.fx, 1000);
        assert_eq!(moved.overlay.offset.x - initial.overlay.offset.x, 2.0);
        assert_eq!(
            moved.overlay.source_crop.x - initial.overlay.source_crop.x,
            2.0
        );
        assert_eq!(moved.overlay.source_crop.y, initial.overlay.source_crop.y);
        player.x += 0.5;
        let subpixel = StageEffects::basement(&player, &game.state.room.appearance.fx, 1000);
        assert!(
            (subpixel.overlay.offset.x - initial.overlay.offset.x - 2.0125).abs()
                < f64::EPSILON * subpixel.overlay.offset.x.abs() * 4.0
        );
        player.y -= 62.5;
        let vertical = StageEffects::basement(&player, &game.state.room.appearance.fx, 1000);
        assert_eq!(
            vertical.overlay.offset.y - initial.overlay.offset.y,
            -1.5625
        );
        assert_eq!(
            vertical.overlay.source_crop.y - initial.overlay.source_crop.y,
            -1.5625
        );
        player.render_offset.x -= 1.0;
        player.render_offset.y -= 3.0;
        let shifted = StageEffects::basement(&player, &game.state.room.appearance.fx, 1000);
        assert!((shifted.overlay.offset.x - vertical.overlay.offset.x + 0.025).abs() < 1e-14);
        assert!((shifted.overlay.offset.y - vertical.overlay.offset.y + 0.075).abs() < 1e-14);
    }
}
