use crate::prelude::*;
use arrayvec::ArrayVec;
use core::time::Duration;
use rt::backtrace::BacktraceFrame;
use rt::paging::PagingToken;
use rt::paging::{Paging, PagingGroup};
use rt::trap::TrapContext;

pub trait Platform {
    type TrapContext: TrapContext;
    type Paging: Paging<Group = Self::PagingGroup>;
    type PagingGroup: PagingGroup;
    const STACK_ALIGN: usize;
    const STACK_OFFSET: usize;
    const ELF_EABI: u16;
    const PAGING_PHYS: Segment<VAddr>;
    const PAGING_KERNEL: Segment<VAddr>;
    const PAGING_HEAP: Segment<VAddr>;
    const PAGING_GLOBAL: Segment<VAddr>;
    unsafe fn backtrace() -> ArrayVec<BacktraceFrame, { config::BACKTRACE_LIMIT }>;
    fn io_write(s: &str);
    fn paging_align(align: usize) -> bool;
    fn paging_permission(permission: Permission) -> bool;
    unsafe fn paging_phys(page_table: &Self::Paging);
    unsafe fn paging_token_switch(_: PagingToken);
    fn process_abort() -> !;
    fn thread_id() -> usize;
    fn thread_flush_ins();
    fn thread_flush_tlb();
    fn time_now() -> u64;
    fn time_timer(time: u64);
    fn time_sub_maybe(start: u64, end: u64) -> Option<Duration>;
    fn time_add_maybe(ins: u64, dur: Duration) -> Option<u64>;
    unsafe fn trap_switch(ctx: &mut Self::TrapContext, pt: rt::paging::PagingToken) -> Trap;
}

pub struct P;
