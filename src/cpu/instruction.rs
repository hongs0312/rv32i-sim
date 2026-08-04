#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    // I-Type 0x03
    Lb   { rd: u32, rs1: u32, imm: i32 },
    Lh   { rd: u32, rs1: u32, imm: i32 },
    Lw   { rd: u32, rs1: u32, imm: i32 },
    Lbu  { rd: u32, rs1: u32, imm: i32 },
    Lhu  { rd: u32, rs1: u32, imm: i32 },

    // I-Type 0x0f
    // 구현 예정

    // I-Type 0x13
    Addi { rd: u32, rs1: u32, imm: i32 },
    // Slli { rd: usize, rs1: u32, imm: i32 },

    // S-Type

    // R-Type
    Add { rd: u32, rs1: u32, rs2: u32 },
    Sub { rd: u32, rs1: u32, rs2: u32 },

    Unknown(u32),
}
