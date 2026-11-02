//! Bit-field helpers for taking instruction words apart.

/// Extracts bits `hi` down to `lo` of `word`, both inclusive, shifted so that bit `lo` becomes bit 0.
///
/// For example, `bits(0x00c1_2083, 6, 0)` is the 7-bit opcode `0b000_0011`.
///
/// Requires `lo <= hi <= 31`. Other arguments overflow, which panics in debug builds.
#[must_use]
pub const fn bits(word: u32, hi: u32, lo: u32) -> u32 {
    let width = hi - lo + 1;
    let mask = if width == 32 { !0 } else { (1 << width) - 1 };
    (word >> lo) & mask
}

/// Reads the low `width` bits of `value` as a two's-complement number and returns it as an `i32`.
///
/// Bits of `value` above `width` are ignored. For example, `sign_extends(0xfff, 12)` is `-1` and
/// `sign_extends(0x7ff, 12)` is `2047`.
///
/// Requires `1 <= width <= 32`.
#[must_use]
pub const fn sign_extends(value: u32, width: u32) -> i32 {
    let shift = 32 - width;
    ((value as i32) << shift) >> shift
}

#[cfg(test)]
mod test {
    use crate::utils::{bits, sign_extends};

    #[test_case::test_case(0x00c1_2083, 6, 0, 0b000_0011)]
    #[test_case::test_case(0x00c1_2083, 31, 20, 12)]
    fn test_bits(word: u32, hi: u32, lo: u32, expected: u32) {
        assert_eq!(bits(word, hi, lo), expected);
    }

    #[test_case::test_case(0x7ff, 12, 2047)]
    #[test_case::test_case(0x800, 12, -2048)]
    #[test_case::test_case(0xfff, 12, -1)]
    fn test_sign_extends(value: u32, width: u32, expected: i32) {
        assert_eq!(sign_extends(value, width), expected);
    }
}
