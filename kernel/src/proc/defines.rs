use crate::prelude::*;
use core::future::Future;
use core::pin::Pin;
use proc::process::Process;
use proc::thread::Thread;

#[derive(Debug)]
pub enum Exception {
    IllegalInstruction,
    Misaligned { op: MemoryOperation },
    PageFault { op: MemoryOperation, addr: VAddr },
    Syscall { id: usize, args: Arguments },
    Breakpoint,
}

#[derive(Debug)]
pub enum Interrupt {
    Timer,
    Software { value: usize },
    Hardware { value: usize },
}

#[derive(Debug)]
pub enum Trap {
    Unknown,
    Exception(Exception),
    Interrupt(Interrupt),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessFault {
    IllegalInstruction,
    Misaligned { op: MemoryOperation },
    Segment { op: MemoryOperation },
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
                EffectKill::Kill => (),
            }
        })
    }
}

#[must_use]
pub enum SideEffect {
    KillProcess(ProcessDeath),
}

impl Environment {
    pub async fn handle_side_effect<T, E: Into<SideEffect>>(
        &self,
        result: Result<T, E>,
    ) -> EffKill<T> {
        use SideEffect::*;
        match result.map_err(Into::into) {
            Ok(x) => Ok(x),
            Err(KillProcess(ProcessDeath::Fault(fault))) => {
                self.process_fault(fault).await.map(|x| x)
            }
            Err(KillProcess(ProcessDeath::Exited(code))) => {
                self.process_exit(code).await.map(|x| x)
            }
        }
    }
}

#[must_use]
pub enum EffectKill {
    Kill,
}

pub type EffKill<T> = Result<T, EffectKill>;

#[must_use]
pub enum EffectSys {
    Errno(Errno),
    EffectKill(EffectKill),
}

impl From<EffectKill> for EffectSys {
    fn from(e: EffectKill) -> Self {
        EffectSys::EffectKill(e)
    }
}

impl From<Errno> for EffectSys {
    fn from(e: Errno) -> Self {
        EffectSys::Errno(e)
    }
}

pub type EffSys<T> = Result<T, EffectSys>;
