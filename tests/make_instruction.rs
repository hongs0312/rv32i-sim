#[test]
fn make_instruction() {
    let opcode: u32 = 0x13; // addi
    let rd: u32 = 8;
    let funct3: u32 = 0x0;
    let rs1: u32 = 0;
    let rs2: u32 = 100;

    // let funct7: u32 = 0x00;

    // // core instruction format에 따라 변환
    // let result = (funct7 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode;

    // let result = (opcode) | (rd << 7) | (funct3 << 12);

    let result = (opcode) | (rd << 7) | (funct3 << 12) | (rs1 << 15) | (rs2 << 20);

    println!("{:#010x}", result);
}
