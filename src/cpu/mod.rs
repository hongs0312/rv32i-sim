pub mod alu;
pub mod control;
pub mod decoder;
pub mod register;

use crate::bus::Bus;

use crate::cpu::alu::*;
use crate::cpu::control::*;
use crate::cpu::decoder::*;
use crate::cpu::register::*;

pub struct Cpu {
    pub pc: u32,
    pub regs: RegisterFile,
    pub alu: Alu,
    pub bus: Bus,

    pub branch_taken: bool,

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

            branch_taken: false, // memory access 단계에서 branch가 taken되었는지 여부를 추적

            if_id_reg: IfIdRegister::default(),
            id_ex_reg: IdExRegister::default(),
            ex_mem_reg: ExMemRegister::default(),
            mem_wb_reg: MemWbRegister::default(),
        }
    }

    // CPU의 한 사이클을 수행하는 메서드
    // 파이프라이닝을 구현하기 위해서 역순으로 각 단계의 레지스터를 업데이트
    pub fn step(&mut self) {
        // 각 단계의 레지스터를 업데이트
        let next_if_id_reg = self.instruction_fetch();
        let next_id_ex_reg = self.instruction_decode(self.if_id_reg.clone());
        let next_ex_mem_reg = self.execute(self.id_ex_reg.clone());
        let next_mem_wb_reg = self.memory_access(self.ex_mem_reg.clone());
        self.write_back(self.mem_wb_reg.clone());

        if self.branch_taken {
            // Branch가 taken되었으면 IF/ID와 ID/EX 레지스터를 초기화
            self.if_id_reg = IfIdRegister::default();
            self.id_ex_reg = IdExRegister::default();
            self.ex_mem_reg = ExMemRegister::default();

            self.branch_taken = false; // Reset branch_taken flag
        } else {
            // Branch가 taken되지 않았으면 다음 단계의 레지스터로 업데이트
            self.if_id_reg = next_if_id_reg;
            self.id_ex_reg = next_id_ex_reg;
            self.ex_mem_reg = next_ex_mem_reg;
        }
        
        self.mem_wb_reg = next_mem_wb_reg;
    }

    fn instruction_fetch(&mut self) -> IfIdRegister {
        let cur_pc = self.pc;

        // Branch가 taken되지 않았으면 PC를 4 증가시켜 다음 명령어로 이동
        if !self.branch_taken {
            self.pc += 4;
        }

        let instruction = self.bus.load32(cur_pc).expect("Fetch failed");

        // 결과를 if_id_reg에 보내서 다음 단계에서 사용할 수 있도록 함
        IfIdRegister {
            pc: cur_pc,
            instruction: instruction,
        }
    }

    fn instruction_decode(&mut self, if_id_reg: IfIdRegister) -> IdExRegister {
        let (pc, instruction) = (if_id_reg.pc, if_id_reg.instruction);

        // 디코더를 사용해 instruction을 분해하여 각 필드를 추출
        let (funct7, rs2, rs1, funct3, rd, opcode) = Decoder::decode(instruction);

        // register file에서 rs1, rs2에 해당하는 값을 읽어옴
        let rs1_data = self.regs.read(rs1);
        let rs2_data = self.regs.read(rs2);

        let imm = imm_gen(instruction); // immediate 값 생성

        // Control Signals 생성
        let control = get_control_signals(opcode);

        // 결과를 id_ex_reg에 보내서 다음 단계에서 사용할 수 있도록 함
        IdExRegister {
            control,
            pc,
            rd,
            rs1_data,
            rs2_data,
            funct3,
            funct7,
            imm,
        }
    }

    fn execute(&mut self, id_ex_reg: IdExRegister) -> ExMemRegister {
        let (control, pc, rd, rs1_data, rs2_data, funct3, funct7, imm) = (
            id_ex_reg.control,
            id_ex_reg.pc,
            id_ex_reg.rd,
            id_ex_reg.rs1_data,
            id_ex_reg.rs2_data,
            id_ex_reg.funct3,
            id_ex_reg.funct7,
            id_ex_reg.imm,
        );

        // ALU 연산을 수행하기 위해 ALU 입력값 결정
        let a = rs1_data;
        let b = if control.alu_src {
            imm as u32 // immediate 값 사용
        } else {
            rs2_data // register 값 사용
        };

        let raw_alu_result = self.alu.get_alu_result(a, b, control.alu_op, funct3, funct7);

        let zero = raw_alu_result == 0; // ALU 결과가 0인지 여부를 판단
        
        // ALU 연산 수행
        let alu_result = match control.jal || control.jalr {
            true => pc.wrapping_add(4), // JAL/JALR 명령어의 경우, ALU 결과는 PC + 4
            false => raw_alu_result, // 그 외의 경우, ALU 결과 그대로 사용
        };

        // Branch target PC 계산
        let target_pc = match control.jalr {
            true => (rs1_data.wrapping_add(imm as u32)) & !1, // JALR 명령어의 경우, target PC를 rs1 + imm로 설정하고 하위 1비트를 0으로 설정
            false => pc.wrapping_add(imm as u32), // Branch 명령어의 경우, target PC를 pc + imm로 설정
        };

        // 결과를 ex_mem_reg에 보내서 다음 단계에서 사용할 수 있도록 함
        ExMemRegister {
            control,
            target_pc,
            zero,
            alu_result,
            rd,
            rs2_data,
        }
    }

    fn memory_access(&mut self, ex_mem_reg: ExMemRegister) -> MemWbRegister {
        let (control, target_pc, zero, alu_result, rd, rs2_data) = (
            ex_mem_reg.control,
            ex_mem_reg.target_pc,
            ex_mem_reg.zero,
            ex_mem_reg.alu_result,
            ex_mem_reg.rd,
            ex_mem_reg.rs2_data,
        );

        let is_jump_taken = (zero && control.branch) || control.jal || control.jalr;

        if is_jump_taken {
            self.pc = target_pc; // Branch가 taken되었으면 PC를 target PC로 설정
            self.branch_taken = true; // Branch가 taken되었음을 표시
        }

        let mem_data = if control.mem_read {
            // Load 명령어 처리
            self.bus.load32(alu_result).expect("Memory read failed")
        } else if control.mem_write {
            // Store 명령어 처리
            self.bus
                .store32(alu_result, rs2_data)
                .expect("Memory write failed");
            0 // Store 명령어는 읽은 값이 없으므로 0 반환
        } else {
            0 // 메모리 접근이 없는 경우 0 반환
        };

        // 결과를 mem_wb_reg에 보내서 다음 단계에서 사용할 수 있도록 함
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
            let write_data = match control.mem_to_reg {
                true => mem_data,    // 메모리 읽기 값 선택
                false => alu_result, // ALU 결과 선택
            };
            
            self.regs.write(rd, write_data, true);
        }
    }
}
