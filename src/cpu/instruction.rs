#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    // I-Type 0x03
    Lb { rd: u32, rs1: u32, imm: i32 },
    Lh { rd: u32, rs1: u32, imm: i32 },
    Lw { rd: u32, rs1: u32, imm: i32 },
    Lbu { rd: u32, rs1: u32, imm: i32 },
    Lhu { rd: u32, rs1: u32, imm: i32 },

    // I-Type 0x0f
    // 구현 예정

    // I-Type 0x13
    Addi { rd: u32, rs1: u32, imm: i32 },
    // Slli { rd: usize, rs1: u32, imm: i32 },

    // S-Type 0x23
    Sb { rs1: u32, rs2: u32, imm: i32 },
    Sh { rs1: u32, rs2: u32, imm: i32 },
    Sw { rs1: u32, rs2: u32, imm: i32 },

    // R-Type 0x33
    Add { rd: u32, rs1: u32, rs2: u32 },
    Sub { rd: u32, rs1: u32, rs2: u32 },

    // U-Type 0x37, 0x17
    Lui { rd: u32, imm: i32 },
    // Auiipc { rd: u32, imm: i32 },

    // SB-Type 0x63
    Beq { rs1: u32, rs2: u32, imm: i32 },
    Bne { rs1: u32, rs2: u32, imm: i32 },
    Blt { rs1: u32, rs2: u32, imm: i32 },
    Bge { rs1: u32, rs2: u32, imm: i32 },

    Unknown(u32),
}
