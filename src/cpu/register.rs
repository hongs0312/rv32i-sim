use crate::cpu::control::ControlSignals;

pub struct RegisterFile {
    regs: [u32; 32],
}

impl RegisterFile {
    pub fn new() -> Self {
        Self { regs: [0; 32] }
    }

    pub fn read(&self, reg: u32) -> u32 {
        if reg == 0 { 0 } else { self.regs[reg as usize] }
    }

    pub fn write(&mut self, reg: u32, value: u32, write_enable: bool) {
        // x0 레지스터는 항상 0이어야 하므로, x0에 쓰기를 시도하면 무시
        if reg != 0 && write_enable {
            self.regs[reg as usize] = value;
        }
    }
}

// pipline register structures
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IfIdRegister {
    pub pc: u32,
    pub instruction: u32,
}
impl Default for IfIdRegister {
    fn default() -> Self {
        Self {
            pc: 0,
            instruction: 0x00000013, // RISC-V NOP (addi x0, x0, 0)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdExRegister {
    pub control: ControlSignals,

    pub pc: u32,

    pub rd: u32,
    pub rs1_data: u32,
    pub rs2_data: u32,

    pub funct3: u32,
    pub funct7: u32,

    pub imm: i32,
}
impl Default for IdExRegister {
    fn default() -> Self {
        Self {
            control: ControlSignals::default(),
            pc: 0,
            rd: 0,
            rs1_data: 0,
            rs2_data: 0,
            funct3: 0,
            funct7: 0,
            imm: 0,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct ExMemRegister {
    pub control: ControlSignals,

    pub target_pc: u32,

    pub zero: bool,
    pub alu_result: u32,

    pub rd: u32,
    pub rs2_data: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct MemWbRegister {
    pub control: ControlSignals,

    pub alu_result: u32,
    pub mem_data: u32,
    pub rd: u32,
}
