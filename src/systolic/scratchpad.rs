/*
    systolic array에 사용되는 scratchpad를 구현한 모듈
    Scratchpad는 A 행렬의 행과 B 행렬의 열을 저장하는 라인 버퍼를 관리하며, 특정 사이클에서 데이터를 읽어오는 기능을 제공합니다.
    OS방식으로 구현된 systolic array에서는 데이터의 입력을 delay를 고려하여 처리해야 하므로, 이를 위해 LineBuffer를 사용합니다.
*/

use crate::systolic::linebuffer::{LineBuffer, StreamValue};

use super::{ARRAY_SIZE, INNER_DIM};

pub struct Scratchpad {
    pub a_rows: [LineBuffer; ARRAY_SIZE], // A 행렬의 행을 저장하는 라인 버퍼
    pub b_cols: [LineBuffer; ARRAY_SIZE], // B 행렬의 열을 저장하는 라인 버퍼
}

impl Scratchpad {
    pub fn new() -> Self {
        Self {
            a_rows: std::array::from_fn(|row_idx| LineBuffer::new(row_idx)),
            b_cols: std::array::from_fn(|col_idx| LineBuffer::new(col_idx)),
        }
    }

    // A 행렬은 가로로 전달되므로 행 단위로 로드
    pub fn load_a(&mut self, matrix: [[u32; INNER_DIM]; ARRAY_SIZE]) {
        for row in 0..ARRAY_SIZE {
            self.a_rows[row].load(matrix[row]);
        }
    }

    // B 행렬은 세로로 전달되므로 열 단위로 로드
    pub fn load_b(&mut self, matrix: [[u32; ARRAY_SIZE]; INNER_DIM]) {
        for col in 0..ARRAY_SIZE {
            let mut col_data = [0; INNER_DIM];

            for k in 0..INNER_DIM {
                col_data[k] = matrix[k][col];
            }

            self.b_cols[col].load(col_data);
        }
    }

    pub fn a_input(&self, row: usize, cycle: usize) -> StreamValue {
        self.a_rows[row].read_at_cycle(cycle)
    }

    pub fn b_input(&self, col: usize, cycle: usize) -> StreamValue {
        self.b_cols[col].read_at_cycle(cycle)
    }

    pub fn clear(&mut self) {
        for row in &mut self.a_rows {
            row.clear();
        }

        for col in &mut self.b_cols {
            col.clear();
        }
    }
}
