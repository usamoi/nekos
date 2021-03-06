use crate::prelude::*;
use alloc::collections::BTreeMap;
use base::cell::SingletonCell;
use futures::task::ArcWake;
use proc::process::Process;
use spin::{Lazy, Mutex};

impl ArcWake for Task {
    fn wake(self: Arc<Self>) {
        self.resched()
    }
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.clone().resched()
    }
}

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

pub(in crate::sched) static SCHEDULER: SingletonCell<Scheduler> = SingletonCell::new();

pub fn spawn(future: Arc<dyn PreemptiveFuture>, priority: Priority) -> Arc<Task> {
    let mut queue = SCHEDULER.queue.lock();
    // todo: fix the bad behavior if all threads are blocked
    let vruntime = queue.vruntime().unwrap_or_else(|| Vruntime::new(0));
    let task = Task::new(future, vruntime, priority);
    queue.insert(task.clone());
    task
}

pub unsafe fn init_global() {
    SCHEDULER.initialize(Scheduler {
        queue: Mutex::new(SchedulerQueue::new()),
    });
}

static INITPROC: Lazy<Arc<Process>> =
    Lazy::new(|| Process::create("initproc").expect("initproc created failed"));

pub fn initproc() -> &'static Arc<Process> {
    &INITPROC
}

pub fn forever() -> ! {
    loop {
        // todo: fix it
        if initproc().is_dead() || initproc().thread_set.threads.lock().len() == 0 {
            panic!(
                "initproc exited unexpectedly, status = {:?}",
                initproc().status()
            );
        }
        if let Some(task) = SCHEDULER.pop() {
            let duration = config::SCHEDULE_TIMESLICE;
            let waker = futures::task::waker(task.clone());
            let cx = &mut core::task::Context::from_waker(&waker);
            task.poll(cx, duration);
        }
    }
}
