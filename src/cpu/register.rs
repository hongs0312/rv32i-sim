use crate::cpu::control::ControlSignals;

pub struct RegisterFile {
    regs: [u32; 32],
}

impl RegisterFile {
    pub fn new() -> Self {
        Self { regs: [0; 32] }
    }

    pub fn read(&self, reg1: u32, reg2: u32) -> (u32, u32) {
        let val1 = if reg1 == 0 { 0 } else { self.regs[reg1 as usize] };
        let val2 = if reg2 == 0 { 0 } else { self.regs[reg2 as usize] };
        (val1, val2)
    }

    pub fn write(&mut self, reg: u32, value: u32, write_enable: bool) {
        // x0 레지스터는 항상 0이어야 하므로, x0에 쓰기를 시도하면 무시
        if reg != 0 && write_enable {
            self.regs[reg as usize] = value;
        }
    }
}

// pipline register structures
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct IfIdRegister {
    pub pc: u32,
    pub instruction: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct ExMemRegister {
    pub control: ControlSignals,

    pub zero: bool,
    pub alu_result: u32,

    pub pc: u32,
    
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

pub fn imm_gen(inst: u32) -> i32 {
    let opcode = inst & 0x7f; // 0~6 bits
    match opcode {
        0x03 | 0x13 | 0x67 => { // I-Type
            let imm = (inst as i32) >> 20; // Sign-extend the immediate
            imm
        }
        0x23 => { // S-Type
            let imm_11_5 = (inst >> 25) & 0x7f; // bits 25~31
            let imm_4_0 = (inst >> 7) & 0x1f;   // bits 7~11
            let imm = ((imm_11_5 << 5) | imm_4_0) as i32;
            (imm << 20) >> 20 // Sign-extend to 32 bits
        }
        0x63 => { // B-Type
            let imm_12 = (inst >> 31) & 0x1;    // bit 31
            let imm_10_5 = (inst >> 25) & 0x3f; // bits 25~30
            let imm_4_1 = (inst >> 8) & 0xf;    // bits 8~11
            let imm_11 = (inst >> 7) & 0x1;     // bit 7
            let imm = ((imm_12 << 12)
                | (imm_11 << 11)
                | (imm_10_5 << 5)
                | (imm_4_1 << 1)) as i32;
            (imm << 19) >> 19 // Sign-extend to 32 bits
        }
        0x37 | 0x17 => { // U-Type
            let imm = (inst & 0xfffff000) as i32; // 상위 20비트만 사용, 하위 12비트는 0으로 채움
            imm
        }
        0x6f => { // J-Type
            let imm_20 = (inst >> 31) & 0x1;      // bit 31
            let imm_10_1 = (inst >> 21) & 0x3ff;  // bits 21~30
            let imm_11 = (inst >> 20) & 0x1;      // bit 20
            let imm_19_12 = (inst >> 12) & 0xff;  // bits 12~19
            let imm = ((imm_20 << 20)
                | (imm_19_12 << 12)
                | (imm_11 << 11)
                | (imm_10_1 << 1)) as i32;
            (imm << 11) >> 11 // Sign-extend to 32 bits
        }

        _ => 0, // 기본값: 0 (알 수 없는 명령어에 대해)
    }
} 