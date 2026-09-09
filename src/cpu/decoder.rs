use crate::cpu::instruction::Instruction;

pub fn decode(inst: u32) -> Instruction {
    // Decode the opcode
    // 0000 0000 0000 0000 0000 0000 0000 0000
    // 31 <- 0순으로 인덱싱
    let opcode = inst & 0x7f; // 0~6 bits
    let rd = ((inst >> 7) & 0x1f) as u32; // 7~11 bits
    let funct3 = (inst >> 12) & 0x7; // 12~14 bits
    let rs1 = ((inst >> 15) & 0x1f) as u32; // 15~19 bits
    let rs2 = ((inst >> 20) & 0x1f) as u32; // 20~24 bits
    let funct7 = (inst >> 25) & 0x7f; // 25~31 bits

    match opcode {
        // 0x33 = 0110011 => R-Type
        0x33 => match (funct3, funct7) {
            (0x0, 0x00) => Instruction::Add { rd, rs1, rs2 },
            (0x0, 0x20) => Instruction::Sub { rd, rs1, rs2 },
            (0x4, 0x00) => Instruction::Xor { rd, rs1, rs2 },
            (0x6, 0x00) => Instruction::Or { rd, rs1, rs2 },
            (0x7, 0x00) => Instruction::And { rd, rs1, rs2 },
            (0x1, 0x00) => Instruction::Sll { rd, rs1, rs2 },
            (0x5, 0x00) => Instruction::Srl { rd, rs1, rs2 },
            (0x5, 0x20) => Instruction::Sra { rd, rs1, rs2 },
            (0x2, 0x00) => Instruction::Slt { rd, rs1, rs2 },
            (0x3, 0x00) => Instruction::Sltu { rd, rs1, rs2 },

            // RV32M Multiply Extension
            (0x0, 0x01) => Instruction::Mul { rd, rs1, rs2 },
            (0x1, 0x01) => Instruction::Mulh { rd, rs1, rs2 },
            (0x2, 0x01) => Instruction::Mulhsu { rd, rs1, rs2 },
            (0x3, 0x01) => Instruction::Mulhu { rd, rs1, rs2 },
            (0x4, 0x01) => Instruction::Div { rd, rs1, rs2 },
            (0x5, 0x01) => Instruction::Divu { rd, rs1, rs2 },
            (0x6, 0x01) => Instruction::Rem { rd, rs1, rs2 },
            (0x7, 0x01) => Instruction::Remu { rd, rs1, rs2 },
            _ => Instruction::Unknown(inst),
        },
        // 0x13 = 0010011 => I-Type Arithmetic Instructions
        0x13 => {
            let imm = (inst as i32) >> 20; // Sign-extend the immediate
            match funct3 {
                0x0 => Instruction::Addi { rd, rs1, imm },
                0x1 => Instruction::Slli { rd, rs1, imm }, // funct7 should be 0x00
                0x2 => Instruction::Slti { rd, rs1, imm },
                0x3 => Instruction::Sltiu { rd, rs1, imm },
                0x4 => Instruction::Xori { rd, rs1, imm },
                0x6 => Instruction::Ori { rd, rs1, imm },
                0x7 => Instruction::Andi { rd, rs1, imm },
                0x5 => match funct7 {
                    // imm[11:5] = funct7
                    0x00 => Instruction::Srli { rd, rs1, imm },
                    0x20 => Instruction::Srai { rd, rs1, imm },
                    _ => Instruction::Unknown(inst),
                },
                _ => Instruction::Unknown(inst),
            }
        }

        // 0x03 = 0000011 => I-Type Load Instructions
        0x03 => {
            let imm = (inst as i32) >> 20; // Sign-extend the immediate
            match funct3 {
                0x0 => Instruction::Lb { rd, rs1, imm },
                0x1 => Instruction::Lh { rd, rs1, imm },
                0x2 => Instruction::Lw { rd, rs1, imm },
                0x4 => Instruction::Lbu { rd, rs1, imm },
                0x5 => Instruction::Lhu { rd, rs1, imm },
                _ => Instruction::Unknown(inst),
            }
        }

        // 0x23 = 0100011 => S-Type Store Instructions
        0x23 => {
            let raw_imm = funct7 << 5 | rd; // S-Type raw immediate
            let imm = ((raw_imm as i32) << 20) >> 20; // Sign-extend the immediate
            match funct3 {
                0x0 => Instruction::Sb { rs1, rs2, imm: imm },
                0x1 => Instruction::Sh { rs1, rs2, imm: imm },
                0x2 => Instruction::Sw { rs1, rs2, imm: imm },
                _ => Instruction::Unknown(inst),
            }
        }

        // 0x63 = 1100011 => B-Type Branch Instructions
        0x63 => {
            // let imm_10_5 = funct7 & 0x3f; // bits 5-10
            // let imm_12 = (funct7 >> 6) & 0x1; // bit 12
            // let imm_11 = rd & 0x1; // bit 11
            // let imm_4_1 = (rd >> 1) & 0xf; // bits 1-4  

            // let raw_imm =
            //     ((imm_12 << 12) | (imm_11 << 11) | (imm_10_5 << 5) | (imm_4_1 << 1)) as i32;
            // let imm = (raw_imm << 19) >> 19; // Sign-extend the immediate

            // B-Type imm 디코딩 예시
            let imm12   = (inst >> 31) & 0x1;
            let imm10_5 = (inst >> 25) & 0x3F;
            let imm4_1  = (inst >> 8) & 0xF;
            let imm11   = (inst >> 7) & 0x1;

            // Bit 0은 무조건 0이므로 (imm4_1 << 1) 형태로 이미 LSB가 0으로 맞춰집니다.
            let raw_imm = (imm12 << 12) | (imm11 << 11) | (imm10_5 << 5) | (imm4_1 << 1);

            // Sign Extension (13비트 -> 32비트 i32)
            let imm = ((raw_imm as i32) << 19) >> 19;

            match funct3 {
                0x0 => Instruction::Beq { rs1, rs2, imm },
                0x1 => Instruction::Bne { rs1, rs2, imm },
                0x4 => Instruction::Blt { rs1, rs2, imm },
                0x5 => Instruction::Bge { rs1, rs2, imm },
                0x6 => Instruction::Bltu { rs1, rs2, imm },
                0x7 => Instruction::Bgeu { rs1, rs2, imm },
                _ => Instruction::Unknown(inst),
            }
        }

        // 0x6f = 1101111 => J-Type (JAL)
        0x6f => {
            let imm_19_12 = (inst >> 12) & 0xff; // bits 12-19
            let imm_11 = (inst >> 20) & 0x1; // bit 11
            let imm_10_1 = (inst >> 21) & 0x3ff; // bits 1-10
            let imm_20 = (inst >> 31) & 0x1; // bit 20

            let raw_imm =
                ((imm_20 << 20) | (imm_19_12 << 12) | (imm_11 << 11) | (imm_10_1 << 1)) as i32;
            let imm = (raw_imm << 11) >> 11; // Sign-extend the immediate

            Instruction::Jal { rd, imm }
        }

        // 0x67 = 1100111 => I-Type (JALR)
        0x67 => {
            let imm = (inst as i32) >> 20; // Sign-extend the immediate
            match funct3 {
                0x0 => Instruction::Jalr { rd, rs1, imm },
                _ => Instruction::Unknown(inst),
            }
        }

        // 0x37 = 0110111 => U-Type (LUI)
        0x37 => {
            let imm = (inst & 0xfffff000) as i32; // 상위 20비트만 사용
            Instruction::Lui { rd, imm }
        }

        // 0x17 = 0010111 => U-Type (AUIPC)
        0x17 => {
            let imm = (inst & 0xfffff000) as i32; // 상위 20비트만 사용
            Instruction::Auipc { rd, imm }
        }

        0x73 => {
            let imm = (inst as i32) >> 20; // Sign-extend the immediate
            match imm {
                0x000 => Instruction::Ecall,
                0x001 => Instruction::Ebreak,
                _ => Instruction::Unknown(inst),
            }
        }

        _ => Instruction::Unknown(inst),
    }
}
