use crate::memory::Dram;

pub struct Bus {
    pub dram: Dram,
}

impl Bus {
    // IF 단계에서 32비트 명령어를 읽어오는 메서드
    pub fn load32(&self, addr: u32) -> Result<u32, ()> {
        let addr = addr as usize;

        if addr % 4 != 0 || addr + 3 >= self.dram.dram.len() {
            return Err(()); // 주소가 4바이트 정렬되지 않았거나 범위를 벗어남
        }

        Ok(self.dram.load32(addr as usize))
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
}
