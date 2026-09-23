// src/hardware/soc.rs
use crate::hardware::cpu::Cpu;
use crate::hardware::bus::{BusState, SystemBus};
use crate::hardware::memory::Dram;
use crate::hardware::systolic::SystolicArray;

pub struct SoC {
    pub cpu: Cpu,                 // 코어 (Bus를 소유하지 않음)
    pub systolic: SystolicArray,  // 가속기
    pub dram: Dram,               // 메인 메모리
    pub bus_state: BusState,      // 시스템 버스 중재 상태
    pub cycle: u64,
}

impl SoC {
    pub fn new(mem_size: usize) -> Self {
        Self {
            cpu: Cpu::new(), // Cpu 내부의 Bus 필드는 삭제해야 합니다.
            systolic: SystolicArray::new(),
            dram: Dram::new(mem_size), // 메모리 용량은 오직 Dram만 압니다!
            bus_state: BusState::Ready,
            cycle: 0,
        }
    }

    // 1 Cycle 틱
    pub fn tick(&mut self) {
        self.tick_with(false);
    }

    pub fn tick_with(&mut self, inject_nop: bool) {
        self.cycle += 1;

        // 1. 가속기 DMA도 CPU와 같은 시스템 버스를 사용합니다.
        let mut dma_bus = SystemBus::memory(self.bus_state, &mut self.dram);
        self.systolic.step(&mut dma_bus);
        self.bus_state = dma_bus.state;

        // 부품들의 참조를 모아 시스템 버스 인터페이스를 생성
        let mut sys_bus = SystemBus::with_systolic(
            self.bus_state,
            &mut self.dram,
            &mut self.systolic,
        );

        // 2. CPU 실행 (CPU가 버스/메모리에 접근할 수 있도록 컨텍스트를 묶어서 전달)
        // Rust의 Borrow Checker를 통과하기 위해 SoC의 필드들을 분리해서 참조로 넘깁니다.
        self.cpu.pipeline_step(&mut sys_bus, inject_nop);
        self.bus_state = sys_bus.state;
    }
}