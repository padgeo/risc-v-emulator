# `guests/`: programs that run inside the emulator

A plain-language tour of this folder. It assumes no background in embedded or "bare-metal" development.

## The big picture

The emulator (`crates/rv-core` and `crates/rv-tui`) imitates a small computer built around a RISC-V CPU. To test and demonstrate it we need programs for that computer. Programs for your PC will not do: they are compiled for a different CPU (x86-64) and expect an operating system (Linux). The emulated machine speaks RISC-V machine code and has no operating system at all.

So this folder is a small, separate project that compiles Rust programs *for the emulated machine*. Two terms come up constantly:

- **host**: your real computer, and the emulator running on it.
- **guest**: a program running *inside* the emulated computer. Hence the folder name.

Nothing in here runs on your PC. The output is an **ELF file** (the standard executable file format on Linux) that the emulator will load into its emulated memory and run. The emulator cannot load files yet; that arrives in roadmap session 2. So far we have only checked that `hello` builds into a well-formed RV32 ELF.

```
hello.rs + lib.rs  ->  compiler  ->  machine code  ->  linker (steered by link.ld)  ->  hello (ELF file)  ->  emulator loads it
```

## What is in the folder

| Path | What it is |
|---|---|
| `src/bin/hello.rs` | The first guest program. It prints a line and exits. |
| `src/lib.rs` | A tiny "runtime": the code *every* guest needs in order to start up, print, exit and handle panics. |
| `link.ld` | The linker script: a floor plan saying which part of the program lives at which memory address. |
| `build.rs` | A build script whose only job is to hand `link.ld` to the linker. |
| `.cargo/config.toml` | Tells cargo to compile for the emulated RISC-V machine instead of for this PC. |
| `Cargo.toml` | The project manifest: package name and compiler options. |
| `Cargo.lock` | Records exact dependency versions. There are no dependencies here, so it is tiny. |
| `target/` | Build output, not tracked by git. The compiled `hello` ELF ends up here. |

`../scripts/build-guests.sh` builds everything.

## Why is this a separate project?

The rest of the repository is compiled for your PC. Guests are compiled for a different CPU, with no operating system, under different rules. If they were part of the main workspace, `cargo build --workspace` would try to compile them for your PC and fail (no operating system, no standard library, hardware addresses that do not exist on a PC). The main workspace also forbids `unsafe` code, while talking to devices needs a little of it (see below).

The split is wired up in three places:

- The root `Cargo.toml` says `exclude = ["guests"]`.
- `guests/Cargo.toml` contains its own empty `[workspace]` table, which makes this folder the root of a workspace of its own.
- `guests/.cargo/config.toml` sets the compile target. Cargo only reads that file when it is run from inside `guests/`, which is why `scripts/build-guests.sh` starts with `cd guests`.

### Decoding the target name `riscv32i-unknown-none-elf`

| Part | Meaning |
|---|---|
| `riscv32i` | 32-bit RISC-V, base integer instruction set only (the "I"). No multiply/divide ("M"), no compressed instructions ("C"), no floating point. This is exactly the instruction set the emulator will implement first, so the compiler can never emit an instruction the emulator does not know. |
| `unknown` | The vendor. Unspecified. |
| `none` | No operating system. This is what "bare metal" means. |
| `elf` | The output file format. |

## File by file

### `Cargo.toml`

Besides the package name, the interesting part is the build profile:

```toml
[profile.release]
panic = "abort"
opt-level = "s"
debug = "line-tables-only"
```

- `panic = "abort"`: normally a Rust panic "unwinds" the stack, which needs runtime support that bare metal does not have. Here a panic simply stops the program (our own handler in `lib.rs` decides what that means). The `dev` profile has the same setting.
- `opt-level = "s"`: optimise for small code. Smaller programs are easier to read in the disassembly pane.
- `debug = "line-tables-only"`: keep a table that maps each machine instruction back to the source line it came from, but nothing else. The emulator never loads it. It is there for a planned stretch goal: showing source code next to the disassembly. It is also why the ELF file is about 700 KB even though only about 1.1 KB of code and data are actually loaded: almost all of the file is debug information, much of it for Rust's own `core` library.

### `.cargo/config.toml`

```toml
[build]
target = "riscv32i-unknown-none-elf"
```

Makes `cargo build` compile for the emulated machine by default.

### `build.rs`

A build script is a small program cargo runs on your PC *before* compiling the crate. Ours prints three instructions for cargo:

- `cargo:rustc-link-search=<this folder>`: let the linker look for files in this folder.
- `cargo:rustc-link-arg-bins=-Tlink.ld`: when linking a program, pass `-Tlink.ld`, which means "use `link.ld` as the linker script".
- `cargo:rerun-if-changed=link.ld`: rebuild if the linker script changes.

Doing it this way means the build works no matter which directory cargo is started from. The linker cargo uses for this target is `rust-lld`, which ships with Rust, so there is nothing extra to install.

### `link.ld`

A **linker** takes the separately compiled pieces of a program (code, constants, variables) and joins them into one finished file, deciding the final memory address of every piece. On your PC the operating system's loader makes those decisions. On bare metal there is no operating system, so the **linker script** makes them. This one describes the emulated machine:

| Line in `link.ld` | What it does |
|---|---|
| `ENTRY(_start)` | Execution begins at the symbol `_start`. |
| `MEMORY { RAM ... ORIGIN = 0x80000000, LENGTH = 16M }` | The machine has 16 MiB of RAM starting at address `0x8000_0000`. These numbers must match `rv_core::memmap`. |
| `.text` | Machine code. `KEEP(*(.text.init))` puts the startup code first, so `_start` is at the very beginning of RAM. |
| `.rodata` | Read-only data: string literals and constants. |
| `.sdata` and `.data` | Variables that start with a value (none in `hello` yet). |
| `.bss` | Variables that start as zero (none yet). `NOLOAD` means they take no space in the file; the loader must zero that region. |
| `__stack_top` | The top of RAM. The **stack** (scratch space for function calls) starts here and grows *downwards*. |
| `__global_pointer$` | The value for the `gp` register, `0x8000_0800` (explained under `lib.rs`). |
| `/DISCARD/` | Drop exception-unwinding tables we do not need. |

The resulting layout for `hello` (addresses are real):

```
0x8100_0000  __stack_top: the stack starts here and grows downwards
     ...     (unused RAM)
0x8000_0480  end of the loaded program (.bss starts here and is empty)
0x8000_0454  .rodata   44 bytes of read-only data
0x8000_0000  .text     1,108 bytes of machine code; the very first instruction is _start

0x1000_0000  UART (a device, not RAM)
0x0010_0000  test finisher (a device, not RAM)
```

### `src/lib.rs`: the mini runtime

This is the heart of the folder. It is what a normal operating system and the Rust standard library would otherwise provide.

**`#![no_std]`**. Rust programs normally link the standard library (`std`): files, threads, heap memory, printing through the operating system. None of that exists here. What remains is `core`, the part of the library that needs no operating system (numbers, string slices, `Option`, text formatting). So the runtime has to supply its own way to print and to stop.

**Devices are "memory-mapped".** Some addresses are not RAM at all. A store to them is really a command to a device:

- `UART = 0x1000_0000`: a serial port. Any byte stored here is console output. In the emulator it will show up in the Console pane.
- `FINISHER = 0x0010_0000`: a special register that stops the machine. Storing `0x5555` means "stop, success". Storing `(code << 16) | 0x3333` means "stop with exit code `code`".

Both addresses are borrowed from QEMU's `virt` board, so the same ELF file should also run on QEMU for cross-checking. The constants are repeated here because this crate shares no code with the host workspace; `rv_core::memmap` is the original.

**The startup code** (`global_asm!`) is a few lines of assembly, the human-readable form of machine code. It runs first, before any Rust code:

| Line | What it does |
|---|---|
| `.section .text.init, "ax"` | Put the following code in a section named `.text.init` (allocatable, executable). `link.ld` places it first. |
| `.globl _start` and `_start:` | Define the label `_start` and make it visible to the linker. This is where the machine starts. |
| `.option push`, `.option norelax`, `.option pop` | Temporarily switch off a linker optimisation for the one line in between (see below). |
| `la gp, __global_pointer$` | Load the address `0x8000_0800` into register `gp`. |
| `la sp, __stack_top` | Load the address `0x8100_0000` (top of RAM) into the stack pointer `sp`. |
| `call main` | Run the program's `main`. |
| `call {exit}` | When `main` returns, call `exit`. The `{exit}` placeholder is filled in with the real name of the Rust function `exit` (that is what `sym exit` does). |

Notes on the less obvious parts:

- **Why `sp` must be set.** A CPU does not come with a stack. Function calls need one, so the very first job is to point `sp` at free memory. The top of RAM is the conventional choice.
- **What `gp` is.** By convention the `gp` register is a "global pointer": a fixed base address that lets compilers reach small global variables with a single short instruction. There are none yet, but standard startup code sets `gp`, so ours does too.
- **Why `norelax`.** Normally the linker may "optimise" address calculations by rewriting them relative to `gp`. That would be circular for the one line that *sets* `gp`, because it would use `gp` before it exists. `norelax` switches the optimisation off for just that line.
- **Why `main`'s result becomes the exit code.** By the RISC-V calling convention a function returns its result in register `a0`, and the first argument of a function also goes in `a0`. So after `main` returns, `a0` already holds exactly what `exit` expects.

**Printing.** `println!("...")` turns into a call to `_print`, which hands the text to Rust's built-in formatting code. That code calls our `Console::write_str`, which stores the bytes one at a time to the UART address with `write_volatile`. "Volatile" tells the compiler that the write has a side effect, so it must not be removed, merged or reordered.

**`exit`** works out the finisher value (`0x5555` for code 0, otherwise `(code << 16) | 0x3333`) and stores it. The emulator will stop the moment it sees that store. The `loop` after it is a safety net: `wfi` ("wait for interrupt") is an idle instruction, so if the machine somehow did not stop, it would idle rather than run off into garbage.

**The panic handler** is required when there is no standard library: Rust needs to know what to do when a program panics. Ours prints `panic: <message>` and exits with code 101, the same code Rust's standard library uses for a panic.

**About `unsafe`.** The compiler cannot prove that an arbitrary address like `0x1000_0000` is valid to write to, so such writes must be wrapped in `unsafe`. Every `unsafe` block in the file carries a `SAFETY:` comment saying why it is sound. The `extern "C"` on `exit` and `main` means "use the standard calling convention", so that assembly and Rust agree on where arguments and results live.

### `src/bin/hello.rs`: the program

```rust
#![no_std]
#![no_main]

use guests::println;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    println!("Hello from RV32I!");
    0
}
```

- `#![no_std]`: same as above.
- `#![no_main]`: do not generate Rust's usual startup wrapper around `main` (it needs an operating system). Our `_start` in `lib.rs` is the real entry point.
- `#[unsafe(no_mangle)]`: Rust normally renames functions internally. This keeps the plain name `main`, so the `call main` in the startup code can find it.
- `-> i32`: the exit code. `0` means success.

Every file in `src/bin/` becomes its own program, named after the file.

## What happens when `hello` runs

1. `scripts/build-guests.sh` compiles `lib.rs` and `hello.rs` for RISC-V. The linker, steered by `link.ld`, produces the ELF file `hello`.
2. *(Planned, roadmap session 2.)* The emulator reads the ELF, copies its code and read-only data (1,152 bytes in total) into emulated RAM at `0x8000_0000`, and sets the program counter (the register holding "which instruction is next") to the ELF's entry point, `_start`.
3. `_start` sets `gp` and `sp`, then calls `main`.
4. `main` calls the print routine with the message.
5. The print routine stores the message's bytes one at a time to `0x1000_0000`. The emulator sees those stores and shows the text in the Console pane.
6. `main` returns 0, and `_start` calls `exit(0)`.
7. `exit` stores `0x5555` to `0x0010_0000`. The emulator sees that and stops with exit code 0.

## A closer look at the machine code (optional)

This is the real content of `hello`: the first 19 instructions of its `.text`, which are `_start` and `main`. The session 1b disassembler will print this same view inside the TUI.

A few registers appear: `zero` is always 0, `ra` holds the return address of the current function call, `sp` is the stack pointer, `gp` is the global pointer, and `a0`/`a1` carry arguments and results. Also note that `la` and `call` look like single instructions in the assembly above, but each becomes *two* machine instructions (an `auipc` that computes an address relative to the current instruction, followed by an `addi` or `jalr`).

| Address | Hex word | Instruction | What it does |
|---|---|---|---|
| `0x80000000` | `00001197` | `auipc gp, 0x1` | `gp` = this address + `0x1000`. First half of `la gp, ...`. |
| `0x80000004` | `80018193` | `addi gp, gp, -2048` | `gp` = `gp` - 2048 = `0x80000800`. Second half. |
| `0x80000008` | `01000117` | `auipc sp, 0x1000` | `sp` = this address + `0x0100_0000`. First half of `la sp, ...`. |
| `0x8000000c` | `ff810113` | `addi sp, sp, -8` | `sp` = `0x81000000`, the top of RAM. |
| `0x80000010` | `00000097` | `auipc ra, 0x0` | `ra` = this address. First half of `call main`. |
| `0x80000014` | `010080e7` | `jalr ra, 16(ra)` | Jump to `ra` + 16 = `0x80000020` (`main`) and leave the return address `0x80000018` in `ra`. |
| `0x80000018` | `00000097` | `auipc ra, 0x0` | First half of `call exit`. |
| `0x8000001c` | `034080e7` | `jalr ra, 52(ra)` | Jump to `0x8000004c` (`exit`). It never returns. |
| `0x80000020` | `ff010113` | `addi sp, sp, -16` | `main` starts: reserve 16 bytes of stack. |
| `0x80000024` | `00112623` | `sw ra, 12(sp)` | Save the return address on the stack, because `main` is about to call another function, which overwrites `ra`. |
| `0x80000028` | `80000537` | `lui a0, 0x80000` | `a0` = `0x80000000`. Build a 32-bit address from two halves: this sets the upper 20 bits. |
| `0x8000002c` | `45450513` | `addi a0, a0, 1108` | `a0` = `0x80000454`: where the message data sits in read-only memory. |
| `0x80000030` | `02500593` | `addi a1, zero, 37` | `a1` = 37: more information about the message. Its exact meaning is internal to Rust's formatting machinery. |
| `0x80000034` | `00000097` | `auipc ra, 0x0` | First half of the call to the print routine. |
| `0x80000038` | `048080e7` | `jalr ra, 72(ra)` | Call the print routine at `0x8000007c`. |
| `0x8000003c` | `00000513` | `addi a0, zero, 0` | `a0` = 0, which is `main`'s return value. |
| `0x80000040` | `00c12083` | `lw ra, 12(sp)` | Restore the saved return address. |
| `0x80000044` | `01010113` | `addi sp, sp, 16` | Give the stack space back. |
| `0x80000048` | `00008067` | `jalr zero, 0(ra)` | Jump to `ra`: return to `_start`. (Written `ret` in assembly.) |

Everything after that is the print routine, `exit`, and Rust's formatting code. The largest function, `core::fmt::write`, is 684 bytes of the 1,108-byte `.text`.

## Try it yourself

All of these only read files. `readelf` comes with binutils, which is already on this machine.

```sh
scripts/build-guests.sh

# The header: look for "Entry point address" (0x80000000).
readelf -h guests/target/riscv32i-unknown-none-elf/release/hello

# The two LOAD lines are what the emulator will copy into RAM.
readelf -lW guests/target/riscv32i-unknown-none-elf/release/hello

# The functions in the program and their addresses.
readelf -sW guests/target/riscv32i-unknown-none-elf/release/hello | grep -w FUNC

# The first 8 machine instructions as hex words (compare with the table above).
od -An -v -tx4 -j 4096 -N 32 guests/target/riscv32i-unknown-none-elf/release/hello
```

## Adding another guest

1. Copy `src/bin/hello.rs` to `src/bin/<name>.rs` and change the body of `main`.
2. Run `scripts/build-guests.sh`.
3. The result is `guests/target/riscv32i-unknown-none-elf/release/<name>`.

`main` must keep the exact signature shown above, and its return value is the program's exit code. Planned additions: `fib` and `sieve` (roadmap session 2) and the stack-overflow demo (session 9).

## Glossary

- **ABI / calling convention**: the agreed rules for where function arguments and results live (in registers `a0`, `a1`, ... and the stack) so that separately compiled code can call each other.
- **Bare metal**: running directly on the hardware with no operating system.
- **ELF**: the executable file format used by Linux and most embedded toolchains. It holds code, data and a header saying where the program starts.
- **Guest / host**: the program running inside the emulated machine / the real computer running the emulator.
- **Linker / linker script**: the tool that joins compiled pieces into one program and assigns addresses / the file that tells it how.
- **Memory-mapped I/O**: devices that are accessed by reading and writing special memory addresses.
- **`no_std`**: Rust without its standard library, for environments with no operating system.
- **Program counter (pc)**: the register holding the address of the next instruction.
- **RISC-V / RV32I**: an open instruction-set specification / its 32-bit base integer version, about 40 instructions.
- **Section**: a named chunk of a compiled program (`.text` for code, `.rodata` for constants, ...).
- **Stack**: memory used for function calls and local variables. `sp` points at its current end, and it grows downwards.
- **Symbol**: a name for an address, such as `_start` or `main`.
- **UART**: a serial port, here used as the console.
