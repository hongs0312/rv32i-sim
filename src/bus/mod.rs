use crate::cpu::pipeline_stage::StageStatus;
use crate::memory::Dram;
use crate::systolic::SystolicArray;

#[derive(Clone, Copy, PartialEq)]
pub enum BusState {
    Ready,
    Processing(u32), // 남은 대기 사이클
}

pub struct Bus {
    pub dram: Dram,
    pub state: BusState,
    pub systolic: SystolicArray,
}

impl Bus {
    pub fn new(dram: Dram) -> Self {
        Self {
            dram,
            state: BusState::Ready,
            systolic: SystolicArray::new(),
        }
    }

    // 캐시에서 16바이트 블록 로드를 요청할 때 사용
    pub fn read_block(&mut self, addr: u32) -> StageStatus<[u8; 16]> {
        let base_addr = (addr & !0xF) as usize;

        match self.state {
            BusState::Ready => {
                self.state = BusState::Processing(4); // 5사이클 중 첫 사이클 소모
                StageStatus::Busy
            }
            BusState::Processing(cycles_left) => {
                if cycles_left > 1 {
                    self.state = BusState::Processing(cycles_left - 1);
                    StageStatus::Busy
                } else {
                    self.state = BusState::Ready; // 완료

                    let mut block = [0u8; 16];
                    block.copy_from_slice(&self.dram.dram[base_addr..base_addr + 16]);
                    StageStatus::Complete(block)
                }
            }
        }
    }

    // 캐시 Eviction 시 블록을 DRAM에 쓸 때 사용
    pub fn write_block(&mut self, addr: u32, block: &[u8; 16]) -> StageStatus<()> {
        let base_addr = (addr & !0xF) as usize;

        match self.state {
            BusState::Ready => {
                self.state = BusState::Processing(4);
                StageStatus::Busy
            }
            BusState::Processing(cycles_left) => {
                if cycles_left > 1 {
                    self.state = BusState::Processing(cycles_left - 1);
                    StageStatus::Busy
                } else {
                    self.state = BusState::Ready;
                    self.dram.dram[base_addr..base_addr + 16].copy_from_slice(block);
                    StageStatus::Complete(())
                }
            }
        }
    }

    // MMIO는 대기 없이 즉시 리턴 (기존 로직 유지)
    pub fn read_mmio(&self, addr: u32) -> u32 {
        /* ... */
        0
    }
    pub fn write_mmio(&mut self, addr: u32, value: u32) -> Result<u32, ()> {
        /* ... */
        Ok(0)
    }
}
