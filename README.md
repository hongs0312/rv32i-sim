# RV32IM C-Code Compiler & Pipelined Simulator

A lightweight **RISC-V (RV32IM)** instruction set simulator written in Rust.  
It automatically compiles C source files using the RISC-V GNU Toolchain, extracts raw binary code, loads it into a simulated 16MB DRAM, and executes the instructions with cycle-by-cycle register inspection.

This simulator features a **5-stage pipeline** with precise hardware stall emulation (e.g., 5-cycle DRAM loads, 3-cycle hardware multipliers) and integrates a custom **Systolic Array Accelerator** via MMIO (Memory-Mapped I/O) to dramatically speed up matrix multiplication workloads.

---

## 🚀 Prerequisites

Ensure you have the following installed in your environment:
* **Rust** (Cargo)
* **RISC-V GNU Toolchain** (`riscv64-unknown-elf-gcc`, `riscv64-unknown-elf-objcopy`)

---

## 💻 Quick Start & Usage

Run C programs directly on the simulator using `cargo run`:

```bash
cargo run -- --source <PATH_TO_C_FILE> [OPTIONS]
OptionsOptionLong FlagDescriptionDefault-s--sourcePath to the target C source filefiles/main.c-p--pipelineEnable 5-stage pipelining executionfalse-v--verboseEnable cycle-by-cycle trace log (PC, Inst, sp, a0)false-m--max-stepsMaximum instruction limit to prevent infinite loops100000⚡ Systolic Array AcceleratorTo overcome the memory wall and accelerate matrix multiplications, this simulator includes a custom 16x16 Systolic Array module. It operates entirely independently of the CPU pipeline through MMIO.FeaturesBuilt-in DMA Controller: Automatically fetches matrix data from DRAM using Burst Mode (initial 5-cycle latency + 1 cycle/word), preventing CPU stalls.Wavefront Computing: Streams data through a 2D PE (Processing Element) grid utilizing a pipelined data skewing mechanism.Hardware Timer: A dedicated MMIO timer for precise cycle profiling.Performance: Achieves up to ~96x speedup on 16x16 matrix multiplication compared to pure RV32IM CPU execution (approx. 840 cycles vs. 81,041 cycles).MMIO Memory MapThe accelerator is mapped to the >= 0x8000_0000 memory address space.AddressRegisterAccessDescription0x8000_0000STATUSR/W0: Idle, 1: Running, 2: Done0x8000_0004ADDR_AWBase address of Matrix A in DRAM0x8000_0008ADDR_BWBase address of Matrix B in DRAM0x8000_000CADDR_CWBase address to store Result Matrix C0x8000_0010STARTWWrite 1 to trigger DMA and computation0x8000_0020TIMERRGlobal hardware cycle counterC-Code ExampleC#define SYSTOLIC_STATUS (*(volatile unsigned int*)0x80000000)
#define SYSTOLIC_ADDR_A (*(volatile unsigned int*)0x80000004)
#define SYSTOLIC_ADDR_B (*(volatile unsigned int*)0x80000008)
#define SYSTOLIC_ADDR_C (*(volatile unsigned int*)0x8000000C)
#define SYSTOLIC_START  (*(volatile unsigned int*)0x80000010)

void matmul_systolic(unsigned int* A, unsigned int* B, unsigned int* C) {
    SYSTOLIC_ADDR_A = (unsigned int)A;
    SYSTOLIC_ADDR_B = (unsigned int)B;
    SYSTOLIC_ADDR_C = (unsigned int)C;
    SYSTOLIC_START = 1;
    while (SYSTOLIC_STATUS != 2) {} // Wait for accelerator
}
📚 ReferenceRISC-V ISA Reference Card
