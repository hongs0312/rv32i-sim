pub struct RegisterFile {
    regs: [u32; 32],
}

impl RegisterFile {
    pub fn read(&self, reg: u32) -> u32 {
        if reg == 0 { 0 } else { self.regs[reg] }
    }

    pub fn write(&mut self, reg: usize, value: u32) {
        if reg != 0 {
            self.regs[reg] = value;
        }
    }
}