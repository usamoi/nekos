use crate::prelude::*;
use crossbeam::queue::SegQueue;

pub struct SignalSet {
    inner: SegQueue<Signal>,
}

impl SignalSet {
    pub const fn new() -> SignalSet {
        SignalSet {
            inner: SegQueue::new(),
        }
    }
    pub fn send(&self, signal: Signal) {
        self.inner.push(signal);
    }
    pub fn receive(&self) -> Option<Signal> {
        self.inner.pop()
    }
}

impl Environment {
    pub async fn handle_signals(&self) -> Flow<()> {
        use Signal::*;
        while let Some(signal) = self.thread.signal_set.receive() {
            match signal {
                KillThread(exit_code) => {
                    self.thread_exit(exit_code).await?;
                }
                StopProcess => {
                    self.thread_fault().await?;
                }
            }
        }
        Flow::Ok(())
    }
}
