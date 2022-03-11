OUTPUT_ARCH(riscv64gc)
ENTRY(_startup)

BASE_ADDRESS = 0xFFFFFFC000000000;

SECTIONS
{
    /* PROVIDE(_phys_address = 0x80000000); */

    . = BASE_ADDRESS;
    PROVIDE(_kernel_address = .);

    PROVIDE(_reserve_start = .);
    . += 2M;
    PROVIDE(_reserve_end = .);

    .text : ALIGN(4K) {
        PROVIDE(_startup_start = .);
        *(.text.startup)
        . = ALIGN(4K);
        PROVIDE(_startup_end = .);

        PROVIDE(_trampoline_start = .);
        *(.text.trampoline)
        . = ALIGN(4K);
        PROVIDE(_trampoline_end = .);

        PROVIDE(_text_start = .);
        *(.stext .stext.*)
        *(.text .text.*)
        . = ALIGN(4K);
        PROVIDE(_text_end = .);
    }

    .rodata : ALIGN(4K) {
        PROVIDE(_rodata_start = .);
        *(.srodata .srodata.*)
        *(.rodata .rodata.*)
        . = ALIGN(4K);
        PROVIDE(_rodata_end = .);
    }

    .debug : ALIGN(4K) {
        PROVIDE(_debug_start = .);

        PROVIDE(_ehframe_start = .);
        KEEP(*(.eh_frame .eh_frame.*))
        PROVIDE(_ehframe_end = .);

        . = ALIGN(4K);
        PROVIDE(_debug_end = .);
    }

    .tdata : ALIGN(4K) {
        PROVIDE(_tdata_start = .);
        *(.tdata .tdata.*)
        . = ALIGN(4K);
        PROVIDE(_tdata_end = .);
    }

    .tbss (NOLOAD) : ALIGN(4K) {
        PROVIDE(_tbss_start = .);
        *(.tbss .tbss.*)
        . = ALIGN(4K);
        PROVIDE(_tbss_end = .);
    }

    .data : ALIGN(4K) {
        PROVIDE(_data_start = .);
        *(.got)
        PROVIDE(__global_pointer$ = .);
        *(.sdata .sdata.*)
        *(.data .data.*)
        . = ALIGN(4K);
        PROVIDE(_data_end = .);
    }

    .bss : ALIGN(4K) {
        PROVIDE(_bss_start = .);
        *(.sbss .sbss.*)
        *(.bss .bss.*)
        . = ALIGN(4K);
        PROVIDE(_bss_end = .);
    }

    .uninit : ALIGN(4K) {
        PROVIDE(_uninit_start = .);
        KEEP(*(.uninit .uninit.*))
        . = ALIGN(4K);
        PROVIDE(_uninit_end = .);
    }

    PROVIDE(_bump_start = .);
    PROVIDE(_bump_end = BASE_ADDRESS + 1024M);

    /DISCARD/ : {}
}
