use clap::Parser;
use rv32i_sim::bus::Bus;
use rv32i_sim::cpu::Cpu;
use rv32i_sim::memory::Dram;
use std::fs;
use std::path::Path;
use std::process::Command;

const RAM_SIZE: usize = 16 * 1024 * 1024; // 16MB
const STACK_TOP: u32 = 0x00FF_FFFC; // 16MB 상단 스택 위치

#[derive(Parser, Debug)]
#[command(author, version, about = "RV32I C-Code Compiler & Pipelined Simulator")]
struct Args {
    #[arg(short, long, default_value = "files/main.c")]
    source: String,

    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    #[arg(short, long, default_value_t = 100000)]
    max_steps: usize,
}

fn compile_and_extract(c_path: &str) -> Vec<u32> {
    println!("[1/3] C 소스 파일 컴파일 중: {}", c_path);

    let elf_path = "temp.elf";
    let bin_path = "temp.bin";

    let gcc_status = Command::new("riscv64-unknown-elf-gcc")
        .args([
            "-O0",
            "-nostdlib",
            "-mabi=ilp32",
            "-march=rv32i",
            "-Ttext=0x0",
            "files/entry.s",
            c_path,
            "-o",
            elf_path,
        ])
        .status()
        .expect("riscv64-unknown-elf-gcc 실행 실패. 환경변수 PATH를 확인하세요.");

    assert!(gcc_status.success(), "C 코드 컴파일 실패");

    let objcopy_status = Command::new("riscv64-unknown-elf-objcopy")
        .args(["-O", "binary", elf_path, bin_path])
        .status()
        .expect("objcopy 실행 실패");

    assert!(objcopy_status.success(), "바이너리 추출 실패");

    let binary_bytes = fs::read(bin_path).expect("바이너리 파일을 읽을 수 없습니다.");

    let _ = fs::remove_file(elf_path);
    let _ = fs::remove_file(bin_path);

    binary_bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
        .collect()
}

fn setup_cpu(program: &[u32]) -> Cpu {
    println!("[2/3] CPU 및 16MB DRAM 초기화 중...");
    let mut dram = Dram::new(RAM_SIZE);

    for (i, &inst) in program.iter().enumerate() {
        let bytes = inst.to_le_bytes();
        let addr = i * 4;
        dram.dram[addr] = bytes[0];
        dram.dram[addr + 1] = bytes[1];
        dram.dram[addr + 2] = bytes[2];
        dram.dram[addr + 3] = bytes[3];
    }

    let bus = Bus { dram };
    let mut cpu = Cpu::new(bus);

    // 스택 포인터(sp = x2) 초기화
    cpu.regs.write(2, STACK_TOP, true);
    cpu
}

fn run_simulation(mut cpu: Cpu, verbose: bool, max_steps: usize) {
    println!("[3/3] 파이프라인 시뮬레이션 시작\n");
    println!("{:=^70}", " Simulation Running ");

    let mut cycle_count = 0;

    loop {
        if cycle_count >= max_steps {
            println!(
                "\n[경고] 최대 실행 클럭 수({})에 도달하여 종료합니다.",
                max_steps
            );
            break;
        }

        // Verbose 출력: 각 파이프라인 Stage의 PC 상태 모니터링
        if verbose {
            println!(
                "[{:05} Cycle] IF_PC: 0x{:08X} | ID_PC: 0x{:08X} | EX_PC: 0x{:08X} | sp: 0x{:08X} | a0: {}",
                cycle_count,
                cpu.pc,
                cpu.if_id_reg.pc,
                cpu.id_ex_reg.pc,
                cpu.regs.read(2),  // sp (x2)
                cpu.regs.read(10)  // a0 (x10)
            );
        }

        // 1. 먼저 1클럭(사이클) 수행
        cpu.step();
        cycle_count += 1;

        // // 2. WB 단계(또는 MEM/WB 레지스터)로 완전히 들어온 명령어의 메모리 주소/값을 확인
        // // ecall 명령어가 Flush되지 않고 WB Stage까지 완전히 도달했을 때만 종료
        // let wb_pc = cpu.mem_wb_reg.rd; // 만약 mem_wb_reg에 PC가 있다면 사용, 없으면 아래 방식 참조
        // let wb_inst = cpu.bus.load32(cpu.mem_wb_reg.alu_result).unwrap_or(0); 

        // 가장 간단한 수정 방법: 
        // CPU 내부 execution 흐름 중 MEM/WB 단계 제어 신호에 is_ecall 등을 두거나,
        // IF/ID stage가 아닌, 파이프라인을 거쳐서 온 mem_wb_reg의 제어 신호를 확인합니다.
        if cpu.mem_wb_reg.control.is_ecall { // (control 신호에 is_ecall 추가 권장)
            let exit_code = cpu.regs.read(10); // a0 (x10)
            println!("\n{:=^70}", " Simulation Finished ");
            println!(
                ">> Program exited with status code: {} (0x{:X})",
                exit_code, exit_code
            );
            println!(">> Total executed cycles: {} cycles", cycle_count);
            break;
        }
    }

}fn main() {
    let args = Args::parse();

    if !Path::new(&args.source).exists() {
        eprintln!("오류: '{}' 파일을 찾을 수 없습니다.", args.source);
        return;
    }

    let program = compile_and_extract(&args.source);
    let cpu = setup_cpu(&program);
    run_simulation(cpu, args.verbose, args.max_steps);
}
