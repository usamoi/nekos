use crate::prelude::*;
use core::future::Future;
use core::pin::Pin;
use proc::process::Process;
use proc::thread::Thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessFault {
    IllegalInstruction,
    Misaligned { access: Access },
    Segment { access: Access },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessDeath {
    Exited(isize),
    Fault(ProcessFault),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    Live,
    Dead(ProcessDeath),
}

impl ProcessStatus {
    pub const fn is_live(self) -> bool {
        matches!(self, ProcessStatus::Live)
    }
    pub const fn is_dead(self) -> bool {
        matches!(self, ProcessStatus::Dead(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadFault {
    ProcessDead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadDeath {
    Exited(isize),
    Fault(ThreadFault),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadStatus {
    Live,
    Dead(ThreadDeath),
}

impl ThreadStatus {
    pub const fn is_live(self) -> bool {
        matches!(self, ThreadStatus::Live)
    }
    pub const fn is_dead(self) -> bool {
        matches!(self, ThreadStatus::Dead(_))
    }
}

#[derive(Debug, Clone)]
pub enum Signal {
    KillThread(isize),
    StopProcess,
}

pub struct Environment {
    pub thread: Arc<Thread>,
    pub process: Arc<Process>,
}

impl Environment {
    pub fn make(thread: Arc<Thread>) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        let this = Environment {
            process: thread.process.clone(),
            thread,
        };
        Box::pin(async move {
            match this.start().await {
                Effect => (),
            }
        })
    }
}
