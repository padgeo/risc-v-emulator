#!/usr/bin/env bash
# Builds the bare-metal RV32I guest programs.
# Output: guests/target/riscv32i-unknown-none-elf/release/<name>
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../guests"
cargo build --release "$@"
