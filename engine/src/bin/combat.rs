//! Conditional source-kernel diagnostic; no live enemy or room routing implied.
use rebirth_cleanroom::{
    combat::{self, ActorHealth, ContactBody, DamageEvent, DamageQueue},
    grid::{Collider, CollisionGeometry, Grid, GridDescriptor},
    motion::{self, WorldVector},
    phase::Clock,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{self, Read};

const MAX_INPUT_BYTES: u64 = 1_048_576;
const MAX_PRESENTATIONS: usize = 36_000;

#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
struct Control {
    x: i32,
    y: i32,
}

impl Control {
    fn validate(&self) -> Result<(), String> {
        if !(-1..=1).contains(&self.x) || !(-1..=1).contains(&self.y) {
            return Err("control components must be -1, 0 or 1".into());
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Admission {
    counter: u32,
    event: DamageEvent,
}

#[derive(Deserialize)]
#[serde(
    tag = "operation",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum Request {
    GridBlast {
        grid: GridDescriptor,
        origin: WorldVector,
        radius: f32,
    },
    GridTrajectory {
        grid: GridDescriptor,
        geometry: CollisionGeometry,
        position: WorldVector,
        velocity: WorldVector,
        clock: Clock,
        inputs: Vec<Control>,
    },
    MassContact {
        player: ContactBody,
        npc: ContactBody,
        control: Control,
        grid: GridDescriptor,
        geometry: CollisionGeometry,
    },
    DeferredDamage {
        health: ActorHealth,
        clock: Clock,
        presentations: usize,
        admissions: Vec<Admission>,
    },
}

fn run(request: Request) -> Result<Value, String> {
    match request {
        Request::GridBlast {
            grid,
            origin,
            radius,
        } => {
            let grid = Grid::new(grid)?;
            let candidates = grid.blast_candidates(origin, radius)?;
            let rocks: Vec<usize> = candidates
                .iter()
                .copied()
                .filter(|&index| {
                    grid.descriptor().cells[index]
                        .entity
                        .as_ref()
                        .is_some_and(|entity| entity.kind == 2)
                })
                .collect();
            Ok(json!({"candidateCells":candidates,"rockCandidates":rocks}))
        }
        Request::GridTrajectory {
            grid,
            geometry,
            mut position,
            mut velocity,
            mut clock,
            inputs,
        } => {
            if inputs.len() > MAX_PRESENTATIONS {
                return Err("trajectory exceeds 36000 presentations".into());
            }
            if !position.is_finite() || !velocity.is_finite() {
                return Err("trajectory vectors must be finite".into());
            }
            for control in &inputs {
                control.validate()?;
            }
            let grid = Grid::new(grid)?;
            let collider = Collider::new(geometry)?;
            let mut frames = Vec::with_capacity(inputs.len() + 1);
            let mut contact = None;
            frames.push(
                json!({"counter":clock.system_counter,"position":position,"velocity":velocity,
                "collidedWithGrid":false,"normal":WorldVector{x:0.0,y:0.0}}),
            );
            for control in inputs {
                let phase = clock.advance()?;
                for _ in 0..phase.player_updates(clock.interpolation_enabled) {
                    (position, velocity) =
                        motion::advance_world(position, velocity, control.x, control.y)?;
                    contact =
                        collider.resolve_player(&grid, &mut position, &mut velocity, false)?;
                }
                frames.push(
                    json!({"counter":clock.system_counter,"phase":phase,"position":position,
                    "velocity":velocity,"collidedWithGrid":contact.is_some(),
                    "normal":contact.map_or(WorldVector{x:0.0,y:0.0},|contact|contact.normal)}),
                );
            }
            Ok(json!({"frames":frames}))
        }
        Request::MassContact {
            mut player,
            mut npc,
            control,
            grid,
            geometry,
        } => {
            control.validate()?;
            let grid = Grid::new(grid)?;
            let collider = Collider::new(geometry)?;
            (player.position, player.velocity) =
                motion::advance_world(player.position, player.velocity, control.x, control.y)?;
            let position = player.position;
            let result = combat::resolve_player_npc_contact(&mut player, &mut npc, |shift| {
                collider.blocks_player_shift(&grid, position, shift)
            })?;
            Ok(
                json!({"player":player,"npc":npc,"colliding":result.colliding,
                "playerDisplacementBlocked":result.player_displacement_blocked}),
            )
        }
        Request::DeferredDamage {
            mut health,
            mut clock,
            presentations,
            admissions,
        } => {
            if presentations > MAX_PRESENTATIONS || admissions.len() > MAX_PRESENTATIONS {
                return Err("damage diagnostic exceeds 36000 presentations or admissions".into());
            }
            health.validate()?;
            let final_counter = clock
                .system_counter
                .checked_add(presentations as u32)
                .ok_or("damage diagnostic counter exhausted")?;
            if admissions
                .iter()
                .any(|event| event.counter <= clock.system_counter || event.counter > final_counter)
            {
                return Err("damage admissions must follow the initial counter within the requested interval".into());
            }
            let mut queue = DamageQueue::default();
            let mut frames = Vec::with_capacity(presentations + 1);
            frames.push(json!({"counter":clock.system_counter,"hp":health.hp,"pendingDamage":0.0,"pendingFlags":0}));
            for _ in 0..presentations {
                let phase = clock.advance()?;
                let applied = queue.advance(&mut health, phase)?;
                for admission in admissions
                    .iter()
                    .filter(|event| event.counter == clock.system_counter)
                {
                    queue.enqueue(health, admission.event)?;
                }
                frames.push(json!({"counter":clock.system_counter,"phase":phase,"hp":health.hp,
                    "pendingDamage":queue.pending_damage(),"pendingFlags":queue.pending_flags(),"applied":applied}));
            }
            Ok(json!({"frames":frames}))
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin()
        .take(MAX_INPUT_BYTES + 1)
        .read_to_string(&mut input)?;
    if input.len() as u64 > MAX_INPUT_BYTES {
        return Err("combat request exceeds 1 MiB".into());
    }
    let result = run(serde_json::from_str(&input)?)?;
    println!(
        "{}",
        serde_json::to_string(&json!({"schema":1,
        "scope":"conditional source kernels; room routing, AI and browser integration are separate", "result":result}))?
    );
    Ok(())
}
