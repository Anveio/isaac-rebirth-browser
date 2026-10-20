//! Room::DamageGrid 5cbb30 -> grid virtual28 (Hurt) receivers, shared by the
//! knife (473323), Larry Jr.'s head (4f365c), laser contacts (4779dc), the
//! regular tear's grid death (5430a5) and the bomb's DestroyGrid (Poop has
//! no Destroy override: 561c00 is Hurt(4)). These receivers are distinct
//! from collision queries and path-cost stores.
use crate::{grid::Grid, motion::WorldVector, State};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GridCallbackOutcome {
    /// Hurt's bool: whether the grid entity took the damage.
    Handled(bool),
    UnadmittedKind(u32),
    /// The receiver's bool is known, but a later branch is not ported. The
    /// draws before that branch ran; its own effects remain unperformed.
    Unadmitted {
        taken: bool,
        branch: &'static str,
    },
}

impl GridCallbackOutcome {
    /// Preserve the actual receiver return independently of its effect frontier.
    /// An unimplemented receiver kind has no source-owned return value.
    pub(crate) fn taken(&self) -> Option<bool> {
        match self {
            Self::Handled(taken) | Self::Unadmitted { taken, .. } => Some(*taken),
            Self::UnadmittedKind(_) => None,
        }
    }

    /// Retain a non-controller callback gap without invalidating C18 history.
    pub(crate) fn record_frontier(&self, state: &mut State, caller: &str) {
        if let Some(channel) = self.frontier() {
            crate::ordinary_placement::invalidate(
                state,
                crate::ordinary_placement::Frontier::GridEffects,
            );
            state.weapon_admission.unadmitted_controller.get_or_insert(
                crate::weapon_plan::UnadmittedController {
                    primary: state.weapon_cache.primary.numeric(),
                    channel: format!("{caller}{channel}"),
                },
            );
        }
    }

    /// The caller's diagnostic channel for an unported receiver or branch.
    pub(crate) fn frontier(&self) -> Option<String> {
        match self {
            Self::Handled(_) => None,
            Self::UnadmittedKind(kind) => Some(format!("GridKind{kind}")),
            Self::Unadmitted { branch, .. } => Some((*branch).to_string()),
        }
    }
}

pub(crate) fn knife_callback(
    state: &mut State,
    geometry: &Grid,
    position: WorldVector,
) -> Result<GridCallbackOutcome, String> {
    let Some(index) = geometry.index(position) else {
        return Ok(GridCallbackOutcome::Handled(false));
    };
    callback_at_index(state, index, 1)
}

/// Room5CBB30(index, damage): empty cells and indices outside the room are
/// a false no-op.
pub(crate) fn callback_at_index(
    state: &mut State,
    index: usize,
    damage: i32,
) -> Result<GridCallbackOutcome, String> {
    let entity = state
        .room
        .grid
        .as_mut()
        .ok_or("grid Hurt requires actual room admission")?
        .cells
        .get_mut(index)
        .and_then(|cell| cell.entity.as_mut());
    let Some(entity) = entity else {
        return Ok(GridCallbackOutcome::Handled(false));
    };
    match entity.kind {
        //Actual factory vtables select the literal561bf0 false-return stub.
        1..=11 | 15..=20 => Ok(GridCallbackOutcome::Handled(false)),
        //5703b0 always takes Bounded3 before testing the existing state.
        12 => {
            let mut strength = damage;
            if state.boot_lifecycle.shared.bounded(3) == 0 {
                strength = (strength + 1).min(4);
            }
            if strength <= 0 || entity.state > 3 {
                return Ok(GridCallbackOutcome::Handled(false));
            }
            entity.state = (entity.state as i32 + strength).min(4) as u32;
            Ok(GridCallbackOutcome::Handled(true))
        }
        //5660e0's destruction/effect/drop callbacks are not admitted yet.
        13 => Ok(GridCallbackOutcome::UnadmittedKind(13)),
        14 => hurt_poop(state, index, damage),
        _ => Err(format!(
            "grid Hurt receiver has unrecognized actual kind{}",
            entity.kind
        )),
    }
}

/// Poop Hurt 56a390 for normal (0) and corn (2) poop. A hit may add one
/// (normal poop: up to two) to the damage and advances the state toward 4;
/// particles scatter on a 1/2 roll below state 4 and always when it breaks.
/// Breaking spawns the 1000.43 explosion, clears the cell (cost 900, class
/// 0) and rolls the drop on a fresh local RNG of the spawn seed.
fn hurt_poop(state: &mut State, index: usize, damage: i32) -> Result<GridCallbackOutcome, String> {
    let grid = state
        .room
        .grid
        .as_mut()
        .ok_or("grid Hurt requires actual room admission")?;
    let width = usize::from(grid.width);
    let poop = grid.cells[index].entity.as_mut().expect("poop cell");
    if !matches!(poop.variant, 0 | 2) {
        // Red, golden and rainbow poop need their colored particles and
        // respawn, coin and full-heart branches.
        return Ok(GridCallbackOutcome::UnadmittedKind(14));
    }
    if poop.state > 3 {
        return Ok(GridCallbackOutcome::Handled(false));
    }
    let shared = &mut state.boot_lifecycle.shared;
    let mut strength = damage;
    if shared.bounded(3) == 0 {
        strength = (strength + 1).min(4);
    }
    if poop.variant == 0 && shared.bounded(3) == 0 {
        strength = (strength + 1).min(4);
    }
    if strength <= 0 {
        return Ok(GridCallbackOutcome::Handled(false));
    }
    // Play("State%d", state + 1): the renderer draws the state's clip.
    poop.state = (poop.state as i32 + strength).min(4) as u32;
    let broken = poop.state == 4;
    let spawn_seed = poop.spawn_seed;
    let pieces = if broken {
        4
    } else if shared.bounded(2) != 0 {
        return Ok(GridCallbackOutcome::Handled(true));
    } else {
        poop.state
    };
    // 561070 -> 5c78b0: the grid centre.
    let centre = WorldVector {
        x: 40.0 + (index % width) as f32 * 40.0,
        y: 120.0 + (index / width) as f32 * 40.0,
    };
    let mut count = shared.bounded(pieces) + 1;
    if state.challenge_id == 23 {
        count *= 3;
    }
    particles(state, centre, count)?;
    if !broken {
        return Ok(GridCallbackOutcome::Handled(true));
    }
    let mut local = crate::items::Rng::new(spawn_seed, [1, 5, 16]);
    //56A701 follows the break particles/local seed and precedes Spawn's raw.
    crate::persistent_effects::add_counter(state, 4, 1)?;
    let seed = state.boot_lifecycle.shared.next_u32();
    admission(state).admit_explosion(seed, centre)?;
    state
        .room
        .path_costs
        .as_mut()
        .ok_or("poop destruction requires source Room.Init path costs")?
        .store(index, 900)?;
    let grid = state.room.grid.as_mut().expect("admitted room grid");
    grid.cells[index]
        .entity
        .as_mut()
        .expect("poop cell")
        .collision_class = 0;
    drop_roll(state, &mut local, centre)
}

fn admission(state: &mut State) -> crate::poop_effects::Admission<'_> {
    crate::poop_effects::Admission {
        game_update_count: state.game_update_count,
        next_effect: &mut state.next_effect,
        effects: &mut state.sprite_effects,
        scheduler: &mut state.actor_scheduler,
    }
}

/// Poop Hurt passes Game::SpawnParticles 5515b0 Effect 1000.58 a speed of
/// one Float + 2.
fn particles(state: &mut State, centre: WorldVector, count: u32) -> Result<(), String> {
    let speed = state.boot_lifecycle.shared.uniform() + 2.0;
    crate::poop_effects::spawn_particles(state, 58, centre, count, speed)
}

/// 56aa90: a 0.1 chance of a heart (1/4) or a coin, or the Petrified Poop
/// trinket (1/20 while the pool holds it); boss rooms mostly drop nothing.
/// Butt Penny, Petrified Poop, Daemon's Tail and E. Coli change the roll.
/// Every partial return here follows successful Hurt; 56a7b3 still returns true.
fn drop_roll(
    state: &mut State,
    local: &mut crate::items::Rng,
    centre: WorldVector,
) -> Result<GridCallbackOutcome, String> {
    let items = state.player_items.as_ref();
    let trinket = |id: u32| items.is_some_and(|items| items.trinket_slots.contains(&id));
    if trinket(2) || trinket(22) || trinket(24) {
        return Ok(GridCallbackOutcome::Unadmitted {
            taken: true,
            branch: "poopDropTrinket",
        });
    }
    let e_coli =
        items.is_some_and(|items| items.owned_counts.get(236).is_some_and(|count| *count > 0));
    let missed = if e_coli {
        GridCallbackOutcome::Unadmitted {
            taken: true,
            branch: "poopDropEColi",
        }
    } else {
        GridCallbackOutcome::Handled(true)
    };
    if local.uniform() >= 0.1 {
        return Ok(missed);
    }
    let boss_room = state
        .room
        .descriptor
        .as_ref()
        .is_some_and(|descriptor| descriptor.template.room_type == 5);
    if local.uniform() >= 0.01 && boss_room {
        return Ok(missed);
    }
    if local.bounded(20) == 0 {
        // RemoveTrinket(2) needs the item pool's trinket availability; the
        // Petrified Poop spawn would then draw MT.next.
        return Ok(GridCallbackOutcome::Unadmitted {
            taken: true,
            branch: "poopDropPetrifiedPoop",
        });
    }
    let variant = if local.uniform() < 0.25 { 10 } else { 20 };
    let seed = local.raw();
    crate::rock_blast::admit_reward(
        state,
        variant,
        None,
        seed,
        centre,
        WorldVector { x: 0.0, y: 0.0 },
    )?;
    Ok(GridCallbackOutcome::Handled(true))
}
