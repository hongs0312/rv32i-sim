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
        // 0x13 = 0010011 => I-Type Arithmetic Instructions
        0x13 => {
            let imm = (inst as i32) >> 20; // Sign-extend the immediate
            match funct3 {
                0x0 => Instruction::Addi { rd, rs1, imm },
                _ => Instruction::Unknown(inst),
            }
        }

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

        // 0x33 = 0110011 => R-Type
        0x33 => match (funct3, funct7) {
            (0x0, 0x00) => Instruction::Add { rd, rs1, rs2 },
            (0x0, 0x20) => Instruction::Sub { rd, rs1, rs2 },
            _ => Instruction::Unknown(inst),
        },
        
        _ => Instruction::Unknown(inst),
    }
}
