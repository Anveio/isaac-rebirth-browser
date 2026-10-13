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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matches_observed_original_seed_readbacks() {
        for (text, value) in [
            ("T1PK6Y0W", 2184569263),
            ("FXSWJZBM", 588143529),
            ("3BMBMFTE", 3350817387),
            ("J4G39N4Q", 1231216718),
            ("M8GDNFAA", 1348684265),
            ("8M72AJ9X", 4246011870),
        ] {
            assert_eq!(decode(text).unwrap(), value);
            assert_eq!(encode(value).unwrap(), text);
        }
    }
    #[test]
    fn nonzero_unsigned_seed_boundaries_and_many_values_round_trip() {
        for value in [1, u32::MAX, 0x8000_0000] {
            assert_eq!(decode(&encode(value).unwrap()).unwrap(), value);
        }
        for value in crate::test_seeds::samples(
            "nonzero_unsigned_seed_boundaries_and_many_values_round_trip",
            10_000,
        ) {
            assert_eq!(decode(&encode(value).unwrap()).unwrap(), value);
        }
    }
    #[test]
    fn reserved_zero_has_no_valid_run_encoding() {
        assert!(encode(0).is_err());
        assert!(decode(&encode_bits(0)).is_err());
    }
    #[test]
    fn rejects_invalid_alphabet_length_and_checksum() {
        for value in ["", "ABCDEFG", "ABCDEFGHI", "IIIIIIII", "T1PK6Y0X", "💩💩"] {
            assert!(decode(value).is_err());
        }
        assert_eq!(decode("t1pk 6y0w").unwrap(), 2184569263);
    }
    #[test]
    fn startup_stage1_stream_matches_independent_original_logs() {
        for (seed, stage) in [
            (2184569263, 1427977769),
            (3350817387, 3049754554),
            (1231216718, 2478271409),
            (1348684265, 1558365435),
            (4246011870, 3308301364),
        ] {
            assert_eq!(stage1_seed(seed), stage);
        }
    }
}
