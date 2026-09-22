use crate::cpu::Cpu;

use super::{IfIdRegister, StageStatus};

pub fn execute(cpu: &mut Cpu, inject_nop: bool) -> StageStatus<IfIdRegister> {
    if inject_nop {
        return StageStatus::Complete(IfIdRegister {
            pc: cpu.pc,
            instruction: 0x00000013,
        });
    }

    let cur_pc = cpu.pc;

    if cur_pc >= 0x8000_0000 {
        panic!(
            "[Error] PC가 MMIO 영역(0x{:08X})을 실행하려고 시도했습니다!",
            cur_pc
        );
    }

    match cpu.i_cache.read(cur_pc, &mut cpu.bus) {
        StageStatus::Busy => StageStatus::Busy,
        StageStatus::Complete(instruction) => StageStatus::Complete(IfIdRegister {
            pc: cur_pc,
            instruction,
        }),
    }
}
