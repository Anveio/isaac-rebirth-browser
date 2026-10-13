//! Bounded diagnostic source-contract entry, separate from gameplay routing.
use rebirth_cleanroom::{layout, room_templates};
use serde::Deserialize;
use std::io::{self, Read};

#[derive(Deserialize)]
#[serde(
    tag = "operation",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum Request {
    Growth {
        generator_seed: u32,
        additional_rooms: u8,
        shape_mask: u8,
    },
    Templates {
        catalog: Vec<room_templates::AuthoredTemplate>,
        selections: Vec<room_templates::SelectionRequest>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().take(2_097_153).read_to_string(&mut input)?;
    if input.len() > 2_097_152 {
        return Err("layout diagnostic request exceeds 2 MiB".into());
    }
    let output = match serde_json::from_str::<Request>(&input)? {
        Request::Growth {
            generator_seed,
            additional_rooms,
            shape_mask,
        } => serde_json::to_value(layout::generate(layout::GrowthRequest {
            generator_seed,
            additional_rooms,
            shape_mask,
        })?)?,
        Request::Templates {
            catalog,
            selections,
        } => {
            if selections.is_empty() || selections.len() > 128 {
                return Err("template diagnostic requires 1..128 selections".into());
            }
            let mut picker = room_templates::TemplatePicker::new(catalog)?;
            let selected = selections
                .into_iter()
                .map(|request| picker.choose(request))
                .collect::<Result<Vec<_>, _>>()?;
            serde_json::to_value(selected)?
        }
    };
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}
