pub mod dma;
pub mod linebuffer;
pub mod processing_element;
pub mod scratchpad;

use crate::hardware::memory::Dram;

use dma::{DmaState, SystolicDma};
use processing_element::ProcessingElement;
use scratchpad::Scratchpad;

pub const ARRAY_SIZE: usize = 16;
pub const INNER_DIM: usize = 16;

#[derive(Clone, Copy, PartialEq)]
pub enum SystolicState {
    Idle,
    Loading,
    Computing,
    Storing,
    Done,
}

pub struct SystolicArray {
    pub status: u32,
    pub addr_c: u32,
    pub state: SystolicState,

    pub dma: SystolicDma,
    pub scratchpad: Scratchpad,
    pub pes: [[ProcessingElement; ARRAY_SIZE]; ARRAY_SIZE],

    pub cycle: usize,
    pub global_time: u32,
}

impl SystolicArray {
    pub fn new() -> Self {
        const INIT_PE: ProcessingElement = ProcessingElement::new();

        Self {
            status: 0,
            addr_c: 0,
            state: SystolicState::Idle,

            dma: SystolicDma::new(5),
            scratchpad: Scratchpad::new(),
            pes: [[INIT_PE; ARRAY_SIZE]; ARRAY_SIZE],

            cycle: 0,
            global_time: 0,
        }
    }

    pub fn start(&mut self, addr_a: u32, addr_b: u32, addr_c: u32) {
        self.status = 1;
        self.addr_c = addr_c;

        self.scratchpad.clear();

        for r in 0..ARRAY_SIZE {
            for c in 0..ARRAY_SIZE {
                self.pes[r][c].clear();
            }
        }

        // DMA 로드 시작
        self.dma.start_load(addr_a, addr_b);
        self.state = SystolicState::Loading;
    }

    pub fn step(&mut self, dram: &mut Dram) {
        self.global_time = self.global_time.wrapping_add(1);

        match self.state {
            SystolicState::Idle | SystolicState::Done => {}

            SystolicState::Loading => {
                self.dma.step(dram);

                if self.dma.state == DmaState::Done {
                    self.scratchpad.load_a(self.dma.temp_a);
                    self.scratchpad.load_b(self.dma.temp_b);

                    self.cycle = 0;
                    self.state = SystolicState::Computing;
                }
            }

            SystolicState::Computing => {
                for row in (0..ARRAY_SIZE).rev() {
                    for col in (0..ARRAY_SIZE).rev() {
                        let a_in = if col == 0 {
                            let stream = self.scratchpad.a_input(row, self.cycle);

                            match stream.valid {
                                true => stream.value,
                                false => 0,
                            }
                        } else {
                            self.pes[row][col - 1].a_reg
                        };

                        let b_in = if row == 0 {
                            let stream = self.scratchpad.b_input(col, self.cycle);

                            match stream.valid {
                                true => stream.value,
                                false => 0,
                            }
                        } else {
                            self.pes[row - 1][col].b_reg
                        };

                        self.pes[row][col].step(a_in, b_in);
                    }
                }

                self.cycle += 1;

                let total_cycles = ARRAY_SIZE * 2 + INNER_DIM - 2;
                if self.cycle >= total_cycles {
                    self.dma.start_store(self.addr_c);
                    self.state = SystolicState::Storing;
                }
            }

            SystolicState::Storing => {
                if let Some((row, col)) = self.dma.store_position() {
                    self.dma.store_step(dram, self.pes[row][col].psum as u32);
                }

                if self.dma.state == DmaState::Done {
                    self.state = SystolicState::Done;
                    self.status = 2;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_systolic_array_mac() {
        // 1. Arrange: 메모리(Dram) 초기화 (예: 64KB 할당)
        let mut dram = Dram::new(64 * 1024);

        let addr_a = 0x1000;
        let addr_b = 0x2000;
        let addr_c = 0x3000;

        // 행렬 A 초기화 (모든 요소를 1로 세팅)
        for row in 0..ARRAY_SIZE {
            for col in 0..INNER_DIM {
                let offset = (row * INNER_DIM + col) * 4;
                dram.store32(addr_a + offset, 1);
            }
        }

        // 행렬 B 초기화 (모든 요소를 1로 세팅)
        for row in 0..INNER_DIM {
            for col in 0..ARRAY_SIZE {
                let offset = (row * ARRAY_SIZE + col) * 4;
                dram.store32(addr_b + offset, 1);
            }
        }

        // 2. Act: 가속기 초기화 및 가동
        let mut systolic = SystolicArray::new();
        systolic.start(addr_a as u32, addr_b as u32, addr_c as u32);

        // 상태가 완료(2)가 될 때까지 사이클을 진행 (클럭 에뮬레이션)
        let mut total_cycles = 0;
        while systolic.status != 2 {
            systolic.step(&mut dram);
            total_cycles += 1;

            // 무한 루프(Deadlock) 방지용 타임아웃
            // 로드(16*16*2) + 연산파도(16+16+16) + 저장(16*16) + 레이턴시 여유 = 약 1000 사이클 내외
            assert!(
                total_cycles < 2000,
                "Simulation timed out! Pipeline stalled."
            );
        }

        // 3. Assert: 결과 행렬 C 검증
        // 1로 가득 찬 16x16 행렬 A와 B를 곱하면, C의 모든 요소는 INNER_DIM(16)이 되어야 합니다.
        for row in 0..ARRAY_SIZE {
            for col in 0..ARRAY_SIZE {
                let offset = (row * ARRAY_SIZE + col) * 4;
                let result = dram.load32(addr_c + offset);

                assert_eq!(
                    result, INNER_DIM as u32,
                    "Mismatch at C[{}][{}]: expected {}, got {}",
                    row, col, INNER_DIM, result
                );
            }
        }

        println!(
            "Success! Systolic Array finished MAC operation in {} cycles.",
            total_cycles
        );
    }
}
