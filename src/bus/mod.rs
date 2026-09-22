use crate::cpu::StageStatus;
use crate::memory::Dram;
use crate::systolic::SystolicArray;

pub enum BusState {
    Ready,
    Processing(u8),
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

    // IF 단계에서 32비트 명령어를 읽어오는 메서드
    // 1사이클 즉시 읽기(L1 캐시 히트 가정)
    pub fn load32(&self, addr: u32) -> Result<u32, ()> {
        let addr = addr as usize;

        if addr % 4 != 0 || addr + 3 >= self.dram.dram.len() {
            return Err(()); // 주소가 4바이트 정렬되지 않았거나 범위를 벗어남
        }

        Ok(self.dram.load32(addr as usize))
    }

    pub fn data_access(
        &mut self,
        addr: u32,
        funct3: u8,
        is_read: bool,
        is_write: bool,
        write_data: u32,
    ) -> StageStatus<Result<u32, ()>> {
        if !is_read && !is_write {
            return StageStatus::Complete(Ok(0)); // 읽기/쓰기 모두 아닌 경우, 즉시 완료
        }

        // [MMIO 라우팅] 0x8000_0000 이상 주소는 DRAM 접근이 아닌 MMIO 접근으로 간주
        if addr >= 0x8000_0000 {
            if is_read {
                return StageStatus::Complete(Ok(self.read_mmio(addr)));
            } else {
                // is_write
                return StageStatus::Complete(self.write_mmio(addr, write_data));
            }
        }

        let dram_latency = 5; // DRAM 접근 지연 사이클 수

        match self.state {
            BusState::Ready => {
                // 첫 메모리 요청 시 대기 상태로 돌입
                self.state = BusState::Processing(dram_latency - 1);
                return StageStatus::Busy;
            }
            BusState::Processing(cycles_left) => {
                if cycles_left > 1 {
                    // 아직 대기중
                    self.state = BusState::Processing(cycles_left - 1);
                    return StageStatus::Busy;
                } else {
                    self.state = BusState::Ready; // 대기 완료 후 Ready 상태로 전환
                }
            }
        }

        // 대기 완료 후 실제 메모리 접근 수행
        if is_read {
            StageStatus::Complete(self.load(addr, funct3))
        } else {
            // is_write
            match self.store(addr, funct3, write_data) {
                Ok(_) => StageStatus::Complete(Ok(0)),   // 쓰기 성공 시 0 반환
                Err(e) => StageStatus::Complete(Err(e)), // 쓰기 실패 시 에러 반환
            }
        }
    }

    // MEM 단계에서 8/16/32비트 데이터를 읽어오는 메서드
    pub fn load(&self, addr: u32, funct3: u8) -> Result<u32, ()> {
        match funct3 {
            0x0 => Ok(self.dram.load8(addr as usize) as i8 as u32),
            0x1 => Ok(self.dram.load16(addr as usize) as i16 as u32),
            0x2 => Ok(self.dram.load32(addr as usize)),
            0x4 => Ok(self.dram.load8(addr as usize) as u32), // LBU
            0x5 => Ok(self.dram.load16(addr as usize) as u32), // LHU
            _ => Err(()),
        }
    }

    // MEM 단계에서 8/16/32비트 데이터를 쓰는 메서드
    pub fn store(&mut self, addr: u32, funct3: u8, value: u32) -> Result<(), ()> {
        match funct3 {
            0x0 => Ok(self.dram.store8(addr as usize, value as u8)),
            0x1 => Ok(self.dram.store16(addr as usize, value as u16)),
            0x2 => Ok(self.dram.store32(addr as usize, value)),
            _ => Err(()),
        }
    }

    fn read_mmio(&self, addr: u32) -> u32 {
        match addr {
            0x8000_0000 => self.systolic.status,
            0x8000_0020 => self.systolic.global_time,
            _ => 0, // 정의되지 않은 MMIO 주소는 0 반환
        }
    }

    fn write_mmio(&mut self, addr: u32, value: u32) -> Result<u32, ()> {
        match addr {
            0x8000_0004 => self.systolic.dma.addr_a = value, // DMA 모듈로 바로 전달
            0x8000_0008 => self.systolic.dma.addr_b = value,
            0x8000_000C => self.systolic.addr_c = value,
            0x8000_0010 => {
                if value == 1 {
                    // 시작 트리거!
                    self.systolic.start(
                        self.systolic.dma.addr_a,
                        self.systolic.dma.addr_b,
                        self.systolic.addr_c,
                    );
                }
            }
            _ => return Err(()), // 정의되지 않은 MMIO 접근
        }
        Ok(0) // MMIO 쓰기 성공 시 0 반환
    }
}
