use crate::prelude::*;
use alloc::collections::BTreeMap;
use core::task::{RawWaker, RawWakerVTable, Waker};
use core::time::Duration;
use proc::process::Process;
use spin::{Lazy, Mutex};

pub struct SchedulerQueue {
    ready: BTreeMap<(Vruntime, usize), Arc<Task>>,
}

impl SchedulerQueue {
    pub fn new() -> Self {
        Self {
            ready: BTreeMap::new(),
        }
    }
    pub fn vruntime(&mut self) -> Option<Vruntime> {
        self.ready
            .first_key_value()
            .map(|((vruntime, _), _)| *vruntime)
    }
    pub fn insert(&mut self, task: Arc<Task>) {
        self.ready
            .insert((task.vruntime(), Arc::as_ptr(&task) as usize), task);
    }
    pub fn pop(&mut self) -> Option<Arc<Task>> {
        self.ready.pop_first().map(|(_, value)| value)
    }
}

pub struct Scheduler {
    queue: Mutex<SchedulerQueue>,
}

impl Scheduler {
    pub fn push(&self, task: Arc<Task>) {
        self.queue.lock().insert(task);
    }
    pub fn pop(&self) -> Option<Arc<Task>> {
        self.queue.lock().pop()
    }
}

pub(in crate::sched) static SCHEDULER: Singleton<Scheduler> = Singleton::new();

pub fn spawn(future: Arc<dyn TaskFuture>, priority: Priority) -> Arc<Task> {
    let mut queue = SCHEDULER.queue.lock();
    // todo: fix the bad behavior if all threads are blocked
    let vruntime = queue.vruntime().unwrap_or(Vruntime::new(0));
    let task = Task::new(future, vruntime, priority);
    queue.insert(task.clone());
    task
}

pub unsafe fn init_boot() {
    SCHEDULER.init(Scheduler {
        queue: Mutex::new(SchedulerQueue::new()),
    });
}

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

static INITPROC: Lazy<Arc<Process>> =
    Lazy::new(|| Process::create(config::PROCESS_INITPROC).expect("initproc created failed"));

pub fn forever() -> ! {
    loop {
        if INITPROC.is_dead() {
            panic!("initproc exited unexpectedly",);
        }
        if let Some(task) = SCHEDULER.pop() {
            let duration = Duration::from_millis(10);
            let waker = make_waker({
                let task = task.clone();
                move || {
                    let mut queue = SCHEDULER.queue.lock();
                    if let Some(queue_vruntime) = queue.vruntime() {
                        let limit_num =
                            queue_vruntime.index() + 1000000u128 / task.priority().index() as u128;
                        let limit = Vruntime::new(limit_num);
                        task.set_vruntime(core::cmp::max(task.vruntime(), limit));
                    }
                    queue.insert(task.clone());
                }
            });
            let cx = &mut core::task::Context::from_waker(&waker);
            let _ = task.future.poll(cx, duration);
            task.set_vruntime(Vruntime::new(
                task.vruntime().index() + 1000000u128 / task.priority().index() as u128,
            ));
        }
    }
}
