mod errors;
pub use self::errors::*;

use crate::prelude::*;
use crossbeam::queue::SegQueue;

pub enum ChannelMessage {
    Bytes(Box<[u8]>),
    Handle(Handle),
}

pub struct Channel {
    connect: Weak<Channel>,
    queue: SegQueue<ChannelMessage>,
}

impl Channel {
    pub fn create() -> (Arc<Channel>, Arc<Channel>) {
        let mut maybe_r = None;
        let l = Arc::new_cyclic(|l: &Weak<Channel>| {
            let r = Arc::new(Channel {
                connect: l.clone(),
                queue: SegQueue::new(),
            });
            let weak_r = Arc::downgrade(&r);
            maybe_r = Some(r);
            Channel {
                connect: weak_r,
                queue: SegQueue::new(),
            }
        });
        (l, maybe_r.unwrap())
    }
    pub fn receive(&self) -> Result<ChannelMessage, ChannelReceiveError> {
        use ChannelReceiveError::*;
        self.queue.pop().ok_or(Empty)
    }
    pub fn send(&self, mail: ChannelMessage) -> Result<(), ChannelSendError> {
        use ChannelSendError::*;
        self.connect.upgrade().ok_or(BadStatus)?.queue.push(mail);
        Ok(())
    }
}
