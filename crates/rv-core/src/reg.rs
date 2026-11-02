//! The 32 integer registers of RV32I.
//!
//! [`Reg`] says *which* register an instruction names, as a number from 0 to 31. It does not hold
//! the register's value: values live in the register file, which is indexed with [`Reg::index`].

use crate::utils;
use std::fmt::Display;

/// One of the 32 integer registers, x0 to x31.
///
/// The field is private, so the only ways to get a `Reg` are [`Reg::new`] and [`Reg::from_bits`],
/// and both guarantee that the number is in `0..32`. That is what makes [`Reg::abi_name`] and
/// [`Reg::index`] safe to use without further checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Reg(u8);

/// ABI names, indexed by register number: `ABI_NAMES[2]` is `"sp"`.
const ABI_NAMES: [&str; 32] = [
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1", "a2", "a3", "a4",
    "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11", "t3", "t4",
    "t5", "t6",
];

impl Reg {
    /// Returns the register numbered `n`, or `None` if `n` is not in `0..32`.
    #[must_use]
    pub fn new(n: u8) -> Option<Self> {
        if n < 32 { Some(Self(n)) } else { None }
    }

    /// Builds a register from the low five bits of `n`; any higher bits are ignored.
    ///
    /// An instruction's register fields are five bits wide, so this is the constructor the decoder
    /// uses.
    #[must_use]
    pub fn from_bits(n: u32) -> Self {
        Self(utils::bits(n, 4, 0) as u8)
    }

    /// Returns the ABI name of this register, such as `"sp"` for x2 or `"a0"` for x10.
    #[must_use]
    pub fn abi_name(self) -> &'static str {
        ABI_NAMES[self.index()]
    }

    /// Returns the register number, 0 to 31, for indexing the register file.
    #[must_use]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

impl Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(self.abi_name())
    }
}

#[cfg(test)]
mod test {
    use super::Reg;

    /// The expected ABI names, written out separately from the table in the implementation, so a
    /// typo in either one is caught.
    const EXPECTED: [&str; 32] = [
        "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1", "a2", "a3", "a4",
        "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11", "t3", "t4",
        "t5", "t6",
    ];

    // The first and last register of every ABI group, plus the special ones.
    #[test_case::test_case(0 => "zero")]
    #[test_case::test_case(1 => "ra")]
    #[test_case::test_case(2 => "sp")]
    #[test_case::test_case(3 => "gp")]
    #[test_case::test_case(4 => "tp")]
    #[test_case::test_case(5 => "t0")]
    #[test_case::test_case(7 => "t2")]
    #[test_case::test_case(8 => "s0")]
    #[test_case::test_case(9 => "s1")]
    #[test_case::test_case(10 => "a0")]
    #[test_case::test_case(17 => "a7")]
    #[test_case::test_case(18 => "s2")]
    #[test_case::test_case(27 => "s11")]
    #[test_case::test_case(28 => "t3")]
    #[test_case::test_case(31 => "t6")]
    fn abi_name_of_group_boundaries(n: u8) -> &'static str {
        Reg::new(n).unwrap().abi_name()
    }

    #[test]
    fn every_register_has_its_abi_name() {
        for n in 0..32u8 {
            let expected = EXPECTED[usize::from(n)];
            let reg = Reg::new(n).unwrap();
            assert_eq!(reg.abi_name(), expected, "name of x{n}");
            assert_eq!(reg.to_string(), expected, "Display of x{n}");
            assert_eq!(reg.index(), usize::from(n), "index of x{n}");
        }
    }

    #[test]
    fn new_rejects_numbers_outside_0_to_31() {
        assert!(Reg::new(31).is_some());
        assert_eq!(Reg::new(32), None);
        assert_eq!(Reg::new(255), None);
    }

    #[test]
    fn from_bits_keeps_only_the_low_five_bits() {
        // Bit 5 is ignored.
        assert_eq!(Reg::from_bits(0b10_0101), Reg::new(5).unwrap());
        // High bits all set, low five clear: x0.
        assert_eq!(Reg::from_bits(0xffff_ffe0), Reg::new(0).unwrap());
        // Every bit set: x31.
        assert_eq!(Reg::from_bits(u32::MAX), Reg::new(31).unwrap());
    }

    #[test]
    fn from_bits_never_produces_an_invalid_register() {
        // Inputs well beyond 0..32. A field-width mistake (six bits instead of five) fails here.
        for n in 0..=1024u32 {
            let reg = Reg::from_bits(n);
            assert!(
                reg.index() < 32,
                "from_bits({n}) gave register number {}",
                reg.index()
            );
            // Looking the name up must not panic either.
            let _ = reg.abi_name();
        }
    }

    #[test]
    fn from_bits_agrees_with_new_for_valid_numbers() {
        for n in 0..32u8 {
            assert_eq!(Some(Reg::from_bits(u32::from(n))), Reg::new(n));
        }
    }

    #[test]
    fn display_honours_width_and_alignment() {
        assert_eq!(format!("[{:>4}]", Reg::new(2).unwrap()), "[  sp]");
        assert_eq!(format!("[{:<4}]", Reg::new(10).unwrap()), "[a0  ]");
    }
}
