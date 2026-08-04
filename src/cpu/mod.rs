use crate::bus::Bus;
use crate::cpu::registers::RegisterFile;
use crate::cpu::decoder::decode;


pub struct Cpu {
    pub pc: u32,
    pub regs: RegisterFile,
    pub bus: Bus,
}

impl Cpu {
    pub fn new(bus: Bus) -> Self {
        Self {
            pc: 0,
            regs: RegisterFile::new(),
            bus,
        }
    }
}

pub fn step(&mut self) {
    // 1. Fetch
    let raw_inst = self.bus.read(self.pc).expect("Fetch failed");

    // 2. Decode
    let inst = decode(raw_inst);

    // 3. Execute & Write Back
    self.pc += 4; // 기본 PC 증가 (4바이트)

    match inst {
        Instruction::Add { rd, rs1, rs2 } => {
            let val = self.regs.read(rs1) + self.regs.read(rs2);
            self.regs.write(rd, val);
        }
        Instruction::Sub { rd, rs1, rs2 } => {
            let val = self.regs.read(rs1) - self.regs.read(rs2);
            self.regs.write(rd, val);
        }
        
        Instruction::Unknown(raw) => panic!("Unknown instruction: {:#x}", raw),
    }
}