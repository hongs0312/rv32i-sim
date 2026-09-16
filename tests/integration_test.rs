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

#[test]
fn test_pipelined_cpu_hazards() {
    // 테스트용 기계어 프로그램 로드 (위의 어셈블리를 바이너리로 변환한 값)
    let instructions = vec![
        0x00a00093, // [0x00] addi x1, x0, 10
        0x01400113, // [0x04] addi x2, x0, 20
        0x002081b3, // [0x08] add  x3, x1, x2   (Forwarding -> x3 = 30)
        0x40118233, // [0x0C] sub  x4, x3, x1   (Forwarding -> x4 = 20, funct7=0x20 수정)
        0x00402023, // [0x10] sw   x4, 0(x0)
        0x00002283, // [0x14] lw   x5, 0(x0)
        0x00128333, // [0x18] add  x6, x5, x1   (Load-Use Stall -> x6 = 30)
        0x00108663, // [0x1C] beq  x1, x1, 12   (Target = 0x1C + 12 = 0x28, 오프셋 수정)
        0x3e700393, // [0x20] addi x7, x0, 999  (Flush되어 실행 안 됨!)
        0x06400413, // [0x24] addi x8, x0, 100  (Branch Target -> x8 = 100)
        0x06400413, // [0x28] addi x8, x0, 100  (Branch Target -> x8 = 100)
    ];

    let mut cpu = setup_cpu(&instructions); // 초기화만 하고 명령어는 나중에 로드

    // 충분한 사이클 동안 파이프라인 펌핑 (NOP 5개를 고려해 약 20 사이클)
    for cycle in 1..=20 {
        // 디버깅용 사이클 출력 (선택 사항)
        println!("--- Cycle {} ---", cycle);

        cpu.pipeline_step(false); // NOP 주입 없이 파이프라인 단계 진행

        println!("PC: {:#010x}", cpu.pc);
        println!("x1: {}, x2: {}", cpu.regs.read(1), cpu.regs.read(2));
        println!(
            "x3(ADD): {}, x4(SUB): {}",
            cpu.regs.read(3),
            cpu.regs.read(4)
        );
        println!(
            "x5(LW): {}, x6(ADD): {}",
            cpu.regs.read(5),
            cpu.regs.read(6)
        );
        println!(
            "x7(Flushed?): {}, x8(Target): {}",
            cpu.regs.read(7),
            cpu.regs.read(8)
        );
    }

    // --- 파이프라인 결과 Assert 검증 ---

    // 1. Forwarding 검증
    assert_eq!(cpu.regs.read(1), 10, "x1 calculation failed");
    assert_eq!(cpu.regs.read(2), 20, "x2 calculation failed");
    assert_eq!(
        cpu.regs.read(3),
        30,
        "EX-EX Forwarding failed: x3 should be 30"
    );
    assert_eq!(
        cpu.regs.read(4),
        20,
        "MEM-EX Forwarding failed: x4 should be 20"
    );

    // 2. Load-Use Hazard Stall 검증
    assert_eq!(cpu.regs.read(5), 20, "Load instruction failed");
    assert_eq!(
        cpu.regs.read(6),
        30,
        "Load-Use hazard stalling failed: x6 should be 30"
    );

    // 3. Control Hazard & Flush 검증
    assert_eq!(
        cpu.regs.read(7),
        0,
        "Control Hazard Flush failed! x7 should be 0 (Instruction was not flushed)"
    );
    assert_eq!(
        cpu.regs.read(8),
        100,
        "Branch Target execution failed: x8 should be 100"
    );
}
