# RV32I C-Code Compiler & Simulator

A lightweight **RISC-V (RV32I)** instruction set simulator written in Rust.  
It automatically compiles C source files using the RISC-V GNU Toolchain, extracts raw binary code, loads it into a simulated 16MB DRAM, and executes the instructions with cycle-by-cycle register inspection.

---

## Prerequisites

Ensure you have the following installed in your environment:
* **Rust** (Cargo)
* **RISC-V GNU Toolchain** (`riscv64-unknown-elf-gcc`, `riscv64-unknown-elf-objcopy`)

---

## Quick Start & Usage

Run C programs directly on the simulator using `cargo run`:

```bash
cargo run -- --source <PATH_TO_C_FILE> [OPTIONS]
```

## Options
| Option | Long Flag | Description | Default |
| --- | --- | --- | --- |
| -s | --source | Path to the target C source file | files/main.c |
| -v | --verbose |Enable cycle-by-cycle trace log (PC, Inst, sp, a0) | false |
| -m | --max-steps | Maximum instruction limit to prevent infinite loops | 100000 |
