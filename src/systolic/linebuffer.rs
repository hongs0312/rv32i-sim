/*
    systolic array의 scratchpad에 사용되는 line buffer를 구현한 모듈
    LineBuffer는 데이터를 저장하고 특정 사이클에서 데이터를 읽어오는 기능을 제공합니다.
    delay 파라미터는 데이터가 유효해지는 데 걸리는 사이클 수를 나타냅니다.
*/

use super::INNER_DIM;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct StreamValue {
    pub value: u32,
    pub valid: bool,
}

pub struct LineBuffer {
    data: [u32; INNER_DIM],
    delay: usize,
}

impl LineBuffer {
    pub fn new(delay: usize) -> Self {
        Self {
            data: [0; INNER_DIM],
            delay,
        }
    }

    pub fn load(&mut self, data: [u32; INNER_DIM]) {
        self.data = data;
    }

    pub fn clear(&mut self) {
        self.data = [0; INNER_DIM];
    }

    pub fn read_at_cycle(&self, cycle: usize) -> StreamValue {
        // cycle이 delay보다 작으면 아직 유효한 데이터가 없으므로 기본값 반환
        // index = cycle - delay
        let Some(index) = cycle.checked_sub(self.delay) else {
            return StreamValue::default();
        };

        match self.data.get(index) {
            Some(&value) => StreamValue { value, valid: true },
            None => StreamValue::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{INNER_DIM, LineBuffer};

    #[test]
    fn line_buffer_delays_values_without_external_crate() {
        let mut buffer = LineBuffer::new(2);
        buffer.load([10; INNER_DIM]);

        assert!(!buffer.read_at_cycle(0).valid);
        assert!(!buffer.read_at_cycle(1).valid);
        assert_eq!(buffer.read_at_cycle(2).value, 10);
        assert!(buffer.read_at_cycle(2).valid);
    }
}
