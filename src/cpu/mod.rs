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
            // R-Type Instructions (0x33)
            Instruction::Add { rd, rs1, rs2 } => {
                let val = self.regs.read(rs1).wrapping_add(self.regs.read(rs2)); // wrapping_add를 사용하여 오버플로우를 허용
                self.regs.write(rd, val);
            }
            Instruction::Sub { rd, rs1, rs2 } => {
                let val = self.regs.read(rs1).wrapping_sub(self.regs.read(rs2)); // wrapping_sub를 사용하여 오버플로우를 허용
                self.regs.write(rd, val);
            }
            Instruction::Xor { rd, rs1, rs2 } => {
                let val = self.regs.read(rs1) ^ self.regs.read(rs2);
                self.regs.write(rd, val);
            }
            Instruction::Or { rd, rs1, rs2 } => {
                let val = self.regs.read(rs1) | self.regs.read(rs2);
                self.regs.write(rd, val);
            }
            Instruction::And { rd, rs1, rs2 } => {
                let val = self.regs.read(rs1) & self.regs.read(rs2);
                self.regs.write(rd, val);
            }
            Instruction::Sll { rd, rs1, rs2 } => {
                let shamt = self.regs.read(rs2) & 0x1f; // Shift amount is lower 5 bits 어차피 32비트이므로
                let val = self.regs.read(rs1) << shamt;
                self.regs.write(rd, val);
            }
            Instruction::Srl { rd, rs1, rs2 } => {
                let shamt = self.regs.read(rs2) & 0x1f; // Shift amount is lower 5 bits
                let val = self.regs.read(rs1) >> shamt;
                self.regs.write(rd, val);
            }
            Instruction::Sra { rd, rs1, rs2 } => {
                let shamt = self.regs.read(rs2) & 0x1f;
                let val = (self.regs.read(rs1) as i32 >> shamt) as u32; // Arithmetic right shift
                self.regs.write(rd, val);
            }
            Instruction::Slt { rd, rs1, rs2 } => {
                // Set if less than (signed)
                let val = if (self.regs.read(rs1) as i32) < (self.regs.read(rs2) as i32) {
                    1
                } else {
                    0
                };
                self.regs.write(rd, val);
            }
            Instruction::Sltu { rd, rs1, rs2 } => {
                // Set if less than (unsigned)
                let val = if self.regs.read(rs1) < self.regs.read(rs2) {
                    1
                } else {
                    0
                };
                self.regs.write(rd, val);
            }

            // R-Type Multiply Extension Instructions (0x33 with funct7 = 0x01)
            Instruction::Mul { rd, rs1, rs2 } => {
                let val = self.regs.read(rs1).wrapping_mul(self.regs.read(rs2));
                self.regs.write(rd, val);
            }
            Instruction::Mulh { rd, rs1, rs2 } => {
                let s1 = self.regs.read(rs1) as i32 as i64; // signed
                let s2 = self.regs.read(rs2) as i32 as i64;
                let result = s1.wrapping_mul(s2);
                let val = (result >> 32) as u32; // 상위 32비트
                self.regs.write(rd, val);
            }
            Instruction::Mulhsu { rd, rs1, rs2 } => {
                let s1 = self.regs.read(rs1) as i32 as i64; // signed
                let s2 = self.regs.read(rs2) as u64 as i64; // unsigned
                let val = ((s1).wrapping_mul(s2) >> 32) as u32; // 상위 32비트
                self.regs.write(rd, val);
            }
            Instruction::Mulhu { rd, rs1, rs2 } => {
                let result = (self.regs.read(rs1) as u64).wrapping_mul(self.regs.read(rs2) as u64);
                let val = (result >> 32) as u32; // 상위 32비트
                self.regs.write(rd, val);
            }
            Instruction::Div { rd, rs1, rs2 } => {
                let dividend = self.regs.read(rs1) as i32;
                let divisor = self.regs.read(rs2) as i32;
                let val = if divisor == 0 {
                    u32::MAX // Division by zero returns -1 (0xFFFFFFFF)
                } else if dividend == i32::MIN && divisor == -1 {
                    dividend as u32 // Overflow case: return dividend (i32::MIN)
                } else {
                    (dividend / divisor) as u32
                };
                self.regs.write(rd, val);
            }
            Instruction::Divu { rd, rs1, rs2 } => {
                let dividend = self.regs.read(rs1);
                let divisor = self.regs.read(rs2);
                let val = if divisor == 0 {
                    u32::MAX // Division by zero returns -1 (0xFFFFFFFF)
                } else {
                    dividend / divisor
                };
                self.regs.write(rd, val);
            }
            Instruction::Rem { rd, rs1, rs2 } => {
                let dividend = self.regs.read(rs1) as i32;
                let divisor = self.regs.read(rs2) as i32;
                let val = if divisor == 0 {
                    dividend as u32 // Remainder by zero returns dividend
                } else if dividend == i32::MIN && divisor == -1 {
                    0 // Overflow case: remainder is 0
                } else {
                    (dividend % divisor) as u32
                };
                self.regs.write(rd, val);
            }
            Instruction::Remu { rd, rs1, rs2 } => {
                let dividend = self.regs.read(rs1);
                let divisor = self.regs.read(rs2);
                let val = if divisor == 0 {
                    dividend // Remainder by zero returns dividend
                } else {
                    dividend % divisor
                };
                self.regs.write(rd, val);
            }

            // I-Type Arithmetic Instructions (0x13)
            Instruction::Addi { rd, rs1, imm } => {
                let val = self.regs.read(rs1).wrapping_add(imm as u32);
                self.regs.write(rd, val);
            }
            Instruction::Xori { rd, rs1, imm } => {
                let val = self.regs.read(rs1) ^ (imm as u32);
                self.regs.write(rd, val);
            }
            Instruction::Ori { rd, rs1, imm } => {
                let val = self.regs.read(rs1) | (imm as u32);
                self.regs.write(rd, val);
            }
            Instruction::Andi { rd, rs1, imm } => {
                let val = self.regs.read(rs1) & (imm as u32);
                self.regs.write(rd, val);
            }
            Instruction::Slli { rd, rs1, imm } => {
                let shamt = (imm & 0x1f) as u32; // Shift amount is lower 5 bits
                let val = self.regs.read(rs1) << shamt;
                self.regs.write(rd, val);
            }
            Instruction::Srli { rd, rs1, imm } => {
                let shamt = (imm & 0x1f) as u32; // Shift amount is lower 5 bits
                let val = self.regs.read(rs1) >> shamt;
                self.regs.write(rd, val);
            }
            Instruction::Srai { rd, rs1, imm } => {
                let shamt = (imm & 0x1f) as u32; // Shift amount is lower 5 bits
                let val = (self.regs.read(rs1) as i32 >> shamt) as u32; // Arithmetic right shift
                self.regs.write(rd, val);
            }
            Instruction::Slti { rd, rs1, imm } => {
                let val = if (self.regs.read(rs1) as i32) < imm {
                    1
                } else {
                    0
                };
                self.regs.write(rd, val);
            }
            Instruction::Sltiu { rd, rs1, imm } => {
                let val = if self.regs.read(rs1) < (imm as u32) {
                    1
                } else {
                    0
                };
                self.regs.write(rd, val);
            }

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

            // B-Type Branch Instructions (0x63)
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
            Instruction::Bltu { rs1, rs2, imm } => {
                if self.regs.read(rs1) < self.regs.read(rs2) {
                    self.pc = cur_pc.wrapping_add(imm as u32);
                }
            }
            Instruction::Bgeu { rs1, rs2, imm } => {
                if self.regs.read(rs1) >= self.regs.read(rs2) {
                    self.pc = cur_pc.wrapping_add(imm as u32);
                }
            }

            // U-Type Jump and Link (0x6F)
            Instruction::Jal { rd, imm } => {
                let target = cur_pc.wrapping_add(imm as u32);
                let return_address = self.pc; // 이미 PC는 4 증가했으므로 PC 레지스터 값을 반환 주소로 사용

                self.regs.write(rd, return_address);
                self.pc = target;
            }

            // I-Type Jump and Link Register (0x67)
            Instruction::Jalr { rd, rs1, imm } => {
                let target = self.regs.read(rs1).wrapping_add(imm as u32) & !1; // 최하위비트를 0으로 설정
                let return_address = self.pc; // 이미 PC는 4 증가했으므로 PC 레지스터 값을 반환 주소로 사용

                self.regs.write(rd, return_address);
                self.pc = target;
            }

            // U-Type Instructions (0x37, 0x17)
            Instruction::Lui { rd, imm } => {
                self.regs.write(rd, imm as u32);
            }
            Instruction::Auipc { rd, imm } => {
                let val = cur_pc.wrapping_add(imm as u32);
                self.regs.write(rd, val);
            }

            // System Instructions (0x73)
            Instruction::Ecall => {
                let a7 = self.regs.read(17); // a7 레지스터는 시스템 호출 번호를 나타냄
                match a7 {
                    93 => {
                        // sys_exit
                        let exit_code = self.regs.read(10); // a0 레지스터는 종료 코드를 나타냄
                        println!("Program exited with code: {}", exit_code);
                        std::process::exit(exit_code as i32);
                    }
                    _ => {
                        println!("Unknown system call: {}", a7);
                    }
                }
            }
            Instruction::Ebreak => {
                // Handle breakpoint (for now, just print a message)
                println!("Breakpoint hit at PC: {:#010x}", cur_pc);
            }
            // Instruction::Fence => {
            //     // Handle fence instruction (for now, just print a message)
            //     println!("Fence instruction at PC: {:#010x}", cur_pc);
            // }
            Instruction::Unknown(raw) => panic!("Unknown instruction: {:#x}", raw),

            _ => panic!("Unknown instruction: {:?}", inst),
        }

        // println!("PC: {:#010x} | a0(x10): {} | sp(x2): {:#010x} | ra(x1): {:#010x}",
        //     cur_pc,
        //     self.regs.read(10),
        //     self.regs.read(2),
        //     self.regs.read(1)
        // );
    }
}
