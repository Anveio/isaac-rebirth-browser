//! Authored group admission uses the room's local spawn stream, never global MT.
//! Selectors for unsupported actors and grids stay typed for their owning systems.
use crate::{motion::WorldVector, mt::uint_to_uniform, room_assets::RoomAsset};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SpawnCategory {
    Entity,
    Grid,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpawnSelector {
    pub group_index: u16,
    pub grid_index: u16,
    pub category: SpawnCategory,
    #[serde(rename = "type")]
    pub entity_type: u16,
    pub variant: u16,
    pub subtype: u16,
    pub position: WorldVector,
    /// RNG11 state after this authored group's weighted draw.
    pub group_seed: u32,
    /// Entities advance a new RNG35 stream once; grids retain the group seed.
    pub spawn_seed: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoomSpawns {
    pub tinted_rock_index: u16,
    pub selectors: Vec<SpawnSelector>,
    pub final_spawn_state: u32,
}

pub(crate) fn next35(mut state: u32) -> u32 {
    state ^= state >> 5;
    state ^= state << 9;
    state ^= state >> 7;
    state
}

fn next11(mut state: u32) -> u32 {
    state ^= state >> 2;
    state ^= state << 7;
    state ^= state >> 7;
    state
}

fn next7(mut state: u32) -> u32 {
    state ^= state >> 1;
    state ^= state << 21;
    state ^= state >> 20;
    state
}

/// Room spawnConfig5c5240's alternate-variant rolls for NPC types 14..88. A
/// local RNG7 on the group seed may promote the authored entry, e.g. Pooter
/// to Super Pooter. The pickup, slot, trapdoor and fire rewrites of the same
/// routine belong to their owning systems.
fn alternate_variant(entity_type: u16, variant: u16, group_seed: u32) -> u16 {
    // 0x67aae0, 0x67ab40 and 0x67aba0 flag tables, indexed by type-14.
    const PRIMARY: [u16; 6] = [14, 16, 24, 25, 26, 27];
    const SECONDARY: [u16; 4] = [41, 53, 55, 60];
    const TERTIARY: [u16; 9] = [29, 30, 38, 39, 42, 44, 57, 61, 88];
    let mut state = group_seed;
    let mut bounded = |range: u32| {
        state = next7(state);
        state % range
    };
    let mut variant = variant;
    if matches!(entity_type, 15 | 16 | 26 | 30 | 39 | 55 | 88) && bounded(100) == 0 {
        variant = 2;
    }
    let primary = PRIMARY.contains(&entity_type);
    let secondary = SECONDARY.contains(&entity_type);
    let tertiary = TERTIARY.contains(&entity_type);
    // 5c530a, 5c5320 and 5c5336: one, two or three escalating chances.
    let first = (primary && !secondary) || (!primary && !secondary && tertiary);
    if !(primary || secondary || tertiary) {
        return variant;
    }
    if first && bounded(21) == 0 {
        return 1;
    }
    if !tertiary && bounded(25) == 0 {
        return 1;
    }
    if bounded(100) == 0 {
        return 1;
    }
    variant
}

/// Fresh room Init 0x5cf190..0x5cf2ee admits groups in authored order. The
/// two initial draws choose the tinted-rock candidate, even without rocks.
pub fn select(asset: &RoomAsset, spawn_seed: u32) -> Result<RoomSpawns, String> {
    if spawn_seed == 0 {
        return Err("room spawn seed must be nonzero".into());
    }
    let width = u16::from(asset.template.width);
    let height = u16::from(asset.template.height);
    if width == 0 || height == 0 || asset.spawns.len() > usize::from(u16::MAX) {
        return Err("room spawn dimensions or group count exceed source bounds".into());
    }
    let grid_width = width + 2;
    if usize::from(grid_width) * usize::from(height + 2) > crate::mt::PADDED_GRID_CELLS {
        return Err("room spawn grid exceeds the source padded grid capacity".into());
    }
    let mut state = next11(spawn_seed);
    let tinted_x = state % u32::from(width) + 1;
    state = next11(state);
    let tinted_y = state % u32::from(height) + 1;
    let tinted_rock_index = (tinted_x + tinted_y * u32::from(grid_width)) as u16;
    let mut selectors = Vec::new();
    for (group_index, group) in asset.spawns.iter().enumerate() {
        state = next11(state);
        if group.entries.is_empty() {
            continue;
        }
        if group.x < -1 || group.y < -1 || group.x > width as i16 || group.y > height as i16 {
            return Err("authored spawn position outside the padded room grid".into());
        }
        let mut total = 0.0_f32;
        for entry in &group.entries {
            if !entry.weight.is_finite() || entry.weight < 0.0 {
                return Err("authored spawn weight must be finite and nonnegative".into());
            }
            total += entry.weight;
        }
        if !total.is_finite() {
            return Err("authored spawn weight overflow".into());
        }
        let target = uint_to_uniform(state) * total;
        let mut cumulative = 0.0_f32;
        // Source picker 0x5d49d0 falls back to its first entry if no strict
        // cumulative comparison wins, including a float rounded up to one.
        let entry = group
            .entries
            .iter()
            .find(|entry| {
                cumulative += entry.weight;
                cumulative > target
            })
            .unwrap_or(&group.entries[0]);
        let category = if (1..1000).contains(&entry.entity_type) {
            SpawnCategory::Entity
        } else {
            SpawnCategory::Grid
        };
        // SpawnEntity5ce6b0 applies spawnConfig5c5240 before Game::Spawn.
        let variant = if category == SpawnCategory::Entity && (14..=88).contains(&entry.entity_type)
        {
            alternate_variant(entry.entity_type, entry.variant, state)
        } else {
            entry.variant
        };
        selectors.push(SpawnSelector {
            group_index: group_index as u16,
            grid_index: (group.x + 1) as u16 + (group.y + 1) as u16 * grid_width,
            category,
            entity_type: entry.entity_type,
            variant,
            subtype: entry.subtype,
            position: WorldVector {
                x: 80.0 + f32::from(group.x) * 40.0,
                y: 160.0 + f32::from(group.y) * 40.0,
            },
            group_seed: state,
            spawn_seed: if category == SpawnCategory::Entity {
                next35(state)
            } else {
                state
            },
        });
    }
    Ok(RoomSpawns {
        tinted_rock_index,
        selectors,
        final_spawn_state: state,
    })
}
