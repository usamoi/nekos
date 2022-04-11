use crate::prelude::*;

#[derive(Debug)]
pub enum ChannelSendError {
    BadStatus,
}

#[derive(Debug)]
pub enum ChannelReceiveError {
    Empty,
}

pub enum ChannelMessage {
    Bytes(Box<[u8]>),
    Handle(Handle),
}

pub struct Channel {}

impl Channel {
    pub fn create() -> (Arc<Channel>, Arc<Channel>) {
        todo!()
    }
    pub async fn recv(&self) -> Result<ChannelMessage, ChannelReceiveError> {
        todo!()
    }
    pub async fn send(&self, _: ChannelMessage) -> Result<(), ChannelSendError> {
        todo!()
    }
}
