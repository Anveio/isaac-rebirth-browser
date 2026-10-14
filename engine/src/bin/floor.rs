//! Full fresh-profile floor contract, separate from supplied-input kernels.
//! With `spawns`, each descriptor also carries the port's own authored spawn
//! selection, so scenario discovery can find rooms that admit an entity, and
//! per room the authored NPC selectors no registered family constructs yet.
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
#[serde(rename_all = "camelCase")]
struct WithSpawns {
    #[serde(flatten)]
    floor: rebirth_cleanroom::floor::Floor,
    /// Selected spawns per descriptor; null where no authored catalogue exists.
    spawns: Vec<Option<rebirth_cleanroom::room_spawns::RoomSpawns>>,
    /// Per descriptor: [type, variant] of authored NPCs (types 10..999) the
    /// port drops on admission.
    unsupported_npcs: Vec<Vec<[u32; 2]>>,
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
        .collect::<Result<Vec<_>, _>>()?;
    let unsupported_npcs = spawns
        .iter()
        .map(|room| {
            room.iter()
                .flat_map(|room| &room.selectors)
                .filter(|selector| {
                    selector.category == rebirth_cleanroom::room_spawns::SpawnCategory::Entity
                        && (10..1000).contains(&selector.entity_type)
                        && !rebirth_cleanroom::supports_npc_selector(selector)
                })
                .map(|selector| [u32::from(selector.entity_type), u32::from(selector.variant)])
                .collect()
        })
        .collect();
    println!(
        "{}",
        serde_json::to_string(&WithSpawns {
            floor,
            spawns,
            unsupported_npcs
        })?
    );
    Ok(())
}
