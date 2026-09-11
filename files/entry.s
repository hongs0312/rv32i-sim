.section .text
.global _start

_start:
    # 1. 스택 포인터 초기화 (16MB 상단)
    lui sp, 0x800

    # 2. C main 함수 호출
    call main

    # 3. main 리턴 값(a0)을 가지고 ecall로 시뮬레이터 종료 요청
    # RISC-V ecall 명령어: 0x00000073
    ecall