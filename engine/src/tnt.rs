//! Source-owned TNT grid Init/Full callback, distinct from actor Bombs.
//! Its rare reward stops at the unowned ItemPool availability query.
use crate::{motion::WorldVector, phase::Phase, State};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Animation {
    Idle,
    IdleMedium,
    ReadyToExplode,
    Blown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Sprite {
    pub animation: Animation,
    pub frame: u32,
    pub playing: bool,
}

impl Sprite {
    fn play(&mut self, animation: Animation) {
        // Every admitted clip is nonlooping. Play(false) on the same clip
        // leaves its real playing byte alone, including an ended clip.
        if self.animation != animation {
            self.animation = animation;
            self.frame = 0;
            self.playing = true;
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RewardFrontier {
    /// Actual next RemoveTrinket query; availability was NOT guessed false.
    pub next_trinket_pool_query: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Controller {
    pub countdown: i32,
    pub sprite: Sprite,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reward_frontier: Option<RewardFrontier>,
}

impl Controller {
    pub(crate) fn initialized(state: u32) -> Self {
        Self {
            countdown: if state as i32 <= 3 { 3 } else { -1 },
            sprite: Sprite {
                animation: Animation::Idle,
                frame: 0,
                playing: true,
            },
            reward_frontier: None,
        }
    }

    fn update(&mut self, state: u32) -> bool {
        match state {
            0 | 1 => self.sprite.play(Animation::Idle),
            2 => self.sprite.play(Animation::IdleMedium),
            3 => self.sprite.play(Animation::ReadyToExplode),
            4 => {
                // TNT has no Sprite.Update in Init, Update, Render or the
                // manager grid dispatch. Do not invent an ANM clock here.
                let playing_blown =
                    self.sprite.playing && self.sprite.animation == Animation::Blown;
                if !playing_blown {
                    self.countdown = self.countdown.wrapping_sub(1);
                    if self.countdown <= 0 {
                        self.sprite.play(Animation::Blown);
                        return self.countdown == 0;
                    }
                }
                // The second counter>0 guard also runs for playing Blown.
                if self.countdown > 0 {
                    self.sprite.play(Animation::ReadyToExplode);
                }
            }
            _ => {}
        }
        false
    }
}

fn frontier(state: &mut State, branch: &'static str) {
    crate::attack_grid::GridCallbackOutcome::Unadmitted {
        taken: true,
        branch,
    }
    .record_frontier(state, "tntUpdate");
}

pub(crate) fn advance(state: &mut State, phase: Phase) -> Result<(), String> {
    if !phase.updates_nonplayers() {
        return Ok(());
    }
    let Some(grid) = state.room.grid.as_ref() else {
        return Ok(());
    };
    let count = grid.cells.len();
    let width = usize::from(grid.width);
    // Re-read each cell after preceding callbacks: a lower-index blast can
    // Hurt a higher-index TNT before its callback on this SAME Full.
    for index in 0..count {
        let Some(entity) = state
            .room
            .grid
            .as_mut()
            .and_then(|grid| grid.cells[index].entity.as_mut())
        else {
            continue;
        };
        if entity.kind != 12 {
            continue;
        }
        entity.collision_class = if (entity.state as i32) < 4 { 2 } else { 0 };
        let seed = entity.spawn_seed;
        let Some(owner) = entity
            .source_owner
            .as_mut()
            .and_then(|owner| owner.tnt.as_mut())
        else {
            frontier(state, "ControllerHistory");
            continue;
        };
        // A recorded reward frontier marks the rare trinket this port skips
        // (below); the blown barrel's ordinary update continues.
        if !owner.update(entity.state) {
            continue;
        }
        let centre = WorldVector {
            x: 40.0 + (index % width) as f32 * 40.0,
            y: 120.0 + (index / width) as f32 * 40.0,
        };
        if !crate::explosion_effects::admit_tnt_children(state, centre)? {
            continue;
        }
        crate::bombs::explode_tnt(state, centre)?;
        let speed = state.boot_lifecycle.shared.uniform() * 4.0 + 2.0;
        let count = state.boot_lifecycle.shared.bounded(3) + 2;
        crate::poop_effects::spawn_particles(state, 27, centre, count, speed)?;
        if seed == 0 {
            return Err("TNT local RNG seed0 reaches the source assertion".into());
        }
        let mut local = crate::items::Rng::new(seed, [1, 5, 16]);
        if local.bounded(100) == 0 {
            let next_query = if local.bounded(2) == 0 { 50 } else { 41 };
            let owner = state
                .room
                .grid
                .as_mut()
                .and_then(|grid| grid.cells[index].entity.as_mut())
                .and_then(|entity| entity.source_owner.as_mut())
                .and_then(|owner| owner.tnt.as_mut())
                .ok_or("TNT controller disappeared during its blast")?;
            owner.reward_frontier = Some(RewardFrontier {
                next_trinket_pool_query: next_query,
            });
            // RemoveTrinket needs two real pool bytes; held slots cannot
            // prove either. No fallback/placement draws or fake empty spawn.
            // Port gap: the rare trinket's Remove/scatter/Init is skipped, and
            // the frontier above records it, so the run keeps going.
            frontier(state, "RareTrinketPool");
        }
        state
            .room
            .path_costs
            .as_mut()
            .ok_or("TNT requires Room.Init costs")?
            .store(index, 900)?;
    }
    Ok(())
}
