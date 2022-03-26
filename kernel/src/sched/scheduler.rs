use crate::prelude::*;
use alloc::collections::BTreeMap;
use core::task::{RawWaker, RawWakerVTable, Waker};
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
    pub queue: Mutex<SchedulerQueue>,
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

pub fn spawn(future: Arc<dyn PreemptiveFuture>, priority: Priority) -> Arc<Task> {
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

fn waker(task: Arc<Task>) -> Waker {
    unsafe fn vtable() -> &'static RawWakerVTable {
        &RawWakerVTable::new(
            |data| {
                Arc::increment_strong_count(data as *const Task);
                RawWaker::new(data, vtable())
            },
            |data| {
                let arc = Arc::from_raw(data as *const Task);
                Task::resched(arc)
            },
            |data| {
                let arc = core::mem::ManuallyDrop::new(Arc::from_raw(data as *const Task));
                Task::resched((*arc).clone())
            },
            |data| Arc::decrement_strong_count(data as *const Task),
        )
    }
    let data = Arc::into_raw(task) as *const ();
    unsafe { Waker::from_raw(RawWaker::new(data, vtable())) }
}

static INITPROC: Lazy<Arc<Process>> =
    Lazy::new(|| Process::create(config::PROCESS_INITPROC).expect("initproc created failed"));

pub fn forever() -> ! {
    loop {
        if INITPROC.is_dead() {
            panic!("initproc exited unexpectedly",);
        }
        if let Some(task) = SCHEDULER.pop() {
            let duration = config::SCHEDULE_TIMESLICE;
            let waker = waker(task.clone());
            let cx = &mut core::task::Context::from_waker(&waker);
            task.poll(cx, duration);
        }
    }
}
