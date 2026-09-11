// 고유한 ALU 제어 신호 정의 (표준 매핑 기준 확장)
const ALU_AND: u8 = 0b0000; // 0
const ALU_OR: u8 = 0b0001; // 1
const ALU_ADD: u8 = 0b0010; // 2
const ALU_XOR: u8 = 0b0011; // 3
const ALU_SUB: u8 = 0b0110; // 6
const ALU_SLT: u8 = 0b0111; // 7
const ALU_SLTU: u8 = 0b1011; // 11 (임의 지정 - 중복 회피)
const ALU_SLL: u8 = 0b1000; // 8  (임의 지정)
const ALU_SRL: u8 = 0b0100; // 4  (임의 지정)
const ALU_SRA: u8 = 0b0101; // 5  (임의 지정)

// Branch 관련 ALU 제어 신호 정의
const ALU_BEQ: u8 = 0b1100; // 12 (Branch Equal)
const ALU_BNE: u8 = 0b1101; // 13 (Branch Not Equal)
const ALU_BLT: u8 = 0b1110; // 14 (Branch Less Than)
const ALU_BGE: u8 = 0b1111; // 15 (Branch Greater Than or Equal)

pub struct Alu;

impl Alu {
    pub fn new() -> Self {
        Alu {}
    }

    fn alu_control(&self, alu_op: u8, funct3: u32, funct7: u32) -> u8 {
        let inst30 = (funct7 >> 5) & 0x1; // funct7의 30번째 비트 추출
        match alu_op {
            0b00 => ALU_ADD, // 메모리참조 명령어
            0b01 => match funct3 {
                // Branch 명령어
                0x0 => ALU_BEQ, // BEQ
                0x1 => ALU_BNE, // BNE
                0x2 => ALU_BLT, // BLT
                0x3 => ALU_BGE, // BGE
                _ => panic!("Unsupported Branch instruction"),
            },
            0b10 => match (funct3, inst30) {
                (0x0, 0) => ALU_ADD, // ADD
                (0x0, 1) => ALU_SUB, // SUB
                (0x4, 0) => ALU_XOR, // XOR
                (0x6, 0) => ALU_OR,  // OR
                (0x7, 0) => ALU_AND, // AND

                (0x1, 0) => ALU_SLL,  // SLL
                (0x2, 0) => ALU_SLT,  // SLT
                (0x3, 0) => ALU_SLTU, // SLTU
                (0x5, 0) => ALU_SRL,  // SRL
                (0x5, 1) => ALU_SRA,  // SRA

                _ => panic!("Unsupported R-Type instruction"),
            },
            0b11 => 0b1111_0000, // LUI/AUIPC (임의 지정 - 실제로는 ALU 연산이 필요 없음)
            _ => panic!("Unsupported ALU Op"),
        }
    }

    #[rustfmt::skip] // match arms를 정렬하지 않음
    fn execute(&self, a: u32, b: u32, alu_control_signal: u8) -> u32 {
        let shamt = b & 0x1F; // RISC-V 시프트 량은 하위 5비트만 사용
        
        match alu_control_signal {
            // LUI/AUIPC: ALU 결과는 immediate 값 (b) 그대로 반환
            0b1111_0000 => b, 

            // 산술 및 논리 연산
            ALU_ADD => a.wrapping_add(b),
            ALU_SUB => a.wrapping_sub(b),
            ALU_XOR => a ^ b,
            ALU_OR => a | b,
            ALU_AND => a & b,

            // Shift 연산
            ALU_SLL => a << shamt,
            ALU_SRL => a >> shamt,
            ALU_SRA => ((a as i32) >> shamt) as u32, // 산술 시프트

            // 비교 연산
            ALU_SLT => if (a as i32) < (b as i32) { 1 } else { 0 },
            ALU_SLTU => if a < b { 1 } else { 0 },

            // Branch 연산
            ALU_BEQ => if a == b { 1 } else { 0 },
            ALU_BNE => if a != b { 1 } else { 0 },
            ALU_BLT => if (a as i32) < (b as i32) { 1 } else { 0 },
            ALU_BGE => if (a as i32) >= (b as i32) { 1 } else { 0 },
            
            _ => panic!("Unsupported ALU control signal"),
        }
    }

    pub fn get_alu_result(&self, a: u32, b: u32, alu_op: u8, funct3: u32, funct7: u32) -> u32 {
        let alu_control_signal = self.alu_control(alu_op, funct3, funct7);

        self.execute(a, b, alu_control_signal)
    }
}
