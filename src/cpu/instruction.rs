#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    // R-Type 0x33
    Add { rd: u32, rs1: u32, rs2: u32 },
    Sub { rd: u32, rs1: u32, rs2: u32 },
    Xor { rd: u32, rs1: u32, rs2: u32 },
    Or { rd: u32, rs1: u32, rs2: u32 },
    And { rd: u32, rs1: u32, rs2: u32 },
    Sll { rd: u32, rs1: u32, rs2: u32 },
    Srl { rd: u32, rs1: u32, rs2: u32 },
    Sra { rd: u32, rs1: u32, rs2: u32 },
    Slt { rd: u32, rs1: u32, rs2: u32 },
    Sltu { rd: u32, rs1: u32, rs2: u32 },

    // I-Type 0x13
    Addi { rd: u32, rs1: u32, imm: i32 },
    Xori { rd: u32, rs1: u32, imm: i32 },
    Ori { rd: u32, rs1: u32, imm: i32 },
    Andi { rd: u32, rs1: u32, imm: i32 },
    Slli { rd: u32, rs1: u32, imm: i32 },
    Srli { rd: u32, rs1: u32, imm: i32 },
    Srai { rd: u32, rs1: u32, imm: i32 },
    Slti { rd: u32, rs1: u32, imm: i32 },
    Sltiu { rd: u32, rs1: u32, imm: i32 },

    // I-Type 0x03
    Lb { rd: u32, rs1: u32, imm: i32 },
    Lh { rd: u32, rs1: u32, imm: i32 },
    Lw { rd: u32, rs1: u32, imm: i32 },
    Lbu { rd: u32, rs1: u32, imm: i32 },
    Lhu { rd: u32, rs1: u32, imm: i32 },

    // S-Type 0x23
    Sb { rs1: u32, rs2: u32, imm: i32 },
    Sh { rs1: u32, rs2: u32, imm: i32 },
    Sw { rs1: u32, rs2: u32, imm: i32 },

    // B-Type 0x63
    Beq { rs1: u32, rs2: u32, imm: i32 },
    Bne { rs1: u32, rs2: u32, imm: i32 },
    Blt { rs1: u32, rs2: u32, imm: i32 },
    Bge { rs1: u32, rs2: u32, imm: i32 },
    Bltu { rs1: u32, rs2: u32, imm: i32 },
    Bgeu { rs1: u32, rs2: u32, imm: i32 },

    // J-Type 0x6f
    Jal { rd: u32, imm: i32 },
    // I-Type 0x67
    Jalr { rd: u32, rs1: u32, imm: i32 },

    // U-Type 0x37, 0x17
    Lui { rd: u32, imm: i32 },
    Auipc { rd: u32, imm: i32 },

    // RV32M Multiply Extension
    Mul { rd: u32, rs1: u32, rs2: u32 },
    Mulh { rd: u32, rs1: u32, rs2: u32 },
    Mulhsu { rd: u32, rs1: u32, rs2: u32 },
    Mulhu { rd: u32, rs1: u32, rs2: u32 },
    Div { rd: u32, rs1: u32, rs2: u32 },
    Divu { rd: u32, rs1: u32, rs2: u32 },
    Rem { rd: u32, rs1: u32, rs2: u32 },
    Remu { rd: u32, rs1: u32, rs2: u32 },

    Ecall, // 시스템 호출
    Ebreak,
    Fence,

    Unknown(u32),
}
