#![no_std]
#![no_main]

use guests::println;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    println!("Hello from RV32I!");
    0
}
