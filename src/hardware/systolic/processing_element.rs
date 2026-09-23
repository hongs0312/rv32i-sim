/*
    systolic array의 processing element를 구현한 모듈
    ProcessingElement는 A와 B 데이터를 입력받아 곱셈을 수행하고, 결과를 누적하여 psum에 저장합니다.
    또한, A와 B 데이터를 다음 PE로 전달하기 위해 a_reg과 b_reg에 저장합니다.
*/

#[derive(Clone, Copy, Default)]
pub struct ProcessingElement {
    pub psum: i32,  // OS 방식: PE 내부에 고정되어 누적되는 결과 (Accumulator)
    pub a_reg: u32, // 옆으로 전달할 A 데이터 래치
    pub b_reg: u32, // 아래로 전달할 B 데이터 래치
}

impl ProcessingElement {
    pub const fn new() -> Self {
        Self {
            psum: 0,
            a_reg: 0,
            b_reg: 0,
        }
    }

    // 매 클럭마다 실행되는 PE 연산
    pub fn step(&mut self, a_in: u32, b_in: u32) -> (u32, u32) {
        let product = (a_in as i32).wrapping_mul(b_in as i32);
        self.psum = self.psum.wrapping_add(product);

        self.a_reg = a_in;
        self.b_reg = b_in;

        (self.a_reg, self.b_reg)
    }

    pub fn clear(&mut self) {
        self.psum = 0;
        self.a_reg = 0;
        self.b_reg = 0;
    }
}

#[test]
fn pe_accumulates_product() {
    // PE 연산 테스트
    let mut pe = ProcessingElement::default();

    pe.step(2, 3); // 2 * 3 = 6
    pe.step(4, 5); // 4 * 5 = 20, 누적 합계 = 6 + 20 = 26

    assert_eq!(pe.psum, 26);
}
