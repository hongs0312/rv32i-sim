pub struct RegisterFile {
    regs: [u32; 32],
}

impl RegisterFile {
    pub fn new() -> Self {
        Self { regs: [0; 32] }
    }

    pub fn read(&self, reg: u32) -> u32 {
        if reg == 0 { 0 } else { self.regs[reg as usize] }
    }

    pub fn write(&mut self, reg: u32, value: u32) {
        if reg != 0 {
            self.regs[reg as usize] = value;
        }
    }
}
