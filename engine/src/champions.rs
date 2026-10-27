//! Entity_NPC::Init47f500 champion roll for ordinary (non-boss) NPCs.
//! A local RNG(spawn seed, shift preset1) draws the roll and the color; color
//! init effects follow the source handlers. Colors 9/10/11 draw the shared MT.
use crate::{items, mt::Mt19937, npcs::Npc, player_items::PlayerItems};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Champion {
    /// Entity_NPC+1040 champion color.
    pub color: u32,
    /// Sprite scale (+2c4) multiplier, ×1.15 for every champion.
    pub scale: f32,
    /// Entity_NPC+f74 speed multiplier written by colors 1, 4 and 7. The
    /// post-AI velocity scale 4f9f30 reads it (`plain_npc`); PORT GAP: the
    /// movement helper 4807e0's below-1 branch does not.
    pub speed: f32,
    /// Entity_NPC+f70 / +1044 timers drawn by colors 10 and 9.
    pub timer: Option<u32>,
}

/// RNG preset1 at 0x679b80+12.
const SHIFTS: [u32; 3] = [1, 5, 16];
const CHAMPION_BELT: usize = 208;

/// Static champion Colors at 0x8ec700 (initializer 0x40aa42..0x40ac23):
/// tint RGBA and offset RGB; colorize is zero for every entry.
pub(crate) fn color(index: u32) -> crate::sprite_color::Color {
    use crate::sprite_color::Color;
    const OFFSET: f32 = crate::sprite_color::OFFSET_200;
    match index {
        0 => Color::tinted([1.0, 0.1, 0.1, 1.0], 0.0),
        1 => Color::tinted([0.6, 0.6, 0.0, 1.0], 0.0),
        2 => Color::tinted([0.1, 0.8, 0.2, 1.0], 0.0),
        3 => Color::tinted([1.0, 0.6, 0.2, 1.0], 0.0),
        4 => Color::tinted([0.2, 0.2, 1.0, 1.0], 0.0),
        5 => Color::tinted([0.0, 0.15, 0.15, 1.0], 0.0),
        6 => Color::tinted([1.0, 1.0, 1.0, 1.0], OFFSET),
        7 => Color::tinted([0.5, 0.5, 0.5, 1.0], 0.0),
        8 => Color::tinted([1.0, 1.0, 1.0, 0.3], OFFSET),
        9 => Color::tinted([0.0, 0.0, 0.0, 1.0], 0.0),
        10 => Color::tinted([1.0, 0.0, 1.0, 1.0], 0.0),
        11 => Color::tinted([0.75, 0.0, 1.0, 1.0], 0.0),
        12 => Color::tinted([0.5, 0.0, 0.0, 1.0], 0.0),
        _ => Color::tinted([0.5, 0.5, 1.0, 1.0], 0.0),
    }
}

pub(crate) struct Roll<'a> {
    pub stage: u32,
    pub difficulty: crate::difficulty::Difficulty,
    pub player_items: Option<&'a PlayerItems>,
}

/// Rolls and applies a champion after the family constructor, in authored
/// selector order. Returns whether the NPC became a champion.
pub(crate) fn roll(npc: &mut Npc, shared: &mut Mt19937, context: &Roll) -> Result<bool, String> {
    let config = npc.config();
    if config.boss || !config.champion {
        return Ok(false);
    }
    let mut rng = items::Rng::new(npc.spawn_seed, SHIFTS);
    let belt = context.player_items.is_some_and(|items| {
        items
            .owned_counts
            .get(CHAMPION_BELT)
            .is_some_and(|count| *count > 0)
    });
    // 47f5a9/47f710: 0.2 with Champion Belt, otherwise 0.05, plus the
    // difficulty's f32 bonus (47f5c6/47f725). Purple Heart's doubling after
    // it is not an admitted runtime state.
    let chance: f32 = if belt { 0.2 } else { 0.05 };
    let chance = chance + crate::difficulty::rules(context.difficulty).champion_chance_bonus;
    if rng.uniform() >= chance {
        return Ok(false);
    }
    let stage = context.stage;
    let pick = rng.uniform();
    let color = if pick < 0.01 && stage > 4 {
        6
    } else if pick < 0.05 && stage > 4 {
        [8, 9, 11][rng.bounded(3) as usize]
    } else if pick < 0.15 {
        [3, 4, 12][rng.bounded(3) as usize]
    } else {
        [0, 1, 2, 5, 7, 10, 13][rng.bounded(7) as usize]
    };
    apply(npc, color, shared)?;
    Ok(true)
}

/// 47f770: double max HP, scale the sprite, contact damage 2, then the
/// color's effects. Init47f500 re-runs it for a champion's kept color when
/// Morph480750 reloads the config.
pub(crate) fn apply(npc: &mut Npc, color: u32, shared: &mut Mt19937) -> Result<(), String> {
    npc.max_hp *= 2.0;
    npc.collision_damage = 2.0;
    let mut champion = Champion {
        color,
        scale: 1.15,
        speed: 1.0,
        timer: None,
    };
    match color {
        0 => npc.max_hp *= 1.333,
        1 => champion.speed = 1.333,
        4 => champion.speed = 0.8,
        7 => {
            npc.max_hp *= 0.333;
            champion.speed = 0.8;
        }
        8 => npc.grid_collision_class = 3,
        9 => champion.timer = Some(shared.bounded(30) + 60),
        10 => champion.timer = Some(shared.bounded(20) + 40),
        11 => {
            // 47f9f0: MT.next seeds a Pulling Effect 1000.57 parented to the
            // champion. PORT GAP: the effect is not constructed. Its Init
            // (4594ac) and Update (45beb8: follow the parent, removed without
            // one) draw nothing, so only its seed is drawn; later actors'
            // sequence ids stay one below the source's.
            shared.next_u32();
        }
        _ => {}
    }
    npc.hp = npc.max_hp;
    npc.champion = Some(champion);
    // Champion tail 47f7b2 calls ResetColor with the rolled colour.
    npc.sprite_color = crate::sprite_color::SpriteColor::reset(npc.base_color());
    Ok(())
}

/// RNG preset 30 at 0x679b80+360: Entity Init 441c80 seeds the NPC's own
/// RNG (+7c0) from its spawn seed with it.
const ENTITY_SHIFTS: [u32; 3] = [4, 3, 17];

/// One Spawn550a40(5, variant, subtype) a dying champion makes.
pub(crate) struct DeathDrop {
    pub variant: u32,
    pub subtype: u32,
    pub spawn_seed: u32,
    pub position: crate::motion::WorldVector,
}

/// Entity_NPC 4f27a0, which 4f2d10 runs for a dead champion (+103f) as the
/// base kill removes it. The NPC's own RNG (+7c0) draws RandomInt(3) at
/// both difficulties (4f27c8), then the color's jump table 66e320 runs; a
/// table entry that drops waits on 4f2960, which the difficulty rule gates.
/// Each drop goes to FindFreeTilePosition 5cd7d0(position, 40) and takes
/// the next own-RNG value as its seed.
///
/// Returns None for a color whose drop the port does not construct yet:
/// 4 (Blue Flies through the player, 508740), 5 (an explosion 554490 and
/// its effects before the gate, then a bomb), 8 (locked chest 60), 9 (red
/// chest 360), 11 (trinket 350) and 13 (4f2852).
pub(crate) fn death_drops(
    npc: &Npc,
    difficulty: crate::difficulty::Difficulty,
    shared: &mut Mt19937,
    free_position: impl Fn(crate::motion::WorldVector) -> crate::motion::WorldVector,
) -> Option<Vec<DeathDrop>> {
    let champion = npc.champion.as_ref()?;
    // Only Dips draw their own RNG before death among the admitted families.
    let mut rng = npc
        .dip
        .as_ref()
        .and_then(|dip| dip.entity_rng)
        .unwrap_or_else(|| items::Rng::new(npc.spawn_seed, ENTITY_SHIFTS));
    let roll = rng.bounded(3);
    let drops = roll == 0 || !crate::difficulty::rules(difficulty).champion_drop_needs_zero_roll;
    let (count, variant, subtype) = match champion.color {
        0 => (1, 10, 1),
        1 => (1, 90, 0),
        2 => (1, 70, 0),
        // 4f2a2a draws the shared MT for the count before the gate.
        3 => (shared.bounded(3) + 1, 20, 0),
        6 => (1, 10, 4),
        7 => (1, 30, 0),
        10 => (0, 0, 0),
        12 => (1, 10, 5),
        _ => return None,
    };
    if !drops {
        return Some(Vec::new());
    }
    Some(
        (0..count)
            .map(|_| DeathDrop {
                variant,
                subtype,
                position: free_position(npc.position),
                spawn_seed: rng.raw(),
            })
            .collect(),
    )
}

/// Room::FindFreeTilePosition 5cd7d0: `position` itself when its cell has
/// grid collision 0, otherwise the centre of the nearest such cell, taking
/// the first within `radius` in index order.
pub(crate) fn free_tile_position(
    grid: &crate::grid::Grid,
    collision: impl Fn(usize) -> u32,
    position: crate::motion::WorldVector,
    radius: f32,
) -> crate::motion::WorldVector {
    let source = grid.index(position);
    // 5c5af0 answers 4 for the -1 of a position outside the grid.
    if source.map_or(4, &collision) == 0 {
        return position;
    }
    // 6129d0 then 612950: the squared length of centre - position.
    let squared = |centre: crate::motion::WorldVector| {
        let (x, y) = (centre.x - position.x, centre.y - position.y);
        x * x + y * y
    };
    let limit = radius * radius;
    // 5c78b0 of -1 is never reached by a position inside the room.
    let mut best = source
        .and_then(|index| grid.position(index))
        .unwrap_or(position);
    let mut nearest = 1_000_000.0_f32;
    for index in 0..grid.descriptor().cells.len() {
        if Some(index) == source || collision(index) != 0 {
            continue;
        }
        let centre = grid.position(index).expect("indexed cell");
        let distance = squared(centre);
        if nearest > distance {
            best = centre;
            if limit > distance {
                break;
            }
            nearest = distance;
        }
    }
    best
}
