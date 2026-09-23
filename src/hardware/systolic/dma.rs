/*
    Systolic Array DMA State Machine
    DMA = Direct Memory Access

    시스템 메모리로부터 Systolic Array로 데이터를 로드하는 모듈
*/

use crate::hardware::bus::SystemBus;
use crate::hardware::cpu::pipeline_stage::StageStatus;

use super::{ARRAY_SIZE, INNER_DIM};

#[derive(Clone, Copy, PartialEq)]
pub enum DmaState {
    Idle,
    LatencyWait { cycles_left: u8, is_a: bool },
    Bursting { is_a: bool, row: usize, col: usize },
    StoringC { row: usize, col: usize },
    Done,
}

pub struct SystolicDma {
    pub state: DmaState,
    pub addr_a: u32,
    pub addr_b: u32,
    pub addr_c: u32,
    pub latency: u8, // 초기 접근 지연 사이클 수

    pub temp_a: [[u32; INNER_DIM]; ARRAY_SIZE],
    pub temp_b: [[u32; ARRAY_SIZE]; INNER_DIM],
}

impl SystolicDma {
    pub fn new(latency: u8) -> Self {
        Self {
            state: DmaState::Idle,
            addr_a: 0,
            addr_b: 0,
            addr_c: 0,
            latency,

            temp_a: [[0; INNER_DIM]; ARRAY_SIZE],
            temp_b: [[0; ARRAY_SIZE]; INNER_DIM],
        }
    }

    pub fn start_load(&mut self, addr_a: u32, addr_b: u32) {
        self.addr_a = addr_a;
        self.addr_b = addr_b;

        self.state = DmaState::LatencyWait {
            cycles_left: self.latency - 1,
            is_a: true,
        };
    }

    pub fn start_store(&mut self, addr_c: u32) {
        self.addr_c = addr_c;
        self.state = DmaState::StoringC { row: 0, col: 0 };
    }

    pub fn store_position(&self) -> Option<(usize, usize)> {
        match self.state {
            DmaState::StoringC { row, col } => Some((row, col)),
            _ => None,
        }
    }

    pub fn store_step(&mut self, bus: &mut SystemBus, value: u32) {
        let DmaState::StoringC { row, col } = self.state else {
            return;
        };

        let offset = (row * ARRAY_SIZE + col) * 4;
        if !matches!(
            bus.write_word(self.addr_c + offset as u32, value),
            StageStatus::Complete(())
        ) {
            return;
        }

        let next_col = col + 1;
        if next_col >= ARRAY_SIZE {
            let next_row = row + 1;
            self.state = if next_row >= ARRAY_SIZE {
                DmaState::Done
            } else {
                DmaState::StoringC {
                    row: next_row,
                    col: 0,
                }
            };
        } else {
            self.state = DmaState::StoringC { row, col: next_col };
        }
    }

    pub fn step(&mut self, bus: &mut SystemBus) {
        match self.state {
            DmaState::Idle | DmaState::Done | DmaState::StoringC { .. } => {}

            DmaState::LatencyWait { cycles_left, is_a } => {
                if cycles_left > 0 {
                    self.state = DmaState::LatencyWait {
                        cycles_left: cycles_left - 1,
                        is_a,
                    };
                } else {
                    self.state = DmaState::Bursting {
                        is_a,
                        row: 0,
                        col: 0,
                    };
                }
            }

            DmaState::Bursting { is_a, row, col } => {
                if is_a {
                    let offset = (row * INNER_DIM + col) * 4;
                    let result = bus.read_word(self.addr_a + offset as u32);
                    let StageStatus::Complete(value) = result else {
                        return;
                    };
                    self.temp_a[row][col] = value;

                    let next_col = col + 1;
                    if next_col >= INNER_DIM {
                        let next_row = row + 1;
                        if next_row >= ARRAY_SIZE {
                            self.state = DmaState::LatencyWait {
                                cycles_left: self.latency - 1,
                                is_a: false,
                            };
                        } else {
                            self.state = DmaState::Bursting {
                                is_a,
                                row: next_row,
                                col: 0,
                            };
                        }
                    } else {
                        self.state = DmaState::Bursting {
                            is_a,
                            row,
                            col: next_col,
                        };
                    }
                } else {
                    let offset = (row * ARRAY_SIZE + col) * 4;
                    let result = bus.read_word(self.addr_b + offset as u32);
                    let StageStatus::Complete(value) = result else {
                        return;
                    };
                    self.temp_b[col][row] = value;

                    let next_col = col + 1;
                    if next_col >= ARRAY_SIZE {
                        let next_row = row + 1;
                        if next_row >= INNER_DIM {
                            self.state = DmaState::Done;
                        } else {
                            self.state = DmaState::Bursting {
                                is_a: false,
                                row: next_row,
                                col: 0,
                            };
                        }
                    } else {
                        self.state = DmaState::Bursting {
                            is_a: false,
                            row,
                            col: next_col,
                        };
                    }
                }
            }
        }
    }
}
