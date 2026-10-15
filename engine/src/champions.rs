//! Entity_NPC::Init47f500 champion roll for ordinary (non-boss) NPCs.
//! A local RNG(spawn seed, shift preset1) draws the roll and the color; color
//! init effects follow the source handlers. Colors 9/10/11 draw the shared MT.
use crate::{content, items, mt::Mt19937, npcs::Npc, player_items::PlayerItems};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Champion {
    /// Entity_NPC+1040 champion color.
    pub color: u32,
    /// Sprite scale (+2c4) multiplier, ×1.15 for every champion.
    pub scale: f32,
    /// Entity_NPC+f74 speed multiplier written by colors 1, 4 and 7.
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
    pub player_items: Option<&'a PlayerItems>,
}

/// Rolls and applies a champion after the family constructor, in authored
/// selector order. Returns whether the NPC became a champion.
pub(crate) fn roll(npc: &mut Npc, shared: &mut Mt19937, context: &Roll) -> Result<bool, String> {
    let config = content::entity(npc.kind, npc.variant);
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
    // 47f5a9/47f710: 0.2 with Champion Belt, otherwise 0.05. Hard mode and the
    // Purple Heart trinket are not admitted runtime states.
    let chance = if belt { 0.2 } else { 0.05 };
    if rng.uniform() >= chance {
        return Ok(false);
    }
    // 47f770: double max HP, scale the sprite, contact damage 2, then color.
    npc.max_hp *= 2.0;
    npc.collision_damage = 2.0;
    let mut champion = Champion {
        color: 0,
        scale: 1.15,
        speed: 1.0,
        timer: None,
    };
    let stage = context.stage;
    let pick = rng.uniform();
    champion.color = if pick < 0.01 && stage > 4 {
        6
    } else if pick < 0.05 && stage > 4 {
        match rng.bounded(3) {
            0 => {
                npc.grid_collision_class = 3;
                8
            }
            1 => {
                champion.timer = Some(shared.bounded(30) + 60);
                9
            }
            _ => return Err("champion color 11 requires its actual light effect owner".into()),
        }
    } else if pick < 0.15 {
        match rng.bounded(3) {
            0 => 3,
            1 => {
                champion.speed = 0.8;
                4
            }
            _ => 12,
        }
    } else {
        match rng.bounded(7) {
            0 => {
                npc.max_hp *= 1.333;
                0
            }
            1 => {
                champion.speed = 1.333;
                1
            }
            2 => 2,
            3 => 5,
            4 => {
                npc.max_hp *= 0.333;
                champion.speed = 0.8;
                7
            }
            5 => {
                champion.timer = Some(shared.bounded(20) + 40);
                10
            }
            _ => 13,
        }
    };
    npc.hp = npc.max_hp;
    npc.champion = Some(champion);
    // Champion tail 47f7b2 calls ResetColor with the rolled colour.
    npc.sprite_color = crate::sprite_color::SpriteColor::reset(npc.base_color());
    Ok(true)
}
