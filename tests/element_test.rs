use rv32i_sim::cpu::alu::Alu;
use rv32i_sim::cpu::decoder::*;
// use rv32i_sim::cpu::register::*;

#[test]
fn alutest() {
    let alu = Alu::new();

    // Test ADD
    assert_eq!(alu.get_alu_result(5, 3, 0b10, 0x0, 0), 8);

    // Test SUB
    assert_eq!(alu.get_alu_result(5, 3, 0b10, 0x0, 1 << 5), 2);

    // Test AND
    assert_eq!(alu.get_alu_result(5, 3, 0b10, 0x7, 0), 1);

    // Test OR
    assert_eq!(alu.get_alu_result(5, 3, 0b10, 0x6, 0), 7);

    // Test XOR
    assert_eq!(alu.get_alu_result(5, 3, 0b10, 0x4, 0), 6);

    // Test SLL
    assert_eq!(alu.get_alu_result(1, 2, 0b10, 0x1, 0), 4);

    // Test SRL
    assert_eq!(alu.get_alu_result(4, 1, 0b10, 0x5, 0), 2);

    // Test SRA
    assert_eq!(alu.get_alu_result(4, 1, 0b10, 0x5, 1 << 5), 2);
}

#[test]
fn imm_gen_test() {
    // Test I-Type
    let inst_i = 0b000000000001_00000_000_00000_0010011; // ADDI x0, x0, 1
    assert_eq!(imm_gen(inst_i), 1);

    // Test S-Type
    let inst_s = 0b0000000_00001_00000_010_00000_0100011; // SW x1, 0(x0)
    assert_eq!(imm_gen(inst_s), 0);

    // Test B-Type
    let inst_b = 0b0000000_00001_00000_000_00000_1100011; // BEQ x0, x1, 0
    assert_eq!(imm_gen(inst_b), 0);
}

#[test]
fn register_file_test() {
    let mut regs = rv32i_sim::cpu::register::RegisterFile::new();

    // Test writing to a register
    regs.write(1, 42, true);
    assert_eq!(regs.read(1), 42);

    // Test reading from x0 (should always be 0)
    assert_eq!(regs.read(0), 0);

    // Test that writing to x0 does not change its value
    regs.write(0, 100, true);
    assert_eq!(regs.read(0), 0);
}
