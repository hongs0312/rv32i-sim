pub struct Dram {
    pub dram: Vec<u8>
}

impl Dram {
    pub fn new(size: usize) -> Self {
        Self { dram: vec![0; size] }
    }

    pub fn load32(&self, addr: usize) -> u32 {

    }

    pub fn store32(&mut self, addr: usize, value: u32) {
        
    }
}