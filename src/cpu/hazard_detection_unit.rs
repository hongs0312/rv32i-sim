/*
    memory hazard 해결을 위한 hazard detection unit과 control hazard 해결을 위한 forwarding unit 구현
    - hazard detection unit은 EX 단계에서 발생하는 memory hazard를 해결하기 위해 사용됨
    - EX 단계에서 load 명령어가 수행될 때, 다음 명령어가 load 명령어의 결과를 필요로 하는 경우, pipeline을 stall 시켜야 함
*/

use crate::cpu::{
    decoder::*,
    register::{ExMemRegister, IdExRegister, MemWbRegister},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForwardA {
    NoForward = 0,      // EX 단계에서 rs1에 대한 forwarding이 필요 없음
    ForwardFromMem = 1, // MEM 단계에서 rs1에 대한 forwarding 필요 (1 cycle 전)
    ForwardFromWb = 2,  // WB 단계에서 rs1에 대한 forwarding 필요 (2 cycle 전)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForwardB {
    NoForward = 0,      // EX 단계에서 rs2에 대한 forwarding이 필요 없음
    ForwardFromMem = 1, // MEM 단계에서 rs2에 대한 forwarding 필요 (1 cycle 전)
    ForwardFromWb = 2,  // WB 단계에서 rs2에 대한 forwarding 필요 (2 cycle 전)
}

// forwarding unit 구현
pub struct ForwardingUnit;

impl ForwardingUnit {
    pub fn get_forward_signals(
        id_ex_reg: &IdExRegister,
        ex_mem_reg: &ExMemRegister,
        mem_wb_reg: &MemWbRegister,
    ) -> (ForwardA, ForwardB) {
        let mut forward_a = ForwardA::NoForward;
        let mut forward_b = ForwardB::NoForward;

        // 단계적으로 거슬러 올라가며 forwarding 필요 여부를 판단

        // MEM 단계에서 rs1에 대한 forwarding 결정
        // 조건 1: MEM/WB 단계에서 레지스터 쓰기 활성화
        // 조건 2: MEM/WB 단계에서 쓰기 대상 레지스터가 0이 아님
        // 조건 3: MEM/WB 단계에서 쓰기 대상 레지스터가 ID/EX 단계에서 읽는 rs1과 동일
        // if mem_wb_reg_write && (mem_wb_rd != 0) && (mem_wb_rd == id_ex_rs1) {
        //     forward_a = ForwardA::ForwardFromWb;
        // }
        // if mem_wb_reg_write && (mem_wb_rd != 0) && (mem_wb_rd == id_ex_rs2) {
        //     forward_b = ForwardB::ForwardFromWb;
        // }

        if mem_wb_reg.control.reg_write && mem_wb_reg.rd != 0 && mem_wb_reg.rd == id_ex_reg.rs1 {
            forward_a = ForwardA::ForwardFromWb;
        }
        if mem_wb_reg.control.reg_write && mem_wb_reg.rd != 0 && mem_wb_reg.rd == id_ex_reg.rs2 {
            forward_b = ForwardB::ForwardFromWb;
        }

        // EX 단계에서 rs1에 대한 forwarding 결정
        // 조건 1: EX/MEM 단계에서 레지스터 쓰기 활성화
        // 조건 2: EX/MEM 단계에서 메모리 읽기 활성화가 아닌 경우 (load 명령어가 아닌 경우)
        // 조건 3: EX/MEM 단계에서 쓰기 대상 레지스터가 0이 아님
        // 조건 4: EX/MEM 단계에서 쓰기 대상 레지스터가 ID/EX 단계에서 읽는 rs1과 동일
        if ex_mem_reg.control.reg_write && (ex_mem_reg.rd != 0) && (ex_mem_reg.rd == id_ex_reg.rs1)
        {
            forward_a = ForwardA::ForwardFromMem;
        }
        if ex_mem_reg.control.reg_write && (ex_mem_reg.rd != 0) && (ex_mem_reg.rd == id_ex_reg.rs2)
        {
            forward_b = ForwardB::ForwardFromMem;
        }

        (forward_a, forward_b)
    }
}

pub struct HazardDetectionUnit;

impl HazardDetectionUnit {
    pub fn check_load_use(id_ex_mem_read: bool, id_ex_rd: u8, if_id_instruction: u32) -> bool {
        // EX 단계에서 load 명령어가 아니거나 쓰기 대상 레지스터가 x0이면 hazard 없음
        if !id_ex_mem_read || id_ex_rd == 0 {
            return false;
        }

        // IF 단계에서 명령어를 디코딩하여 rs1, rs2 레지스터를 확인
        let (_, rs2, rs1, _, _, _) = Decoder::decode(if_id_instruction);

        // load-use hazard 발생 여부를 판단하기 위해, IF 단계에서 읽은 명령어가 rs1 또는 rs2를 사용하는지 확인

        // load-use hazard 발생 여부를 판단
        // 조건: EX 단계에서 load 명령어가 수행 중이고, IF 단계에서 읽은 명령어가 EX 단계의 rd 레지스터를 사용하고 있는 경우
        (rs1 == id_ex_rd) || (rs2 == id_ex_rd)
    }
}

// // IF 단계에서 읽은 명령어가 rs1을 사용하는지 확인
// fn check_if_instruction_uses_rs1(opcode: u8) -> bool {
//     match opcode {
//         0x33 => true, // R-Type
//         0x13 => true, // I-Type (Immediate ALU)
//         0x03 => true, // I-Type (Load)
//         0x67 => true, // I-Type (JALR)
//         0x23 => true, // S-Type (Store)
//         0x63 => true, // B-Type (Branch)
//         _ => false,
//     }
// }

// fn check_if_instruction_uses_rs2(opcode: u8) -> bool {
//     match opcode {
//         0x33 => true, // R-Type
//         0x23 => true, // S-Type (Store)
//         0x63 => true, // B-Type (Branch)
//         _ => false,
//     }
// }
