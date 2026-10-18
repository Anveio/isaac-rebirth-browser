//! Boss-room exit: the Room clear award trapdoor and boss pedestal, the Grid17
//! TrapDoor callback and the Game StageTransition that admits the next level.
//! Native evidence: docs/stage-progression.md.
use crate::motion::WorldVector;
use crate::{phase::Phase, State};
use serde::{Deserialize, Serialize};

/// GridEntity_TrapDoor (vtable 674630) sprite clips used by its Update5711a0.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TrapdoorClip {
    Closed,
    OpenAnimation,
}

impl TrapdoorClip {
    pub fn name(self) -> &'static str {
        match self {
            Self::Closed => "Closed",
            Self::OpenAnimation => "Open Animation",
        }
    }
    fn frames(self) -> u32 {
        match self {
            Self::Closed => crate::content::TRAPDOOR_CLOSED_FRAMES,
            Self::OpenAnimation => crate::content::TRAPDOOR_OPEN_ANIMATION_FRAMES,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Trapdoor {
    pub grid_index: usize,
    /// GridEntity+10: InitSubclass570fd0 writes 0; a finished open clip writes 1.
    pub state: u32,
    pub spawn_seed: u32,
    pub clip: TrapdoorClip,
    pub frame: f32,
    pub playing: bool,
}

impl Trapdoor {
    /// InitSubclass570fd0: state0 (571049) and Play Closed without force.
    pub(crate) fn reinitialize(&mut self) {
        self.state = 0;
        self.clip = TrapdoorClip::Closed;
        self.frame = 0.0;
        self.playing = true;
    }

    fn world_position(&self, width: usize) -> WorldVector {
        WorldVector {
            x: (40 + self.grid_index % width * 40) as f32,
            y: (120 + self.grid_index / width * 40) as f32,
        }
    }

    /// Sprite.Play40fd80 without force restarts only a different clip or a
    /// stopped looping clip; both trapdoor clips are authored nonlooping.
    fn play(&mut self, clip: TrapdoorClip) {
        if self.clip != clip {
            self.clip = clip;
            self.frame = 0.0;
            self.playing = true;
        }
    }

    /// Sprite.Update40f9f0 at the end of the grid callback.
    fn advance_sprite(&mut self) {
        if !self.playing {
            return;
        }
        self.frame += 1.0;
        if self.frame.floor() >= self.clip.frames() as f32 {
            self.frame = (self.clip.frames() - 1) as f32;
            self.playing = false;
        }
    }
}

/// Game+a0e0 StageTransition: state, counter, same-stage byte and animation.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StageTransition {
    pub state: u32,
    pub counter: u32,
    pub same_stage: bool,
    pub animation: u32,
}

impl StageTransition {
    /// IsActive5e37f0.
    pub fn active(&self) -> bool {
        self.state != 0
    }
}

fn player_world(state: &State) -> WorldVector {
    WorldVector {
        x: ((state.player.x - 32.0) / 0.65) as f32,
        y: ((state.player.y + 46.0) / 0.65) as f32,
    }
}

fn set_player_world(state: &mut State, position: WorldVector) {
    state.player.x = f64::from(position.x) * 0.65 + 32.0;
    state.player.y = f64::from(position.y) * 0.65 - 46.0;
}

fn grid_position(width: usize, index: usize) -> WorldVector {
    WorldVector {
        x: (40 + index % width * 40) as f32,
        y: (120 + index / width * 40) as f32,
    }
}

/// Room::SpawnClearAward5c9240, boss-room branch5c9740 only. The caller is
/// the Room clear boundary5d19f1, after doors open and the clear SFX.
pub(crate) fn spawn_boss_award(state: &mut State) -> Result<(), String> {
    let descriptor = &state.floor.descriptors[state.current_room];
    if descriptor.template.room_type != 5 {
        return Ok(());
    }
    if (descriptor.template.width, descriptor.template.height) != (13, 7) {
        return Err("boss clear award admits only 1x1 boss rooms".into());
    }
    // 5c9745..5c975f: effective stage adds one under Curse of the Labyrinth.
    // The supported run admits no curses, so it is the numeric stage.
    let stage = state.stage;
    if !(1..=5).contains(&stage) && stage != 7 {
        return Err("boss clear award admits only the trapdoor stages".into());
    }
    // 5c97e4: presentation-specific award branches (Mom, horsemen, ...).
    if matches!(state.room.boss_presentation_index, 9..=12 | 0x13..=0x16 | 0x26 | 0x37) {
        return Err("special boss clear award is not admitted".into());
    }
    let grid = state
        .room
        .grid
        .as_ref()
        .ok_or("boss clear award requires the admitted room grid")?;
    let width = usize::from(grid.width);
    let center_index = width / 2 + usize::from(grid.height) / 2 * width;
    let center = grid_position(width, center_index);
    // 5c929e..5c9324: trapdoor at centre-80y, collectible at centre+80y.
    let trapdoor_index = room_index(
        width,
        WorldVector {
            x: center.x,
            y: center.y - 80.0,
        },
    );
    let collectible_position = WorldVector {
        x: center.x,
        y: center.y + 80.0,
    };
    let collectible_index = room_index(width, collectible_position);
    // 5c97c1 fix_award_pos5c8260 clears the trapdoor cell before its MT draw.
    clear_award_cell(state, trapdoor_index)?;
    // Room+8 descriptor+50 seeds the award RNG with shift index 35.
    let mut rng = crate::items::Rng::new(
        state.floor.descriptors[state.current_room].following_seed,
        [5, 9, 7],
    );
    // 5c97c6 Random() -> SpawnGridEntity(index, 17, 0, seed).
    let trapdoor_seed = state.boot_lifecycle.shared.next_u32();
    let grid = state.room.grid.as_mut().expect("checked grid");
    grid.cells[trapdoor_index].entity = Some(crate::grid::GridEntity {
        kind: 17,
        variant: 0,
        state: 0,
        collision_class: 0,
        spawn_seed: trapdoor_seed,
        source_owner: None,
    });
    // InitSubclass570fd0 loads the trapdoor actor and plays Closed.
    state.room.trapdoor = Some(Trapdoor {
        grid_index: trapdoor_index,
        state: 0,
        spawn_seed: trapdoor_seed,
        clip: TrapdoorClip::Closed,
        frame: 0.0,
        playing: true,
    });
    // 5c9825: GetCollectible5c51e0(Room, rng) then Spawn(5,100,...,rng).
    let selection_seed = rng.raw();
    let pools = state
        .item_pools
        .as_mut()
        .ok_or("boss award requires the run item pools")?;
    let pool = pools.room_pool(5, &mut state.boot_lifecycle.shared);
    let subtype = pools.collectible(pool, true, selection_seed)?;
    // 5c9894 fix_award_pos5c8320 clears the collectible cell after selection.
    clear_award_cell(state, collectible_index)?;
    let spawn_seed = rng.raw();
    crate::pickups::admit_collectible(state, subtype, collectible_position, spawn_seed)?;
    Ok(())
}

fn room_index(width: usize, position: WorldVector) -> usize {
    let column = ((position.x - 40.0) / 40.0).round() as usize;
    let row = ((position.y - 120.0) / 40.0).round() as usize;
    column + row * width
}

/// GridEntity_TrapDoor::Update5711a0, dispatched in the Room grid pass before
/// entities. Distances use the player's previous full-update position.
pub(crate) fn advance_trapdoor(state: &mut State, phase: Phase) -> Result<(), String> {
    if !phase.updates_nonplayers() || state.stage_transition.active() {
        return Ok(());
    }
    let Some(width) = state.room.grid.as_ref().map(|grid| usize::from(grid.width)) else {
        return Ok(());
    };
    let Some(trapdoor) = state.room.trapdoor.as_mut() else {
        return Ok(());
    };
    let position = trapdoor.world_position(width);
    // The trapdoor opens on every trapdoor stage, but a stage whose next
    // SetNextStage585620 target has no admitted generator never takes the
    // player: like an unadmitted doorway, the frontier refuses the entry.
    let admitted = next_stage(state.stage).is_ok();
    let player = player_world(state);
    let (dx, dy) = (player.x - position.x, player.y - position.y);
    let distance = (dx * dx + dy * dy).sqrt();
    let trapdoor = state.room.trapdoor.as_mut().expect("checked trapdoor");
    // 571225: a near player keeps it closed until the open clip passes frame9.
    if 50.0 > distance && trapdoor.frame as i32 <= 9 {
        trapdoor.play(TrapdoorClip::Closed);
    } else {
        trapdoor.play(TrapdoorClip::OpenAnimation);
        if !trapdoor.playing {
            trapdoor.state = 1;
        }
    }
    let enter = 21.0 > distance && trapdoor.state == 1 && admitted;
    if enter && !state.player_animation.guard {
        // Game+a1d8 is 16 only in the same-stage branch; normal runs pass false.
        start(state, false, 0);
        crate::player_animation::force_play(state, crate::player_animation::Clip::Trapdoor);
        set_player_world(state, position);
    }
    state
        .room
        .trapdoor
        .as_mut()
        .expect("checked trapdoor")
        .advance_sprite();
    Ok(())
}

/// Game::StartStageTransition5508d0 -> StageTransition::Start5e3800.
fn start(state: &mut State, same_stage: bool, animation: u32) {
    state.stage_transition = StageTransition {
        state: 1,
        counter: 0,
        same_stage,
        animation,
    };
    state.player.vx = 0.0;
    state.player.vy = 0.0;
    state.player.controls_enabled = false;
}

/// One full Game update while the transition is active (54f9da). The Game
/// counter, Room and fast updates stay frozen; StageTransition::Update5e38b0
/// ends every call by incrementing its counter, including after a reset.
pub(crate) enum Step {
    /// States1 and4 call the Player's vtable Update first, then finish with
    /// `after_player_update`.
    UpdatePlayer,
    /// State2 counter49: SetNextStage585620 then ShowNightmareScene595580.
    NextStage,
    /// State2 counter50: Appear clip, Player Update, visible, then
    /// Level::Init58be30 and the first Level update584200.
    AdmitLevel,
    Done,
}

pub(crate) fn advance(state: &mut State) -> Step {
    let transition = &mut state.stage_transition;
    let step = match transition.state {
        1 | 4 => return Step::UpdatePlayer,
        2 if transition.counter > 49 => {
            transition.counter = 0;
            transition.state = 3;
            Step::AdmitLevel
        }
        2 if transition.counter == 49 => Step::NextStage,
        3 if transition.counter > 49 => {
            transition.counter = 0;
            transition.state = 4;
            // 5e394e Level::ShowName585500(false) starts the stage streak.
            state.run_introduction = crate::RunIntroduction {
                age: 0,
                active: true,
                title: stage_name(state.stage).into(),
                subtitle: String::new(),
                stage: true,
            };
            Step::Done
        }
        _ => Step::Done,
    };
    state.stage_transition.counter += 1;
    step
}

/// The remainder of states1 and4 after the Player callback.
pub(crate) fn after_player_update(state: &mut State) {
    let guard = state.player_animation.guard;
    let transition = &mut state.stage_transition;
    match transition.state {
        1 => {
            let animation = transition.animation;
            if !guard && animation != 1 {
                state.player.visible = false;
            }
            let transition = &mut state.stage_transition;
            if (!guard || animation == 1)
                && (transition.counter > 20 || (animation == 1 && transition.counter > 5))
            {
                // 5e3aa7 also stops every playing SFX voice (SFX table5e1d00).
                transition.counter = 0;
                transition.state = 2;
            }
        }
        4 if !guard => {
            transition.state = 0;
            state.player.controls_enabled = true;
        }
        _ => {}
    }
    state.stage_transition.counter += 1;
}

/// StageTransition::Render5e3bd0 black quad alpha (GetFadeValue5e3c90).
pub fn fade(transition: &StageTransition) -> f32 {
    match transition.state {
        2 => transition.counter as f32 / 50.0,
        3 => 1.0 - transition.counter as f32 / 50.0,
        _ => 0.0,
    }
}

/// Level::SetNextStage585620 for the admitted ordinary stages. Alternate
/// stages need Seeds stage bit0 clear and a Manager unlock byte that a fresh
/// profile does not own; Curse of the Labyrinth is not admitted. Caves II
/// (stage 4) has no admitted generator, so Caves I's trapdoor stays shut.
pub(crate) fn next_stage(stage: u32) -> Result<u32, String> {
    match stage {
        1 => Ok(2),
        2 => Ok(3),
        _ => Err(format!(
            "stage {stage} has no admitted next-stage generator"
        )),
    }
}

/// Level::ShowName585500 names; alternate (Cellar) stages stay locked on a
/// fresh profile, so SetNextStage585620 never selects them here.
fn stage_name(stage: u32) -> &'static str {
    match stage {
        2 => "BASEMENT II",
        3 => "CAVES I",
        4 => "CAVES II",
        _ => "BASEMENT I",
    }
}

/// fix_award_pos5c8260/5c8320: an occupying grid object takes its vtable+30
/// destroy with immediate=1. Poop has no Destroy override (561c00 is
/// Hurt(4)), so a tail poop runs Poop Hurt56a390 with its draws, particles,
/// explosion and drop roll. An ordinary or tinted rock (an authored rock on a
/// Larry Jr. room's award cell) breaks through Rock.Destroy56c670, whose
/// immediate flag only changes bomb rocks. Other occupants' destroys,
/// unported Hurt branches and the 5c7f30 relocation when collision remains
/// are not admitted: those cells keep their object and the award still
/// spawns there.
fn clear_award_cell(state: &mut crate::State, index: usize) -> Result<(), String> {
    let occupant = state
        .room
        .grid
        .as_ref()
        .ok_or("boss clear award requires the admitted room grid")?
        .cells[index]
        .entity
        .as_ref()
        .map(|entity| entity.kind);
    match occupant {
        Some(14) => {
            let outcome = crate::attack_grid::callback_at_index(state, index, 4)?;
            outcome.record_frontier(state, "bossAward");
        }
        Some(2 | 4) => {
            crate::rock_blast::destroy_rock(state, index)?;
        }
        _ => {}
    }
    Ok(())
}
