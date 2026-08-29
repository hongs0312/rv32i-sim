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

        // 현재 PC를 저장하여 분기 및 점프 명령어에서 사용할 수 있도록 함
        let cur_pc = self.pc;

        // 3. Execute & Write Back
        self.pc += 4; // 기본 PC 증가 (4바이트)

        match inst {
            // I-Type Load Instructions (0x03)
            Instruction::Lb { rd, rs1, imm } => {
                let addr = self.regs.read(rs1).wrapping_add(imm as u32);
                let val = self.bus.load8(addr).expect("Load failed") as i8 as u32; // Sign-extend이므로 한 번 변환 후 u32로 변환
                self.regs.write(rd, val);
            }
            Instruction::Lh { rd, rs1, imm } => {
                let addr = self.regs.read(rs1).wrapping_add(imm as u32);
                let val = self.bus.load16(addr).expect("Load failed") as i16 as u32; // Sign-extend
                self.regs.write(rd, val);
            }
            Instruction::Lw { rd, rs1, imm } => {
                let addr = self.regs.read(rs1).wrapping_add(imm as u32);
                let val = self.bus.load32(addr).expect("Load failed");
                self.regs.write(rd, val);
            }
            Instruction::Lbu { rd, rs1, imm } => {
                let addr = self.regs.read(rs1).wrapping_add(imm as u32);
                let val = self.bus.load8(addr).expect("Load failed") as u32;
                self.regs.write(rd, val);
            }
            Instruction::Lhu { rd, rs1, imm } => {
                let addr = self.regs.read(rs1).wrapping_add(imm as u32);
                let val = self.bus.load16(addr).expect("Load failed") as u32;
                self.regs.write(rd, val);
            }

            // I-Type Arithmetic Instructions (0x13)
            Instruction::Addi { rd, rs1, imm } => {
                let val = self.regs.read(rs1).wrapping_add(imm as u32);
                self.regs.write(rd, val);
            }

            // S-Type Store Instructions (0x23)
            Instruction::Sb { rs1, rs2, imm } => {
                let addr = self.regs.read(rs1).wrapping_add(imm as u32);
                let val = self.regs.read(rs2) as u8;
                self.bus.store8(addr, val).expect("Store failed");
            }
            Instruction::Sh { rs1, rs2, imm } => {
                let addr = self.regs.read(rs1).wrapping_add(imm as u32);
                let val = self.regs.read(rs2) as u16;
                self.bus.store16(addr, val).expect("Store failed");
            }
            Instruction::Sw { rs1, rs2, imm } => {
                let addr = self.regs.read(rs1).wrapping_add(imm as u32);
                let val = self.regs.read(rs2);
                self.bus.store32(addr, val).expect("Store failed");
            }

            // R-Type Instructions (0x33)
            Instruction::Add { rd, rs1, rs2 } => {
                let val = self.regs.read(rs1).wrapping_add(self.regs.read(rs2)); // wrapping_add를 사용하여 오버플로우를 허용
                self.regs.write(rd, val);
            }
            Instruction::Sub { rd, rs1, rs2 } => {
                let val = self.regs.read(rs1).wrapping_sub(self.regs.read(rs2)); // wrapping_sub를 사용하여 오버플로우를 허용
                self.regs.write(rd, val);
            }

            // U-Type Instructions (0x37, 0x17)
            Instruction::Lui { rd, imm } => {
                self.regs.write(rd, imm as u32);
            }

            // SB-Type Branch Instructions (0x63)
            Instruction::Beq { rs1, rs2, imm } => {
                if self.regs.read(rs1) == self.regs.read(rs2) {
                    self.pc = cur_pc.wrapping_add(imm as u32);
                }
            }
            Instruction::Bne { rs1, rs2, imm } => {
                if self.regs.read(rs1) != self.regs.read(rs2) {
                    self.pc = cur_pc.wrapping_add(imm as u32);
                }
            }
            Instruction::Blt { rs1, rs2, imm } => {
                if (self.regs.read(rs1) as i32) < (self.regs.read(rs2) as i32) {
                    self.pc = cur_pc.wrapping_add(imm as u32);
                }
            }
            Instruction::Bge { rs1, rs2, imm } => {
                if (self.regs.read(rs1) as i32) >= (self.regs.read(rs2) as i32) {
                    self.pc = cur_pc.wrapping_add(imm as u32);
                }
            }

            // I-Type Jump and Link Register (0x67)
            Instruction::Jalr { rd, rs1, imm } => {
                let target = self.regs.read(rs1).wrapping_add(imm as u32) & !1; // 최하위비트를 0으로 설정
                let return_address = self.pc; // 이미 PC는 4 증가했으므로 PC 레지스터 값을 반환 주소로 사용

                self.regs.write(rd, return_address);
                self.pc = target;
            }

            // U-Type Jump and Link (0x6F)
            Instruction::Jal { rd, imm } => {
                let target = cur_pc.wrapping_add(imm as u32);
                let return_address = self.pc; // 이미 PC는 4 증가했으므로 PC 레지스터 값을 반환 주소로 사용

                self.regs.write(rd, return_address);
                self.pc = target;
            }

            Instruction::Unknown(raw) => panic!("Unknown instruction: {:#x}", raw),
        }
    }
}
