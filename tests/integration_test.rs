use rv32i_sim::bus::Bus;
use rv32i_sim::cpu::Cpu;
use rv32i_sim::memory::Dram;

/// 테스트용 헬퍼 함수: u32 리틀 엔디안 명령어 슬라이스를 DRAM에 메모리 바이트로로드
fn create_test_cpu(program: &[u32]) -> Cpu {
    let mut dram = Dram::new(1024); // 1KB 테스트 메모리

    // u32 명령어들을 리틀 엔디안 바이트 배열로 바꾸어 메모리에 저장
    for (i, &inst) in program.iter().enumerate() {
        let bytes = inst.to_le_bytes();
        let addr = i * 4;
        dram.dram[addr] = bytes[0];
        dram.dram[addr + 1] = bytes[1];
        dram.dram[addr + 2] = bytes[2];
        dram.dram[addr + 3] = bytes[3];
    }

    let bus = Bus { dram };
    Cpu::new(bus)
}

#[test]
fn test_addi_and_add_execution() {
    // -------------------------------------------------------------
    // 어셈블리 코드:
    // 0x00: addi x1, x0, 10   (x1 = 10)  -> 0x00a00093
    // 0x04: addi x2, x0, 20   (x2 = 20)  -> 0x01400093
    // 0x08: add  x3, x1, x2   (x3 = 30)  -> 0x002081b3
    // -------------------------------------------------------------
    let program = vec![
        0x00a00093, // addi x1, x0, 10
        0x01400113, // addi x2, x0, 20
        0x002081b3, // add  x3, x1, x2
    ];

    let mut cpu = create_test_cpu(&program);

    // 1클럭 실행: addi x1, x0, 10
    cpu.step();
    assert_eq!(cpu.regs.read(1), 10, "x1 레지스터는 10이어야 합니다.");
    assert_eq!(cpu.pc, 4);

    // 2클럭 실행: addi x2, x0, 20
    cpu.step();
    assert_eq!(cpu.regs.read(2), 20, "x2 레지스터는 20이어야 합니다.");
    assert_eq!(cpu.pc, 8);

    // 3클럭 실행: add x3, x1, x2
    cpu.step();
    assert_eq!(
        cpu.regs.read(3),
        30,
        "x3 레지스터는 30(10 + 20)이어야 합니다."
    );
    assert_eq!(cpu.pc, 12);
}

#[test]
fn test_x0_register_hardwired_zero() {
    // -------------------------------------------------------------
    // x0 레지스터에 값을 쓰려고 해도 항상 0으로 유지되는지 검증
    // 0x00: addi x0, x0, 999  (x0에 999 쓰기 시도) -> 0x3e700013
    // -------------------------------------------------------------
    let program = vec![
        0x3e700013, // addi x0, x0, 999
    ];

    let mut cpu = create_test_cpu(&program);

    cpu.step();

    // x0는 무조건 0이어야 함
    assert_eq!(
        cpu.regs.read(0),
        0,
        "x0 레지스터는 어떤 값을 써도 0을 유지해야 합니다."
    );
}

#[test]
fn test_negative_immediate_sign_extension() {
    // -------------------------------------------------------------
    // 음수 Immediate (Sign Extension) 검증
    // 0x00: addi x1, x0, -5   (x1 = -5) -> 0xffb00093
    // -------------------------------------------------------------
    let program = vec![
        0xffb00093, // addi x1, x0, -5
    ];

    let mut cpu = create_test_cpu(&program);

    cpu.step();

    // 2의 보수로 표현된 -5 (0xffff_fffb)
    assert_eq!(
        cpu.regs.read(1) as i32,
        -5,
        "음수 부호 확장이 정상 동작해야 합니다."
    );
}

#[test]
fn test_load_and_store_instructions() {
    // -------------------------------------------------------------
    // Load/Store 명령어 검증
    // 0x00: addi x1, x0, 42       (x1 = 42) -> 0x02a00093
    // 0x04: sw   x1, 0(x0)        (메모리[0] = 42) -> 0x00102023 (수정)
    // 0x08: lw   x2, 0(x0)        (x2 = 메모리[0]) -> 0x00002103 (수정)
    // -------------------------------------------------------------
    let program = vec![
        0x02a00093, // addi x1, x0, 42
        0x00102023, // sw   x1, 0(x0)
        0x00002103, // lw   x2, 0(x0)
    ];

    let mut cpu = create_test_cpu(&program);

    // 1클럭 실행: addi x1, x0, 42
    cpu.step();
    assert_eq!(cpu.regs.read(1), 42, "x1 레지스터는 42이어야 합니다.");
    assert_eq!(cpu.pc, 4);

    // 2클럭 실행: sw x1, 0(x0)
    cpu.step();
    let stored_value = cpu.bus.load32(0).expect("메모리 로드 실패");
    assert_eq!(
        stored_value, 42,
        "메모리[0]에는 42가 저장되어야 합니다."
    );
    assert_eq!(cpu.pc, 8);
    
    // 3클럭 실행: lw x2, 0(x0)
    cpu.step();
    assert_eq!(
        cpu.regs.read(2),
        42,
        "x2 레지스터는 메모리[0]에서 로드한 값 42이어야 합니다."
    );
    assert_eq!(cpu.pc, 12);
}