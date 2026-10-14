//! Every simulation decision draws from one entropy source.
//!
//! Under Antithesis each draw calls the SDK so the platform can steer and
//! replay exploration deterministically. Locally a seeded SplitMix64 stream
//! makes each case reproducible from its printed seed. Units never read the
//! clock, OS randomness or the SDK directly.
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Seeded,
    Antithesis,
}

impl fmt::Display for Mode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Seeded => "seeded",
            Self::Antithesis => "antithesis",
        })
    }
}

pub struct Entropy {
    mode: Mode,
    state: u64,
    draws: u64,
}

pub fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut value = *state;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

impl Entropy {
    pub fn seeded(seed: u64) -> Self {
        Self {
            mode: Mode::Seeded,
            state: seed,
            draws: 0,
        }
    }

    pub fn antithesis() -> Self {
        Self {
            mode: Mode::Antithesis,
            state: 0,
            draws: 0,
        }
    }

    pub fn draws(&self) -> u64 {
        self.draws
    }

    pub fn u64(&mut self) -> u64 {
        self.draws += 1;
        match self.mode {
            Mode::Seeded => splitmix64(&mut self.state),
            Mode::Antithesis => antithesis_sdk::random::get_random(),
        }
    }

    pub fn u32(&mut self) -> u32 {
        (self.u64() >> 32) as u32
    }

    /// Unbiased draw in `0..bound` (Lemire's multiply/reject method).
    pub fn below(&mut self, bound: u64) -> u64 {
        assert!(bound > 0, "entropy bound must be positive");
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let product = u128::from(self.u64()) * u128::from(bound);
            if (product as u64) >= threshold {
                return (product >> 64) as u64;
            }
        }
    }

    pub fn index(&mut self, length: usize) -> usize {
        self.below(length as u64) as usize
    }

    /// Inclusive range.
    pub fn range(&mut self, low: u32, high: u32) -> u32 {
        assert!(low <= high, "entropy range is empty");
        low + self.below(u64::from(high - low) + 1) as u32
    }

    pub fn chance(&mut self, numerator: u64, denominator: u64) -> bool {
        self.below(denominator) < numerator
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.index(items.len())]
    }

    /// A nonzero seed biased toward boundary values that ordinary uniform
    /// draws almost never produce.
    pub fn nonzero_u32(&mut self) -> u32 {
        match self.below(16) {
            0 => *self.pick(&[1, 2, 3, u32::MAX, u32::MAX - 1, 0x8000_0000, 0x7fff_ffff]),
            1 => 1 << self.below(32),
            _ => self.u32().max(1),
        }
    }
}
