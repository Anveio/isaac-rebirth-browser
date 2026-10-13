//! Component verifier: explicit source inputs, independent of gameplay routing.
use rebirth_cleanroom::{
    motion::WorldVector,
    mt::{DescriptorSeeds, Mt19937, SavedGameSeeds},
    phase::{Clock, Phase},
    projectiles::{Eye, Parameters, TearMotion},
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{self, Read};

const MAX_INPUT: usize = 1_048_576;
const MAX_STEPS: usize = 36_000;

#[derive(Deserialize)]
#[serde(tag = "kernel", rename_all = "camelCase", deny_unknown_fields)]
enum Request {
    Mt {
        #[serde(rename = "bootEpochSeed")]
        boot_epoch_seed: u32,
        consumers: Vec<Consumer>,
    },
    Phase {
        #[serde(rename = "systemCounter")]
        system_counter: u32,
        #[serde(rename = "interpolationEnabled")]
        interpolation_enabled: bool,
        presentations: usize,
    },
    Projectile {
        #[serde(rename = "playerWorld")]
        player_world: WorldVector,
        #[serde(rename = "storedVelocity")]
        stored_velocity: WorldVector,
        eye: Eye,
        #[serde(rename = "eyeUniform")]
        eye_uniform: f32,
        #[serde(rename = "finalFallingUniform")]
        final_falling_uniform: f32,
        parameters: Parameters,
        #[serde(rename = "birthPhase")]
        birth_phase: Phase,
        steps: Vec<Step>,
    },
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
enum Consumer {
    Uint32,
    Uniform,
    Bounded {
        bound: u32,
    },
    InitializeDescriptorGrid {
        #[serde(default)]
        samples: Vec<GridQuery>,
    },
    ConstructSavedGame {
        #[serde(default)]
        samples: Vec<GridQuery>,
    },
    ClearSavedGame {
        #[serde(default)]
        samples: Vec<GridQuery>,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GridQuery {
    room_record: usize,
    grid_index: usize,
}

fn grid_samples(table: &DescriptorSeeds, queries: &[GridQuery]) -> Result<Value, String> {
    if queries.len() > 64 {
        return Err("too many descriptor seed samples".into());
    }
    queries.iter().map(|query| table.seed_pair(query.room_record, query.grid_index)
        .map(|seeds| json!({"roomRecord":query.room_record,"gridIndex":query.grid_index,"seeds":seeds})))
        .collect::<Result<Vec<_>,_>>().map(|samples| json!(samples))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    phase: Phase,
    #[serde(default)]
    collides: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin()
        .take((MAX_INPUT + 1) as u64)
        .read_to_string(&mut input)?;
    if input.len() > MAX_INPUT {
        return Err("physics request exceeds 1 MiB".into());
    }
    let response = match serde_json::from_str::<Request>(&input)? {
        Request::Mt {
            boot_epoch_seed,
            consumers,
        } => {
            if consumers.len() > MAX_STEPS {
                return Err("too many diagnostic MT consumers".into());
            }
            if consumers
                .iter()
                .filter(|consumer| {
                    matches!(
                        consumer,
                        Consumer::InitializeDescriptorGrid { .. }
                            | Consumer::ConstructSavedGame { .. }
                            | Consumer::ClearSavedGame { .. }
                    )
                })
                .count()
                > 64
            {
                return Err("too many diagnostic descriptor initializations".into());
            }
            let mut rng = Mt19937::new(boot_epoch_seed);
            let outputs: Vec<Value> = consumers
                .into_iter()
                .map(|consumer| match consumer {
                    Consumer::Uint32 => Ok(json!(rng.next_u32())),
                    Consumer::Uniform => Ok(json!(rng.uniform())),
                    Consumer::Bounded { bound } => Ok(json!(rng.bounded(bound))),
                    Consumer::InitializeDescriptorGrid { samples } => {
                        let table = DescriptorSeeds::initialize(&mut rng);
                        Ok(json!({"gridSeeds":grid_samples(&table, &samples)?}))
                    }
                    Consumer::ConstructSavedGame { samples } => {
                        let saved = SavedGameSeeds::construct(&mut rng);
                        Ok(json!({"gridSeeds":grid_samples(&saved.descriptor_seeds, &samples)?,"playerRngSeed":saved.player_rng_seed}))
                    }
                    Consumer::ClearSavedGame { samples } => {
                        let saved = SavedGameSeeds::clear(&mut rng);
                        Ok(json!({"gridSeeds":grid_samples(&saved.descriptor_seeds, &samples)?,"playerRngSeed":saved.player_rng_seed}))
                    }
                })
                .collect::<Result<_,String>>()?;
            json!({"schema":1,"kernel":"mt","outputs":outputs,"state":rng})
        }
        Request::Phase {
            system_counter,
            interpolation_enabled,
            presentations,
        } => {
            if presentations > MAX_STEPS {
                return Err("too many diagnostic presentations".into());
            }
            let mut clock = Clock::new(system_counter, interpolation_enabled);
            let phases = (0..presentations)
                .map(|_| clock.advance())
                .collect::<Result<Vec<_>, _>>()?;
            json!({"schema":1,"kernel":"phase","phases":phases,"state":clock})
        }
        Request::Projectile {
            player_world,
            stored_velocity,
            eye,
            eye_uniform,
            final_falling_uniform,
            parameters,
            birth_phase,
            steps,
        } => {
            if steps.len() > MAX_STEPS {
                return Err("too many diagnostic projectile phases".into());
            }
            let origin = rebirth_cleanroom::projectiles::eye_origin(
                player_world,
                stored_velocity,
                eye,
                eye_uniform,
            )?;
            let mut tear =
                TearMotion::spawn(origin, stored_velocity, parameters, final_falling_uniform)?;
            tear.present_birth(birth_phase)?;
            let mut states = vec![tear];
            for step in steps {
                tear.advance(step.phase, step.collides)?;
                states.push(tear);
            }
            json!({"schema":1,"kernel":"projectile","states":states})
        }
    };
    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}
