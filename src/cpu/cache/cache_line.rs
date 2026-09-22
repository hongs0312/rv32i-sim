// 16바이트 크기의 캐시 라인
const CACHE_LINE_SIZE: usize = 16;

#[derive(Clone, Copy)]
pub struct CacheLine {
    pub valid: bool,
    pub dirty: bool,
    pub tag: u32,
    pub data: [u8; CACHE_LINE_SIZE],
}

impl CacheLine {
    pub fn new() -> Self {
        Self {
            valid: false,
            dirty: false,
            tag: 0,
            data: [0; CACHE_LINE_SIZE],
        }
    }

    pub fn read32(&self, offset: usize) -> u32 {
        let bytes = &self.data[offset..offset + 4];
        u32::from_le_bytes(bytes.try_into().expect("Slice with incorrect length"))
    }

    pub fn read8(&self, offset: usize) -> u8 {
        self.data[offset]
    }

    pub fn write32(&mut self, offset: usize, value: u32) {
        let bytes = value.to_le_bytes();
        self.data[offset..offset + 4].copy_from_slice(&bytes);
        self.dirty = true; // 데이터가 변경되었음을 표시
    }

    pub fn write8(&mut self, offset: usize, value: u8) {
        self.data[offset] = value;
        self.dirty = true; // 데이터가 변경되었음을 표시
    }
}
