use crate::cpu::Cpu;
use crate::cpu::elements::{
    control::get_control_signals,
    decoder::{Decoder, imm_gen},
};

use super::{IdExRegister, IfIdRegister};

pub fn execute(cpu: &mut Cpu, if_id_reg: IfIdRegister) -> IdExRegister {
    let (pc, instruction) = (if_id_reg.pc, if_id_reg.instruction);
    let (funct7, rs2, rs1, funct3, rd, opcode) = Decoder::decode(instruction);

    let rs1_data = cpu.regs.read(rs1);
    let rs2_data = cpu.regs.read(rs2);
    let imm = imm_gen(instruction);

    let control = get_control_signals(opcode, funct3, funct7);

    IdExRegister {
        control,
        pc,
        rd,
        rs1,
        rs2,
        rs1_data,
        rs2_data,
        imm,
    }
}
