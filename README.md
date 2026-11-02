# RISC-V time-travel debugger

A RISC-V (RV32I) emulator in Rust with a [Ratatui](https://ratatui.rs) debugger that can run a program *backwards*: undo the last instruction, reverse-continue to a breakpoint, or jump to the instruction that last wrote a byte of memory.

> **Status: scaffolding.** The workspace, CI and a placeholder UI are in place; the emulator itself is not written yet. Progress is tracked in [`docs/roadmap.md`](docs/roadmap.md) and design decisions in [`docs/design.md`](docs/design.md).

## How it will work

Every executed instruction records the old values of whatever it changed. Undoing an instruction means applying that record in reverse, and the same record powers change highlighting and "who wrote this?" queries.

## Repository layout

| Path | Contents |
|---|---|
| `crates/rv-core` | Emulator core: decoder, machine state, history. Performs no I/O. |
| `crates/rv-tui` | Ratatui frontend. |
| `guests/` | Bare-metal RV32I programs to run in the emulator, built for `riscv32i-unknown-none-elf` in a workspace of their own. New to this? Start with [`guests/README.md`](guests/README.md). |
| `docs/` | Roadmap and design notes. |
| `scripts/` | Helper scripts. |

## Getting started

Requires a recent stable Rust (1.88 or newer). Running `rustup toolchain install` in the repository root reads `rust-toolchain.toml` and installs clippy, rustfmt and the RV32 target.

```sh
cargo run -p rv-tui          # placeholder UI; q or Esc quits
cargo test --workspace
scripts/build-guests.sh      # -> guests/target/riscv32i-unknown-none-elf/release/hello
```
