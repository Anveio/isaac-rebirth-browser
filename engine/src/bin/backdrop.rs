//! Reference verifier entry: supply a room decoration seed, never a run seed.
use rebirth_cleanroom::Room;
use serde::Deserialize;
use std::io::{self, Read};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    decoration_seed: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().take(4097).read_to_string(&mut input)?;
    if input.len() > 4096 {
        return Err("backdrop request exceeds 4 KiB".into());
    }
    let request: Request = serde_json::from_str(&input)?;
    println!(
        "{}",
        serde_json::to_string(&Room::from_decoration_seed(request.decoration_seed).appearance)?
    );
    Ok(())
}
