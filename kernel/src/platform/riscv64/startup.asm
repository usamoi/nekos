    .section .text.startup
    .globl _startup
_startup:
	.option push
	.option norelax
    call _satp
    ld ra, _startup_address
    ret
    .option pop

    .section .text
    .balign 8
_startup_address:
    .8byte _startup_virtual
_startup_virtual:
    .option push
    .option norelax
    // gp
    la gp, __global_pointer$
    // tp
    la tp, _brk_tls_start
    // sp
    la sp, _brk_stack_top
    // call
    call _start
    .option pop

    .section .text
    .globl _wake
_wake:
	.option push
	.option norelax
    call _satp
    ld ra, _wake_address
    ret
    .option pop

    .section .text
    .balign 8
_wake_address:
    .8byte _wake_virtual
_wake_virtual:
    .option push
    .option norelax
    // satp
    ld t0, SATP
    csrw satp, t0
    sfence.vma
    fence.i
    // gp
    la gp, __global_pointer$
    // tp
    ld tp, (-1 * 8) (a1)
    // sp
    mv sp, a1
    // call
    call _start2
    .option pop
