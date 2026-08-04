use crate::memory::Dram;

pub struct Bus {
    pub dram: Dram,
}

impl Bus {
    pub fn load8(&self, addr: u32) -> Result<u8, ()> {
        Ok(self.dram.load8(addr as usize))
    }

    pub fn load16(&self, addr: u32) -> Result<u16, ()> {
        Ok(self.dram.load16(addr as usize))
    }

    pub fn load32(&self, addr: u32) -> Result<u32, ()> {
        Ok(self.dram.load32(addr as usize))
    }

    pub fn store32(&mut self, addr: u32, value: u32) -> Result<(), ()> {
        self.dram.store32(addr as usize, value);
        Ok(())
    }
}
