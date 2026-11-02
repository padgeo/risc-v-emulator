//! Core of the RISC-V time-travel debugger.
//!
//! This crate owns everything about the emulated machine and nothing about how it is shown: it
//! performs no terminal or file I/O, so frontends and tests drive it through plain function calls.
//!
//! Modules are added in build order (see `docs/roadmap.md`): instruction decoding and
//! disassembly, memory and devices, the CPU, machine-mode traps, and the history that makes
//! stepping backwards possible.

#![warn(missing_docs)]

pub mod memmap;
