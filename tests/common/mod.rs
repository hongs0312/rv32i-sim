use rv32i_sim::hardware::soc::SoC;

pub const RAM_SIZE: usize = 16 * 1024 * 1024;

pub fn load_program(program: &[u32]) -> SoC {
    let mut soc = SoC::new(RAM_SIZE);
    for (index, instruction) in program.iter().copied().enumerate() {
        let address = index * 4;
        soc.dram.dram[address..address + 4].copy_from_slice(&instruction.to_le_bytes());
    }
    soc
}

pub fn run_cycles(soc: &mut SoC, cycles: usize) {
    for _ in 0..cycles {
        soc.tick();
    }
}
