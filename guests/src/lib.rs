//! Minimal runtime for bare-metal RV32I guest programs.
//!
//! It provides an entry point (`_start`), console output ([`println!`]) and a way to stop the
//! machine ([`exit`]). The device addresses mirror `rv_core::memmap` in the host workspace. They
//! are repeated here because this crate is built for a different target and shares no code with
//! it.

#![no_std]

use core::arch::{asm, global_asm};
use core::fmt::{self, Write};
use core::panic::PanicInfo;
use core::ptr::write_volatile;

/// Console transmit register (`rv_core::memmap::UART_BASE`).
const UART: *mut u8 = 0x1000_0000 as *mut u8;
/// Test-finisher register (`rv_core::memmap::FINISHER_BASE`).
const FINISHER: *mut u32 = 0x0010_0000 as *mut u32;
/// Value that stops the machine with exit code 0.
const FINISHER_PASS: u32 = 0x5555;
/// Low half of the value that stops the machine with a non-zero exit code.
const FINISHER_FAIL: u32 = 0x3333;

// Entry point: set up `gp` and `sp`, run the program's `main`, then exit with its return value.
// `main` leaves its result in a0, which is exactly where `exit` expects its argument.
global_asm!(
    r#"
    .section .text.init, "ax"
    .globl _start
_start:
    .option push
    .option norelax
    la   gp, __global_pointer$
    .option pop
    la   sp, __stack_top
    call main
    call {exit}
"#,
    exit = sym exit,
);

struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            // SAFETY: UART is a memory-mapped device register of the machine this runs on.
            unsafe { write_volatile(UART, byte) };
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    // Writing to the console cannot fail.
    let _ = Console.write_fmt(args);
}

/// Prints to the console.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => { $crate::_print(format_args!($($arg)*)) };
}

/// Prints to the console, with a newline.
#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => { $crate::print!("{}\n", format_args!($($arg)*)) };
}

/// Stops the machine with the given exit code.
pub extern "C" fn exit(code: i32) -> ! {
    let word = if code == 0 {
        FINISHER_PASS
    } else {
        ((code as u32) << 16) | FINISHER_FAIL
    };
    // SAFETY: FINISHER is a memory-mapped device register of the machine this runs on.
    unsafe { write_volatile(FINISHER, word) };
    loop {
        // The finisher stops the machine. If it somehow does not, wait here.
        // SAFETY: `wfi` has no memory effects.
        unsafe { asm!("wfi", options(nomem, nostack)) };
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("panic: {info}");
    exit(101)
}
