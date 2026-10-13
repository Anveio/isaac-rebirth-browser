//! Full fresh-profile floor contract, separate from supplied-input kernels.
use serde::Deserialize;
use std::io::{self, Read};
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    stage_seed: u32,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().take(4097).read_to_string(&mut input)?;
    if input.len() > 4096 {
        return Err("floor request exceeds 4096 bytes".into());
    }
    let request: Request = serde_json::from_str(&input)?;
    let floor = rebirth_cleanroom::floor::generate_basement(request.stage_seed)?;
    println!("{}", serde_json::to_string(&floor)?);
    Ok(())
}
