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
    // gp
    .option push
    .option norelax    
    la gp, __global_pointer$
    .option pop
    // tp
    li tp, 0
    // sp
    la sp, _stack_top
    // call
    call _start

    .section .text
    .globl _startup2
_startup2:
	.option push
	.option norelax
    call _satp
    ld ra, _startup2_address
    ret
    .option pop

    .section .text
    .balign 8
_startup2_address:
    .8byte _startup2_virtual
_startup2_virtual:
    // gp
    .option push
    .option norelax
    la gp, __global_pointer$
    .option pop
    // tp
    li tp, 0
    // sp
    mv sp, a1
    // satp
    ld t0, SATP
    csrw satp, t0
    sfence.vma
    fence.i
    // call
    call _start2
