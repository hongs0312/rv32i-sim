/*
    CPU 모듈
    - CPU는 5단계 파이프라인 구조를 가지며, 각 단계는 별도의 레지스터를 사용하여 데이터를 전달함
*/

pub mod alu;
pub mod branch;
pub mod control;
pub mod decoder;
pub mod hazard_detection_unit;
pub mod register;

use crate::bus::Bus;
use crate::cpu::alu::*;
use crate::cpu::branch::*;
use crate::cpu::control::*;
use crate::cpu::decoder::*;
use crate::cpu::hazard_detection_unit::*;
use crate::cpu::register::*;

// 파이프라인 단계의 상태를 표현하는 Enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StageStatus<T> {
    Busy,
    Complete(T),
}

pub struct Cpu {
    pub pc: u32,
    pub regs: RegisterFile,
    pub alu: Alu,
    pub bus: Bus,

    pub if_id_reg: IfIdRegister,
    pub id_ex_reg: IdExRegister,
    pub ex_mem_reg: ExMemRegister,
    pub mem_wb_reg: MemWbRegister,
}

impl Cpu {
    pub fn new(bus: Bus) -> Self {
        Self {
            pc: 0,
            regs: RegisterFile::new(),
            alu: Alu::new(),
            bus,

            if_id_reg: IfIdRegister::default(),
            id_ex_reg: IdExRegister::default(),
            ex_mem_reg: ExMemRegister::default(),
            mem_wb_reg: MemWbRegister::default(),
        }
    }

    pub fn pipeline_step(&mut self, inject_nop: bool) {
        // 1. WB 및 MEM 단계 실행 (역순)
        self.write_back(self.mem_wb_reg);

        let mem_status = self.memory_access(self.ex_mem_reg);
        let is_mem_busy = matches!(mem_status, StageStatus::Busy);

        let next_mem_wb_reg = match mem_status {
            StageStatus::Complete(reg) => reg,
            StageStatus::Busy => MemWbRegister::default(),
        };

        // 3. EX 단계 실행
        let ex_status = match is_mem_busy {
            true => StageStatus::Busy,
            false => self.execute(self.id_ex_reg, &next_mem_wb_reg),
        };
        let is_ex_busy = matches!(ex_status, StageStatus::Busy);

        // EX 단계 래치 결과 및 분기 판정 제어 신호 추출
        let (next_ex_mem_reg, pcsrc, branch_target) = match ex_status {
            StageStatus::Complete(ex_reg) => {
                let branch_taken = ex_reg.control.branch
                    && get_branch_condition(ex_reg.alu_result, ex_reg.zero, ex_reg.control.funct3);

                let is_pcsrc = ex_reg.control.jump || branch_taken;

                (ex_reg, is_pcsrc, ex_reg.target_pc)
            }
            StageStatus::Busy => {
                (ExMemRegister::default(), false, 0)
            }
        };

        // 4. 앞단 실행 (ID, IF)
        let mut is_stall = HazardDetectionUnit::check_load_use(
            self.id_ex_reg.control.mem_read,
            self.id_ex_reg.rd,
            self.if_id_reg.instruction,
        );

        // 👇 [버그 픽스 1] Ghost Stall 무시: 어차피 점프(pcsrc)로 인해 버려질 명령어들이 만든 스톨은 무시합니다!
        if pcsrc {
            is_stall = false;
        }

        let next_id_ex_reg = self.instruction_decode(self.if_id_reg);
        let next_if_id_reg = self.instruction_fetch(inject_nop);

        let freeze_all = is_mem_busy || is_ex_busy || is_stall;
        let freeze_ex = is_mem_busy || is_ex_busy;

        // 5. 래치 업데이트
        if !freeze_all {
            self.update_pc(branch_target, pcsrc, inject_nop);

            self.if_id_reg = match pcsrc {
                true => IfIdRegister::default(),
                false => next_if_id_reg,
            };
        }

        if !freeze_ex {
            self.id_ex_reg = match pcsrc || is_stall {
                true => IdExRegister::default(),
                false => next_id_ex_reg,
            };
        }

        if !is_mem_busy {
            self.ex_mem_reg = match is_ex_busy {
                true => ExMemRegister::default(),
                false => next_ex_mem_reg,
            }
        }

        self.mem_wb_reg = match is_mem_busy {
            true => MemWbRegister::default(),
            false => next_mem_wb_reg,
        };
    }

    fn update_pc(&mut self, branch_target: u32, pcsrc: bool, inject_nop: bool) {
        if pcsrc {
            self.pc = branch_target;
        } else if !inject_nop {
            self.pc = self.pc.wrapping_add(4);
        }
    }

    fn instruction_fetch(&mut self, inject_nop: bool) -> IfIdRegister {
        let cur_pc = self.pc;

        let instruction = match inject_nop {
            true => 0x00000013, // NOP instruction
            false => self.bus.load32(self.pc).expect("Fetch failed"),
        };

        IfIdRegister {
            pc: cur_pc,
            instruction,
        }
    }

    fn instruction_decode(&mut self, if_id_reg: IfIdRegister) -> IdExRegister {
        let (pc, instruction) = (if_id_reg.pc, if_id_reg.instruction);
        let (funct7, rs2, rs1, funct3, rd, opcode) = Decoder::decode(instruction);

        let rs1_data = self.regs.read(rs1);
        let rs2_data = self.regs.read(rs2);
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

    fn execute(&mut self, id_ex_reg: IdExRegister, next_mem_wb_reg: &MemWbRegister) -> StageStatus<ExMemRegister> {
        let (control, pc, rd, imm) = (
            id_ex_reg.control,
            id_ex_reg.pc,
            id_ex_reg.rd,
            id_ex_reg.imm,
        );
        let (funct3, funct7) = (control.funct3, control.funct7);

        let rs1_data = self.regs.read(id_ex_reg.rs1);
        let rs2_data = self.regs.read(id_ex_reg.rs2);

        // 👇 [버그 픽스 2] 무조건 점프(JAL) 중 rd=x0 인 경우 무시되는 것을 막기 위해 !control.jump 조건 추가
        if !control.reg_write && !control.mem_write && !control.branch && !control.jump && !control.is_ecall {
            return StageStatus::Complete(ExMemRegister::default());
        }

        let (forward_a, forward_b) =
            ForwardingUnit::get_forward_signals(&id_ex_reg, next_mem_wb_reg, &self.mem_wb_reg);

        let rs1_data_forwarded = match forward_a {
            ForwardA::NoForward => rs1_data,
            ForwardA::ForwardFromMem => match next_mem_wb_reg.control.wb_src {
                true => next_mem_wb_reg.mem_data,
                false => next_mem_wb_reg.alu_result,
            },
            ForwardA::ForwardFromWb => match self.mem_wb_reg.control.wb_src {
                true => self.mem_wb_reg.mem_data,
                false => self.mem_wb_reg.alu_result,
            },
        };

        let rs2_data_forwarded = match forward_b {
            ForwardB::NoForward => rs2_data,
            ForwardB::ForwardFromMem => match next_mem_wb_reg.control.wb_src {
                true => next_mem_wb_reg.mem_data,
                false => next_mem_wb_reg.alu_result,
            },
            ForwardB::ForwardFromWb => match self.mem_wb_reg.control.wb_src {
                true => self.mem_wb_reg.mem_data,
                false => self.mem_wb_reg.alu_result,
            },
        };

        let a = match control.alu_src_a {
            true => pc,
            false => rs1_data_forwarded,
        };
        let b = match control.alu_src_b {
            true => imm as u32,
            false => rs2_data_forwarded,
        };

        let alu_status = self
            .alu
            .execute_with_cycles(a, b, control.alu_op, funct3, funct7);

        let (raw_alu_result, zero) = match alu_status {
            StageStatus::Busy => return StageStatus::Busy,
            StageStatus::Complete(res) => res,
        };

        let target_pc = match control.is_jalr {
            true => raw_alu_result & !1,
            false => pc.wrapping_add(imm as u32),
        };

        let alu_result = match control.jump {
            true => pc.wrapping_add(4),
            false => raw_alu_result,
        };

        let result_reg = ExMemRegister {
            control,
            target_pc,
            zero,
            alu_result,
            rd,
            rs2_data: rs2_data_forwarded,
        };
        StageStatus::Complete(result_reg)
    }

    fn memory_access(&mut self, ex_mem_reg: ExMemRegister) -> StageStatus<MemWbRegister> {
        let control = ex_mem_reg.control;
        let bus_status = self.bus.data_access(
            ex_mem_reg.alu_result,
            control.funct3,
            control.mem_read,
            control.mem_write,
            ex_mem_reg.rs2_data,
        );

        let mem_data_result = match bus_status {
            StageStatus::Busy => return StageStatus::Busy,
            StageStatus::Complete(res) => res.expect("Memory access failed"),
        };

        let result_reg = MemWbRegister {
            control,
            alu_result: ex_mem_reg.alu_result,
            mem_data: mem_data_result,
            rd: ex_mem_reg.rd,
        };
        StageStatus::Complete(result_reg)
    }

    fn write_back(&mut self, mem_wb_reg: MemWbRegister) {
        let (control, alu_result, mem_data, rd) = (
            mem_wb_reg.control,
            mem_wb_reg.alu_result,
            mem_wb_reg.mem_data,
            mem_wb_reg.rd,
        );

        if control.reg_write {
            let write_data = match control.wb_src {
                false => alu_result,
                true => mem_data,
            };
            self.regs.write(rd, write_data, true);
        }
    }
}