#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    // I-Type 0x03
    Lb   { rd: usize, rs1: usize, imm: i32 },
    Lh   { rd: usize, rs1: usize, imm: i32 },
    Lw   { rd: usize, rs1: usize, imm: i32 },
    Lbu  { rd: usize, rs1: usize, imm: i32 },
    Lhu  { rd: usize, rs1: usize, imm: i32 },

    // I-Type 0x0f
    // 구현 예정

    // I-Type 0x13
    Addi { rd: usize, rs1: usize, imm: i32 },
    Slli { rd: usize, rs1: usize, imm: i32 },

    // S-Type

    // R-Type
    Add { rd: usize, rs1: usize, rs2: usize },
    Sub { rd: usize, rs1: usize, rs2: usize },


    Unknown(u32),
}