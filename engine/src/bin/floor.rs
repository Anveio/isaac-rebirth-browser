//! Full fresh-profile floor contract, separate from supplied-input kernels.
//! With `spawns`, each descriptor also carries the port's own authored spawn
//! selection, so scenario discovery can find rooms that admit an entity.
use serde::{Deserialize, Serialize};
use std::io::{self, Read};
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    stage_seed: u32,
    #[serde(default)]
    spawns: bool,
}
#[derive(Serialize)]
struct WithSpawns {
    #[serde(flatten)]
    floor: rebirth_cleanroom::floor::Floor,
    /// Selected spawns per descriptor; null where no authored catalogue exists.
    spawns: Vec<Option<rebirth_cleanroom::room_spawns::RoomSpawns>>,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().take(4097).read_to_string(&mut input)?;
    if input.len() > 4096 {
        return Err("floor request exceeds 4096 bytes".into());
    }
    let request: Request = serde_json::from_str(&input)?;
    let floor = rebirth_cleanroom::floor::generate_basement(request.stage_seed)?;
    if !request.spawns {
        println!("{}", serde_json::to_string(&floor)?);
        return Ok(());
    }
    let spawns = floor
        .descriptors
        .iter()
        .map(|descriptor| {
            rebirth_cleanroom::room_assets::for_descriptor(descriptor)
                .ok()
                .map(|asset| rebirth_cleanroom::room_spawns::select(&asset, descriptor.spawn_seed))
                .transpose()
        })
        .collect::<Result<_, _>>()?;
    println!("{}", serde_json::to_string(&WithSpawns { floor, spawns })?);
    Ok(())
}
