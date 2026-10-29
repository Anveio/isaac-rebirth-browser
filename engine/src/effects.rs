//! Stage effects expressed as world state rather than renderer corrections.
//!
//! The two mask passes, source rectangles, and modulation values are observable
//! public OpenGL submissions shared by independently seeded Basement captures.
//! Original decoration RNG initialization remains unverified.
use crate::{content, room::RoomFx, Bomb, Explosion, Player};
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

/// Both native mask actors snap their projected anchor to half a logical
/// pixel before authored extents and integer mask corner rounding.
/// The NPC render anchor: the f32 world-to-screen mapping, then half-pixel
/// snapping, as the ordinary NPC draw places its sprite.
fn render_anchor(position: crate::motion::WorldVector) -> Point {
    let x = (position.x - 60.0) * 0.65_f32 + 71.0;
    let y = 44.0 + (position.y - 140.0) * 0.65_f32;
    let snap = |value: f32| f64::from((value * 2.0 + 0.5).floor()) / 2.0;
    Point {
        x: snap(x),
        y: snap(y),
    }
}

fn sprite_anchor(x: f64, y: f64) -> Point {
    let snap = |value: f64| (value * 2.0).round() / 2.0;
    Point {
        x: snap(x),
        y: snap(y),
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Light {
    pub owner_id: u32,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub alpha: f64,
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
    /// The entity's own "shadow" sprite layer, drawn into the shadow surface
    /// in place of the shadow.png ellipse (destination_rect is then unused).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer: Option<ShadowLayer>,
}

/// Entity +0x30 441f50: RenderLayer of the sprite layer named "shadow" at the
/// entity's render position, main clip and overlay alike, with the sprite's
/// own scale and flip and the default colour (the sprite colour is swapped
/// out around the call). The layer's +0x64 enable byte is set only for this
/// call, so the ordinary render never draws it.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShadowLayer {
    pub actor: String,
    pub animation: String,
    pub frame: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overlay: Option<ShadowLayerOverlay>,
    pub x: f64,
    pub y: f64,
    pub flip_x: bool,
    pub scale: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShadowLayerOverlay {
    pub animation: String,
    pub frame: f64,
}

/// Whether the actor has a layer named exactly "shadow" (410870 compares the
/// whole name, case-sensitively).
pub fn has_shadow_layer(actor: &str) -> bool {
    content::SHADOW_LAYER_ACTORS.contains(&actor)
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
                ShadowDefinition {
                    entity_type: 4,
                    variant: 0,
                    size_percent: content::BOMB_SHADOW_PERCENT,
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
        let anchor = sprite_anchor(x, y);
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
                x: anchor.x - width / 2.0,
                y: anchor.y - height / 2.0,
                width,
                height,
            }),
            layer: None,
        }
    }

    pub fn sprite_layer(owner_id: u32, layer: ShadowLayer) -> Self {
        Self {
            owner_id,
            x: layer.x,
            y: layer.y,
            width: 0.0,
            height: 0.0,
            asset: "gfx/shadow.png".into(),
            authored_size_percent: 0.0,
            source_size: Viewport {
                width: content::SHADOW_SOURCE_WIDTH,
                height: content::SHADOW_SOURCE_HEIGHT,
            },
            destination_rect: Rect {
                x: layer.x,
                y: layer.y,
                width: 0.0,
                height: 0.0,
            },
            layer: Some(layer),
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
    /// The public native draw trace shows a six-ANM2-frame flash. Its gradient
    /// uses the authored off-center pivot at 150% scale, rather than assuming
    /// a symmetric sprite rectangle around the physical bomb position.
    pub fn add_explosion_lights(&mut self, explosions: &[Explosion]) {
        for explosion in explosions {
            let frame = u32::from(explosion.animation_frame);
            if frame >= 6 {
                continue;
            }
            let scale = 1.5;
            let width = content::LIGHT_WIDTH * scale;
            let height = content::LIGHT_HEIGHT * scale;
            let anchor = sprite_anchor(explosion.x, explosion.y - 1.0);
            let x = anchor.x - content::LIGHT_PIVOT_X * scale;
            let y = anchor.y - content::LIGHT_PIVOT_Y * scale;
            let progress = frame as f32 / 6.0;
            self.lights.push(Light {
                owner_id: explosion.id,
                x: x + width / 2.0,
                y: y + height / 2.0,
                radius: width / 2.0,
                alpha: f64::from(1.0_f32 - progress * progress),
                destination_rect: self.mask_raster.rectangle(Rect {
                    x,
                    y,
                    width,
                    height,
                }),
            });
        }
    }
    /// Planted bombs use authored entity shadow scales and the independently
    /// observed sprite ground anchor, then the same signed raster as players.
    pub fn add_bomb_shadows(&mut self, bombs: &[Bomb]) {
        for bomb in bombs
            .iter()
            .filter(|bomb| bomb.motion.lifetime == crate::bombs::BombLifetime::Alive)
        {
            self.shadow_casters.push(ShadowCaster::authored(
                bomb.id,
                bomb.x,
                bomb.y - 1.0,
                if matches!(bomb.variant, 3 | 4) {
                    content::TROLL_BOMB_SHADOW_PERCENT
                } else {
                    content::BOMB_SHADOW_PERCENT
                },
                &self.mask_raster,
            ));
        }
    }

    /// Tear shadow follows ground prediction, separately from the height used
    /// by its sprite. Fresh regular tears use authored shadow size8.
    pub fn add_tear_shadows(&mut self, tears: &[crate::Tear]) {
        for tear in tears {
            self.shadow_casters.push(ShadowCaster::authored(
                tear.id,
                f64::from(tear.motion.presented_position.x) * crate::motion::WORLD_SCALE + 32.0,
                f64::from(tear.motion.presented_position.y) * crate::motion::WORLD_SCALE - 47.0,
                content::TEAR_SHADOW_PERCENT,
                &self.mask_raster,
            ));
        }
    }

    pub fn add_pickup_shadows(&mut self, pickups: &[crate::Pickup]) {
        for pickup in pickups
            .iter()
            .filter(|pickup| !pickup.collected && !pickup.removed)
        {
            if has_shadow_layer(&pickup.actor) {
                let anchor = sprite_anchor(pickup.x, pickup.y - 1.0);
                self.shadow_casters.push(ShadowCaster::sprite_layer(
                    pickup.id,
                    ShadowLayer {
                        actor: pickup.actor.clone(),
                        animation: pickup.animation.clone(),
                        frame: pickup.animation_frame,
                        overlay: pickup.overlay.as_ref().map(|animation| ShadowLayerOverlay {
                            animation: animation.clone(),
                            frame: f64::from(pickup.overlay_frame),
                        }),
                        x: anchor.x,
                        y: anchor.y,
                        flip_x: false,
                        scale: 1.0,
                    },
                ));
                continue;
            }
            if pickup.shadow_size <= 0.0 {
                continue;
            }
            self.shadow_casters.push(ShadowCaster::authored(
                pickup.id,
                pickup.x,
                pickup.y - 1.0,
                pickup.shadow_size,
                &self.mask_raster,
            ));
        }
    }

    pub fn add_familiar_shadows(&mut self, familiars: &[crate::familiars::Familiar]) {
        for familiar in familiars
            .iter()
            .filter(|actor| actor.visible && actor.active && !actor.removed)
        {
            self.shadow_casters.push(ShadowCaster::authored(
                familiar.id,
                f64::from(familiar.presented_position.x) * crate::motion::WORLD_SCALE + 32.0,
                f64::from(familiar.presented_position.y) * crate::motion::WORLD_SCALE - 47.0,
                f64::from(familiar.shadow_size),
                &self.mask_raster,
            ));
        }
    }

    /// DrawShadows 452920 draws every visible, live entity whose shadow size
    /// (+30c, XML percent / 100) times sprite Scale.X (+2c4) is at least
    /// 0.001, at its render position without PositionOffset. Champions carry
    /// the ×1.15 sprite scale.
    pub fn add_npc_shadows(&mut self, npcs: &[crate::npcs::Npc]) {
        for npc in npcs
            .iter()
            .filter(|npc| npc.visible && !npc.dead && !npc.removed)
        {
            let actor = if npc.monstro.is_some() {
                "gfx/020.000_monstro.anm2"
            } else {
                npc.actor.as_str()
            };
            if has_shadow_layer(actor) {
                let scale = npc
                    .sprite_scale
                    .or(npc.champion.as_ref().map(|champion| champion.scale))
                    .unwrap_or(1.0);
                let anchor = render_anchor(npc.presented_position);
                self.shadow_casters.push(ShadowCaster::sprite_layer(
                    npc.id,
                    ShadowLayer {
                        actor: actor.into(),
                        animation: npc.animation.clone(),
                        frame: f64::from(npc.animation_clock),
                        overlay: npc.overlay.as_ref().map(|overlay| ShadowLayerOverlay {
                            animation: overlay.animation.clone(),
                            frame: f64::from(overlay.clock),
                        }),
                        x: anchor.x,
                        y: anchor.y,
                        flip_x: npc
                            .monstro
                            .as_ref()
                            .map_or(npc.flip_x, |monstro| monstro.flip_x),
                        scale: f64::from(scale),
                    },
                ));
                continue;
            }
            let scale = npc.champion.as_ref().map_or(1.0, |champion| champion.scale);
            let size = f64::from(npc.shadow_size * scale);
            if size / 100.0 < 0.001 {
                continue;
            }
            self.shadow_casters.push(ShadowCaster::authored(
                npc.id,
                f64::from(npc.presented_position.x) * crate::motion::WORLD_SCALE + 32.0,
                f64::from(npc.presented_position.y) * crate::motion::WORLD_SCALE - 47.0,
                size,
                &self.mask_raster,
            ));
        }
    }

    /// Light and sprite anchors remain separate in the current logical model.
    /// Observed raster geometry alone does not identify native physical units.
    /// `world_offset` is the world layer's CEA00 shake plus Room+15B8
    /// scroll; the fx overlay is a world sprite that moves with both.
    pub fn basement(
        player: &Player,
        fx: &RoomFx,
        run_tick: u32,
        world_offset: crate::motion::WorldVector,
    ) -> Self {
        let offset = Point {
            x: (player.x + player.render_offset.x - fx.overlay_anchor.x) * fx.overlay_parallax.x,
            y: (player.y + player.render_offset.y - fx.overlay_anchor.y) * fx.overlay_parallax.y,
        };
        let mask_raster = Raster {
            pixel_size: 1.0,
            rounding: Rounding::HalfAwayFromZero,
        };
        let light_anchor = sprite_anchor(player.x, player.y - 1.0);
        let light_rect = mask_raster.rectangle(Rect {
            x: light_anchor.x - content::LIGHT_PIVOT_X,
            y: light_anchor.y - content::LIGHT_PIVOT_Y,
            width: content::LIGHT_WIDTH,
            height: content::LIGHT_HEIGHT,
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
                    x: content::LIGHT_CROP_X,
                    y: content::LIGHT_CROP_Y,
                    width: content::LIGHT_WIDTH,
                    height: content::LIGHT_HEIGHT,
                },
            },
            lights: vec![Light {
                owner_id: 0,
                x: player.x,
                y: player.y,
                radius: 150.0,
                alpha: 1.0,
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
                // The authored overlay keeps its natural logical texture
                // size: the viewport crops it, never scales it to a room.
                // 5473d0 places it at the Room centre plus CEA00 + Room+15B8,
                // so the crop moves against the world offset.
                source_crop: Rect {
                    x: fx.overlay_origin.x + offset.x - f64::from(world_offset.x),
                    y: fx.overlay_origin.y + offset.y - f64::from(world_offset.y),
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
