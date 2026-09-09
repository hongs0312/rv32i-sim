use clap::Parser;
use rv32i_sim::bus::Bus;
use rv32i_sim::cpu::Cpu;
use rv32i_sim::memory::Dram;
use std::fs;
use std::path::Path;
use std::process::Command;

const RAM_SIZE: usize = 16 * 1024 * 1024; // 16MB
// const STACK_TOP: u32 = 0x0100_0000; // 16MB 지점을 스택 상단으로 설정

#[derive(Parser, Debug)]
#[command(author, version, about = "RV32I C-Code Compiler & Simulator")]
struct Args {
    /// 실행할 C 소스 파일 경로
    #[arg(short, long, default_value = "files/main.c")]
    source: String,

    /// 한 스텝씩 진행하며 레지스터 상태 출력
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// 최대 실행 클럭 수 제한 (무한 루프 방지)
    #[arg(short, long, default_value_t = 100000)]
    max_steps: usize,
}

/// 1. C언어 컴파일 및 바이너리 추출
fn compile_and_extract(c_path: &str) -> Vec<u32> {
    println!("[1/3] C 소스 파일 컴파일 중: {}", c_path);

    let elf_path = "temp.elf";
    let bin_path = "temp.bin";

    // GCC 컴파일 수행
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

    // 바이너리 추출
    let objcopy_status = Command::new("riscv64-unknown-elf-objcopy")
        .args(["-O", "binary", elf_path, bin_path])
        .status()
        .expect("objcopy 실행 실패");

    assert!(objcopy_status.success(), "바이너리 추출 실패");

    // 생성된 바이너리 파일 읽기
    let binary_bytes = fs::read(bin_path).expect("바이너리 파일을 읽을 수 없습니다.");

    // 임시 파일 삭제
    let _ = fs::remove_file(elf_path);
    let _ = fs::remove_file(bin_path);

    // u8 바이트 배열을 u32 명령어 배열로 변환
    binary_bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
        .collect()
}

/// 2. CPU 및 DRAM 초기화
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
    let cpu = Cpu::new(bus);

    // // 스택 포인터(sp)를 메모리 상단(16MB)으로 설정
    // cpu.regs.write(2, STACK_TOP);
    cpu
}

/// 3. 실행 및 실시간 피드백
fn run_simulation(mut cpu: Cpu, verbose: bool, max_steps: usize) {
    println!("[3/3] 시뮬레이션 시작\n");
    println!("{:=^60}", " Simulation Running ");

    let mut step_count = 0;

    loop {
        if step_count >= max_steps {
            println!(
                "\n[경고] 최대 실행 명령어 수({})에 도달하여 종료합니다.",
                max_steps
            );
            break;
        }

        let current_pc = cpu.pc;

        // 메모리에서 현재 명령어 로드
        let inst = match cpu.bus.load32(current_pc) {
            Ok(val) => val,
            Err(_) => {
                println!("\n[오류] PC 0x{:08X} 접근 실패 (Out of Bounds)", current_pc);
                break;
            }
        };

        // ecall (0x00000073) 체크 - 프로그램 종료 시스콜
        if inst == 0x00000073 {
            let exit_code = cpu.regs.read(10); // a0
            println!("\n{:=^60}", " Simulation Finished ");
            println!(
                ">> Program exited with status code: {} (0x{:X})",
                exit_code, exit_code
            );
            println!(">> Total executed instructions: {} steps", step_count);
            break;
        }

        // Verbose 옵션 시 단계별 실행 상태 출력
        if verbose {
            println!(
                "[{:05}] PC: 0x{:08X} | Inst: 0x{:08X} | sp: 0x{:08X} | a0: {}",
                step_count,
                current_pc,
                inst,
                cpu.regs.read(2),
                cpu.regs.read(10)
            );
        }

        cpu.step();
        step_count += 1;
    }
}

fn main() {
    let args = Args::parse();

    if !Path::new(&args.source).exists() {
        eprintln!("오류: '{}' 파일을 찾을 수 없습니다.", args.source);
        return;
    }

    let program = compile_and_extract(&args.source);
    let cpu = setup_cpu(&program);
    run_simulation(cpu, args.verbose, args.max_steps);
}
