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

fn pickup_catalog(path: &Path, animations: &Path, entities_xml: &str) -> String {
    let catalog: Value =
        serde_json::from_str(&fs::read_to_string(path).expect("read authored pickup catalog"))
            .expect("parse authored pickup catalog");
    let actors: Value = serde_json::from_str(
        &fs::read_to_string(animations).expect("read authored animation catalog"),
    )
    .expect("parse authored animation catalog");
    let mut output = String::from("pub static DEFINITIONS: &[Definition] = &[\n");
    for pickup in catalog["pickups"]
        .as_array()
        .expect("pickup definitions missing")
    {
        let actor = pickup["actor"].as_str().expect("pickup actor missing");
        let mut clips = String::new();
        for animation in actors[actor]["Animations"]
            .as_array()
            .expect("authored pickup actor missing")
        {
            clips.push_str(&format!(
                "Clip {{ name: {:?}, frames: {}, looping: {} }},",
                animation["Name"]
                    .as_str()
                    .expect("pickup clip name missing"),
                animation["FrameNum"]
                    .as_u64()
                    .expect("pickup frame count missing"),
                animation["Loop"]
                    .as_bool()
                    .expect("pickup loop flag missing")
            ));
        }
        let variant = u32::try_from(pickup["variant"].as_u64().expect("pickup variant"))
            .expect("bounded variant");
        let mass = entity_number(entities_xml, 5, variant, "collisionMass");
        let friction = entity_number(entities_xml, 5, variant, "friction");
        let points = entity_number(entities_xml, 5, variant, "numGridCollisionPoints") as u16;
        output.push_str(&format!("Definition {{ variant: {}, subtype: {}, actor: {actor:?}, collision_radius: {}_f32, collision_mass: {mass:?}_f32, collision_points: {points}, friction: {friction:?}_f32, shadow_size: {}_f64, clips: &[{clips}] }},\n",
            pickup["variant"].as_u64().expect("pickup variant missing"),
            pickup["subtype"].as_u64().expect("pickup subtype missing"),
            pickup["collisionRadius"].as_f64().expect("pickup collision radius missing"),
            pickup["shadowSize"].as_f64().expect("pickup shadow size missing")));
    }
    output.push_str("];\n");
    let chest = &actors["gfx/005.050_chest.anm2"];
    let drop_id = chest["Events"]
        .as_array()
        .expect("authored Chest event list")
        .iter()
        .find(|event| event["Name"] == "DropSound")
        .and_then(|event| event["Id"].as_u64())
        .expect("authored Chest DropSound event");
    let drop_frame = chest["Animations"]
        .as_array()
        .expect("authored Chest animations")
        .iter()
        .find(|clip| clip["Name"] == "Appear")
        .and_then(|clip| clip["Triggers"].as_array())
        .and_then(|triggers| {
            triggers
                .iter()
                .find(|trigger| trigger["EventId"] == drop_id)
        })
        .and_then(|trigger| trigger["AtFrame"].as_u64())
        .expect("authored Chest Appear DropSound frame");
    output.push_str(&format!(
        "pub const CHEST_DROP_SOUND_FRAME: u32 = {drop_frame};\n"
    ));
    output
}

fn familiar_catalog(path: &Path, entities_xml: &str) -> String {
    let actors: Value = serde_json::from_str(
        &fs::read_to_string(path).expect("read authored Familiar/effect animations"),
    )
    .expect("parse authored Familiar/effect animations");
    let mut output = String::from("pub static DEFINITIONS: &[Definition] = &[\n");
    for (entity_type, variant, actor) in [
        (3, 56, "gfx/003.056_leech.anm2"),
        (1000, 5, "gfx/1000.005_blood particle.anm2"),
        (1000, 7, "gfx/1000.007_blood splat.anm2"),
        (1000, 15, "gfx/1000.015_poof01.anm2"),
        (1000, 49, "gfx/1000.049_heart.anm2"),
    ] {
        let catalogue = &actors[actor];
        let default = catalogue["DefaultAnimation"]
            .as_str()
            .expect("authored default clip");
        output.push_str(&format!("Definition {{ entity_type: {entity_type}, variant: {variant}, actor: {actor:?}, default_clip: {default:?}, "));
        for (field, attribute) in [
            ("base_hp", "baseHP"),
            ("collision_radius", "collisionRadius"),
            ("collision_mass", "collisionMass"),
            ("collision_damage", "collisionDamage"),
            ("friction", "friction"),
            ("shadow_size", "shadowSize"),
        ] {
            output.push_str(&format!(
                "{field}: {:?}_f32, ",
                entity_number(entities_xml, entity_type, variant, attribute)
            ));
        }
        let points = entity_number(entities_xml, entity_type, variant, "numGridCollisionPoints");
        assert_eq!(
            points.fract(),
            0.0,
            "integer authored collision point count"
        );
        output.push_str(&format!("grid_points: {points:.0}, clips: &["));
        for clip in catalogue["Animations"]
            .as_array()
            .expect("authored actor clips")
        {
            output.push_str(&format!(
                "Clip {{ name: {:?}, frames: {}, looping: {} }},",
                clip["Name"].as_str().expect("clip name"),
                clip["FrameNum"].as_u64().expect("clip frame count"),
                clip["Loop"].as_bool().expect("clip loop flag")
            ));
        }
        output.push_str("] },\n");
    }
    output.push_str("];\n");
    output
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
    let pickups = manifest.parent().unwrap().join("web/pickup-data.json");
    let npcs = manifest.parent().unwrap().join("web/npc-data.json");
    println!("cargo:rerun-if-changed={}", entities.display());
    println!("cargo:rerun-if-changed={}", image.display());
    println!("cargo:rerun-if-changed={}", animations.display());
    println!("cargo:rerun-if-changed={}", pickups.display());
    println!("cargo:rerun-if-changed={}", npcs.display());
    let xml = fs::read_to_string(entities).expect("read authored entity definitions");
    let (width, height) = png_dimensions(&image);
    let player = entity_number(&xml, 1, 0, "shadowSize");
    let tear = entity_number(&xml, 2, 0, "shadowSize");
    let bomb = entity_number(&xml, 4, 0, "shadowSize");
    let troll_bomb = entity_number(&xml, 4, 3, "shadowSize");
    let collision_radius = entity_number(&xml, 1, 0, "collisionRadius");
    let collision_mass = entity_number(&xml, 1, 0, "collisionMass");
    let mut generated = format!(
        "pub const SHADOW_SOURCE_WIDTH: f64 = {width}.0;\n\
         pub const SHADOW_SOURCE_HEIGHT: f64 = {height}.0;\n\
         pub const PLAYER_SHADOW_PERCENT: f64 = {player:?};\n\
         pub const TEAR_SHADOW_PERCENT: f64 = {tear:?};\n\
         pub const BOMB_SHADOW_PERCENT: f64 = {bomb:?};\n\
         pub const TROLL_BOMB_SHADOW_PERCENT: f64 = {troll_bomb:?};\n\
         pub const PLAYER_COLLISION_RADIUS_WORLD: f64 = {collision_radius:?};\n\
         pub const PLAYER_COLLISION_MASS_WORLD: f64 = {collision_mass:?};\n"
    );
    let player_actors: Value = serde_json::from_str(
        &fs::read_to_string(&animations).expect("read authored player animations"),
    )
    .expect("parse authored player animations");
    let pickup_clip = player_actors["gfx/001.000_player.anm2"]["Animations"]
        .as_array()
        .expect("authored player clips")
        .iter()
        .find(|clip| clip["Name"] == "Pickup")
        .expect("authored Pickup clip");
    assert_eq!(pickup_clip["Loop"], false, "Player Pickup is nonlooping");
    generated.push_str(&format!(
        "pub const PLAYER_PICKUP_FRAMES: u32 = {};\n",
        pickup_clip["FrameNum"]
            .as_u64()
            .expect("Pickup frame count")
    ));
    for name in ["Trapdoor", "Appear", "Death", "UseItem"] {
        let clip = player_actors["gfx/001.000_player.anm2"]["Animations"]
            .as_array()
            .expect("authored player clips")
            .iter()
            .find(|clip| clip["Name"] == name)
            .expect("authored guarded player clip");
        assert_eq!(clip["Loop"], false, "guarded Player clip is nonlooping");
        let frames = clip["FrameNum"].as_u64().expect("Player clip frames");
        assert!(frames > 0 && frames <= u64::from(u32::MAX));
        generated.push_str(&format!(
            "pub const PLAYER_{}_FRAMES: u32 = {frames};\n",
            name.to_uppercase()
        ));
    }
    let keeper_tag = xml
        .split("<entity ")
        .find(|tag| attribute(tag, "id") == Some("17") && attribute(tag, "variant") == Some("1"))
        .expect("admitted secret Keeper config is authored");
    let keeper_shut_doors = attribute(keeper_tag, "shutdoors").unwrap_or("true") == "true";
    let keeper_boss = entity_number(&xml, 17, 1, "boss") != 0.0;
    generated.push_str(&format!("pub const KEEPER_CAN_SHUT_DOORS: bool = {keeper_shut_doors};\npub const KEEPER_BOSS: bool = {keeper_boss};\n"));
    for field in [
        "baseHP",
        "collisionRadius",
        "collisionMass",
        "collisionDamage",
        "friction",
        "shadowSize",
    ] {
        generated.push_str(&format!(
            "pub const KEEPER_{}: f32 = {:?}_f32;\n",
            field.to_uppercase(),
            entity_number(&xml, 17, 1, field)
        ));
    }
    let npc_assets: Value =
        serde_json::from_str(&fs::read_to_string(npcs).expect("read authored NPC animations"))
            .expect("authored NPC animation JSON");
    let keeper_animations = npc_assets["gfx/017.002_secret room keeper.anm2"]["Animations"]
        .as_array()
        .expect("secret Keeper animations are authored");
    let keeper_frames: Vec<u32> = (1..=9)
        .map(|subtype| {
            let name = format!("Guy{subtype}");
            u32::try_from(
                keeper_animations
                    .iter()
                    .find(|animation| animation["Name"] == name)
                    .expect("every derived Keeper subtype has an authored clip")["FrameNum"]
                    .as_u64()
                    .expect("bounded authored Keeper frame count"),
            )
            .expect("Keeper frame count fits native actor clock")
        })
        .collect();
    generated.push_str(&format!(
        "pub const KEEPER_ANIMATION_FRAMES: [u32; 9] = {keeper_frames:?};\n"
    ));
    let light = authored_light_frame(&animations);
    let npc_animations = manifest.parent().unwrap().join("web/npc-data.json");
    println!("cargo:rerun-if-changed={}", npc_animations.display());
    let npc_clips: Value = serde_json::from_str(
        &fs::read_to_string(npc_animations).expect("read authored NPC animations"),
    )
    .expect("parse authored NPC animations");
    let boss_scene = npc_clips["gfx/ui/boss/versusscreen.anm2"]["Animations"]
        .as_array()
        .expect("boss Scene animations missing")
        .iter()
        .find(|clip| clip["Name"] == "Scene")
        .expect("authored boss Scene missing");
    generated.push_str(&format!(
        "pub const BOSS_SCENE_FRAMES: u32 = {};\n",
        boss_scene["FrameNum"].as_u64().unwrap()
    ));
    for size in [5, 6, 7] {
        let projectile_name = format!("RegularTear{size}");
        let projectile_clip = npc_clips["gfx/009.000_projectile.anm2"]["Animations"]
            .as_array()
            .expect("projectile clips missing")
            .iter()
            .find(|clip| clip["Name"] == projectile_name)
            .expect("regular projectile clip missing");
        generated.push_str(&format!(
            "pub const PROJECTILE_REGULAR{size}_FRAMES: u32 = {};\n",
            projectile_clip["FrameNum"].as_u64().unwrap()
        ));
    }
    let level2 = &npc_clips["gfx/213.000_level2fly.anm2"]["Animations"][0];
    assert_eq!(level2["Name"], "Idle");
    assert_eq!(level2["Loop"], true);
    generated.push_str(&format!(
        "pub const LEVEL2_FLY_IDLE_FRAMES: u32 = {};\n",
        level2["FrameNum"].as_u64().unwrap()
    ));
    for clip in npc_clips["gfx/020.000_monstro.anm2"]["Animations"]
        .as_array()
        .expect("Monstro clips missing")
    {
        let name = clip["Name"].as_str().unwrap();
        generated.push_str(&format!(
            "pub const MONSTRO_{}_FRAMES: u32 = {};\n",
            name.to_uppercase(),
            clip["FrameNum"].as_u64().unwrap()
        ));
        if name == "Death" {
            for trigger in clip["Triggers"]
                .as_array()
                .expect("Monstro Death triggers missing")
            {
                let event = match trigger["EventId"].as_u64().expect("Monstro event missing") {
                    0 => "BLOODSTART",
                    1 => "EXPLOSION",
                    _ => panic!("unexpected Monstro Death event"),
                };
                generated.push_str(&format!(
                    "pub const MONSTRO_{event}_FRAME: u32 = {};\n",
                    trigger["AtFrame"].as_u64().unwrap()
                ));
            }
        }
    }
    for clip in npc_clips["gfx/244.000_round worm.anm2"]["Animations"]
        .as_array()
        .expect("Round Worm clips missing")
    {
        let name = clip["Name"].as_str().unwrap();
        generated.push_str(&format!(
            "pub const WORM_{}_FRAMES: u32 = {};\n",
            name.to_uppercase(),
            clip["FrameNum"].as_u64().unwrap()
        ));
        if name == "DigOut" {
            let trigger = clip["Triggers"]
                .as_array()
                .expect("DigOut triggers missing")
                .iter()
                .find(|trigger| trigger["EventId"] == 0)
                .expect("authored Shoot trigger missing");
            generated.push_str(&format!(
                "pub const WORM_SHOOT_FRAME: u32 = {};\n",
                trigger["AtFrame"].as_u64().expect("Shoot frame missing")
            ));
        }
    }
    for name in ["Flickering", "NoFire", "Dissapear"] {
        let clips = npc_clips["gfx/033.000_fireplace.anm2"]["Animations"]
            .as_array()
            .expect("ordinary Fire clips missing");
        let clip = clips
            .iter()
            .find(|clip| clip["Name"] == name)
            .expect("ordinary Fire clip missing");
        let frames = clip["FrameNum"].as_u64().expect("Fire frame count missing");
        for subtype in 2..=3 {
            let suffix = format!("{name}{subtype}");
            let sibling = clips
                .iter()
                .find(|clip| clip["Name"] == suffix)
                .expect("Fire subtype clip missing");
            assert_eq!(sibling["FrameNum"], clip["FrameNum"]);
            assert_eq!(sibling["Loop"], clip["Loop"]);
        }
        generated.push_str(&format!(
            "pub const FIRE_{}_FRAMES: u32 = {frames};\n",
            name.to_uppercase()
        ));
    }
    let ember = npc_clips["gfx/1000.066_ember particle.anm2"]["Animations"]
        .as_array()
        .expect("Ember clips missing")
        .iter()
        .find(|clip| clip["Name"] == "Idle")
        .expect("Ember Idle missing");
    generated.push_str(&format!(
        "pub const EMBER_IDLE_FRAMES: u32 = {};\n",
        ember["FrameNum"].as_u64().unwrap()
    ));
    for kind in [9, 13, 18, 20, 33, 214, 244] {
        let tag = xml
            .split("<entity ")
            .find(|tag| {
                attribute(tag, "id").and_then(|value| value.parse::<u32>().ok()) == Some(kind)
                    && attribute(tag, "variant").and_then(|value| value.parse::<u32>().ok())
                        == Some(0)
            })
            .expect("admitted NPC config is authored");
        if kind != 9 {
            if kind == 244 {
                // Config44bd90 parses the gibs child; its amount is not an
                // entity-root attribute. Lookup449590 exposes +48/+50.
                let child = tag
                    .split("<gibs ")
                    .nth(1)
                    .and_then(|child| child.split('>').next())
                    .expect("authored Round Worm gibs child absent");
                let amount = attribute(child, "amount")
                    .unwrap_or("0")
                    .parse::<u32>()
                    .expect("invalid authored Round Worm gib amount");
                let mut flags = 0;
                for (name, bit) in [
                    ("blood", 1),
                    ("bone", 2),
                    ("gut", 4),
                    ("eye", 8),
                    ("large", 16),
                ] {
                    if attribute(child, name).unwrap_or("0") != "0" {
                        flags |= bit;
                    }
                }
                generated.push_str(&format!(
                    "pub const NPC_244_GIB_AMOUNT: u32 = {amount};\npub const NPC_244_GIB_FLAGS: u32 = {flags};\n"
                ));
            }
            let can_shut_doors = attribute(tag, "shutdoors").unwrap_or("true") == "true";
            let boss = entity_number(&xml, kind, 0, "boss") != 0.0;
            generated.push_str(&format!("pub const NPC_{kind}_CAN_SHUT_DOORS: bool = {can_shut_doors};\npub const NPC_{kind}_BOSS: bool = {boss};\n"));
            let points = entity_number(&xml, kind, 0, "numGridCollisionPoints") as u16;
            let class = match attribute(tag, "gridCollision") {
                Some("none") => 0,
                Some("walls") => 3,
                Some("ground") => 5,
                None => {
                    if points > 0 {
                        5
                    } else {
                        3
                    }
                }
                Some(value) => panic!("unsupported authored NPC grid class {value}"),
            };
            generated.push_str(&format!(
                "pub const NPC_{kind}_GRID_CLASS: u32 = {class};\n"
            ));
            if matches!(kind, 20 | 214 | 244) {
                generated.push_str(&format!(
                    "pub const NPC_{kind}_GRID_POINTS: u16 = {points};\n"
                ));
            }
        }
        for field in [
            "baseHP",
            "collisionRadius",
            "collisionMass",
            "collisionDamage",
            "friction",
            "shadowSize",
        ] {
            if kind == 9 && !matches!(field, "collisionRadius" | "collisionMass") {
                continue;
            }
            generated.push_str(&format!(
                "pub const NPC_{kind}_{}: f32 = {:?}_f32;\n",
                field.to_uppercase(),
                entity_number(&xml, kind, 0, field)
            ));
        }
    }
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
    fs::write(
        output.join("pickup_catalog.rs"),
        pickup_catalog(&pickups, &animations, &xml),
    )
    .expect("write pickup catalog");
    let audio_path = manifest.parent().unwrap().join("web/audio-data.json");
    println!("cargo:rerun-if-changed={}", audio_path.display());
    let audio: Value = serde_json::from_str(
        &fs::read_to_string(audio_path).expect("read authored audio catalogue"),
    )
    .expect("parse authored audio catalogue");
    let mut counts = [0_u8; 324];
    for sound in audio["sounds"].as_array().expect("sound catalogue missing") {
        let id = sound["id"].as_u64().expect("sound ID missing") as usize;
        let samples = sound["samples"].as_array().expect("sound samples missing");
        assert!(
            id < counts.len() && counts[id] == 0 && !samples.is_empty() && samples.len() <= 5,
            "invalid authored sound entry"
        );
        assert!(
            samples.iter().all(|sample| sample["weight"] == 1),
            "base sample selection requires equal authored weights"
        );
        counts[id] = samples.len() as u8;
    }
    fs::write(
        output.join("audio_catalog.rs"),
        format!("pub const SOUND_COUNTS: [u8; 324] = {counts:?};\n"),
    )
    .expect("write authored audio catalogue");
    let mut tracks = String::from("pub const MUSIC_TRACKS: &[MusicTrack] = &[\n");
    for track in audio["music"].as_array().expect("music catalogue missing") {
        let id = track["id"].as_u64().expect("music ID missing");
        assert!(id <= 101, "invalid authored music ID");
        let path = track["path"].as_str().expect("music path missing");
        let optional = |field: &str| match track[field].as_str() {
            Some(path) => format!("Some({path:?})"),
            None => "None".into(),
        };
        tracks.push_str(&format!(
            "MusicTrack {{ id: {id}, path: {path:?}, intro: {}, layer: {}, layer_intro: {}, looping: {} }},\n",
            optional("intro"), optional("layer"), optional("layerintro"),
            track["loop"].as_bool().expect("music loop missing"),
        ));
    }
    tracks.push_str("];\n");
    fs::write(output.join("music_catalog.rs"), tracks).expect("write authored music catalogue");
    let item_decoder = manifest
        .parent()
        .unwrap()
        .join("tools/foundation/item-catalog.py");
    for path in [
        &item_decoder,
        &resources.join("items.xml"),
        &resources.join("itempools.xml"),
    ] {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    let items = std::process::Command::new("python3")
        .arg(item_decoder)
        .arg("--rust")
        .output()
        .expect("decode pinned base collectible resources");
    assert!(
        items.status.success(),
        "collectible catalogue rejected: {}",
        String::from_utf8_lossy(&items.stderr)
    );
    let familiar_path = manifest.parent().unwrap().join("web/familiar-data.json");
    println!("cargo:rerun-if-changed={}", familiar_path.display());
    fs::write(
        output.join("familiar_catalog.rs"),
        familiar_catalog(&familiar_path, &xml),
    )
    .expect("write authored Familiar/effect catalogue");
    fs::write(output.join("item_catalog.rs"), items.stdout)
        .expect("write authored collectible catalogue");
}
