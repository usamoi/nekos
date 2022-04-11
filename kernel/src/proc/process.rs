use crate::prelude::*;
use crossbeam::atomic::AtomicCell;
use proc::handle_set::HandleSet;
use proc::loader::LoadError;
use proc::loader::{load, ImageTls};
use proc::thread::Thread;
use proc::thread::ThreadCreateError;
use proc::thread_set::ThreadSet;
use proc::vmm::UserSpace;

#[derive(Debug)]
pub enum ProcessCreateError {
    LoadError,
    OutOfMemory,
    OutOfVirtualMemory,
}

impl From<LoadError> for ProcessCreateError {
    fn from(_e: LoadError) -> Self {
        use ProcessCreateError::*;
        LoadError
    }
}

partially!(ProcessSpawnError, ProcessCreateError; OutOfMemory, OutOfVirtualMemory);

#[derive(Debug)]
pub enum ProcessSpawnError {
    BadStatus,
    OutOfMemory,
    OutOfVirtualMemory,
}

fully!(ThreadCreateError, ProcessSpawnError; OutOfMemory, OutOfVirtualMemory);

#[derive(Debug)]
pub enum ProcessStopError {
    BadStatus,
}

pub struct Process {
    status: AtomicCell<ProcessStatus>,
    pub space: Arc<UserSpace>,
    pub handle_set: HandleSet,
    pub thread_set: ThreadSet,
    pub load_tls: Option<ImageTls>,
}

impl Process {
    pub fn status(&self) -> ProcessStatus {
        self.status.load()
    }
    pub fn is_live(&self) -> bool {
        self.status().is_live()
    }
    pub fn is_dead(&self) -> bool {
        self.status().is_dead()
    }
    pub fn create(name: &str) -> Result<Arc<Process>, ProcessCreateError> {
        let load = load(name)?;
        let process = Arc::new(Process {
            status: AtomicCell::new(ProcessStatus::Live),
            space: load.space,
            handle_set: HandleSet::new(),
            thread_set: ThreadSet::new(),
            load_tls: load.tls,
        });
        let _ = process.handle_set.extend(0, Handle::new(process.clone()));
        process.spawn(load.pc, 0).out::<ProcessCreateError>()?;
        Ok(process)
    }
    pub fn spawn(
        self: &Arc<Self>,
        pc: VAddr,
        opaque: usize,
    ) -> Result<Arc<Thread>, ProcessSpawnError> {
        use ProcessSpawnError::*;
        if self.is_dead() {
            return Err(BadStatus);
        }
        let thread = Thread::create(self, pc, opaque)?;
        self.thread_set.insert(thread.clone());
        Ok(thread)
    }
    pub fn stop(&self, death: ProcessDeath) -> Result<(), ProcessStopError> {
        use ProcessStatus::*;
        use ProcessStopError::*;
        if self.status.compare_exchange(Live, Dead(death)).is_err() {
            return Err(BadStatus);
        }
        self.thread_set.broadcast(Signal::StopProcess);
        Ok(())
    }
}

impl Environment {
    pub async fn process_fault(&self, fault: ProcessFault) -> EffKill<!> {
        use proc::process::ProcessStopError::*;
        use ProcessDeath::*;
        match self.process.stop(Fault(fault)) {
            Ok(()) => {
                warn!("process fault: {:?}", fault);
            }
            Err(BadStatus) => (),
        }
        self.thread_fault().await
    }
    pub async fn process_exit(&self, exit_code: isize) -> EffKill<!> {
        use proc::process::ProcessStopError::*;
        use ProcessDeath::*;
        match self.process.stop(Exited(exit_code)) {
            Ok(()) => (),
            Err(BadStatus) => (),
        }
        self.thread_fault().await
    }
}
