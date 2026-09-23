ENTRY(_start)

SECTIONS
{
    . = 0x00000000;
    
    /* 1. 무조건 .init(entry.s)을 0번지 맨 앞에 강제 배치! */
    .init : { *(.init) }
    
    /* 2. 그 다음부터 일반 코드와 데이터를 차례대로 쌓음 */
    .text : { *(.text .text.*) }
    .rodata : { *(.rodata .rodata.*) }
    .data : { *(.data .data.* .sdata .sdata.*) }
    .bss : { *(.bss .bss.* .sbss .sbss.*) }
}