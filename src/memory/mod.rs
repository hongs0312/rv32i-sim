pub struct Dram {
    pub dram: Vec<u8>,
}

impl Dram {
    pub fn new(size: usize) -> Self {
        Self {
            dram: vec![0; size],
        }
    }

    pub fn load32(&self, addr: usize) -> u32 {
        let bytes = &self.dram[addr..addr + 4];
        u32::from_le_bytes(bytes.try_into().expect("Slice with incorrect length"))
    }

    pub fn store32(&mut self, addr: usize, value: u32) {
        let bytes = value.to_le_bytes();
        self.dram[addr..addr + 4].copy_from_slice(&bytes);
    }
}
