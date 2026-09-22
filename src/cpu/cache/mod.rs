mod cache_line;

use crate::{bus::Bus, cpu::StageStatus};
use cache_line::CacheLine;

pub struct L1Cache {
    lines: [CacheLine; 64],
}

impl L1Cache {
    pub fn new() -> Self {
        Self {
            lines: [CacheLine::new(); 64],
        }
    }

    pub fn read(&mut self, addr: u32, bus: &mut Bus) -> StageStatus<u32> {
        let offset = (addr & 0xF) as usize;
        let index = ((addr >> 4) & 0x3F) as usize;
        let tag = addr >> 10;

        let line = &mut self.lines[index];

        if line.valid && line.tag == tag {
            // [Hit] 1사이클 즉시 반환
            return StageStatus::Complete(line.read32(offset));
        }

        // [Miss] 버스에 블록 로드 요청

        match bus.read_block(addr) {
            StageStatus::Busy => StageStatus::Busy, // 버스가 바쁘면 캐시도 바쁨
            StageStatus::Complete(new_block) => {
                // 버스가 데이터를 가져오면 캐시 라인 업데이트
                line.data = new_block;
                line.valid = true;
                line.tag = tag;
                line.dirty = false;

                StageStatus::Complete(line.read32(offset))
            }
        }
    }
}
