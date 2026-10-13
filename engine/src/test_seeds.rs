//! Seed-independent tests use fresh entropy, with a single replayable process seed.
use std::collections::BTreeSet;
use std::io::Read;
use std::sync::OnceLock;

const REPLAY_ENV: &str = "FOUNDATION_TEST_SEED";
static PROCESS_SEED: OnceLock<u32> = OnceLock::new();

fn process_seed() -> u32 {
    *PROCESS_SEED.get_or_init(|| {
        let seed = match std::env::var(REPLAY_ENV) {
            Ok(value) => value
                .parse::<u32>()
                .ok()
                .filter(|value| *value != 0)
                .expect("FOUNDATION_TEST_SEED must be a nonzero unsigned 32-bit integer"),
            Err(std::env::VarError::NotPresent) => {
                let mut entropy = std::fs::File::open("/dev/urandom")
                    .expect("open /dev/urandom for property-test seeds");
                loop {
                    let mut bytes = [0; 4];
                    entropy
                        .read_exact(&mut bytes)
                        .expect("read /dev/urandom for property-test seeds");
                    let value = u32::from_ne_bytes(bytes);
                    if value != 0 {
                        break value;
                    }
                }
            }
            Err(error) => panic!("cannot read {REPLAY_ENV}: {error}"),
        };
        eprintln!("Rust property-test replay: {REPLAY_ENV}={seed}");
        seed
    })
}

/// Derive an independent stream for each named test from the recorded entropy.
/// This test-only SplitMix64 sampler is unrelated to production seed routing.
pub fn samples(test: &str, count: usize) -> Vec<u32> {
    let mut state = u64::from(process_seed());
    for byte in test.bytes() {
        state = (state ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3);
    }
    let mut result = Vec::with_capacity(count);
    let mut chosen = BTreeSet::new();
    while result.len() != count {
        state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        let value = (value ^ (value >> 31)) as u32;
        if value != 0 && chosen.insert(value) {
            result.push(value);
        }
    }
    result
}

pub fn one(test: &str) -> u32 {
    samples(test, 1)[0]
}
