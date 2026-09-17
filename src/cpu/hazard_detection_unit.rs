/*
    memory hazard 해결을 위한 hazard detection unit과 control hazard 해결을 위한 forwarding unit 구현
    - hazard detection unit은 EX 단계에서 발생하는 memory hazard를 해결하기 위해 사용됨
    - EX 단계에서 load 명령어가 수행될 때, 다음 명령어가 load 명령어의 결과를 필요로 하는 경우, pipeline을 stall 시켜야 함
*/

use crate::cpu::{
    decoder::*,
    register::{IdExRegister, MemWbRegister},
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
        mem_reg: &MemWbRegister,
        wb_reg: &MemWbRegister,
    ) -> (ForwardA, ForwardB) {
        let mut forward_a = ForwardA::NoForward;
        let mut forward_b = ForwardB::NoForward;

        // MEM Stage Forwarding
        if mem_reg.control.reg_write && (mem_reg.rd != 0) && (mem_reg.rd == id_ex_reg.rs1) {
            forward_a = ForwardA::ForwardFromMem;
        }
        if mem_reg.control.reg_write && (mem_reg.rd != 0) && (mem_reg.rd == id_ex_reg.rs2) {
            forward_b = ForwardB::ForwardFromMem;
        }

        // WB forwarding has lower priority than MEM forwarding.
        if forward_a == ForwardA::NoForward
            && wb_reg.control.reg_write
            && (wb_reg.rd != 0)
            && (wb_reg.rd == id_ex_reg.rs1)
        {
            forward_a = ForwardA::ForwardFromWb;
        }
        if forward_b == ForwardB::NoForward
            && wb_reg.control.reg_write
            && (wb_reg.rd != 0)
            && (wb_reg.rd == id_ex_reg.rs2)
        {
            forward_b = ForwardB::ForwardFromWb;
        }

        (forward_a, forward_b)
    }
}

pub struct HazardDetectionUnit;

impl HazardDetectionUnit {
    // load-use hazard detection
    // EX 단계에서 load 명령어가 수행될 때, 다음 명령어가 load 명령어의 결과를 필요로 하는 경우, pipeline을 stall 시켜야 함
    pub fn check_load_use(id_ex_mem_read: bool, id_ex_rd: u8, if_id_instruction: u32) -> bool {
        // EX 단계에서 load 명령어가 아니거나 쓰기 대상 레지스터가 x0이면 hazard 없음
        if !id_ex_mem_read || id_ex_rd == 0 {
            return false;
        }

        // IF 단계에서 명령어를 디코딩하여 rs1, rs2 레지스터를 확인
        let (_, rs2, rs1, _, _, _) = Decoder::decode(if_id_instruction);

        // load-use hazard 발생 여부를 판단
        (rs1 == id_ex_rd) || (rs2 == id_ex_rd)
    }
}
