//! The machine's physical memory map.
//!
//! This is a subset of QEMU's `virt` board on purpose: a guest ELF linked for this map should run
//! unchanged under `qemu-system-riscv32 -M virt`, which gives us a reference to cross-check
//! against. `guests/link.ld` and the guest runtime in `guests/src/lib.rs` must agree with these
//! values.

/// First address of RAM. Guests are linked here and execution starts at the ELF entry point.
pub const RAM_BASE: u32 = 0x8000_0000;

/// Size of RAM in bytes (16 MiB).
pub const RAM_SIZE: u32 = 16 * 1024 * 1024;

/// Transmit register of a 16550-style UART: a byte stored here is console output.
pub const UART_BASE: u32 = 0x1000_0000;

/// "Test finisher" register (`sifive_test` on QEMU) that lets a guest stop the machine.
///
/// Storing [`FINISHER_PASS`] exits with code 0. Storing `(code << 16) | FINISHER_FAIL` exits with
/// `code`.
pub const FINISHER_BASE: u32 = 0x0010_0000;

/// Value stored to [`FINISHER_BASE`] to stop the machine successfully.
pub const FINISHER_PASS: u32 = 0x5555;

/// Low half of the value stored to [`FINISHER_BASE`] to stop the machine with a failure code.
pub const FINISHER_FAIL: u32 = 0x3333;

const _: () = assert!(
    RAM_BASE.checked_add(RAM_SIZE).is_some(),
    "RAM must not wrap around the 32-bit address space"
);
