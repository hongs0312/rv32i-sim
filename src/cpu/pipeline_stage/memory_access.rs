use crate::cpu::Cpu;

use super::{ExMemRegister, MemWbRegister, StageStatus};

pub fn execute(cpu: &mut Cpu, ex_mem_reg: ExMemRegister) -> StageStatus<MemWbRegister> {
    let control = ex_mem_reg.control;
    let addr = ex_mem_reg.alu_result;

    if !control.mem_read && !control.mem_write {
        return StageStatus::Complete(MemWbRegister {
            control,
            alu_result: addr,
            mem_data: 0,
            rd: ex_mem_reg.rd,
        });
    }

    // MMIO 라우팅
    if addr >= 0x8000_0000 {
        if control.mem_write {
            let _ = cpu.bus.write_mmio(addr, ex_mem_reg.rs2_data);
            // MMIO Write Complete
        }
        // MMIO Read Complete 등 처리...
        return StageStatus::Complete(/* ... */);
    }

    // 일반 메모리는 캐시를 찌름
    let cache_status = if control.mem_read {
        cpu.d_cache.read(addr, &mut cpu.bus)
    } else {
        // write 로직 (구현 필요)
        cpu.d_cache.write(addr, ex_mem_reg.rs2_data, &mut cpu.bus)
    };

    match cache_status {
        StageStatus::Busy => StageStatus::Busy,
        StageStatus::Complete(mem_data) => {
            // 여기서 funct3에 따른 데이터 마스킹(lb, lh, lw)을 처리하면 깔끔합니다.
            let masked_data = cpu.apply_funct3_mask(mem_data, control.funct3);

            StageStatus::Complete(MemWbRegister {
                control,
                alu_result: addr,
                mem_data: masked_data,
                rd: ex_mem_reg.rd,
            })
        }
    }
}
