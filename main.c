int fibo(int n) {
    if (n <= 1) {
        return n;
    }
    return fibo(n - 1) + fibo(n - 2);
}

int main() {
    int result = fibo(10);
    return result;
}

void __attribute__((naked, section(".text.entry"))) _start() {
    __asm__ volatile (
        "li sp, 0x10000\n\t"     // 링커 기호 대신 직접 상수로 스택 탑 주소 설정
        "call main\n\t"
        "li a7, 93\n\t"          // sys_exit
        "ecall\n\t"
    );
}