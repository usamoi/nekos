use crate::prelude::*;
use crate::sched::scheduler::spawn;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;
use crossbeam::atomic::AtomicCell;
use proc::process::Process;
use proc::signal_set::SignalSet;
use proc::vmm::AreaFindMapError;
use rt::paging::Paging;
use rt::thread::current;
use rt::time::Instant;
use rt::trap::User;
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
    pub switch: Mutex<User>,
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
            let stack_bot = process
                .space
                .root
                .find_map(memory, Permission::RW)
                .out::<ThreadCreateError>()?;
            let stack_top = stack_bot + size;
            stack_top - P::STACK_OFFSET
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
        let switch = {
            let mut switch = User::new(pt, pc, sp, tp);
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

impl PreemptiveFuture for Thread {
    fn poll(&self, cx: &mut Context, duration: Duration) -> Poll<()> {
        let mut guard = self.future.get().unwrap().lock();
        current().set_timer(Instant::now() + duration);
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
                Exception(Misaligned { access, .. }) => {
                    self.process_fault(ProcessFault::Misaligned { access })
                        .await?;
                }
                Exception(PageFault { access, addr }) => {
                    self.handle_page_fault(addr, access).await?;
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
                    base::future::yield_now().await;
                }
                Interrupt(Software { .. }) => {
                    self.handle_signals().await?;
                }
                Interrupt(Hardware { value }) => {
                    panic!("hardware interrupt: not supported (value = {})", value);
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
