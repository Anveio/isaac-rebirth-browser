//! Default CameraStyle1 histories and bounded room scrolling. Actor/world
//! positions stay authoritative; this controller only supplies render offsets.
use crate::{motion::WorldVector, phase::Phase, Player, Room};
use serde::{Deserialize, Serialize};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };
const SENTINEL: WorldVector = WorldVector { x: -1.0, y: -1.0 };

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct History {
    pub capacity: usize,
    pub head: usize,
    /// Chronological order, matching the source observer's ring readback.
    pub samples: Vec<WorldVector>,
}
impl History {
    fn filled(capacity: usize, value: WorldVector) -> Self {
        Self {
            capacity,
            head: 0,
            samples: vec![value; capacity],
        }
    }
    fn append(&mut self, value: WorldVector) {
        self.samples.rotate_left(1);
        *self
            .samples
            .last_mut()
            .expect("camera history has capacity") = value;
        self.head = (self.head + 1) % self.capacity;
    }
    fn mean(&self, peak: f32, fallback: Option<WorldVector>) -> WorldVector {
        // 42bc80 accumulates each chronological weighted vector and weight in
        // f32, then multiplies the vector by the reciprocal total weight.
        let count = self.samples.len() as f32;
        let peak_index = (count * peak) as usize;
        let mut total = ZERO;
        let mut weights = 0.0_f32;
        for (index, &sample) in self.samples.iter().enumerate() {
            let weight = if index <= peak_index {
                (index as f32 - -1.0) / (peak_index as f32 - -1.0)
            } else {
                (index as f32 - (count + 1.0)) / (peak_index as f32 - (count + 1.0))
            };
            let sample = if sample == SENTINEL {
                fallback.unwrap_or(sample)
            } else {
                sample
            };
            weights += weight;
            total = total + sample * weight;
        }
        total * (1.0 / weights)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Camera {
    pub scroll: WorldVector,
    pub last_player_position: WorldVector,
    pub histories: Vec<History>,
}

fn player_position(player: &Player) -> WorldVector {
    WorldVector {
        x: ((player.x - 32.0) / 0.65) as f32,
        y: ((player.y + 46.0) / 0.65) as f32,
    }
}
fn projected(point: WorldVector) -> WorldVector {
    WorldVector {
        x: (point.x - 320.0) * 0.65 + 240.0,
        y: (point.y - 280.0) * 0.65 + 135.0,
    }
}
fn minimum_scroll(room: &Room) -> WorldVector {
    let (width, height) = room
        .grid
        .as_ref()
        .map_or((15, 9), |grid| (grid.width, grid.height));
    WorldVector {
        x: 480.0 - ((f32::from(width) + 2.0) * 26.0 + room.appearance.origin.x as f32 * 2.0),
        y: 270.0 - ((f32::from(height) + 2.0) * 26.0 + room.appearance.origin.y as f32 * 2.0),
    }
}
fn target(point: WorldVector) -> WorldVector {
    let point = projected(point);
    WorldVector {
        x: 240.0 - point.x,
        y: 135.0 - point.y,
    }
}
fn clamp_focus(room: &Room, point: WorldVector) -> WorldVector {
    let minimum = minimum_scroll(room);
    let point = target(point);
    WorldVector {
        x: point.x.clamp(minimum.x, 0.0),
        y: point.y.clamp(minimum.y, 0.0),
    }
}
fn ease(value: f32, low: f32, mode: u32) -> f32 {
    if value <= low {
        return low;
    }
    if value >= 0.0 {
        return 0.0;
    }
    let t = (value - low) / (0.0 - low);
    let negative_square = -(t * t);
    // Source42b6a0 uses different fifth-degree curves on the two axes.
    // Preserve its separate multiply/add/subtract order rather than a fused
    // polynomial or the door-proximity smoothstep.
    let curved = if mode == 1 {
        let ten_t = 10.0 * t;
        let cubic = ((4.0 * t) * t) * t;
        negative_square * (((cubic - t * ten_t) + ten_t) - 5.0)
    } else {
        let square = (15.0 * t) * t;
        let cubic = ((t * 6.0) * t) * t;
        negative_square * (((cubic - square) + t * 14.0) - 6.0)
    };
    low + (0.0 - low) * curved
}

impl Camera {
    pub fn new(room: &Room, incoming_slot: Option<u8>) -> Self {
        let point = incoming_slot
            .and_then(|slot| {
                room.appearance
                    .doors
                    .iter()
                    .find(|door| door.slot == slot)
                    .map(|door| door.world_position)
            })
            .unwrap_or(ZERO);
        Self {
            scroll: clamp_focus(room, point),
            last_player_position: ZERO,
            histories: vec![
                History::filled(60, ZERO),
                History::filled(90, ZERO),
                History::filled(300, ZERO),
                History::filled(60, ZERO),
                History::filled(4, ZERO),
                History::filled(120, SENTINEL),
            ],
        }
    }
    /// Source42d780 admits default-style camera updates after each active game
    /// presentation. Room freezes are handled by the game/transition caller.
    pub fn advance(&mut self, room: &Room, player: &Player, aim: WorldVector, phase: Phase) {
        if phase == Phase::Skipped {
            return;
        }
        let position = player_position(player);
        let velocity = WorldVector {
            x: (player.vx / 0.65) as f32,
            y: (player.vy / 0.65) as f32,
        };
        let mut enemy = ZERO;
        let mut enemy_weight = 0.0_f32;
        for npc in &room.npcs {
            if !npc.can_shut_doors || !npc.visible || npc.removed {
                continue;
            }
            let delta = WorldVector {
                x: npc.presented_position.x - position.x,
                y: npc.presented_position.y - position.y,
            };
            let distance = (delta.x * delta.x + delta.y * delta.y).sqrt();
            if distance >= 550.0 {
                continue;
            }
            let weight = (distance - 550.0) / (0.0 - 550.0);
            enemy = enemy + (delta * weight) * 0.4;
            enemy_weight += if npc.boss { 0.6 } else { 1.0 };
        }
        if enemy_weight != 0.0 {
            enemy = enemy * (1.0 / enemy_weight);
        }
        self.histories[0].append(aim * 35.0);
        self.histories[1].append((velocity * 0.075) * 60.0);
        self.histories[2].append(enemy);
        // Fresh single Isaac has no cooperative relative-player focus.
        self.histories[3].append(ZERO);
        // The shake consumer and special item focus callbacks remain separate.
        self.histories[4].append(ZERO);
        // DefaultStyle1 clears its special-target flag after BOTH phases. With
        // no callback-supplied target, each phase appends the sentinel again.
        self.histories[5].append(SENTINEL);
        let focus = (((position + self.histories[0].mean(0.75, None))
            + self.histories[1].mean(0.85, None))
            + self.histories[2].mean(0.75, None))
            + self.histories[3].mean(0.75, None);
        let focus = self.histories[5].mean(0.5, Some(focus));
        let target = target(focus);
        let minimum = minimum_scroll(room);
        self.scroll = WorldVector {
            x: ease(target.x, minimum.x, 2),
            y: ease(target.y, minimum.y, 1),
        };
        for door in &room.appearance.doors {
            let dx = position.x - door.world_position.x;
            let dy = position.y - door.world_position.y;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance < 30.0 {
                let t = ((distance - 30.0) / (10.0 - 30.0)).min(1.0);
                let weight = (t * t) * (3.0 - 2.0 * t);
                let target = clamp_focus(room, door.world_position);
                self.scroll = WorldVector {
                    x: self.scroll.x + (target.x - self.scroll.x) * weight,
                    y: self.scroll.y + (target.y - self.scroll.y) * weight,
                };
            }
        }
        self.last_player_position = position;
    }
}
