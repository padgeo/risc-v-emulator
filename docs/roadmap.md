# Roadmap

Twelve working sessions of about three hours each (roughly two weeks of near-daily evenings), plus two or three buffer sessions. Sessions 3 and 4 are the usual time sink. Tick items off as they land.

- [x] **1a. Scaffold.** Workspace, CI, placeholder UI, first guest program. *Done when:* fmt, clippy and tests pass and `hello` builds to an RV32 ELF.
- [ ] **1b. Decoder.** Decoder and disassembler for all RV32I formats. *Done when:* output matches `llvm-objdump` (aliases off) on a compiled guest, and random 32-bit words never panic the decoder.
- [ ] **2. Execution.** Memory, ELF loader, full RV32I execution, console and finisher devices. *Done when:* Rust guests (fib, sieve) print the expected output and per-instruction tests pass.
- [ ] **3. Traps.** CSRs, traps, `mret`, the `tohost` convention; build `riscv-tests`. *Done when:* the first `rv32ui-p` tests pass.
- [ ] **4. Conformance.** *Done when:* the full `rv32ui-p` set is green and skipped tests are documented.
- [ ] **5. Time travel.** Delta log, `step_back`, reverse-run, reversible console. *Done when:* a property test shows that going forward N steps and back N restores an identical state hash.
- [ ] **6. Measure.** Benchmarks, bounded history, determinism test. *Done when:* the README reports MIPS with and without recording.
- [ ] **7. TUI core.** Registers and disassembly panes, step/back/continue. *Done when:* you can step a program forward and back and changed registers highlight.
- [ ] **8. Memory and console.** Hex/stack view with change highlighting, console pane, breakpoints. *Done when:* a run honours a time budget and breakpoints can be set in the gutter.
- [ ] **9. Reverse debugging.** Reverse-continue, watchpoints, "who wrote this?", timeline bar. *Done when:* the stack-smash demo works end to end.
- [ ] **10. Polish.** Step-over/finish, trap banner, help overlay, error handling. *Done when:* bad ELF files and traps are handled gracefully.
- [ ] **11. UI tests.** Pane snapshot tests (`TestBackend` + `insta`), last-writer property test. *Done when:* CI is green and clippy is clean.
- [ ] **12. Ship.** README GIF, architecture and trade-off docs, release. *Done when:* someone can install and run it in a minute.

**If behind, cut in this order:** recency heat in the hex view; step-over/finish; timeline scrubbing; the machine-mode trap tests.

**Stretch goals, by impact:**

1. M extension (multiply/divide) and its tests.
2. Source-level view via `gimli`/`addr2line` next to the disassembly.
3. Checkpoints plus deterministic replay for unbounded history.
4. A GDB remote stub with reverse packets, so real GDB can drive the emulator.
5. Compressed instructions (C extension), so typical `rv32imc` binaries run.
6. Lockstep trace comparison against a reference simulator such as Spike or QEMU.
