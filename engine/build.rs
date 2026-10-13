use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let marker = format!("{name}=\"");
    let start = tag.find(&marker)? + marker.len();
    let end = tag[start..].find('"')? + start;
    Some(&tag[start..end])
}

fn entity_number(xml: &str, id: u32, variant: u32, field: &str) -> f64 {
    let tag = xml
        .split("<entity ")
        .find(|tag| {
            attribute(tag, "id").and_then(|value| value.parse::<u32>().ok()) == Some(id)
                && attribute(tag, "variant").and_then(|value| value.parse::<u32>().ok())
                    == Some(variant)
        })
        .unwrap_or_else(|| panic!("authored entity {id}/{variant} missing"));
    let value = attribute(tag, field)
        .unwrap_or_else(|| panic!("entity {id}/{variant} {field} missing"))
        .parse::<f64>()
        .unwrap_or_else(|_| panic!("invalid entity {id}/{variant} {field}"));
    assert!(
        value.is_finite() && value >= 0.0,
        "invalid authored entity numeric value"
    );
    value
}

fn png_dimensions(path: &Path) -> (u32, u32) {
    let bytes = fs::read(path).expect("read authored shadow PNG");
    assert_eq!(
        bytes.get(..8),
        Some(b"\x89PNG\r\n\x1a\n".as_slice()),
        "invalid shadow PNG"
    );
    assert_eq!(
        bytes.get(12..16),
        Some(b"IHDR".as_slice()),
        "PNG IHDR missing"
    );
    let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    assert!(width > 0 && height > 0, "empty shadow PNG");
    (width, height)
}

fn authored_light_frame(path: &Path) -> Value {
    let actors: Value =
        serde_json::from_str(&fs::read_to_string(path).expect("read authored animation catalog"))
            .expect("parse authored animation catalog");
    let actor = &actors["gfx/lightgradient.anm2"];
    let animation = actor["Animations"]
        .as_array()
        .expect("lightgradient animations missing")
        .iter()
        .find(|animation| animation["Name"] == "Default")
        .expect("lightgradient Default animation missing");
    let layer = animation["LayerAnimations"]
        .as_array()
        .expect("lightgradient layers missing")
        .iter()
        .find(|layer| layer["LayerId"] == 0)
        .expect("lightgradient Main layer missing");
    layer["Frames"][0].clone()
}

fn frame_number(frame: &Value, name: &str) -> f64 {
    let value = frame[name]
        .as_f64()
        .unwrap_or_else(|| panic!("lightgradient {name} missing"));
    assert!(value.is_finite(), "non-finite authored lightgradient value");
    value
}

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let resources = manifest
        .parent()
        .unwrap()
        .join("references/isaac/rebirth/resources");
    let entities = resources.join("entities2.xml");
    let image = resources.join("gfx/shadow.png");
    let animations = manifest.parent().unwrap().join("web/scene-data.json");
    println!("cargo:rerun-if-changed={}", entities.display());
    println!("cargo:rerun-if-changed={}", image.display());
    println!("cargo:rerun-if-changed={}", animations.display());
    let xml = fs::read_to_string(entities).expect("read authored entity definitions");
    let (width, height) = png_dimensions(&image);
    let player = entity_number(&xml, 1, 0, "shadowSize");
    let tear = entity_number(&xml, 2, 0, "shadowSize");
    let bomb = entity_number(&xml, 4, 0, "shadowSize");
    let collision_radius = entity_number(&xml, 1, 0, "collisionRadius");
    let mut generated = format!(
        "pub const SHADOW_SOURCE_WIDTH: f64 = {width}.0;\n\
         pub const SHADOW_SOURCE_HEIGHT: f64 = {height}.0;\n\
         pub const PLAYER_SHADOW_PERCENT: f64 = {player:?};\n\
         pub const TEAR_SHADOW_PERCENT: f64 = {tear:?};\n\
         pub const BOMB_SHADOW_PERCENT: f64 = {bomb:?};\n\
         pub const PLAYER_COLLISION_RADIUS_WORLD: f64 = {collision_radius:?};\n"
    );
    let light = authored_light_frame(&animations);
    for (constant, attribute) in [
        ("LIGHT_CROP_X", "XCrop"),
        ("LIGHT_CROP_Y", "YCrop"),
        ("LIGHT_WIDTH", "Width"),
        ("LIGHT_HEIGHT", "Height"),
        ("LIGHT_PIVOT_X", "XPivot"),
        ("LIGHT_PIVOT_Y", "YPivot"),
    ] {
        generated.push_str(&format!(
            "pub const {constant}: f64 = {:?};\n",
            frame_number(&light, attribute)
        ));
    }
    let output = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    fs::write(output.join("shadow_catalog.rs"), generated).expect("write shadow catalog");
}
