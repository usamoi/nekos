mod errors;
pub use self::errors::*;

use crate::prelude::*;

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
