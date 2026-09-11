#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct ControlSignals {
    // Execution 제어 신호
    pub alu_src: bool, // 1bit: ALU 두 번째 입력 선택
    pub alu_op: u8,    // 2-bit: Main Control이 ALU Control로 넘기는 힌트 신호 (00, 01, 10 등)
    pub jal: bool,        // 1bit: JAL 명령어 여부
    pub jalr: bool,       // 1bit: JALR 명령어 여부

    // Memory 제어 신호
    pub branch: bool,    // 1bit: Branch 명령어 여부
    pub mem_read: bool,  // 1bit: 메모리 읽기 활성화 (lw 등)
    pub mem_write: bool, // 1bit: 메모리 쓰기 활성화 (sw 등)

    // Write Back 제어 신호
    pub reg_write: bool,  // 1bit: 레지스터file 쓰기 활성화
    pub mem_to_reg: bool, // 1bit: register에 쓸 값 선택 (0: ALU 결과 vs 1: Memory 읽기 값)

    // System 제어 신호
    pub is_ecall: bool, // 1bit: ECALL 명령어 여부
}

// Control 신호를 opcode에 따라 설정하는 함수
pub fn get_control_signals(opcode: u32) -> ControlSignals {
    let mut control = ControlSignals::default();

    match opcode {
        0x00 => {} // NOP (No Operation)
        0x37 => {
            // U-Type (LUI)
            control.alu_src = true; // ALU 두 번째 입력은 immediate
            control.alu_op = 0b10; // ALU는 ADD 연산 수행
            control.reg_write = true; // 레지스터file 쓰기 활성화
        }
        0x17 => {
            // U-Type (AUIPC)
            control.alu_src = true; // ALU 두 번째 입력은 immediate
            control.alu_op = 0b10; // ALU는 ADD 연산 수행
            control.reg_write = true; // 레지스터file 쓰기 활성화
        }
        0x33 => {
            // R-Type
            control.alu_src = false;
            control.alu_op = 0b10;
            control.reg_write = true;
        }
        0x13 => {
            // I-Type (Immediate ALU)
            control.alu_src = true;
            control.alu_op = 0b10;
            control.reg_write = true;
        }
        0x03 => {
            // I-Type (Load)
            control.alu_src = true;
            control.alu_op = 0b00; // ALU는 주소 계산만 수행
            control.mem_read = true; // 메모리 읽기 활성화
            control.reg_write = true; // 레지스터file 쓰기 활성화
            control.mem_to_reg = true; // 메모리 읽기 값 선택
        }
        0x23 => {
            // S-Type (Store)
            control.alu_src = true; // ALU 두 번째 입력은 immediate
            control.alu_op = 0b00; // ALU는 주소 계산만 수행
            control.mem_write = true; // 메모리 쓰기 활성화
        }
        0x63 => {
            // B-Type (Branch)
            control.alu_src = false; // ALU 두 번째 입력은 register
            control.alu_op = 0b01; // ALU는 비교 연산 수행
            control.branch = true; // Branch 명령어 활성화
        }
        0x6F | 0x67 => {
            // J-Type (JAL, JALR)
            control.alu_src = true; // ALU 두 번째 입력은 immediate
            control.alu_op = 0b00; // ALU는 주소 계산만 수행
            control.reg_write = true; // 레지스터file 쓰기 활성화
            if opcode == 0x6F {
                control.jal = true; // JAL 명령어 활성화
            } else {
                control.jalr = true; // JALR 명령어 활성화
            }
        }
        0x73 => {
            // System (ECALL, EBREAK)
            control.is_ecall = true; // ECALL 명령어 활성화
        }

        _ => panic!("Unsupported opcode: {:#x}", opcode),
    }

    control
}
