ENTRY(_startup)

BASE_ADDRESS = 0xFFFFFFC000000000;

PHDRS
{
    text PT_LOAD;
    rodata PT_LOAD;
    data PT_LOAD;
    bss PT_LOAD;
    tls PT_TLS;
    brk PT_LOAD;
}

SECTIONS
{
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
    } :text

    .rodata : ALIGN(4K) {
        PROVIDE(_rodata_start = .);
        *(.srodata .srodata.*)
        *(.rodata .rodata.*)
        . = ALIGN(4K);
        PROVIDE(_rodata_end = .);
    } :rodata

    .tdata : ALIGN(4K) {
        PROVIDE(_tdata_start = .);
        *(.tdata .tdata.*)
        . = ALIGN(4K);
        PROVIDE(_tdata_end = .);
    } :tls

    .tbss : ALIGN(4K) {
        PROVIDE(_tbss_start = .);
        *(.tbss .tbss.*)
        . = ALIGN(4K);
        PROVIDE(_tbss_end = .);
    } :tls

    .data : ALIGN(4K) {
        PROVIDE(_data_start = .);
        *(.sdata .sdata.*)
        PROVIDE(__global_pointer$ = .);
        *(.data .data.*)
        . = ALIGN(4K);
        PROVIDE(_data_end = .);
    } :data

    .bss : ALIGN(4K) {
        PROVIDE(_bss_start = .);
        *(.sbss .sbss.*)
        *(.bss .bss.*)
        . = ALIGN(4K);
        PROVIDE(_bss_end = .);
    } :bss

    . = ALIGN(4K);
    PROVIDE(_brk_start = .);
    .brk : {
        PROVIDE(_brk_tls_start = .);
        . += SIZEOF(.tdata) + SIZEOF(.tbss);
        PROVIDE(_brk_tls_end = .);
        KEEP(*(.brk.stack))
        KEEP(*(.brk.heap))
        KEEP(*(.brk .brk.*))
    } :brk
    . = ALIGN(4K);
    PROVIDE(_brk_ptr = .);
    PROVIDE(_brk_end = BASE_ADDRESS + 1024M);

    /DISCARD/ : {}
}
