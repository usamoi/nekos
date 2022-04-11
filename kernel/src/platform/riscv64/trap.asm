    .equ ctx_regs,       0 * 8
    .equ ctx_fregs,      32 * 8
    .equ ctx_sstatus,    64 * 8
    .equ ctx_sepc,       65 * 8
    .equ status,         66 * 8
    .equ fault_counter,  67 * 8
    .equ fault_handler,  68 * 8
    .equ fault_gp,       69 * 8
    .equ fault_tp,       70 * 8
    .equ fault_sp,       71 * 8
    .equ switch_sp,      72 * 8
    .equ switch_satp,    73 * 8

    .attribute arch, "rv64gc"
    .section .text.trampoline
    .globl _trampoline_trap_handler
    .balign 4
_trampoline_trap_handler:
    csrrw x31, sscratch, x31
    sd x1, (ctx_regs + 1 * 8) (x31)
    sd x2, (ctx_regs + 2 * 8) (x31)
    sd x3, (ctx_regs + 3 * 8) (x31)
    sd x4, (ctx_regs + 4 * 8) (x31)
    sd x5, (ctx_regs + 5 * 8) (x31)
    sd x6, (ctx_regs + 6 * 8) (x31)
    sd x7, (ctx_regs + 7 * 8) (x31)
    sd x8, (ctx_regs + 8 * 8) (x31)
    sd x9, (ctx_regs + 9 * 8) (x31)
    sd x10, (ctx_regs + 10 * 8) (x31)
    sd x11, (ctx_regs + 11 * 8) (x31)
    sd x12, (ctx_regs + 12 * 8) (x31)
    sd x13, (ctx_regs + 13 * 8) (x31)
    sd x14, (ctx_regs + 14 * 8) (x31)
    sd x15, (ctx_regs + 15 * 8) (x31)
    sd x16, (ctx_regs + 16 * 8) (x31)
    sd x17, (ctx_regs + 17 * 8) (x31)
    sd x18, (ctx_regs + 18 * 8) (x31)
    sd x19, (ctx_regs + 19 * 8) (x31)
    sd x20, (ctx_regs + 20 * 8) (x31)
    sd x21, (ctx_regs + 21 * 8) (x31)
    sd x22, (ctx_regs + 22 * 8) (x31)
    sd x23, (ctx_regs + 23 * 8) (x31)
    sd x24, (ctx_regs + 24 * 8) (x31)
    sd x25, (ctx_regs + 25 * 8) (x31)
    sd x26, (ctx_regs + 26 * 8) (x31)
    sd x27, (ctx_regs + 27 * 8) (x31)
    sd x28, (ctx_regs + 28 * 8) (x31)
    sd x29, (ctx_regs + 29 * 8) (x31)
    sd x30, (ctx_regs + 30 * 8) (x31)
    fsd f0, (ctx_fregs + 0 * 8) (x31)
    fsd f1, (ctx_fregs + 1 * 8) (x31)
    fsd f2, (ctx_fregs + 2 * 8) (x31)
    fsd f3, (ctx_fregs + 3 * 8) (x31)
    fsd f4, (ctx_fregs + 4 * 8) (x31)
    fsd f5, (ctx_fregs + 5 * 8) (x31)
    fsd f6, (ctx_fregs + 6 * 8) (x31)
    fsd f7, (ctx_fregs + 7 * 8) (x31)
    fsd f8, (ctx_fregs + 8 * 8) (x31)
    fsd f9, (ctx_fregs + 9 * 8) (x31)
    fsd f10, (ctx_fregs + 10 * 8) (x31)
    fsd f11, (ctx_fregs + 11 * 8) (x31)
    fsd f12, (ctx_fregs + 12 * 8) (x31)
    fsd f13, (ctx_fregs + 13 * 8) (x31)
    fsd f14, (ctx_fregs + 14 * 8) (x31)
    fsd f15, (ctx_fregs + 15 * 8) (x31)
    fsd f16, (ctx_fregs + 16 * 8) (x31)
    fsd f17, (ctx_fregs + 17 * 8) (x31)
    fsd f18, (ctx_fregs + 18 * 8) (x31)
    fsd f19, (ctx_fregs + 19 * 8) (x31)
    fsd f20, (ctx_fregs + 20 * 8) (x31)
    fsd f21, (ctx_fregs + 21 * 8) (x31)
    fsd f22, (ctx_fregs + 22 * 8) (x31)
    fsd f23, (ctx_fregs + 23 * 8) (x31)
    fsd f24, (ctx_fregs + 24 * 8) (x31)
    fsd f25, (ctx_fregs + 25 * 8) (x31)
    fsd f26, (ctx_fregs + 26 * 8) (x31)
    fsd f27, (ctx_fregs + 27 * 8) (x31)
    fsd f28, (ctx_fregs + 28 * 8) (x31)
    fsd f29, (ctx_fregs + 29 * 8) (x31)
    fsd f30, (ctx_fregs + 30 * 8) (x31)
    fsd f31, (ctx_fregs + 31 * 8) (x31)
    csrrw t0, sscratch, x31
    sd t0, (ctx_regs + 31 * 8) (x31)
    csrr t0, sstatus
    sd t0, ctx_sstatus (x31)
    csrr t0, sepc
    sd t0, ctx_sepc (x31)
    ld t0, status (x31)
    li t1, 0
    bleu t0, t1, _fn_fault
    li t1, 1
    sd x0, status (x31)
    bleu t0, t1, _fn_switch
_fn_fault:
    .option push
	.option norelax
    ld t0, fault_counter (x31)
    bnez t0, _abort
    addi t0, t0, 1
    sd t0, fault_counter (x31)
    ld ra, fault_handler (x31)
    ld gp, fault_gp (x31)
    ld tp, fault_tp (x31)
    ld sp, fault_sp (x31)
    ret
    .option pop
_fn_switch:
    ld t0, switch_satp (x31)
    csrw satp, t0
    sfence.vma
    fence.i
    ld sp, switch_sp (x31)
    ld ra, (0 * 8) (sp)
    ld gp, (1 * 8) (sp)
    ld tp, (2 * 8) (sp)
    ld s0, (3 * 8) (sp)
    ld s1, (4 * 8) (sp)
    ld s2, (5 * 8) (sp)
    ld s3, (6 * 8) (sp)
    ld s4, (7 * 8) (sp)
    ld s5, (8 * 8) (sp)
    ld s6, (9 * 8) (sp)
    ld s7, (10 * 8) (sp)
    ld s8, (11 * 8) (sp)
    ld s9, (12 * 8) (sp)
    ld s10, (13 * 8) (sp)
    ld s11, (14 * 8) (sp)
    addi sp, sp, 15 * 8
    ret
_abort:
    li a7, 1
    li a0, 'A'
    ecall
    li a0, 'B'
    ecall
    li a0, 'O'
    ecall
    li a0, 'R'
    ecall
    li a0, 'T'
    ecall
    li a0, '\n'
    ecall
    li a7, 8
    ecall

    .attribute arch, "rv64gc"
    .section .text.trampoline
    .globl _trampoline_switch
    // extern "C" fn _trampoline_switch(_: PageTableToken)
_trampoline_switch:
    addi sp, sp, - 15 * 8
    sd ra, (0 * 8) (sp)
    sd gp, (1 * 8) (sp)
    sd tp, (2 * 8) (sp)
    sd s0, (3 * 8) (sp)
    sd s1, (4 * 8) (sp)
    sd s2, (5 * 8) (sp)
    sd s3, (6 * 8) (sp)
    sd s4, (7 * 8) (sp)
    sd s5, (8 * 8) (sp)
    sd s6, (9 * 8) (sp)
    sd s7, (10 * 8) (sp)
    sd s8, (11 * 8) (sp)
    sd s9, (12 * 8) (sp)
    sd s10, (13 * 8) (sp)
    sd s11, (14 * 8) (sp)
    csrr x31, sscratch
    li t0, 1
    sd t0, status (x31)
    sd sp, switch_sp (x31)
    ld t0, ctx_sstatus (x31)
    csrw sstatus, t0
    ld t0, ctx_sepc (x31)
    csrw sepc, t0
    csrw satp, a0
    sfence.vma
    fence.i
    ld x1, (ctx_regs + 1 * 8) (x31)
    ld x2, (ctx_regs + 2 * 8) (x31)
    ld x3, (ctx_regs + 3 * 8) (x31)
    ld x4, (ctx_regs + 4 * 8) (x31)
    ld x5, (ctx_regs + 5 * 8) (x31)
    ld x6, (ctx_regs + 6 * 8) (x31)
    ld x7, (ctx_regs + 7 * 8) (x31)
    ld x8, (ctx_regs + 8 * 8) (x31)
    ld x9, (ctx_regs + 9 * 8) (x31)
    ld x10, (ctx_regs + 10 * 8) (x31)
    ld x11, (ctx_regs + 11 * 8) (x31)
    ld x12, (ctx_regs + 12 * 8) (x31)
    ld x13, (ctx_regs + 13 * 8) (x31)
    ld x14, (ctx_regs + 14 * 8) (x31)
    ld x15, (ctx_regs + 15 * 8) (x31)
    ld x16, (ctx_regs + 16 * 8) (x31)
    ld x17, (ctx_regs + 17 * 8) (x31)
    ld x18, (ctx_regs + 18 * 8) (x31)
    ld x19, (ctx_regs + 19 * 8) (x31)
    ld x20, (ctx_regs + 20 * 8) (x31)
    ld x21, (ctx_regs + 21 * 8) (x31)
    ld x22, (ctx_regs + 22 * 8) (x31)
    ld x23, (ctx_regs + 23 * 8) (x31)
    ld x24, (ctx_regs + 24 * 8) (x31)
    ld x25, (ctx_regs + 25 * 8) (x31)
    ld x26, (ctx_regs + 26 * 8) (x31)
    ld x27, (ctx_regs + 27 * 8) (x31)
    ld x28, (ctx_regs + 28 * 8) (x31)
    ld x29, (ctx_regs + 29 * 8) (x31)
    ld x30, (ctx_regs + 30 * 8) (x31)
    fld f0, (ctx_fregs + 0 * 8) (x31)
    fld f1, (ctx_fregs + 1 * 8) (x31)
    fld f2, (ctx_fregs + 2 * 8) (x31)
    fld f3, (ctx_fregs + 3 * 8) (x31)
    fld f4, (ctx_fregs + 4 * 8) (x31)
    fld f5, (ctx_fregs + 5 * 8) (x31)
    fld f6, (ctx_fregs + 6 * 8) (x31)
    fld f7, (ctx_fregs + 7 * 8) (x31)
    fld f8, (ctx_fregs + 8 * 8) (x31)
    fld f9, (ctx_fregs + 9 * 8) (x31)
    fld f10, (ctx_fregs + 10 * 8) (x31)
    fld f11, (ctx_fregs + 11 * 8) (x31)
    fld f12, (ctx_fregs + 12 * 8) (x31)
    fld f13, (ctx_fregs + 13 * 8) (x31)
    fld f14, (ctx_fregs + 14 * 8) (x31)
    fld f15, (ctx_fregs + 15 * 8) (x31)
    fld f16, (ctx_fregs + 16 * 8) (x31)
    fld f17, (ctx_fregs + 17 * 8) (x31)
    fld f18, (ctx_fregs + 18 * 8) (x31)
    fld f19, (ctx_fregs + 19 * 8) (x31)
    fld f20, (ctx_fregs + 20 * 8) (x31)
    fld f21, (ctx_fregs + 21 * 8) (x31)
    fld f22, (ctx_fregs + 22 * 8) (x31)
    fld f23, (ctx_fregs + 23 * 8) (x31)
    fld f24, (ctx_fregs + 24 * 8) (x31)
    fld f25, (ctx_fregs + 25 * 8) (x31)
    fld f26, (ctx_fregs + 26 * 8) (x31)
    fld f27, (ctx_fregs + 27 * 8) (x31)
    fld f28, (ctx_fregs + 28 * 8) (x31)
    fld f29, (ctx_fregs + 29 * 8) (x31)
    fld f30, (ctx_fregs + 30 * 8) (x31)
    fld f31, (ctx_fregs + 31 * 8) (x31)
    ld x31, (ctx_regs + 31 * 8) (x31)
    sret
