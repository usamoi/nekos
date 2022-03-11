    .section .text
    .globl _satp
_satp:
	.option push
	.option norelax
    la t0, _pt
    srli t0, t0, 12
    li t1, 0b1001 << 60
    or t0, t0, t1
    csrw satp, t0
    sfence.vma
    fence.i
    ret
    .option pop

    .section .rodata
    .balign 4096
_pt:
    .space 512 * 8, 0x0
    // todo
