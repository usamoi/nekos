use crate::prelude::*;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;
use crossbeam::atomic::AtomicCell;
use proc::process::Process;
use proc::signal_set::SignalSet;
use proc::vmm::AreaFindMapError;
use rt::time::local;
use rt::time::Instant;
use sched::scheduler::spawn;
use spin::{Mutex, Once};
use user::objects::memory::Memory;
use user::objects::memory::MemoryCreateError;

#[derive(Debug)]
pub enum ThreadCreateError {
    OutOfMemory,
    OutOfVirtualMemory,
}

partially!(MemoryCreateError, ThreadCreateError; OutOfMemory);
partially!(AreaFindMapError, ThreadCreateError; OutOfVirtualMemory);

pub struct Thread {
    pub status: AtomicCell<ThreadStatus>,
    pub signal_set: SignalSet,
    pub trapping: Mutex<<P as Platform>::Trapping>,
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
        let sp = {
            let memory = Memory::create(config::THREAD_STACK_LAYOUT).out::<ThreadCreateError>()?;
            let size = memory.layout().size();
            let stack_bot = process
                .space
                .root
                .find_map(memory, Permission::RW)
                .out::<ThreadCreateError>()?;
            let stack_top = stack_bot + size;
            stack_top - P::ABI_STACK_OFFSET
        };
        let tp = match &process.load_tls {
            None => VAddr::new(0),
            Some(tls) => {
                let memory = Memory::create(tls.layout).out::<ThreadCreateError>()?;
                memory.write(0, &tls.content);
                process
                    .space
                    .root
                    .find_map(memory, Permission::RW)
                    .out::<ThreadCreateError>()?
            }
        };
        let thread = Arc::new(Thread {
            status: AtomicCell::new(ThreadStatus::Live),
            signal_set: SignalSet::new(),
            future: Once::new(),
            trapping: Mutex::new(<P as Platform>::Trapping::new(
                Privilege::User,
                pc,
                sp,
                tp,
                opaque,
            )),
            process: process.clone(),
        });
        thread
            .future
            .call_once(|| Mutex::new(Environment::make(thread.clone())));
        spawn(thread.clone(), Priority::DEFAULT);
        Ok(thread)
    }
}

impl PreemptiveFuture for Thread {
    fn poll(&self, cx: &mut Context, duration: Duration) -> Poll<()> {
        let mut future = self.future.get().unwrap().lock();
        local().timer((Instant::now() + duration).value());
        future.as_mut().poll(cx)
    }
}

impl Environment {
    pub async fn thread_fault(&self) -> Flow<!> {
        self.thread
            .status
            .store(ThreadStatus::Dead(ThreadDeath::Fault(
                ThreadFault::ProcessDead,
            )));
        self.thread
            .process
            .thread_set
            .on_thread_killed(&self.thread);
        Flow::Eff(Effect)
    }
    pub async fn thread_exit(&self, exit_code: isize) -> Flow<!> {
        self.thread
            .status
            .store(ThreadStatus::Dead(ThreadDeath::Exited(exit_code)));
        self.thread
            .process
            .thread_set
            .on_thread_killed(&self.thread);
        Flow::Eff(Effect)
    }
    pub async fn forever(&self) -> Flow<!> {
        use Exception::*;
        use Interrupt::*;
        use Trap::*;
        loop {
            self.handle_signals().await?;
            let trap = unsafe {
                P::trap_switch(
                    &mut self.thread.trapping.lock(),
                    self.process.space.page_table.as_ref(),
                )
            };
            match trap {
                TrapUnknown => {
                    panic!("unknown");
                }
                TrapException(IllegalInstruction) => {
                    self.process_fault(ProcessFault::IllegalInstruction).await?;
                }
                TrapException(Misaligned { access, .. }) => {
                    self.process_fault(ProcessFault::Misaligned { access })
                        .await?;
                }
                TrapException(PageFault { access, addr }) => {
                    self.handle_page_fault(addr, access).await?;
                }
                TrapException(Breakpoint) => {
                    self.thread.trapping.lock().solve_breakpoint();
                }
                TrapException(Syscall { id, args }) => {
                    let mut trapping = self.thread.trapping.lock();
                    match self.handle_syscall(id, args).await.shift()? {
                        Ok(value) => {
                            trapping.solve_syscall(Some(0), Some(value));
                        }
                        Err(errno) => {
                            trapping.solve_syscall(Some(u32::from(errno) as usize), None);
                        }
                    }
                }
                TrapInterrupt(Timer) => {
                    base::future::yield_now().await;
                }
                TrapInterrupt(Software { .. }) => {
                    self.handle_signals().await?;
                }
                TrapInterrupt(Hardware { value }) => {
                    panic!("hardware interrupt: not supported (value = {})", value);
                }
            }
        }
    }
    pub async fn start(&self) -> Effect {
        match self.forever().await {
            Flow::Ok(_infallible) => _infallible,
            Flow::Err(_infallible) => _infallible,
            Flow::Eff(eff) => eff,
        }
    }
}
