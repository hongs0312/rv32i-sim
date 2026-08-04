pub mod decoder;
pub mod instruction;
pub mod register;

use crate::bus::Bus;
use crate::cpu::decoder::decode;
use crate::cpu::instruction::Instruction;
use crate::cpu::register::RegisterFile;

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

    pub fn step(&mut self) {
        // 1. Fetch
        let raw_inst = self.bus.load32(self.pc).expect("Fetch failed");
        // println!("Fetched instruction: {:#010x} at PC: {:#010x}", raw_inst, self.pc);

        // 2. Decode
        let inst = decode(raw_inst);
        // println!("Decoded instruction: {:?}", inst);

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
            Instruction::Addi { rd, rs1, imm } => {
                let val = self.regs.read(rs1) + imm as u32;
                println!("Executing ADDI: x{} = x{} + {} => {}", rd, rs1, imm, val);
                self.regs.write(rd, val);
            }

            Instruction::Unknown(raw) => panic!("Unknown instruction: {:#x}", raw),
        }
    }
}
