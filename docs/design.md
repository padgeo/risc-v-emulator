# Design notes

Decisions are recorded as they are made. Status is **accepted** (reflected in the code), **planned** (agreed, not built yet) or **open**.

## Workspace split: engine without I/O (accepted)

`rv-core` is a library that performs no I/O. `rv-tui` is a thin frontend that drives it through plain calls. Keeping the engine free of terminal concerns makes it testable without a UI, and lets the UI be tested separately with Ratatui's `TestBackend`.

## Memory map: a subset of QEMU `virt` (accepted)

RAM at `0x8000_0000` (16 MiB), a UART transmit register at `0x1000_0000` and a test-finisher register at `0x0010_0000`. These match QEMU's `virt` board, so a guest ELF built for this machine should also run unchanged under `qemu-system-riscv32 -M virt`, which gives us a reference to cross-check against. `rv-core::memmap`, `guests/link.ld` and `guests/src/lib.rs` must agree.

## Guest I/O is memory-mapped; `ecall` is a real trap (accepted)

Guests print by storing bytes to the UART and exit by writing to the finisher. `ecall` is not a host-call shortcut: it raises an environment-call trap as the specification says, which the official `riscv-tests` rely on. Memory-mapped devices are also easier to make reversible than a special system-call path.

## Time travel: undo log of per-instruction deltas (planned)

Each retired instruction records the old values of what it changed: the pc, at most one register, at most one memory location, CSRs and device state. Undo applies a delta in reverse. The same record drives change highlighting and "who wrote this?" queries. Traps change several CSRs at once and need a larger record. Devices must be reversible too (console output is truncated when stepping back past a write). History is a bounded ring buffer.

Alternatives considered:

- Full snapshots per step: too much memory.
- Periodic checkpoints plus deterministic replay: unbounded history, but more machinery. A possible later extension.

## Open questions

- Misaligned loads and stores: trap, or emulate them? (This affects the `ma_data` test in `riscv-tests`.)
- Default size of the history window.
- Single-threaded budgeted run loop, or an emulator thread?
