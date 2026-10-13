//! Platform-independent f32 transcendental functions.
//!
//! Browsers run wasm32 std, whose sinf/cosf are the Rust `libm`
//! implementation. Native std calls the host C library instead, which
//! disagrees in the last bit on about 2% of inputs. Engine code calls these so
//! native builds (DST, replay and floor tools) compute the shipped artifact's
//! exact results; clippy.toml rejects the std methods.
pub fn sin(x: f32) -> f32 {
    libm::sinf(x)
}

pub fn cos(x: f32) -> f32 {
    libm::cosf(x)
}
