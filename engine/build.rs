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

fn nightmare_catalog(xml: &str, path: &Path) -> String {
    let actors: Value =
        serde_json::from_str(&fs::read_to_string(path).expect("read authored stage animations"))
            .expect("parse authored stage animations");
    let root_tag = xml.split("<nightmares ").nth(1).expect("nightmares root");
    let root = attribute(root_tag, "root")
        .expect("nightmares resources root")
        .trim_start_matches("resources/");
    let progress = format!(
        "{root}{}",
        attribute(root_tag, "progressAnm2").expect("progress actor")
    );
    let progress_frames = actors[&progress]["Animations"]
        .as_array()
        .expect("authored progress definition")
        .iter()
        .find(|clip| clip["Name"] == "Levels")
        .and_then(|clip| clip["FrameNum"].as_u64())
        .expect("authored progress Levels clip");
    assert!(progress_frames > 0 && progress_frames <= u64::from(u32::MAX));
    let mut output = format!(
        "pub const PROGRESS_ACTOR: &str = {progress:?};\npub static PROGRESS_DEFINITION: Definition = Definition {{ actor: {progress:?}, frames: {progress_frames} }};\n"
    );
    output.push_str("pub static DEFINITIONS: &[Definition] = &[\n");
    for tag in xml.split("<nightmare ").skip(1) {
        let actor = format!("{root}{}", attribute(tag, "anm2").expect("nightmare actor"));
        let clip = actors[&actor]["Animations"]
            .as_array()
            .expect("authored nightmare clips")
            .iter()
            .find(|clip| clip["Name"] == "Scene")
            .expect("authored nightmare Scene");
        let frames = clip["FrameNum"].as_u64().expect("Scene frames");
        assert!(frames > 0 && frames <= u64::from(u32::MAX));
        assert_eq!(clip["Loop"], false, "nightmare Scene is nonlooping");
        output.push_str(&format!(
            "Definition {{ actor: {actor:?}, frames: {frames} }},\n"
        ));
    }
    output.push_str("];");
    output
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
        let drop_id = actors[actor]["Events"].as_array().and_then(|events| {
            events
                .iter()
                .find(|event| event["Name"] == "DropSound")
                .and_then(|event| event["Id"].as_u64())
        });
        let mut clips = String::new();
        for animation in actors[actor]["Animations"]
            .as_array()
            .expect("authored pickup actor missing")
        {
            let drop_sound = drop_id
                .and_then(|id| {
                    animation["Triggers"].as_array().and_then(|triggers| {
                        triggers
                            .iter()
                            .find(|trigger| trigger["EventId"] == id)
                            .and_then(|trigger| trigger["AtFrame"].as_u64())
                    })
                })
                .map_or("None".to_string(), |frame| format!("Some({frame})"));
            clips.push_str(&format!(
                "Clip {{ name: {:?}, frames: {}, looping: {}, drop_sound: {drop_sound} }},",
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

/// ItemConfig57BFA0 reads the authored card/rune achievement word. Card IDs
/// remain separate from the four entities2.xml visual families.
fn card_catalog(xml: &str) -> String {
    let mut achievements = [None; 41];
    let mut definitions: [Option<String>; 41] = std::array::from_fn(|_| None);
    for tag in xml
        .split("<card ")
        .skip(1)
        .chain(xml.split("<rune ").skip(1))
    {
        let head = &tag[..tag.find('>').expect("card tag end")];
        let id: usize = attribute(head, "id")
            .expect("card id")
            .parse()
            .expect("card id integer");
        assert!(
            id < achievements.len() && achievements[id].is_none(),
            "invalid authored card id"
        );
        achievements[id] = Some(attribute(head, "achievement").map_or(-1, |value| {
            value.parse::<i32>().expect("card achievement integer")
        }));
        definitions[id] = Some(format!(
            "PocketCardDefinition {{ name: {:?}, description: {:?} }},\n",
            attribute(head, "name").expect("card name"),
            attribute(head, "description").unwrap_or("")
        ));
    }
    assert!(
        achievements.iter().all(Option::is_some),
        "incomplete authored card catalogue"
    );
    let values: Vec<_> = achievements.into_iter().map(Option::unwrap).collect();
    let cards: String = definitions.into_iter().map(Option::unwrap).collect();
    //57BF80 indexes the authored final Pill effect, not a held color.
    let mut pills: [Option<String>; 28] = std::array::from_fn(|_| None);
    for tag in xml.split("<pilleffect ").skip(1) {
        let head = &tag[..tag.find('>').expect("Pill effect tag end")];
        let id: usize = attribute(head, "id")
            .expect("Pill effect id")
            .parse()
            .expect("Pill effect id integer");
        assert!(
            id < pills.len() && pills[id].is_none(),
            "invalid authored Pill effect id"
        );
        pills[id] = Some(format!(
            "PocketPillDefinition {{ name: {:?} }},\n",
            attribute(head, "name").expect("Pill effect name")
        ));
    }
    assert!(
        pills.iter().all(Option::is_some),
        "incomplete authored Pill effect catalogue"
    );
    let pills: String = pills.into_iter().map(Option::unwrap).collect();
    format!("pub const CARD_ACHIEVEMENTS: [i32; 41] = {values:?};\npub const CARD_DEFINITIONS: &[PocketCardDefinition] = &[\n{cards}];\npub const PILL_DEFINITIONS: &[PocketPillDefinition] = &[\n{pills}];\n")
}

/// tools/foundation/extract-alpha-masks.py output: one alpha>0.5 bitfield
/// per clip of Creep (Red), rows as hex with bit x for texel x.
fn creep_mask_catalog(path: &Path) -> String {
    let data: Value = serde_json::from_str(&fs::read_to_string(path).expect("read creep masks"))
        .expect("parse creep masks");
    let mut output = String::from("pub static CREEP_MASKS: &[CreepMask] = &[\n");
    for (name, mask) in data["masks"].as_object().expect("creep masks") {
        let rows: Vec<String> = mask["rows"]
            .as_array()
            .expect("mask rows")
            .iter()
            .map(|row| {
                let bits = u128::from_str_radix(row.as_str().expect("mask row"), 16)
                    .expect("hex mask row");
                format!("{bits}")
            })
            .collect();
        output.push_str(&format!(
            "CreepMask {{ clip: {name:?}, left: {:?}_f32, top: {:?}_f32, width: {}, rows: &[{}] }},\n",
            mask["left"].as_f64().expect("mask left"),
            mask["top"].as_f64().expect("mask top"),
            mask["width"].as_u64().expect("mask width"),
            rows.join(",")
        ));
    }
    output.push_str("];\n");
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
        (1000, 2, "gfx/1000.002_blood explosion.anm2"),
        (1000, 3, "gfx/1000.003_fly explosion.anm2"),
        (1000, 7, "gfx/1000.007_blood splat.anm2"),
        (1000, 12, "gfx/1000.012_tear poofa.anm2"),
        (1000, 13, "gfx/1000.013_tear poofb.anm2"),
        (1000, 79, "gfx/1000.012_tear poof_small.anm2"),
        (1000, 80, "gfx/1000.012_tear poof_verysmall.anm2"),
        (1000, 15, "gfx/1000.015_poof01.anm2"),
        (1000, 17, "gfx/1000.017_poof04.anm2"),
        (1000, 42, "gfx/1000.042_blood gush.anm2"),
        (1000, 49, "gfx/1000.049_heart.anm2"),
        (1000, 58, "gfx/1000.058_poop particle.anm2"),
        (1000, 5, "gfx/1000.005_blood particle.anm2"),
        (1000, 27, "gfx/1000.027_wood particle.anm2"),
        (1000, 35, "gfx/1000.035_tooth particle.anm2"),
        (1000, 43, "gfx/1000.043_poop explosion.anm2"),
        (1000, 21, "gfx/1000.021_tiny bug.anm2"),
        (1000, 33, "gfx/1000.033_tiny fly.anm2"),
        (1000, 68, "gfx/1000.021_tiny bug.anm2"),
        (1000, 22, "gfx/1000.022_creep (red).anm2"),
        (1000, 23, "gfx/1000.023_creep (green).anm2"),
        (1000, 24, "gfx/1000.024_creep (yellow).anm2"),
        (1000, 1, "gfx/1000.001_bomb explosion.anm2"),
        (1000, 30, "gfx/1000.030_dr. fetus target.anm2"),
        (1000, 31, "gfx/1000.031_dr. fetus rocket.anm2"),
        (1000, 18, "gfx/1000.018_bomb crater.anm2"),
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

fn attack_catalog(path: &Path) -> String {
    let actors: Value =
        serde_json::from_str(&fs::read_to_string(path).expect("read authored attack animations"))
            .expect("parse authored attack animations");
    let actor = "gfx/008.000_moms knife.anm2";
    let catalogue = &actors[actor];
    let default = catalogue["DefaultAnimation"]
        .as_str()
        .expect("knife default clip");
    let mut output = format!("pub static KNIFE_DEFINITION: Definition = Definition {{ entity_type: 8, variant: 0, actor: {actor:?}, default_clip: {default:?}, ");
    for field in [
        "base_hp",
        "collision_radius",
        "collision_mass",
        "collision_damage",
        "friction",
        "shadow_size",
    ] {
        output.push_str(&format!("{field}: crate::content::entity(8, 0).{field}, "));
    }
    output.push_str("grid_points: crate::content::entity(8, 0).grid_points, clips: &[");
    for clip in catalogue["Animations"]
        .as_array()
        .expect("knife authored clips")
    {
        output.push_str(&format!(
            "crate::familiars::Clip {{ name: {:?}, frames: {}, looping: {} }},",
            clip["Name"].as_str().expect("knife clip name"),
            clip["FrameNum"].as_u64().expect("knife clip length"),
            clip["Loop"].as_bool().expect("knife clip loop")
        ));
    }
    output.push_str("] };\n");
    output
}

fn information_catalog(xml: &str, path: &Path) -> String {
    let actors: Value =
        serde_json::from_str(&fs::read_to_string(path).expect("read authored stage animations"))
            .expect("parse authored stage animations");
    let root_tag = xml.split("<giantbook ").nth(1).expect("giantbook root");
    let root = attribute(root_tag, "anm2root")
        .expect("giantbook authored root")
        .trim_start_matches("resources/");
    let mut output = String::new();
    for (id, constant) in [("13", "ETERNAL"), ("16", "BLACK_HEART")] {
        let entry = xml
            .split("<entry ")
            .find(|entry| attribute(entry, "id") == Some(id))
            .expect("authored information entry");
        let actor = format!(
            "{root}{}",
            attribute(entry, "anm2").expect("information actor")
        )
        .to_lowercase();
        let spritesheet = format!(
            "{root}{}",
            attribute(entry, "gfx").expect("information graphics")
        )
        .to_lowercase();
        let name = attribute(entry, "anim").expect("information clip");
        let clip = actors[&actor]["Animations"]
            .as_array()
            .expect("authored information definition")
            .iter()
            .find(|clip| clip["Name"] == name)
            .expect("authored information clip");
        let frames = clip["FrameNum"].as_u64().expect("information frames");
        assert!(frames > 0 && frames <= u64::from(u32::MAX));
        assert_eq!(
            clip["Loop"], false,
            "admitted information clip is nonlooping"
        );
        assert!(
            clip["Triggers"]
                .as_array()
                .expect("information triggers")
                .is_empty(),
            "information event callbacks require their actual owner"
        );
        output.push_str(&format!(
            "pub static {constant}_DEFINITION: Definition = Definition {{ actor: {actor:?}, clip: {name:?}, frames: {frames}, spritesheet: {spritesheet:?} }};\n"
        ));
    }
    output
}

/// Every authored entities2.xml row, in document order. Lookups keep the
/// first (id, variant) row, matching the native config search.
fn entity_catalog(xml: &str) -> String {
    let mut output = String::from("pub const ENTITIES: &[EntityConfig] = &[\n");
    for tag in xml.split("<entity ").skip(1) {
        let head = &tag[..tag.find('>').expect("entity tag end")];
        let number = |field: &str| -> f64 {
            let value: f64 = attribute(head, field)
                .unwrap_or_else(|| panic!("entity {field} missing"))
                .parse()
                .unwrap_or_else(|_| panic!("invalid entity {field}"));
            assert!(value.is_finite(), "non-finite authored entity value");
            value
        };
        let integer = |field: &str| -> u32 {
            attribute(head, field).map_or(0, |value| value.parse().expect("entity integer"))
        };
        let points = number("numGridCollisionPoints") as u16;
        let grid_class = match attribute(head, "gridCollision") {
            Some("none") => 0,
            Some("walls") => 3,
            Some("ground") => 5,
            None if points > 0 => 5,
            None => 3,
            Some(value) => panic!("unsupported authored grid class {value}"),
        };
        let gibs = tag.split("<gibs ").nth(1).unwrap_or("");
        let mut gib_flags = 0;
        for (name, bit) in [
            ("blood", 1),
            ("bone", 2),
            ("gut", 4),
            ("eye", 8),
            ("large", 16),
            ("poop", 32),
        ] {
            if attribute(gibs, name) == Some("1") {
                gib_flags |= bit;
            }
        }
        let gib_amount: u32 =
            attribute(gibs, "amount").map_or(0, |value| value.parse().expect("gib amount"));
        let actor = format!(
            "gfx/{}",
            attribute(head, "anm2path").unwrap_or("").to_lowercase()
        );
        output.push_str(&format!(
            "EntityConfig {{ id: {}, variant: {}, subtype: {}, name: {:?}, actor: {actor:?}, portrait: {}, \
             base_hp: {:?}_f32, stage_hp: {:?}_f32, boss: {}, champion: {}, collision_damage: {:?}_f32, \
             collision_mass: {:?}_f32, collision_radius: {:?}_f32, friction: {:?}_f32, shadow_size: {:?}_f32, \
             grid_points: {points}, grid_class: {grid_class}, can_shut_doors: {}, gib_amount: {gib_amount}, gib_flags: {gib_flags} }},\n",
            integer("id"),
            integer("variant"),
            integer("subtype"),
            attribute(head, "name").unwrap_or(""),
            attribute(head, "portrait").map_or("None".into(), |value| format!("Some({value})")),
            number("baseHP"),
            attribute(head, "stageHP").map_or(0.0, |value| value.parse::<f64>().expect("stageHP")),
            number("boss") != 0.0,
            number("champion") != 0.0,
            number("collisionDamage"),
            number("collisionMass"),
            number("collisionRadius"),
            number("friction"),
            number("shadowSize"),
            attribute(head, "shutdoors").unwrap_or("true") == "true",
        ));
    }
    output.push_str("];\n");
    output
}

/// Every clip of every authored NPC actor: frame count, loop flag and the
/// (event name, frame) triggers its anm2 authors.
/// One authored layer-0 keyframe as `content::Keyframe`.
fn main_keyframe(frame: &Value) -> String {
    let number = |name: &str| {
        let value = frame[name].as_f64().expect("authored keyframe number") as f32;
        assert!(value.is_finite(), "non-finite authored keyframe {name}");
        value
    };
    let delay = frame["Delay"].as_u64().expect("authored keyframe delay");
    assert!(delay <= u64::from(u32::MAX));
    format!(
        "Keyframe {{ delay: {delay}, size: [{:?}, {:?}], pivot: [{:?}, {:?}], position: [{:?}, {:?}] }}",
        number("Width"),
        number("Height"),
        number("XPivot"),
        number("YPivot"),
        number("XPosition"),
        number("YPosition"),
    )
}

fn clip_catalog(actors: &Value) -> String {
    let mut output = String::from("pub const CLIPS: &[Clip] = &[\n");
    for (actor, definition) in actors.as_object().expect("NPC actor map") {
        let names: std::collections::BTreeMap<u64, &str> = definition["Events"]
            .as_array()
            .map(|events| {
                events
                    .iter()
                    .map(|event| {
                        (
                            event["Id"].as_u64().expect("event id"),
                            event["Name"].as_str().expect("event name"),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        for clip in definition["Animations"].as_array().expect("actor clips") {
            let events: Vec<String> = clip["Triggers"]
                .as_array()
                .map(|triggers| {
                    triggers
                        .iter()
                        .map(|trigger| {
                            let id = trigger["EventId"].as_u64().expect("trigger event");
                            format!(
                                "({:?}, {})",
                                names.get(&id).expect("trigger names an authored event"),
                                trigger["AtFrame"].as_u64().expect("trigger frame")
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            let main_layer: Vec<String> = clip["LayerAnimations"]
                .as_array()
                .expect("clip layers")
                .iter()
                .find(|layer| layer["LayerId"] == 0)
                .and_then(|layer| layer["Frames"].as_array())
                .map(|frames| frames.iter().map(main_keyframe).collect())
                .unwrap_or_default();
            output.push_str(&format!(
                "Clip {{ actor: {actor:?}, name: {:?}, frames: {}, looping: {}, events: &[{}], main_layer: &[{}] }},\n",
                clip["Name"].as_str().expect("clip name"),
                clip["FrameNum"].as_u64().expect("clip frames"),
                clip["Loop"].as_bool().expect("clip loop flag"),
                events.join(", "),
                main_layer.join(", "),
            ));
        }
    }
    output.push_str("];\n");
    output
}

/// bossportraits.xml: VS-screen presentation id, boss type/variant (encoded
/// in the authored portrait name `Portrait_<type>.<variant>_...`) and portrait.
fn boss_catalog(xml: &str) -> String {
    let mut output = String::from("pub const BOSS_PRESENTATIONS: &[BossPresentation] = &[\n");
    for tag in xml.split("<boss ").skip(1) {
        let id: u32 = attribute(tag, "id")
            .expect("boss id")
            .parse()
            .expect("boss id number");
        let portrait = attribute(tag, "portrait").expect("boss portrait");
        // The final Isaac boss reuses PlayerPortrait_01; its entity is not
        // encoded in the name, so it has no row until that boss is admitted.
        let Some(identity) = portrait
            .strip_prefix("Portrait_")
            .and_then(|rest| rest.split('_').next())
        else {
            continue;
        };
        let (kind, variant) = identity.split_once('.').expect("portrait type.variant");
        // 551a10 maps entity types to ids in code, not by file name:
        // 551ccd gives Gurglings 237 id 56, whose sheet is named 276.0.
        let (kind, variant) = if id == 56 {
            ("237", "1")
        } else {
            (kind, variant)
        };
        output.push_str(&format!(
            "BossPresentation {{ id: {id}, entity_type: {kind}, variant: {variant}, portrait: {portrait:?} }},\n",
            kind = kind.parse::<u32>().expect("portrait type"),
            variant = variant.parse::<u32>().expect("portrait variant"),
        ));
    }
    output.push_str("];\n");
    output
}

fn laser_root_catalog(actors: &Value) -> String {
    let actor = &actors["gfx/007.001_thick red laser.anm2"];
    let layers = actor["Layers"].as_array().expect("Laser authored layers");
    let clip = actor["Animations"]
        .as_array()
        .expect("Laser authored clips")
        .iter()
        .find(|clip| clip["Name"] == "LargeRedLaser")
        .expect("Laser default clip");
    let root = clip["LayerAnimations"]
        .as_array()
        .expect("Laser layer clips")
        .iter()
        .find(|layer| layer["LayerId"] == 0)
        .expect("Laser root layer");
    let mut out = format!(
        "const LASER_LAYER_COUNT: usize = {};\nconst LASER_ROOT_EXTENTS: &[(u32, f32)] = &[\n",
        layers.len()
    );
    for frame in root["Frames"].as_array().expect("Laser root frames") {
        let delay = frame["Delay"].as_u64().expect("Laser root frame delay");
        let x = frame["XPosition"].as_f64().expect("Laser root X") as f32;
        let width = frame["Width"].as_f64().expect("Laser root width") as f32;
        assert!(delay > 0 && delay <= u64::from(u32::MAX) && x.is_finite() && width.is_finite());
        out.push_str(&format!("({delay}, {:?}),\n", x + width));
    }
    out.push_str("];\n");
    let thin_layers = actors["gfx/007.002_thin red laser.anm2"]["Layers"]
        .as_array()
        .expect("Technology authored layers");
    assert_eq!(
        thin_layers.len(),
        1,
        "Technology actual47842C single-layer branch"
    );
    out.push_str(&format!(
        "const THIN_LASER_LAYER_COUNT: usize = {};\n",
        thin_layers.len()
    ));
    out
}

/// bossportraits.xml: VS-screen presentation id, boss type/variant (encoded
/// in the authored portrait name `Portrait_<type>.<variant>_...`) and portrait.
fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let resources = manifest
        .parent()
        .unwrap()
        .join("references/isaac/rebirth/resources");
    let giantbook = resources.join("giantbook.xml");
    let challenges = resources.join("challenges.xml");
    println!("cargo:rerun-if-changed={}", challenges.display());
    let information_animations = manifest.parent().unwrap().join("web/information-data.json");
    println!("cargo:rerun-if-changed={}", giantbook.display());
    println!(
        "cargo:rerun-if-changed={}",
        information_animations.display()
    );
    let entities = resources.join("entities2.xml");
    let image = resources.join("gfx/shadow.png");
    let animations = manifest.parent().unwrap().join("web/scene-data.json");
    let pickups = manifest.parent().unwrap().join("web/pickup-data.json");
    let npcs = manifest.parent().unwrap().join("web/npc-data.json");
    let stage_animations = manifest.parent().unwrap().join("web/stage-data.json");
    let nightmares = resources.join("nightmares.xml");
    println!("cargo:rerun-if-changed={}", stage_animations.display());
    println!("cargo:rerun-if-changed={}", nightmares.display());
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
    for name in [
        "Trapdoor", "Appear", "Death", "UseItem", "Hit", "Sad", "Happy",
    ] {
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
    let grid_animations = manifest.parent().unwrap().join("web/grid-data.json");
    println!("cargo:rerun-if-changed={}", grid_animations.display());
    let grid_assets: Value = serde_json::from_str(
        &fs::read_to_string(grid_animations).expect("read authored grid animations"),
    )
    .expect("authored grid animation JSON");
    for (constant, name) in [("CLOSED", "Closed"), ("OPEN_ANIMATION", "Open Animation")] {
        let clip = grid_assets["gfx/grid/door_11_trapdoor.anm2"]["Animations"]
            .as_array()
            .expect("authored trapdoor clips")
            .iter()
            .find(|clip| clip["Name"] == name)
            .expect("authored trapdoor clip");
        assert_eq!(clip["Loop"], false, "trapdoor clip is nonlooping");
        generated.push_str(&format!(
            "pub const TRAPDOOR_{constant}_FRAMES: u32 = {};\n",
            clip["FrameNum"].as_u64().expect("trapdoor clip frames")
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
    let shopkeeper_animations = npc_assets["gfx/017.001_shopkeeper.anm2"]["Animations"]
        .as_array()
        .expect("Shopkeeper animations are authored");
    let shopkeeper_clips: Vec<(u32, bool)> = (1..=9)
        .map(|subtype| {
            let name = format!("Shopkeeper {subtype}");
            let clip = shopkeeper_animations
                .iter()
                .find(|animation| animation["Name"] == name)
                .expect("every derived Shopkeeper subtype has an authored clip");
            (
                u32::try_from(clip["FrameNum"].as_u64().expect("Shopkeeper frame count"))
                    .expect("Shopkeeper frame count fits native actor clock"),
                clip["Loop"].as_bool().expect("Shopkeeper clip loop flag"),
            )
        })
        .collect();
    generated.push_str(&format!(
        "pub const SHOPKEEPER_ANIMATION_FRAMES: [u32; 9] = {:?};\n\
         pub const SHOPKEEPER_ANIMATION_LOOPS: [bool; 9] = {:?};\n",
        shopkeeper_clips
            .iter()
            .map(|clip| clip.0)
            .collect::<Vec<_>>(),
        shopkeeper_clips
            .iter()
            .map(|clip| clip.1)
            .collect::<Vec<_>>()
    ));
    let light = authored_light_frame(&animations);
    let npc_animations = manifest.parent().unwrap().join("web/npc-data.json");
    println!("cargo:rerun-if-changed={}", npc_animations.display());
    let npc_clips: Value = serde_json::from_str(
        &fs::read_to_string(npc_animations).expect("read authored NPC animations"),
    )
    .expect("parse authored NPC animations");
    // npc-data.json is generated from this allowlist; an actor added to the
    // JSON alone would be dropped by the next regeneration.
    let npc_list = manifest
        .parent()
        .unwrap()
        .join("tools/foundation/port-npc-animations.txt");
    println!("cargo:rerun-if-changed={}", npc_list.display());
    let listed = fs::read_to_string(npc_list).expect("read NPC animation allowlist");
    let listed = listed
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect::<std::collections::BTreeSet<_>>();
    for actor in npc_clips
        .as_object()
        .expect("NPC animations are a map")
        .keys()
    {
        assert!(
            listed.contains(actor.as_str()),
            "web/npc-data.json has {actor}, which tools/foundation/port-npc-animations.txt does not list"
        );
    }
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
    let level2 = &npc_clips["gfx/213.000_level2fly.anm2"]["Animations"][0];
    assert_eq!(level2["Name"], "Idle");
    assert_eq!(level2["Loop"], true);
    generated.push_str(&format!(
        "pub const LEVEL2_FLY_IDLE_FRAMES: u32 = {};\n",
        level2["FrameNum"].as_u64().unwrap()
    ));
    for clip in npc_clips["gfx/216.000_dip.anm2"]["Animations"]
        .as_array()
        .expect("Dip clips missing")
    {
        let name = clip["Name"].as_str().unwrap();
        assert!(matches!(name, "Idle" | "Move"));
        assert_eq!(clip["Loop"], true);
        assert_eq!(clip["Triggers"].as_array().unwrap().len(), 0);
        generated.push_str(&format!(
            "pub const DIP_{}_FRAMES: u32 = {};\n",
            name.to_uppercase(),
            clip["FrameNum"].as_u64().unwrap()
        ));
    }
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
            let guide = clip["NullAnimations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|track| track["NullId"] == 0)
                .expect("Monstro actual BloodGuide0 missing");
            assert_eq!(guide["Visible"], true);
            generated.push_str("pub static MONSTRO_DEATH_GUIDE: &[(f32,f32,f32,f32)] = &[\n");
            let mut guide_frames = 0_u64;
            for frame in guide["Frames"]
                .as_array()
                .expect("BloodGuide frames missing")
            {
                assert_eq!(
                    frame["Interpolated"], false,
                    "Current Death null frames are discrete"
                );
                let x = frame["XPosition"].as_f64().unwrap() as f32;
                let y = frame["YPosition"].as_f64().unwrap() as f32;
                // Packed raw scales were multiplied by100 only for ANM2 export.
                let scale_x = (frame["XScale"].as_f64().unwrap() / 100.0) as f32;
                let scale_y = (frame["YScale"].as_f64().unwrap() / 100.0) as f32;
                let duration = frame["Delay"].as_u64().unwrap();
                for _ in 0..duration {
                    generated.push_str(&format!("({x:?},{y:?},{scale_x:?},{scale_y:?}),\n"));
                }
                guide_frames += duration;
            }
            assert_eq!(guide_frames, clip["FrameNum"].as_u64().unwrap());
            generated.push_str("];\n");
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
    fs::copy(challenges, output.join("challenges.xml"))
        .expect("copy actual authored Challenge resource");
    fs::write(
        output.join("information_catalog.rs"),
        information_catalog(
            &fs::read_to_string(giantbook).expect("read authored information selection"),
            &information_animations,
        ),
    )
    .expect("write authored information definition");
    fs::write(
        output.join("nightmare_catalog.rs"),
        nightmare_catalog(
            &fs::read_to_string(nightmares).expect("read authored nightmare selection"),
            &stage_animations,
        ),
    )
    .expect("write authored nightmare definitions");

    fs::write(output.join("shadow_catalog.rs"), generated).expect("write shadow catalog");
    fs::write(output.join("entity_catalog.rs"), entity_catalog(&xml))
        .expect("write entity catalog");
    fs::write(
        output.join("laser_root_catalog.rs"),
        laser_root_catalog(&npc_clips),
    )
    .expect("write authored Laser root definitions");
    fs::write(output.join("clip_catalog.rs"), clip_catalog(&npc_clips))
        .expect("write clip catalog");
    let portraits = resources.join("bossportraits.xml");
    println!("cargo:rerun-if-changed={}", portraits.display());
    fs::write(
        output.join("boss_catalog.rs"),
        boss_catalog(&fs::read_to_string(portraits).expect("read authored boss portraits")),
    )
    .expect("write boss catalog");
    fs::write(
        output.join("pickup_catalog.rs"),
        pickup_catalog(&pickups, &animations, &xml),
    )
    .expect("write pickup catalog");
    let pockets = resources.join("pocketitems.xml");
    println!("cargo:rerun-if-changed={}", pockets.display());
    fs::write(output.join("card_catalog.rs"), {
        let art_path = manifest.parent().unwrap().join("web/pocket-data.json");
        println!("cargo:rerun-if-changed={}", art_path.display());
        let art: Value =
            serde_json::from_str(&fs::read_to_string(art_path).expect("read authored pocket art"))
                .expect("authored pocket art JSON");
        let sparkle = art["gfx/ui/ui_cardspills.anm2"]["Animations"]
            .as_array()
            .expect("pocket clips")
            .iter()
            .find(|clip| clip["Name"] == "Sparkle")
            .expect("pocket Sparkle");
        assert_eq!(
            sparkle["Loop"], false,
            "copied pocket Sparkle is nonlooping"
        );
        let frames = sparkle["FrameNum"].as_u64().expect("pocket Sparkle frames");
        assert!(frames > 0 && frames <= u64::from(u32::MAX));
        format!(
            "{}pub const POCKET_SPARKLE_FRAMES: u32 = {frames};\n",
            card_catalog(&fs::read_to_string(pockets).expect("read authored pocket configuration"))
        )
    })
    .expect("write authored card catalogue");
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
        &resources.join("costumes2.xml"),
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
    let creep_masks = manifest.parent().unwrap().join("web/creep-mask-data.json");
    println!("cargo:rerun-if-changed={}", creep_masks.display());
    fs::write(
        output.join("creep_masks.rs"),
        creep_mask_catalog(&creep_masks),
    )
    .expect("write creep masks");
    let familiar_path = manifest.parent().unwrap().join("web/familiar-data.json");
    println!("cargo:rerun-if-changed={}", familiar_path.display());
    fs::write(
        output.join("familiar_catalog.rs"),
        familiar_catalog(&familiar_path, &xml),
    )
    .expect("write authored Familiar/effect catalogue");
    let attack_path = manifest.parent().unwrap().join("web/attack-data.json");
    println!("cargo:rerun-if-changed={}", attack_path.display());
    fs::write(
        output.join("attack_catalog.rs"),
        attack_catalog(&attack_path),
    )
    .expect("write authored attack catalogue");
    fs::write(output.join("item_catalog.rs"), items.stdout)
        .expect("write authored collectible catalogue");
}
