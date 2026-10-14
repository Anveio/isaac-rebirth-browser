//! Platform-independent transcendental math.
//!
//! Browsers run wasm32 std, whose float functions are the Rust `libm`
//! implementation. Native std calls the host C library instead, which
//! disagrees in the last bit on some inputs, so native replay and tooling
//! could diverge from the shipped artifact. Engine code calls these so every
//! build computes the shipped artifact's exact results; `engine/clippy.toml`
//! disallows the std functions they replace (see docs/dst.md).
pub fn sin(x: f32) -> f32 {
    libm::sinf(x)
}

pub fn cos(x: f32) -> f32 {
    libm::cosf(x)
}

pub fn atan2(y: f64, x: f64) -> f64 {
    libm::atan2(y, x)
}
