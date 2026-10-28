//! Platform-independent transcendental math.
//!
//! Browsers run wasm32 std, whose float functions are the Rust `libm`
//! implementation. Native std calls the host C library instead, which
//! disagrees in the last bit on some inputs, so native replay and tooling
//! could diverge from the shipped artifact. Engine code calls these so every
//! build computes the shipped artifact's exact results; `engine/clippy.toml`
//! disallows the std functions they replace (see docs/dst.md).
//!
//! The original links the host glibc (2.39 here, x86_64), whose sinf, cosf
//! and sincosf share one algorithm (sysdeps/ieee754/flt-32/s_sincosf.h,
//! sincosf_poly.h). The Rust `libm` sinf/cosf (a musl/FreeBSD port) differ
//! from it in the last bit on about 1.3% of inputs, so these two follow
//! glibc's double-precision evaluation instead. Every step is a plain IEEE
//! double operation, identical in wasm32 and native builds. Checked bit for
//! bit against the host glibc on 400,000 inputs, including large ones; the
//! FMA multiarch build gives the same floats.

/// sin(y), as glibc 2.39 `sinf`.
pub fn sin(y: f32) -> f32 {
    glibc_sincosf(y, false)
}

/// cos(y), as glibc 2.39 `cosf`.
pub fn cos(y: f32) -> f32 {
    glibc_sincosf(y, true)
}

/// The sincos_t table: sign of sine per quadrant, 2/PI * 2^24, PI/2, the
/// cosine polynomial (negated in the second entry) and the sine polynomial.
struct SinCos {
    c0: f64,
    c1: f64,
    c2: f64,
    c3: f64,
    c4: f64,
}
const SIGN: [f64; 4] = [1.0, -1.0, -1.0, 1.0];
const HPI_INV: f64 = f64::from_bits(0x4164_5f30_6dc9_c883);
const HPI: f64 = f64::from_bits(0x3ff9_21fb_5444_2d18);
const PI63: f64 = f64::from_bits(0x3c19_21fb_5444_2d18);
const C1: f64 = f64::from_bits(0xbfdf_ffff_fd0c_621c);
const C2: f64 = f64::from_bits(0x3fa5_5553_e106_8f19);
const C3: f64 = f64::from_bits(0xbf56_c087_e89a_359d);
const C4: f64 = f64::from_bits(0x3ef9_9343_027b_f8c3);
const S1: f64 = f64::from_bits(0xbfc5_5554_5995_a603);
const S2: f64 = f64::from_bits(0x3f81_1076_0523_0bc4);
const S3: f64 = f64::from_bits(0xbf29_94eb_3774_cf24);
const TABLE: [SinCos; 2] = [
    SinCos {
        c0: 1.0,
        c1: C1,
        c2: C2,
        c3: C3,
        c4: C4,
    },
    SinCos {
        c0: -1.0,
        c1: -C1,
        c2: -C2,
        c3: -C3,
        c4: -C4,
    },
];
/// 4/PI to 192 bits, 8 new bits per entry.
const INV_PIO4: [u32; 24] = [
    0xa2, 0xa2f9, 0xa2f983, 0xa2f9836e, 0xf9836e4e, 0x836e4e44, 0x6e4e4415, 0x4e441529, 0x441529fc,
    0x1529fc27, 0x29fc2757, 0xfc2757d1, 0x2757d1f5, 0x57d1f534, 0xd1f534dd, 0xf534ddc0, 0x34ddc0db,
    0xddc0db62, 0xc0db6295, 0xdb629599, 0x6295993c, 0x95993c43, 0x993c4390, 0x3c439041,
];

fn abstop12(x: f32) -> u32 {
    (x.to_bits() >> 20) & 0x7ff
}

/// sinf_poly: the sine polynomial for even N, the cosine one for odd N.
fn sinf_poly(x: f64, x2: f64, p: &SinCos, n: i32) -> f32 {
    if n & 1 == 0 {
        let x3 = x * x2;
        let s1 = S2 + x2 * S3;
        let x7 = x3 * x2;
        let s = x + x3 * S1;
        (s + x7 * s1) as f32
    } else {
        let x4 = x2 * x2;
        let c2 = p.c3 + x2 * p.c4;
        let c1 = p.c0 + x2 * p.c1;
        let x6 = x4 * x2;
        let c = c1 + x4 * p.c2;
        (c + x6 * c2) as f32
    }
}

/// reduce_fast without TOINT_INTRINSICS (x86_64): the quadrant lands in
/// bits 24..31 of the prescaled product.
fn reduce_fast(x: f64) -> (f64, i32) {
    let r = x * HPI_INV;
    let n = ((r as i32).wrapping_add(0x80_0000)) >> 24;
    (x - f64::from(n) * HPI, n)
}

/// reduce_large: a 32x96-bit multiply by 4/PI for |y| >= 120.
fn reduce_large(xi: u32) -> (f64, i32) {
    let arr = &INV_PIO4[((xi >> 26) & 15) as usize..];
    let shift = (xi >> 23) & 7;
    let xi = ((xi & 0xff_ffff) | 0x80_0000) << shift;
    let mut res0 = u64::from(xi.wrapping_mul(arr[0]));
    let res1 = u64::from(xi) * u64::from(arr[4]);
    let res2 = u64::from(xi) * u64::from(arr[8]);
    res0 = (res2 >> 32) | (res0 << 32);
    res0 = res0.wrapping_add(res1);
    let n = res0.wrapping_add(1 << 61) >> 62;
    res0 = res0.wrapping_sub(n << 62);
    ((res0 as i64) as f64 * PI63, n as i32)
}

fn glibc_sincosf(y: f32, cosine: bool) -> f32 {
    let x = f64::from(y);
    let k = i32::from(cosine);
    if abstop12(y) < abstop12(core::f32::consts::FRAC_PI_4) {
        if abstop12(y) < abstop12(f32::from_bits(0x3980_0000)) {
            return if cosine { 1.0 } else { y };
        }
        sinf_poly(x, x * x, &TABLE[0], k)
    } else if abstop12(y) < abstop12(120.0) {
        let (x, n) = reduce_fast(x);
        let s = SIGN[(n & 3) as usize];
        let p = &TABLE[usize::from(n & 2 != 0)];
        sinf_poly(x * s, x * x, p, n ^ k)
    } else if abstop12(y) < abstop12(f32::INFINITY) {
        let xi = y.to_bits();
        let sign = (xi >> 31) as i32;
        let (x, n) = reduce_large(xi);
        let s = SIGN[((n + sign) & 3) as usize];
        let p = &TABLE[usize::from((n + sign) & 2 != 0)];
        sinf_poly(x * s, x * x, p, n ^ k)
    } else {
        // __math_invalidf: NaN for infinities and NaN.
        #[allow(clippy::eq_op)]
        let d = y - y;
        #[allow(clippy::eq_op)]
        let nan = d / d;
        nan
    }
}

pub fn pow(x: f64, y: f64) -> f64 {
    libm::pow(x, y)
}

pub fn atan2(y: f64, x: f64) -> f64 {
    libm::atan2(y, x)
}

/// Native atan2f consumers retain their actual single-precision call boundary.
pub fn atan2_f32(y: f32, x: f32) -> f32 {
    libm::atan2f(y, x)
}
