OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS
{
    .tdata : ALIGN(4K) {
        *(.tdata .tdata.*)
        . = ALIGN(4K);
    }

    .tbss : ALIGN(4K) {
        *(.tbss .tbss.*)
        . = ALIGN(4K);
    }

    . = 0x8048000;

    .text : ALIGN(4K) {
        *(.stext .stext.*)
        *(.text .text.*)
        . = ALIGN(4K);
    }
    
    .rodata : ALIGN(4K) {
        *(.srodata .srodata.*)
        *(.rodata .rodata.*)
        . = ALIGN(4K);
    }
    
    .data : ALIGN(4K) {
        *(.sdata .sdata.*)
        *(.data .data.*)
        . = ALIGN(4K);
    }

    .bss : ALIGN(4K) {
        *(.sbss .sbss.*)
        *(.bss .bss.*)
        . = ALIGN(4K);
    }

    /DISCARD/ : {
        *(.eh_frame)
    }
}
