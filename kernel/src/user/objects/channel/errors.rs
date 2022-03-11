#[derive(Debug)]
pub enum ChannelSendError {
    BadStatus,
}

#[derive(Debug)]
pub enum ChannelReceiveError {
    Empty,
}
