use crate::prelude::*;
use common::basic::Singleton;
use core::task::{RawWaker, RawWakerVTable, Waker};
use core::time::Duration;
use crossbeam::queue::SegQueue;
use proc::process::Process;
use spin::Lazy;

pub fn make_waker(callback: impl Fn()) -> Waker {
    // do not remove the type annotation!
    let f: Arc<Box<dyn Fn()>> = Arc::new(Box::new(callback));
    unsafe fn vtable() -> &'static RawWakerVTable {
        type T = Box<dyn Fn()>;
        &RawWakerVTable::new(
            |data| {
                Arc::increment_strong_count(data as *const T);
                RawWaker::new(data, vtable())
            },
            |data| Arc::from_raw(data as *const T)(),
            |data| (*(data as *const T))(),
            |data| Arc::decrement_strong_count(data as *const T),
        )
    }
    unsafe { Waker::from_raw(RawWaker::new(Arc::into_raw(f) as *const (), vtable())) }
}

pub struct Scheduler {
    ready: SegQueue<Arc<Task>>,
}

static SCHEDULER: Singleton<Scheduler> = Singleton::new();

pub unsafe fn init_boot() {
    SCHEDULER.init(Scheduler {
        ready: SegQueue::new(),
    });
}

static INITPROC: Lazy<Arc<Process>> =
    Lazy::new(|| Process::create(config::PROCESS_INITPROC).expect("initproc created failed"));

pub fn spawn(worker: Arc<dyn Pollable>, priority: Priority) {
    SCHEDULER.ready.push(Task::new(worker, priority));
}

pub fn forever() -> ! {
    fn find() -> Option<Arc<Task>> {
        if let Some(ans) = SCHEDULER.ready.pop() {
            return Some(ans);
        }
        None
    }
    loop {
        if INITPROC.is_dead() {
            panic!("initproc exited unexpectedly",);
        }
        if let Some(task) = find() {
            if unsafe { task.poll(|| (), Duration::from_millis(10)) }.is_pending() {
                SCHEDULER.ready.push(task);
            }
        }
    }
}
