use rv32i_sim::bus::Bus;
use rv32i_sim::cpu::*;
use rv32i_sim::memory::Dram;

const RAM_SIZE: usize = 16 * 1024 * 1024; // 16MB

fn setup_cpu(program: &[u32]) -> Cpu {
    println!("CPU 및 16MB DRAM 초기화 중...");
    let mut dram = Dram::new(RAM_SIZE);

    for (i, &inst) in program.iter().enumerate() {
        let bytes = inst.to_le_bytes();
        let addr = i * 4;
        dram.dram[addr] = bytes[0];
        dram.dram[addr + 1] = bytes[1];
        dram.dram[addr + 2] = bytes[2];
        dram.dram[addr + 3] = bytes[3];
    }

    let bus = Bus { dram };
    let cpu = Cpu::new(bus);

    // // 스택 포인터(sp = x2) 초기화
    // cpu.regs.write(2, STACK_TOP, true);
    cpu
}

fn single_cycle_step(cpu: &mut Cpu) {
    cpu.pipeline_step(false); // NOP 주입 없이 파이프라인 단계 진행

    for _ in 0..4 {
        cpu.pipeline_step(true); // NOP 주입하여 파이프라인 단계 진행
    }
}

// fn instruction_test_reference() {
//     // R-Type 명령어 테스트를 위한 간단한 프로그램
//     let program = vec![
//         // === [초기화] I-Type으로 테스트 피연산자 설정 ===
//         0x00a00093, // ADDI x1, x0, 10    (x1 = 10)
//         0xffc00113, // ADDI x2, x0, -4    (x2 = -4 / 0xFFFFFFFC)
//         0x00200193, // ADDI x3, x0, 2     (x3 = 2)
//     ];

//     let mut cpu = setup_cpu(&program);

//     single_cycle_step(&mut cpu);
//     assert_eq!(cpu.regs.read(1), 10); // x1 = 10

//     single_cycle_step(&mut cpu);
//     assert_eq!(cpu.regs.read(2) as i32, -4); // x2 = -4 (0xFFFFFFFC)

//     single_cycle_step(&mut cpu);
//     assert_eq!(cpu.regs.read(3), 2); // x3 = 2
// }

#[test]
fn r_type_instruction_test() {
    // R-Type 명령어 테스트를 위한 간단한 프로그램
    let program = vec![
        // === [초기화] I-Type으로 테스트 피연산자 설정 ===
        0x00a00093, // ADDI x1, x0, 10    (x1 = 10)
        0xffc00113, // ADDI x2, x0, -4    (x2 = -4 / 0xFFFFFFFC)
        0x00200193, // ADDI x3, x0, 2     (x3 = 2)
        // === [RV32I R-Type (10종)] ===
        0x00208233, // ADD   x4,  x1, x2  (10 + -4 = 6)
        0x403082b3, // SUB   x5,  x1, x3  (10 - 2 = 8)
        0x00309333, // SLL   x6,  x1, x3  (10 << 2 = 40)
        0x001123b3, // SLT   x7,  x2, x1  (-4 < 10 signed -> 1)
        0x00113433, // SLTU  x8,  x2, x1  (0xFFFFFFFC < 10 unsigned -> 0)
        0x0030c4b3, // XOR   x9,  x1, x3  (10 ^ 2 = 8)
        0x00315533, // SRL   x10, x2, x3  (0xFFFFFFFC >> 2 logical = 0x3FFFFFFE)
        0x403155b3, // SRA   x11, x2, x3  (-4 >> 2 arithmetic = -1 / 0xFFFFFFFF)
        0x0030e633, // OR    x12, x1, x3  (10 | 2 = 10)
        0x0030f6b3, // AND   x13, x1, x3  (10 & 2 = 2)
        // === [RV32M R-Type (8종)] ===
        0x02208733, // MUL    x14, x1, x2  (10 * -4 = -40 / 0xFFFFFFD8)
        0x022097b3, // MULH   x15, x1, x2  (High 32bit of signed 10 * -4 = 0xFFFFFFFF)
        0x0220a833, // MULHSU x16, x1, x2  (High 32bit of signed 10 * unsigned 0xFFFFFFFC = 0x00000009)
        0x0220b8b3, // MULHU  x17, x1, x2  (High 32bit of unsigned 10 * unsigned 0xFFFFFFFC = 0x00000009)
        0x02314933, // DIV    x18, x2, x3  (-4 / 2 signed = -2 / 0xFFFFFFFE)
        0x023159b3, // DIVU   x19, x2, x3  (0xFFFFFFFC / 2 unsigned = 0x7FEFFFFE)
        0x0230ea33, // REM    x20, x1, x3  (10 % 2 signed = 0)
        0x02317ab3, // REMU   x21, x2, x3  (0xFFFFFFFC % 2 unsigned = 0)
    ];

    let mut cpu = setup_cpu(&program);

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(1), 10); // x1 = 10

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(2) as i32, -4); // x2 = -4 (0xFFFFFFFC)

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(3), 2); // x3 = 2

    // --- [RV32I R-Type (10종)] ---
    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(4), 6); // ADD: x1 + x2 = 10 + (-4) = 6

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(5), 8); // SUB: x1 - x3 = 10 - 2 = 8

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(6), 40); // SLL: x1 << x3 = 10 << 2 = 40 (0x28)

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(7), 1); // SLT: x2 < x1 (signed) -> -4 < 10 -> 1

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(8), 0); // SLTU: x2 < x1 (unsigned) -> 0xFFFFFFFC < 10 -> 0

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(9), 8); // XOR: x1 ^ x3 = 10 ^ 2 = 8

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(10), 0x3FFFFFFF); // SRL: x2 >> x3 (logical) -> 0xFFFFFFFC >> 2

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(11), 0xFFFFFFFF); // SRA: x2 >> x3 (arithmetic) -> -4 >> 2 = -1

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(12), 10); // OR: x1 | x3 = 10 | 2 = 10

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(13), 2); // AND: x1 & x3 = 10 & 2 = 2

    // --- [RV32M R-Type (8종)] ---
    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(14), 0xFFFFFFD8); // MUL: x1 * x2 = 10 * (-4) = -40 (0xFFFFFFD8)

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(15), 0xFFFFFFFF); // MULH: High 32-bit of signed (10 * -4) = -1

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(16), 9); // MULHSU: High 32-bit of signed 10 * unsigned 0xFFFFFFFC = 9

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(17), 9); // MULHU: High 32-bit of unsigned 10 * unsigned 0xFFFFFFFC = 9

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(18), 0xFFFFFFFE); // DIV: x2 / x3 (signed) = -4 / 2 = -2 (0xFFFFFFFE)

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(19), 0x7FFFFFFE); // DIVU: x2 / x3 (unsigned) = 0xFFFFFFFC / 2 = 0x7FFFFFFE

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(20), 0); // REM: x1 % x3 (signed) = 10 % 2 = 0

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(21), 0); // REMU: x2 % x3 (unsigned) = 0xFFFFFFFC % 2 = 0
}

#[test]
fn i_type_instruction_test() {
    // I-Type 명령어 테스트를 위한 프로그램 (기계어 바이너리)
    let program = vec![
        // === [1. 산술 및 논리 연산 (ALU Immediate)] ===
        0x00a00093, // ADDI  x1,  x0, 10       (x1 = 0 + 10 = 10)
        0xffc08113, // ADDI  x2,  x1, -4       (x2 = 10 + -4 = 6)
        0x0011a193, // SLTI  x3,  x3, 1        (x3 = 0 < 1 signed -> 1)
        0xfff13213, // SLTIU x4,  x2, -1       (x4 = 6 < 0xFFFFFFFF unsigned -> 1)
        0x0032c293, // XORI  x5,  x5, 3        (x5 = 0 ^ 3 = 3)
        0x00f16313, // ORI   x6,  x2, 15       (x6 = 6 | 15 = 15)
        0x00337393, // ANDI  x7,  x6, 3        (x7 = 15 & 3 = 3)
        0x00211413, // SLLI  x8,  x2, 2        (x8 = 6 << 2 = 24)
        0x00115493, // SRLI  x9,  x2, 1        (x9 = 6 >> 1 logical = 3)
        0x4010d513, // SRAI  x10, x2, 1        (x2를 -4로 재설정 후 테스트)
        // === [2. 메모리 로드 연산 (Load Instructions)] ===
        // (미리 메모리 주소 0x100에 데이터 0x87654321을 저장해두었다고 가정)
        0x10000593, // ADDI  x11, x0, 0x100    (x11 = 0x100, Base Address)
        0x00058603, // LB    x12, 0(x11)       (Byte: 0x21 -> 0x00000021)
        0x00158683, // LBU   x13, 1(x11)       (Byte: 0x43 -> 0x00000043)
        0x00059703, // LH    x14, 0(x11)       (Halfword: 0x4321 -> 0x00004321)
        0x0005d783, // LHU   x15, 0(x11)       (Sign-extension 테스트용)
        0x0005a803, // LW    x16, 0(x11)       (Word: 0x87654321)
        // === [3. 점프 연산 (JALR)] ===
        0x00800893, // ADDI  x17, x0, 8        (Target Base = 8)
        0x00488967, // JALR  x18, 4(x17)       (x18 = Next PC, PC = (8 + 4) & ~1 = 12)
    ];

    let mut cpu = setup_cpu(&program);

    // --- [1. ALU Immediate 연산 검증] ---
    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(1), 10); // ADDI: x1 = 10

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(2), 6); // ADDI: x2 = 10 + (-4) = 6

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(3), 1); // SLTI: 0 < 1 -> 1

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(4), 1); // SLTIU: 6 < 0xFFFFFFFF (unsigned) -> 1

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(5), 3); // XORI: 0 ^ 3 = 3

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(6), 15); // ORI: 6 | 15 = 15

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(7), 3); // ANDI: 15 & 3 = 3

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(8), 24); // SLLI: 6 << 2 = 24

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(9), 3); // SRLI: 6 >> 1 logical = 3

    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(9), 3); // SRLI: 6 >> 1 = 3

    // --- [2. Memory Load 연산 검증] ---
    // 테스트용 데이터 미리 메모리에 주입 (Little-Endian: 0x87654321)
    let _ = cpu.bus.store(0x100, 0x2, 0x87654321);

    single_cycle_step(&mut cpu); // ADDI x11 (주소 설정)
    assert_eq!(cpu.regs.read(11), 0x100);

    single_cycle_step(&mut cpu); // LB
    assert_eq!(cpu.regs.read(12), 0x21);

    single_cycle_step(&mut cpu); // LBU
    assert_eq!(cpu.regs.read(13), 0x43);

    single_cycle_step(&mut cpu); // LH
    assert_eq!(cpu.regs.read(14), 0x4321);

    // 음수 부호 확장(Sign-Extension) 테스트를 위한 데이터 배치
    let _ = cpu.bus.store(0x100, 0x1, 0x0080);

    single_cycle_step(&mut cpu); // LHU (Zero-Extension)
    assert_eq!(cpu.regs.read(15), 0x00000080);

    let _ = cpu.bus.store(0x100, 0x2, 0x87654321); // 원래 값 복구

    single_cycle_step(&mut cpu); // LW
    assert_eq!(cpu.regs.read(16), 0x87654321);

    // --- [3. JALR 연산 검증] ---
    let current_pc = cpu.pc;
    single_cycle_step(&mut cpu); // ADDI x17

    single_cycle_step(&mut cpu); // JALR
    assert_eq!(cpu.regs.read(18), current_pc + 8); // Return Address (Next PC) 저장 확인
    assert_eq!(cpu.pc, 12); // Target PC = (x17 + imm) & !1 = (8 + 4) = 12
}

#[test]
fn s_type_instruction_test() {
    // S-Type (Store) 명령어 테스트 프로그램
    let program = vec![
        // === [초기화] 주소 및 데이터 설정 ===
        0x10000093, // ADDI x1, x0, 256   (x1 = 0x100, 기본 메모리 주소)
        0x08000113, // ADDI x2, x0, 128   (x2 = 0x80 / 128)
        0xffc00193, // ADDI x3, x0, -4    (x3 = -4 / 0xFFFFFFFC)
        // === [S-Type Store 실행] ===
        0x00208023, // SB x2, 0(x1)       (Mem[0x100] = 0x80, 1 Byte Store)
        0x00309223, // SH x3, 4(x1)       (Mem[0x104] = 0xFFFC, 2 Byte Store)
        0x0020a823, // SW x2, 16(x1)      (Mem[0x110] = 0x00000080, 4 Byte Store)
    ];

    let mut cpu = setup_cpu(&program);

    // 1. 초기화 단계 실행 (3 cycle)
    single_cycle_step(&mut cpu); // x1 = 0x100
    single_cycle_step(&mut cpu); // x2 = 128
    single_cycle_step(&mut cpu); // x3 = -4

    assert_eq!(cpu.regs.read(1), 0x100);
    assert_eq!(cpu.regs.read(2), 128);
    assert_eq!(cpu.regs.read(3) as i32, -4);

    // 2. SB (Store Byte) 검증
    single_cycle_step(&mut cpu);
    // 0x100 주소에서 1바이트 읽기 (0x80)
    let byte_val = cpu.bus.load(0x100, 0x0).unwrap() as u8;
    assert_eq!(byte_val, 0x80);

    // 3. SH (Store Halfword) 검증
    single_cycle_step(&mut cpu);
    // 0x104 주소에서 2바이트(Halfword) 읽기 (0xFFFC)
    let half_val = cpu.bus.load(0x104, 0x1).unwrap() as u16;
    assert_eq!(half_val, 0xFFFC);

    // 4. SW (Store Word) 검증
    single_cycle_step(&mut cpu);
    // 0x110 주소에서 4바이트(Word) 읽기 (128)
    let word_val = cpu.bus.load(0x110, 0x2).unwrap();
    assert_eq!(word_val, 128);
}

#[test]
fn b_type_instruction_test() {
    // B-Type (Branch) 명령어 테스트 프로그램
    let program = vec![
        // PC: 0x00 | 초기화
        0x00500093, // ADDI x1, x0, 5    (x1 = 5)
        0x00500113, // ADDI x2, x0, 5    (x2 = 5)
        0x00a00193, // ADDI x3, x0, 10   (x3 = 10)
        // PC: 0x0C | BEQ x1, x2, +8 (Target: 0x14) -> Equal이므로 분기 성공!
        0x00208463, // BEQ x1, x2, 8
        // PC: 0x10 | Branch Taken 시 건너뛰어야 하는 트랩 명령어
        0x00100213, // ADDI x4, x0, 1    (실행되면 안 됨!)
        // PC: 0x14 | 분기 성공 지점 (Target PC)
        0x00100293, // ADDI x5, x0, 1    (x5 = 1)
        // PC: 0x18 | BLT x3, x1, +8 (Target: 0x20) -> 10 < 5 조건 거짓이므로 분기 실패!
        0x0011c463, // BLT x3, x1, 8     (Branch Not Taken)
        // PC: 0x1C | 분기 실패 시 정상적으로 다음 라인 실행
        0x00200313, // ADDI x6, x0, 2    (x6 = 2)
    ];

    let mut cpu = setup_cpu(&program);

    // 1. 레지스터 초기화 (3 cycle)
    single_cycle_step(&mut cpu); // PC: 0x04, x1 = 5
    single_cycle_step(&mut cpu); // PC: 0x08, x2 = 5
    single_cycle_step(&mut cpu); // PC: 0x0C, x3 = 10

    // 2. BEQ 실행 (Branch Taken)
    single_cycle_step(&mut cpu);

    // BEQ 조건 성립으로 0x10(ADDI x4)을 건너뛰고 Target PC(0x14)로 이동했는지 검증
    assert_eq!(cpu.pc, 0x14);
    assert_eq!(cpu.regs.read(4), 0); // x4는 실행되지 않아 0이어야 함

    // 3. Target 주소(0x14) 명령어 실행
    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(5), 1);
    assert_eq!(cpu.pc, 0x18);

    // 4. BLT 실행 (Branch Not Taken)
    single_cycle_step(&mut cpu);
    // 조건 불성립이므로 PC가 0x20으로 점프하지 않고 PC + 4 (0x1C)로 진행해야 함
    assert_eq!(cpu.pc, 0x1C);

    // 5. Fall-through 명령어(0x1C) 정상 실행 검증
    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(6), 2);
}

#[test]
fn u_type_instruction_test() {
    // U-Type 명령어 테스트 (LUI, AUIPC)
    // 메모리 시작 주소 / Initial PC = 0x0000_0000 가정
    let program = vec![
        // 0x00: LUI x1, 0x12345        -> x1 = 0x1234_5000
        0x123450b7,
        // 0x04: AUIPC x2, 0x00002      -> x2 = PC(0x0004) + 0x0000_2000 = 0x0000_2004
        0x00002117,
        // 0x08: ADDI x3, x1, 0x678     -> x3 = 0x1234_5000 + 0x678 = 0x1234_5678 (32비트 상합)
        0x67808193,
    ];

    let mut cpu = setup_cpu(&program);

    // 1. LUI (Load Upper Immediate) 테스트
    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(1), 0x1234_5000);

    // 2. AUIPC (Add Upper Immediate to PC) 테스트
    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(2), 0x0000_2004);

    // 3. LUI + ADDI 조합으로 32비트 풀 상수가 잘 만들어지는지 테스트
    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(3), 0x1234_5678);
}

#[test]
fn j_type_instruction_test() {
    // J-Type (JAL) 및 JALR 명령어 테스트
    // 메모리 시작 주소 / Initial PC = 0x0000_0000 가정
    let program = vec![
        // 0x00: JAL x1, 8 (offset +8 -> 0x08로 점프)
        //       x1 = return address (PC + 4 = 0x04)
        0x008000ef,
        // 0x04: ADDI x2, x0, 99 (점프 성공 시 건너뛰어야 함)
        0x06300113, // 0x08: ADDI x3, x0, 10 (점프 타겟: x3 = 10)
        0x00a00193,
        // 0x0C: JALR x4, 0(x1) (x1=0x04로 복귀 및 점프)
        //       x4 = return address (PC + 4 = 0x10)
        0x00008267,
    ];

    let mut cpu = setup_cpu(&program);

    // 1. JAL 실행 (0x00 -> 0x08로 오프셋 점프)
    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(1), 0x0000_0004); // x1에 복귀 주소 PC+4 저장 확인
    assert_eq!(cpu.pc, 0x0000_0008); // PC가 0x08로 점프했는지 확인

    // 2. 점프 타겟 명령 실행 (0x08)
    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(3), 10); // x3 = 10
    assert_eq!(cpu.regs.read(2), 0); // 건너뛴 0x04의 x2는 여전히 0이어야 함

    // 3. JALR 실행 (x1 레지스터 주소 기반 점프)
    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(4), 0x0000_0010); // x4에 복귀 주소 PC+4(0x10) 저장 확인
    assert_eq!(cpu.pc, 0x0000_0004); // PC가 x1(0x04) 위치로 복귀했는지 확인

    // 4. 건너뛰었던 0x04 위치의 명령 실행
    single_cycle_step(&mut cpu);
    assert_eq!(cpu.regs.read(2), 99); // x2 = 99 정상 실행 확인
}
