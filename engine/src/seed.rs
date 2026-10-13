//! Public Rebirth seed encoding, independently transcribed from the documented
//! alphabet/checksum/XOR format. No seed-specific acceptance or generation path.
//! Source: SolarPolarMan, https://gist.github.com/ChildishGiant/bf39d195dbdc9093417bc8f5b85b57e8
const ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTWXYZ01234V6789";
const MASK: u32 = 0x0fef_7ffd;
/// Public xorshift startup stream, shift-index 27. Stage 1 is its second draw;
/// this limited routing has been checked against fresh original-game logs.
pub fn startup_next(mut value: u32) -> u32 {
    value ^= value >> 3;
    value ^= value << 23;
    value ^= value >> 25;
    value
}
pub fn stage1_seed(value: u32) -> u32 {
    startup_next(startup_next(value))
}
fn checksum(mut seed: u32) -> u8 {
    let mut checksum: u32 = 0;
    while seed != 0 {
        checksum = (checksum + (seed & 255)) & 255;
        checksum = ((checksum << 1) + (checksum >> 7)) & 255;
        seed >>= 5;
    }
    checksum as u8
}
fn encode_bits(seed: u32) -> String {
    let packed = (u64::from(seed ^ MASK) << 8) | u64::from(checksum(seed));
    (0..8)
        .map(|index| ALPHABET[((packed >> ((7 - index) * 5)) & 31) as usize] as char)
        .collect()
}
pub fn encode(seed: u32) -> Result<String, String> {
    if seed == 0 {
        return Err("seed zero is reserved".into());
    }
    Ok(encode_bits(seed))
}
pub fn decode(text: &str) -> Result<u32, String> {
    let compact: String = text.chars().filter(|character| *character != ' ').collect();
    if compact.len() != 8 || !compact.is_ascii() {
        return Err("seed requires eight code characters".into());
    }
    let mut packed = 0u64;
    for glyph in compact.to_ascii_uppercase().bytes() {
        let value = ALPHABET
            .iter()
            .position(|character| *character == glyph)
            .ok_or("invalid seed alphabet character")?;
        packed = (packed << 5) | value as u64;
    }
    let seed = (packed >> 8) as u32 ^ MASK;
    if seed == 0 {
        return Err("seed zero is reserved".into());
    }
    if packed as u8 != checksum(seed) {
        return Err("seed checksum does not match".into());
    }
    Ok(seed)
}
