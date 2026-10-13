//! Public seed codes and the run-seed xorshift handoff (`engine/src/seed.rs`).
use crate::harness::Case;
use crate::props;
use rebirth_cleanroom::seed;
use serde_json::json;

/// The documented public alphabet (no I, O, U or 5; V stands in for 5),
/// restated independently of the engine's private constant.
const ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTWXYZ01234V6789";

pub fn run(case: &mut Case) {
    let e = &mut *case.entropy;
    props::always!(
        seed::encode(0).is_err(),
        "seed_codec: reserved seed zero never encodes"
    );
    for _ in 0..(case.steps / 4).max(64) {
        let value = e.nonzero_u32();
        let Ok(code) = seed::encode(value) else {
            props::always!(
                false,
                "seed_codec: every nonzero seed encodes",
                json!({ "seed": value })
            );
            continue;
        };
        props::always!(true, "seed_codec: every nonzero seed encodes");
        props::always!(
            code.len() == 8 && code.bytes().all(|glyph| ALPHABET.contains(&glyph)),
            "seed_codec: codes are eight public alphabet glyphs",
            json!({ "seed": value, "code": code })
        );
        props::always!(
            seed::decode(&code) == Ok(value),
            "seed_codec: decode inverts encode",
            json!({ "seed": value, "code": code, "decoded": seed::decode(&code) })
        );
        // The game displays codes as two space-separated quartets.
        let display = format!("{} {}", &code[..4], &code[4..]).to_ascii_lowercase();
        props::always!(
            seed::decode(&display) == Ok(value),
            "seed_codec: decode ignores case and spaces",
            json!({ "seed": value, "display": display })
        );

        // Arbitrary glyph strings are accepted exactly when canonical.
        let random: String = (0..8).map(|_| *e.pick(ALPHABET) as char).collect();
        match seed::decode(&random) {
            Ok(decoded) => {
                props::sometimes!(true, "seed_codec: a random code passes its checksum");
                props::always!(
                    seed::encode(decoded).as_deref() == Ok(random.as_str()),
                    "seed_codec: every accepted code is the canonical encoding of its seed",
                    json!({ "code": random, "decoded": decoded })
                );
            }
            Err(_) => props::sometimes!(true, "seed_codec: a random code is rejected"),
        }

        // A single substituted glyph is either rejected or canonical.
        let mut mutated = code.clone().into_bytes();
        let position = e.index(8);
        mutated[position] = *e.pick(ALPHABET);
        let mutated = String::from_utf8(mutated).expect("ASCII alphabet");
        if mutated != code {
            if let Ok(decoded) = seed::decode(&mutated) {
                props::always!(
                    decoded != value && seed::encode(decoded).as_deref() == Ok(mutated.as_str()),
                    "seed_codec: a substituted glyph never aliases the original seed",
                    json!({ "code": code, "mutated": mutated, "decoded": decoded })
                );
            }
        }

        let excluded = *e.pick(b"IOU5!-");
        let mut foreign = code.clone().into_bytes();
        foreign[e.index(8)] = excluded;
        let foreign = String::from_utf8(foreign).expect("ASCII glyph");
        props::always!(
            seed::decode(&foreign).is_err(),
            "seed_codec: codes with glyphs outside the alphabet are rejected",
            json!({ "code": foreign })
        );
        let length = e.index(16);
        let truncated: String = code.chars().cycle().take(length).collect();
        props::always!(
            length == 8 || seed::decode(&truncated).is_err(),
            "seed_codec: codes without exactly eight glyphs are rejected",
            json!({ "code": truncated })
        );

        // Xorshift steps are bijections fixing zero, so a nonzero run seed
        // can never reach generate_basement's reserved stage seed.
        let stage = seed::stage1_seed(value);
        props::always!(
            stage != 0 && seed::startup_next(value) != 0,
            "seed_codec: run-seed derivation never yields reserved zero",
            json!({ "seed": value, "stage": stage })
        );
        match seed::RunSeeds::new(value) {
            Ok(mut run_seeds) => {
                let fields_nonzero = run_seeds.initialized_fields.iter().all(|field| *field != 0);
                let pool = run_seeds.item_pool_seed();
                props::always!(
                    fields_nonzero && pool != 0 && run_seeds.start_seed == value,
                    "seed_codec: run seed fields and item-pool seed are nonzero",
                    json!({ "seed": value, "fields": run_seeds.initialized_fields, "pool": pool })
                );
            }
            Err(error) => props::always!(
                false,
                "seed_codec: run seed fields and item-pool seed are nonzero",
                json!({ "seed": value, "error": error })
            ),
        };
    }
}
