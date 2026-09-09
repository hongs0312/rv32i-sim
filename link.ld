SECTIONS
{
    . = 0x00000000;
    .text : {
        *(.text.entry)
        *(.text*)
    }
    .data : { *(.data*) }
    .bss : { *(.bss*) }

    /* 64KB 지점을 스택 상단으로 지정 */
    . = 0x10000;
    __stack_top = .;
}