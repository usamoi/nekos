mod errors;
pub use self::errors::*;
mod signal_set;
pub use self::signal_set::*;

use crate::prelude::*;
use crate::sched::scheduler::spawn;
use arch::cpu::local;
use arch::time::MachineInstant;
use arch::trampoline::switch::Switch;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;
use crossbeam::atomic::AtomicCell;
use proc::process::Process;
use spin::{Mutex, Once};
use user::objects::memory::Memory;

pub struct Thread {
    pub status: AtomicCell<ThreadStatus>,
    pub signal_set: SignalSet,
    pub switch: Mutex<Switch>,
    // warning: this mutex MUST unlock quickly after lock
    pub process: Arc<Process>,
    future: Once<Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>>,
}

impl Thread {
    pub fn status(&self) -> ThreadStatus {
        self.status.load()
    }
    pub fn create(
        process: &Arc<Process>,
        pc: VAddr,
        opaque: usize,
    ) -> Result<Arc<Thread>, ThreadCreateError> {
        let pt = process.space.page_table.token();
        let sp = {
            let memory = Memory::create(config::THREAD_STACK_LAYOUT).out::<ThreadCreateError>()?;
            let size = memory.layout().size();
            let ptr = process
                .space
                .root
                .find_map(memory, MapPermission::RW)
                .out::<ThreadCreateError>()?;
            ptr + size
        };
        let tp = match &process.load_tls {
            None => VAddr::new(0),
            Some(tls) => {
                let memory = Memory::create(tls.layout).out::<ThreadCreateError>()?;
                memory.write(0, &tls.content);
                process
                    .space
                    .root
                    .find_map(memory, MapPermission::RW)
                    .out::<ThreadCreateError>()?
            }
        };
        let switch = {
            let mut switch = Switch::new(pt, pc, sp, tp);
            switch.set_opaque(opaque);
            switch
        };
        let thread = Arc::new(Thread {
            status: AtomicCell::new(ThreadStatus::Live),
            signal_set: SignalSet::new(),
            future: Once::new(),
            switch: Mutex::new(switch),
            process: process.clone(),
        });
        thread
            .future
            .call_once(|| Mutex::new(Environment::make(thread.clone())));
        spawn(thread.clone(), Priority::DEFAULT);
        Ok(thread)
    }
}

impl TaskFuture for Thread {
    fn poll(&self, cx: &mut Context, duration: Duration) -> Poll<()> {
        let mut guard = self.future.get().unwrap().lock();
        local().local_set_timer(MachineInstant::now() + duration);
        let future = guard.as_mut();
        let ans = future.poll(cx);
        drop(guard);
        ans
    }
}

impl Environment {
    pub async fn thread_fault(&self) -> EffKill<!> {
        self.thread
            .status
            .store(ThreadStatus::Dead(ThreadDeath::Fault(
                ThreadFault::ProcessDead,
            )));
        self.thread
            .process
            .thread_set
            .on_thread_killed(&self.thread);
        Err(EffectKill::Kill)
    }
    pub async fn thread_exit(&self, exit_code: isize) -> EffKill<!> {
        self.thread
            .status
            .store(ThreadStatus::Dead(ThreadDeath::Exited(exit_code)));
        self.thread
            .process
            .thread_set
            .on_thread_killed(&self.thread);
        Err(EffectKill::Kill)
    }
    pub async fn forever(&self) -> EffKill<!> {
        use self::Exception::*;
        use self::Interrupt::*;
        use self::Trap::*;
        loop {
            self.handle_signals().await?;
            let trap = unsafe { self.thread.switch.lock().switch() };
            match trap {
                Unknown => {
                    panic!("unknown");
                }
                Exception(IllegalInstruction) => {
                    self.process_fault(ProcessFault::IllegalInstruction).await?;
                }
                Exception(Misaligned { op: kind }) => {
                    self.process_fault(ProcessFault::Misaligned { op: kind })
                        .await?;
                }
                Exception(PageFault { op: kind, addr }) => {
                    self.handle_page_fault(addr, kind).await?;
                }
                Exception(Breakpoint) => {
                    trace!("breakpoint");
                    self.thread.switch.lock().solve_breakpoint();
                }
                Exception(Syscall { id, args }) => {
                    let ret = match self.handle_syscall(id, args).await {
                        Ok(value) => Ok(value),
                        Err(EffectSys::Errno(errno)) => Err(errno),
                        Err(EffectSys::EffectKill(fault)) => return Err(fault),
                    };
                    self.thread.switch.lock().solve_syscall(ret);
                }
                Interrupt(Timer) => {
                    yield_now().await;
                }
                Interrupt(Software) => {
                    self.handle_signals().await?;
                }
                Interrupt(Hardware) => {
                    panic!("hardware interrupt: not supported");
                }
            }
        }
    }
    pub async fn start(&self) -> EffectKill {
        match self.forever().await {
            Ok(_infallible) => _infallible,
            Err(death) => death,
        }
    }
}
