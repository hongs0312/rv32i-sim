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

// cpu를 이루는 모듈
use crate::cpu::alu::*;
use crate::cpu::branch::*;
use crate::cpu::control::*;
use crate::cpu::decoder::*;
use crate::cpu::register::*;

// Hazard Detection Unit 모듈
use crate::cpu::hazard_detection_unit::*;

// Memory Bus 모듈
use crate::bus::Bus;

pub struct Cpu {
    pub pc: u32,
    pub regs: RegisterFile,
    pub alu: Alu,

    /// CPU가 접근할 수 있는 메모리 인터페이스
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

    // CPU의 한 사이클을 수행하는 메서드
    // 파이프라이닝을 구현하기 위해서 역순으로 각 단계의 레지스터를 업데이트
    pub fn pipeline_step(&mut self, inject_nop: bool) {
        // 1. WB 및 MEM 단계 실행 (역순)
        self.write_back(self.mem_wb_reg.clone());
        let next_mem_wb_reg = self.memory_access(self.ex_mem_reg.clone());

        // 2. EX 단계 실행
        let next_ex_mem_reg = self.execute(self.id_ex_reg.clone());

        // 3. EX 단계의 연산 결과(next_ex_mem_reg)를 기반으로 Branch/Jump 판정
        let branch_taken = next_ex_mem_reg.control.branch
            && get_branch_condition(
                next_ex_mem_reg.alu_result,
                next_ex_mem_reg.zero,
                next_ex_mem_reg.control.funct3,
            );
        let pcsrc = next_ex_mem_reg.control.jump || branch_taken;
        let branch_target = next_ex_mem_reg.target_pc;

        // 4. Load-Use Hazard Detection
        let is_stall = HazardDetectionUnit::check_load_use(
            self.id_ex_reg.control.mem_read,
            self.id_ex_reg.rd,
            self.if_id_reg.instruction,
        );

        let next_id_ex_reg = self.instruction_decode(self.if_id_reg.clone());
        let next_if_id_reg = self.instruction_fetch(inject_nop);

        // 5. 제어 흐름 업데이트 (Flush > Stall > Normal)
        if pcsrc {
            // Branch/Jump Taken: Target PC로 업데이트 후 선행 파이프라인(IF/ID, ID/EX) Flush
            // self.update_pc(branch_target, true, inject_nop);
            self.pc = branch_target;

            self.if_id_reg = IfIdRegister::default();
            self.id_ex_reg = IdExRegister::default();

            // next_ex_mem_reg(점프/분기 명령어 본체)는 WB까지 전진하여 레지스터 쓰기 수행
            self.ex_mem_reg = next_ex_mem_reg;
        } else if is_stall {
            // Load-Use Hazard: ID/EX에 NOP(Bubble) 삽입, IF/ID 및 PC는 동결
            self.id_ex_reg = IdExRegister::default();
            self.ex_mem_reg = next_ex_mem_reg;
        } else {
            self.update_pc(branch_target, false, inject_nop);

            self.if_id_reg = next_if_id_reg;
            self.id_ex_reg = next_id_ex_reg;
            self.ex_mem_reg = next_ex_mem_reg;
        }

        // 6. MEM/WB 레지스터 업데이트
        self.mem_wb_reg = next_mem_wb_reg;
    }

    fn update_pc(&mut self, branch_target: u32, pcsrc: bool, inject_nop: bool) {
        if inject_nop {
            return;
        }

        let next_pc = self.pc.wrapping_add(4);
        self.pc = if pcsrc { branch_target } else { next_pc };
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

    fn execute(&mut self, id_ex_reg: IdExRegister) -> ExMemRegister {
        // 인자로 전달받은 id_ex_reg 참조
        let (forward_a, forward_b) =
            ForwardingUnit::get_forward_signals(&id_ex_reg, &self.ex_mem_reg, &self.mem_wb_reg);

        let (control, pc, rd, rs1_data, rs2_data, imm) = (
            id_ex_reg.control,
            id_ex_reg.pc,
            id_ex_reg.rd,
            id_ex_reg.rs1_data,
            id_ex_reg.rs2_data,
            id_ex_reg.imm,
        );
        let (funct3, funct7) = (control.funct3, control.funct7);

        let rs1_data_forwarded = match forward_a {
            ForwardA::NoForward => rs1_data,
            ForwardA::ForwardFromMem => self.ex_mem_reg.alu_result,
            ForwardA::ForwardFromWb => match self.mem_wb_reg.control.wb_src {
                true => self.mem_wb_reg.mem_data,
                false => self.mem_wb_reg.alu_result,
            },
        };

        let rs2_data_forwarded = match forward_b {
            ForwardB::NoForward => rs2_data,
            ForwardB::ForwardFromMem => self.ex_mem_reg.alu_result,
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

        let (raw_alu_result, zero) = self
            .alu
            .get_alu_result(a, b, control.alu_op, funct3, funct7);

        let target_pc = match control.is_jalr {
            true => raw_alu_result & !1,
            false => pc.wrapping_add(imm as u32),
        };

        let alu_result = match control.jump {
            true => pc.wrapping_add(4),
            false => raw_alu_result,
        };

        ExMemRegister {
            control,
            target_pc,
            zero,
            alu_result,
            rd,
            rs2_data: rs2_data_forwarded,
        }
    }

    fn memory_access(&mut self, ex_mem_reg: ExMemRegister) -> MemWbRegister {
        let (control, alu_result, rd, rs2_data) = (
            ex_mem_reg.control,
            ex_mem_reg.alu_result,
            ex_mem_reg.rd,
            ex_mem_reg.rs2_data,
        );
        let funct3 = control.funct3;

        let mem_data = if control.mem_read {
            self.bus
                .load(alu_result, funct3)
                .expect("Memory read failed")
        } else if control.mem_write {
            self.bus
                .store(alu_result, funct3, rs2_data)
                .expect("Memory write failed");
            0
        } else {
            0
        };

        MemWbRegister {
            control,
            alu_result,
            mem_data,
            rd,
        }
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
