//! Gemini 79.0 and Gemini Baby 79.10: ai 4cb8d0 (parent 48fd50, baby
//! 4bc640). The authored parent spawns its baby (linked parent/child, first
//! update at once) and 18 umbilical cord pieces 79.20 it drapes between the
//! two. The parent chases with FindGridPath, rages and pants; the tethered
//! baby drifts toward the player and spits predictive shots, and rages
//! alone once orphaned.
use crate::{
    audio::SoundRequest,
    content,
    entities::{Birth, Family, NpcWorld, SpawnContext},
    motion::WorldVector,
    mt::Mt19937,
    npcs::{Npc, Overlay},
    pathfinder::{self, Pathfinder},
    phase::Phase,
    plain_npc::{self, ShooterTimers, ShotAdjust, ShotMode, ShotParameters},
    room_spawns::{SpawnCategory, SpawnSelector},
};
use serde::{Deserialize, Serialize};

pub(crate) const FAMILY: Family = Family {
    kinds: &[79],
    spawn,
    advance,
    post_contact_grid: crate::entities::ground_collision,
    entity_flags: crate::entities::ordinary_flags,
    occupancy: crate::entities::Occupancy::None,
};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };
const BABY: u16 = 10;
const CORD: u16 = 20;
const CORDS: usize = 18;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeminiState {
    /// Entity_NPC+f58 state counter.
    pub counter: i32,
    /// +1038 walk speed.
    pub speed: f32,
}

fn spawn(id: u32, selector: &SpawnSelector, context: &SpawnContext) -> Result<Option<Npc>, String> {
    // Steven 79.1 and The Blighted Ovum 79.2 have their own branches; base
    // Basement rooms author Gemini 79.0.0, which spawns 79.10 and 79.20.
    let authored = selector.variant == 0 && selector.subtype == 0;
    if selector.category != SpawnCategory::Entity
        || selector.entity_type != 79
        || !(authored || selector.variant == BABY || selector.variant == CORD)
    {
        return Ok(None);
    }
    if !selector.position.is_finite() || selector.spawn_seed == 0 {
        return Err("Gemini requires bounded position and nonzero spawn seed".into());
    }
    let config = content::entity(79, selector.variant.into());
    let mut npc = Npc {
        // Config47f573 SetFrame(default, 0) leaves the default clip stopped.
        animation: config_default(selector.variant).into(),
        actor: config.actor.into(),
        pathfinder: (selector.variant == 0).then(Pathfinder::new),
        shooter: (selector.variant == BABY).then(ShooterTimers::default),
        gemini: Some(GeminiState {
            counter: 0,
            speed: 0.0,
        }),
        ..Npc::authored(id, config, selector)
    };
    npc.max_hp = content::initial_hp(config, context.stage);
    npc.hp = npc.max_hp;
    Ok(Some(npc))
}

fn config_default(variant: u16) -> &'static str {
    match variant {
        0 => "Appear",
        BABY => "Walk01",
        _ => "CordPiece",
    }
}

fn advance(
    npc: &mut Npc,
    phase: Phase,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
) -> Result<(), String> {
    if npc.variant == u32::from(CORD) {
        // The cord's wrapper clears APPEAR before 4f9a46 tests it.
        return plain_npc::advance_with(npc, phase, random, world, births, ai, false);
    }
    plain_npc::advance_dead_ai(npc, phase, random, world, births, ai)
}

fn sound(random: &mut Mt19937, world: &mut NpcWorld, id: u32) {
    world.audio.admit(
        random,
        world.system_counter,
        SoundRequest {
            id,
            volume: 1.0,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
}

/// 4cb8d0: friction x0.8, then the parent, baby or cord body.
fn ai(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    *friction *= 0.8;
    if npc.variant == u32::from(CORD) {
        return Ok(());
    }
    let mut state = npc.gemini.take().ok_or("Gemini requires its AI state")?;
    let result = if npc.variant == 0 {
        parent(npc, &mut state, random, world, births, friction)
    } else {
        baby(npc, &mut state, random, world, births, friction)
    };
    npc.gemini = Some(state);
    result
}

fn selector(npc: &Npc, variant: u16, offset: WorldVector, spawn_seed: u32) -> SpawnSelector {
    SpawnSelector {
        group_index: 0,
        grid_index: 0,
        category: SpawnCategory::Entity,
        entity_type: 79,
        variant,
        subtype: 0,
        position: npc.position + offset,
        group_seed: spawn_seed,
        spawn_seed,
    }
}

fn hold_overlay(npc: &mut Npc, animation: &str) {
    npc.overlay = Some(Overlay {
        animation: animation.into(),
        frame: 0,
        clock: 0.0,
        playing: false,
    });
}

fn play_overlay(npc: &mut Npc, animation: &str) {
    if npc.overlay.as_ref().is_some_and(|overlay| overlay.animation == animation && overlay.playing) {
        return;
    }
    npc.overlay = Some(Overlay {
        animation: animation.into(),
        frame: 0,
        clock: 0.0,
        playing: true,
    });
}

fn chase(
    npc: &mut Npc,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    speed: f32,
    friction: &mut f32,
) -> Result<(), String> {
    let target = world.player;
    pathfinder::find_grid_path(npc, random, world, target, 0.83 * speed, 900, friction)
}

/// 48fd50.
fn parent(
    npc: &mut Npc,
    state: &mut GeminiState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    match npc.state {
        0 => {
            // 48fee0: the baby 8 above (updated at once), then 18 cords.
            let seed = random.next_u32();
            births.push(Birth::Child(selector(
                npc,
                BABY,
                WorldVector { x: 0.0, y: -8.0 },
                seed,
            )));
            for _ in 0..CORDS {
                let seed = random.next_u32();
                births.push(Birth::Attached(selector(npc, CORD, ZERO, seed)));
            }
            npc.state = 4;
        }
        4 => {
            plain_npc::walk_frame(npc, "WalkHori", "WalkVert", 0.1);
            state.speed = 1.4;
            chase(npc, random, world, state.speed, friction)?;
            hold_overlay(npc, "IdleHead");
            if random.bounded(60) == 0 {
                sound(random, world, 122);
                state.counter = 0;
                npc.state = 8;
            }
        }
        8 => {
            state.speed = (0.04 + state.speed) * 0.98;
            chase(npc, random, world, state.speed, friction)?;
            plain_npc::walk_frame(npc, "WalkHori", "WalkVert", 0.1);
            play_overlay(npc, "RageBody");
            state.counter += 1;
            if state.counter > 100 {
                state.counter = 0;
                npc.state = 3;
            }
        }
        3 => {
            npc.overlay = None;
            plain_npc::play(npc, "Pant", false);
            state.speed = 0.0;
            state.counter += 1;
            if state.counter > 100 {
                npc.state = 4;
                plain_npc::set_frame(npc, "WalkVert", 0);
                hold_overlay(npc, "IdleHead");
            }
        }
        _ => {}
    }
    tether(npc, world)
}

/// 48fda0: forget a detached child; pull the baby beyond 76 and drape the
/// cords between the two; with no baby (or dead) remove the cords.
fn tether(npc: &mut Npc, world: &mut NpcWorld) -> Result<(), String> {
    let child = npc.child.and_then(|id| world.peers.get(id).map(|baby| (id, baby.parent)));
    if let Some((_, parent)) = child {
        if parent.is_none() {
            npc.child = None;
        }
    }
    let baby = npc
        .child
        .and_then(|id| world.peers.get(id))
        .map(|baby| (baby.position, baby.base_friction));
    let (Some((baby_position, baby_friction)), false) = (baby, npc.dead) else {
        for piece in world
            .peers
            .before
            .iter_mut()
            .chain(world.peers.after.iter_mut())
            .filter(|other| other.kind == 79 && other.variant == u32::from(CORD) && other.parent == Some(npc.id))
        {
            piece.removed = true;
            piece.dead = true;
        }
        return Ok(());
    };
    let delta = WorldVector {
        x: npc.position.x - baby_position.x,
        y: npc.position.y - baby_position.y,
    };
    let length = (delta.x * delta.x + delta.y * delta.y).sqrt();
    if length > 76.0 && baby_friction != 0.0 {
        let pull = ((delta * (length - 76.0)) * 0.01) * (1.0 / length);
        if let Some(baby) = npc.child.and_then(|id| world.peers.get_mut(id)) {
            baby.velocity = baby.velocity
                + WorldVector {
                    x: pull.x / baby_friction,
                    y: pull.y / baby_friction,
                };
        }
    }
    let slack = if 196.0 - length > 0.0 { (196.0 - length) / 196.0 } else { 0.0 };
    let mut k = 0.0_f32;
    for piece in world
        .peers
        .before
        .iter_mut()
        .chain(world.peers.after.iter_mut())
        .filter(|other| other.kind == 79 && other.variant == u32::from(CORD) && other.parent == Some(npc.id))
    {
        let sag = 16.0 - ((k - 9.0).abs() * 0.5).powi(2);
        let t = (16.0 - k) / 18.0;
        piece.position.x = (npc.position.x - delta.x * t) - 6.0;
        piece.position.y = (((npc.position.y - delta.y * t) - 20.0) + (sag * slack) * 1.5) + k * 0.8;
        piece.presented_position = piece.position;
        piece.visible = true;
        k += 1.0;
    }
    Ok(())
}

/// 4bc640.
fn baby(
    npc: &mut Npc,
    _: &mut GeminiState,
    random: &mut Mt19937,
    world: &mut NpcWorld,
    births: &mut Vec<Birth>,
    friction: &mut f32,
) -> Result<(), String> {
    if npc.state == 0 {
        npc.state = 4;
        return Ok(());
    }
    npc.grid_collision_class = 3;
    let toward = WorldVector {
        x: world.player.x - npc.position.x,
        y: world.player.y - npc.position.y,
    };
    let unit = normalized(toward);
    if npc.parent.is_none() {
        // 4.3b: orphaned rage, clamped to the room (top 30, bottom 10).
        let rage_playing = npc.animation == "Rage" && npc.animation_playing;
        let walking = npc.animation == "Walk02" && npc.animation_playing;
        if npc.animation == "Rage" && plain_npc::finished(npc) {
            plain_npc::play(npc, "Walk02", false);
        } else if !walking && !rage_playing {
            plain_npc::play(npc, "Rage", true);
        }
        let (minimum, maximum) = crate::laser_npc_runtime::room_bounds_from_grid(world.grid);
        npc.position.x = npc.position.x.min(maximum.x).max(minimum.x);
        npc.position.y = npc.position.y.min(maximum.y - 10.0).max(minimum.y + 30.0);
        // 4bceaf 612bb0 divides: v /= 0.85.
        npc.velocity = WorldVector {
            x: npc.velocity.x / 0.85,
            y: npc.velocity.y / 0.85,
        };
        npc.velocity = npc.velocity
            + WorldVector {
                x: (unit.x * 0.5) / *friction,
                y: (unit.y * 0.5) / *friction,
            };
        npc.collision_mass = 8.0;
    } else {
        npc.velocity = npc.velocity
            + WorldVector {
                x: (unit.x * 0.25) / *friction,
                y: (unit.y * 0.25) / *friction,
            };
        let parameter_angle = (f64::from(random.uniform()) * std::f64::consts::PI) as f32;
        if npc.shooter.as_ref().is_some_and(|timers| timers.delay == 0) {
            sound(random, world, 318);
            sound(random, world, 178);
        }
        let shot = ShotParameters {
            mode: ShotMode::Single,
            range: 300.0,
            cooldown: 5,
            delay: 17,
            speed: 7.0,
            parameter_angle,
            lead: 5.0,
            adjust: Some(ShotAdjust {
                height: -15.0,
                falling_speed: 3.0,
                velocity_scale: 1.2,
            }),
        };
        if plain_npc::try_shoot_player(npc, random, world, births, shot)? {
            plain_npc::play(npc, "Attack01", true);
        } else if plain_npc::finished(npc) {
            plain_npc::play(npc, "Walk01", false);
        }
    }
    if toward.x.abs() > toward.y.abs() {
        npc.flip_x = toward.x < 0.0;
    }
    Ok(())
}

fn normalized(vector: WorldVector) -> WorldVector {
    let length = (vector.x * vector.x + vector.y * vector.y).sqrt();
    if length > 0.0 {
        vector * (1.0 / length)
    } else {
        vector
    }
}
