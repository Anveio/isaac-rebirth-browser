//! Native-world grid queries, regular blast selection and player penetration.
//!
//! Room descriptors include the boundary ring. These kernels reconstruct the
//! source predicates given resolved grid objects; they do not resolve authored
//! spawn selectors, implement grid drops or infer a floor graph. Source evidence
//! and independently captured fixtures live in docs/combat-evidence.md.
use crate::motion::WorldVector;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The native pathfinder reserves 448 room-cell entries (28 × 16).
pub const MAX_GRID_CELLS: usize = 448;
const MAX_COLLISION_POINTS: u16 = 128;
// A resource bound for diagnostic calls, not a recovered gameplay radius tier.
const MAX_QUERY_RADIUS: f32 = 4096.0;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GridCell {
    pub selector: u32,
    pub entity: Option<GridEntity>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GridEntity {
    pub kind: u32,
    pub variant: u32,
    pub state: u32,
    pub collision_class: u32,
    pub spawn_seed: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GridDescriptor {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<GridCell>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid {
    descriptor: GridDescriptor,
}

impl Grid {
    /// Collision geometry of the ordinary 1x1 room's boundary ring. This
    /// profile has no admitted grid actors or random spawn seeds; floor/room
    /// admission supplies those separately, including live door apertures.
    pub fn closed_1x1_boundary() -> Self {
        let width = 15;
        let height = 9;
        let cells = (0..width * height)
            .map(|index| GridCell {
                selector: 0,
                entity: (index < width
                    || index >= width * (height - 1)
                    || index % width == 0
                    || index % width == width - 1)
                    .then_some(GridEntity {
                        kind: 15,
                        variant: 0,
                        state: 0,
                        collision_class: 4,
                        spawn_seed: 0,
                    }),
            })
            .collect();
        Self {
            descriptor: GridDescriptor {
                width,
                height,
                cells,
            },
        }
    }

    pub fn new(descriptor: GridDescriptor) -> Result<Self, String> {
        let count = usize::from(descriptor.width) * usize::from(descriptor.height);
        if descriptor.width == 0
            || descriptor.height == 0
            || count > MAX_GRID_CELLS
            || descriptor.cells.len() != count
        {
            return Err("grid dimensions require exactly 1..448 cells".into());
        }
        Ok(Self { descriptor })
    }

    pub fn descriptor(&self) -> &GridDescriptor {
        &self.descriptor
    }

    /// Room::GetGridIndex uses floorf on the original 40-unit lattice.
    pub fn index(&self, point: WorldVector) -> Option<usize> {
        if !point.is_finite() {
            return None;
        }
        let column = ((point.x - 40.0) / 40.0 + 0.5).floor();
        let row = ((point.y - 120.0) / 40.0 + 0.5).floor();
        if column < 0.0
            || row < 0.0
            || column >= f32::from(self.descriptor.width)
            || row >= f32::from(self.descriptor.height)
        {
            return None;
        }
        Some(column as usize + row as usize * usize::from(self.descriptor.width))
    }

    pub fn position(&self, index: usize) -> Option<WorldVector> {
        if index >= self.descriptor.cells.len() {
            return None;
        }
        let width = usize::from(self.descriptor.width);
        Some(WorldVector {
            x: 40.0 + (index % width) as f32 * 40.0,
            y: 120.0 + (index / width) as f32 * 40.0,
        })
    }

    pub fn collision_at(&self, point: WorldVector) -> u32 {
        self.index(point).map_or(4, |index| self.collision(index))
    }

    pub fn collision(&self, index: usize) -> u32 {
        let Some(cell) = self.descriptor.cells.get(index) else {
            return 4;
        };
        if let Some(entity) = &cell.entity {
            return entity.collision_class;
        }
        match cell.selector {
            3000 => 1,
            1000.. => 3,
            _ => 0,
        }
    }

    /// Source CheckLine(mode=2): ordinary rocks permit the line; kind3 blocks
    /// it, kind16 doors bypass collision-class rejection, and walls block it.
    /// The source advances ten units before testing, including the final step
    /// that may overshoot the endpoint by less than ten units.
    fn blast_line_clear(&self, from: WorldVector, to: WorldVector) -> bool {
        let delta = WorldVector {
            x: from.x - to.x,
            y: from.y - to.y,
        };
        let distance = (delta.x * delta.x + delta.y * delta.y).sqrt();
        if distance <= 0.0 {
            return true;
        }
        let step = WorldVector {
            x: delta.x / distance * 10.0,
            y: delta.y / distance * 10.0,
        };
        let mut point = from;
        let mut travelled = 0.0;
        while travelled < distance {
            point.x -= step.x;
            point.y -= step.y;
            let Some(index) = self.index(point) else {
                return false;
            };
            let kind = self.descriptor.cells[index]
                .entity
                .as_ref()
                .map(|entity| entity.kind);
            if kind == Some(3) {
                return false;
            }
            if kind != Some(16) && !matches!(self.collision(index), 0..=3) {
                return false;
            }
            travelled += 10.0;
        }
        true
    }

    /// Sorted source grid candidates, prior to each object's virtual Destroy.
    /// This differs from the explosion's strict entity-center radius predicate.
    pub fn blast_candidates(&self, origin: WorldVector, radius: f32) -> Result<Vec<usize>, String> {
        if !origin.is_finite() || !radius.is_finite() || radius <= 0.0 || radius > MAX_QUERY_RADIUS
        {
            return Err("blast query requires a finite origin and radius in (0,4096]".into());
        }
        let rays = (radius * 8.0 / 40.0).ceil() as u32;
        let angle_step = (std::f64::consts::TAU / f64::from(rays)) as f32;
        let mut selected = BTreeSet::new();
        if let Some(index) = self.index(origin) {
            selected.insert(index);
        }
        let mut angle = 0.0_f32;
        let mut direction = WorldVector { x: 1.0, y: 0.0 };
        for _ in 0..rays {
            let mut distance = 0.0_f32;
            while distance < radius {
                distance = (distance + 20.0).min(radius);
                let point = WorldVector {
                    x: origin.x + direction.x * distance,
                    y: origin.y + direction.y * distance,
                };
                let Some(index) = self.index(point) else {
                    continue;
                };
                let Some(entity) = &self.descriptor.cells[index].entity else {
                    continue;
                };
                selected.insert(index);
                if entity.kind == 3 {
                    break;
                }
            }
            angle += angle_step;
            direction = WorldVector {
                x: angle.cos(),
                y: angle.sin(),
            };
        }
        Ok(selected
            .into_iter()
            .filter(|&index| {
                self.blast_line_clear(
                    origin,
                    self.position(index).expect("selected index is valid"),
                )
            })
            .collect())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CollisionGeometry {
    pub radius: f32,
    pub scale: WorldVector,
    pub points: u16,
}

/// Source collision points are cached when geometry changes, not rebuilt each
/// gameplay update. Their order matters to the f32 sum of the contact normal.
#[derive(Clone, Debug, PartialEq)]
pub struct Collider {
    geometry: CollisionGeometry,
    samples: Vec<WorldVector>,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GridContact {
    pub normal: WorldVector,
    pub previous_velocity: WorldVector,
    pub velocity_changed: bool,
}

impl Collider {
    pub fn new(geometry: CollisionGeometry) -> Result<Self, String> {
        if !geometry.radius.is_finite()
            || geometry.radius < 0.0
            || !geometry.scale.is_finite()
            || geometry.scale.x < 0.0
            || geometry.scale.y < 0.0
            || geometry.points > MAX_COLLISION_POINTS
            || geometry.radius * geometry.scale.x > MAX_QUERY_RADIUS
            || geometry.radius * geometry.scale.y > MAX_QUERY_RADIUS
        {
            return Err(
                "collision geometry requires finite nonnegative sizes and at most128 points".into(),
            );
        }
        let mut samples = Vec::with_capacity(usize::from(geometry.points));
        for index in (0..geometry.points)
            .step_by(2)
            .chain((1..geometry.points).step_by(2))
        {
            let doubled = f32::from(index) + f32::from(index);
            let angle =
                (f64::from(doubled) * std::f64::consts::PI / f64::from(geometry.points)) as f32;
            let sample = WorldVector {
                x: angle.cos() * (geometry.radius * geometry.scale.x),
                y: angle.sin() * (geometry.radius * geometry.scale.y),
            };
            if !sample.is_finite() {
                return Err("collision point geometry overflowed".into());
            }
            samples.push(sample);
        }
        Ok(Self { geometry, samples })
    }

    pub fn geometry(&self) -> CollisionGeometry {
        self.geometry
    }
    pub fn samples(&self) -> &[WorldVector] {
        &self.samples
    }

    /// Source displacement admission used before mass-weighted player pushing.
    pub fn blocks_player_shift(
        &self,
        grid: &Grid,
        position: WorldVector,
        shift: WorldVector,
    ) -> bool {
        let point = position + shift;
        self.samples
            .iter()
            .any(|&sample| !matches!(grid.collision_at(point + sample), 0 | 5))
    }

    /// Fresh player ground-collision class5. Both native player update phases
    /// call this solver. Render prediction may request position-only correction.
    /// The caller handles source room-bound clamping and special grid damage.
    pub fn resolve_player(
        &self,
        grid: &Grid,
        position: &mut WorldVector,
        velocity: &mut WorldVector,
        preview: bool,
    ) -> Result<Option<GridContact>, String> {
        if !position.is_finite() || !velocity.is_finite() {
            return Err("player collision requires finite world vectors".into());
        }
        let mut normal = WorldVector { x: 0.0, y: 0.0 };
        for &sample in &self.samples {
            if !matches!(grid.collision_at(*position + sample), 0 | 5) {
                normal = normal + sample;
            }
        }
        let length_squared = normal.x * normal.x + normal.y * normal.y;
        if length_squared < 0.001 {
            return Ok(None);
        }
        let inverse_length = 1.0 / length_squared.sqrt();
        normal = normal * inverse_length;
        let previous_velocity = *velocity;
        let inward = -normal.x * velocity.x - normal.y * velocity.y;
        let velocity_changed = inward < 0.0 && !preview;
        let next_velocity = if velocity_changed {
            *velocity + normal * (inward * 0.9 - 0.1)
        } else {
            *velocity
        };
        let speed_squared = next_velocity.x * next_velocity.x + next_velocity.y * next_velocity.y;
        if !next_velocity.is_finite() || !speed_squared.is_finite() {
            return Err("player collision overflowed velocity".into());
        }
        let speed = speed_squared.sqrt().max(2.0);
        let scaled_normal = WorldVector {
            x: normal.x * self.geometry.scale.x,
            y: normal.y * self.geometry.scale.y,
        };
        let radius = (scaled_normal.x * scaled_normal.x + scaled_normal.y * scaled_normal.y).sqrt()
            * self.geometry.radius;
        let mut upper = radius;
        let mut lower = radius - speed;
        for _ in 0..12 {
            let middle = (upper - lower) * 0.5 + lower;
            if matches!(grid.collision_at(*position + normal * middle), 0 | 5) {
                lower = middle;
            } else {
                upper = middle;
            }
        }
        let next_position = *position + normal * (lower - radius);
        if !next_position.is_finite() {
            return Err("player collision overflowed position".into());
        }
        *position = next_position;
        *velocity = next_velocity;
        Ok(Some(GridContact {
            normal,
            previous_velocity,
            velocity_changed,
        }))
    }
}
