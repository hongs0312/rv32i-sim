#[cfg(test)]
mod tests {
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

        let bus = Bus::new(dram);
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

    #[test]
    fn test_single_cycle_execution() {
        let instructions = vec![
            0x00a00093, // [0x00] addi x1, x0, 10
            0x01400113, // [0x04] addi x2, x0, 20
            0x002081b3, // [0x08] add  x3, x1, x2   (Forwarding -> x3 = 30)
            0x40118233, // [0x0C] sub  x4, x3, x1   (Forwarding -> x4 = 20)
            0x00402023, // [0x10] sw   x4, 0(x0)
            0x00002283, // [0x14] lw   x5, 0(x0)
            0x00128333, // [0x18] add  x6, x5, x1   (Load-Use Stall -> x6 = 30)
            0x00108663, // [0x1C] beq  x1, x1, 12   (Target = 0x1C + 12 = 0x28)
            0x3e700393, // [0x20] addi x7, x0, 999
            0x06400413, // [0x24] addi x8, x0, 100
            0x06400413, // [0x28] addi x8, x0, 100
        ];

        let mut cpu = setup_cpu(&instructions); // 초기화만 하고 명령어는 나중에 로드

        for _ in 0..40 {
            single_cycle_step(&mut cpu);
        }

        assert_eq!(cpu.regs.read(1), 10);
        assert_eq!(cpu.regs.read(2), 20);
        assert_eq!(cpu.regs.read(3), 30);
        assert_eq!(cpu.regs.read(4), 20);
        assert_eq!(cpu.regs.read(5), 20);
        assert_eq!(cpu.regs.read(6), 30);
        assert_eq!(cpu.regs.read(7), 0);
        assert_eq!(cpu.regs.read(8), 100);
    }

    #[test]
    fn test_pipeline_trap() {
        let instructions = vec![
            0x00a00093, // [0x00] addi x1, x0, 10
            // 💡 1. 일부러 x1을 Destination으로 쓰는 무의미한 연산 주입 (포워딩 오염 테스트)
            0x00000013, // NOP (addi x0, x0, 0)
            0x00108133, // add x2, x1, x1  (x2 = 20)
            // 💡 2. JALR 연산 (PC 및 rs1 데이터 계산 검증)
            0x00000013, // NOP
            0x008000ef, // jal x1, 8       (x1 = 0x14, Target = 0x18)
            0x3e700393, // [0x14] addi x7, x0, 999 (실행 안 되어야 함!)
            0x06400413, // [0x18] addi x8, x0, 100
        ];

        let mut cpu = setup_cpu(&instructions); // 초기화만 하고 명령어는 나중에 로드

        // ... 실행 및 검증
        for _ in 0..40 {
            single_cycle_step(&mut cpu);
        }

        assert_eq!(cpu.regs.read(1), 0x14); // JALR로 인해 x1에 Target 주소 저장
        assert_eq!(cpu.regs.read(2), 20); // x2 = 20 (x1 + x1)
        assert_eq!(cpu.regs.read(7), 0); // x7 = 0 (Flush되어 실행 안 됨)
        assert_eq!(cpu.regs.read(8), 100); // x8 = 100 (Branch Target)
    }

    #[test]
    fn nop_test() {
        let instructions = vec![
            0x00000013, // NOP (addi x0, x0, 0)
            0x00a00093, 0x00108133, // add x2, x1, x1  (x2 = 20)
            0x008000ef, // jal x1, 8       (x1 = 0x14, Target = 0x18)
        ];

        let mut cpu = setup_cpu(&instructions); // 초기화만 하고 명령어는 나중에 로드

        // ... 실행 및 검증
        for _ in 0..20 {
            single_cycle_step(&mut cpu);
        }
    }
}
