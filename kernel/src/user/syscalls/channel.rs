use crate::prelude::*;
use common::inherit::IntExt;
use user::objects::channel::*;

impl Object for Channel {}

impl_syscall!(CHANNEL_CREATE, 0xe3f0302cu32);

#[repr(C)]
#[derive(Clone, Copy)]
struct ChannelCreateReturn {
    l: HandleID,
    r: HandleID,
}

#[async_trait::async_trait]
impl Syscalls<{ Syscall::CHANNEL_CREATE }> for Syscall {
    type Arg0 = VAddr;
    async fn syscall(env: &Environment, (ret_addr, ..): Self::Args) -> EffSys<isize> {
        let (l, r) = Channel::create();
        let l = env.process.handle_set.push(Handle::new(l));
        let r = env.process.handle_set.push(Handle::new(r));
        let ret_value = ChannelCreateReturn { l, r };
        env.handle_side_effect(env.process.space.write_value(ret_addr, ret_value))
            .await?;
        Ok(0)
    }
}

impl_syscall!(CHANNEL_SEND_BYTES, 0x72a3d296u32);
impl_errno!(CHANNEL_SEND_BYTES_BAD_STATUS, 0xec598f86u32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::CHANNEL_SEND_BYTES }> for Syscall {
    type Arg0 = Handle<Channel>;
    type Arg1 = VAddr;
    type Arg2 = usize;
    async fn syscall(
        env: &Environment,
        (channel, bytes_addr, bytes_len, ..): Self::Args,
    ) -> EffSys<isize> {
        use ChannelSendError::*;
        let mut buffer = vec![0u8; bytes_len].into_boxed_slice();
        env.handle_side_effect(env.process.space.read_buffer(bytes_addr, &mut buffer))
            .await?;
        channel
            .send(ChannelMessage::Bytes(buffer))
            .map_err(|e| match e {
                BadStatus => Errno::CHANNEL_SEND_BYTES_BAD_STATUS,
            })?;
        Ok(0)
    }
}

impl_syscall!(CHANNEL_SEND_HANDLE, 0x314aa333u32);
impl_errno!(CHANNEL_SEND_HANDLE_BAD_STATUS, 0xf7f6aea9u32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::CHANNEL_SEND_HANDLE }> for Syscall {
    type Arg0 = Handle<Channel>;
    type Arg1 = Handle;
    async fn syscall(_: &Environment, (channel, handle, ..): Self::Args) -> EffSys<isize> {
        use ChannelSendError::*;
        channel
            .send(ChannelMessage::Handle(handle))
            .map_err(|e| match e {
                BadStatus => Errno::CHANNEL_SEND_HANDLE_BAD_STATUS,
            })?;
        Ok(0)
    }
}

impl_syscall!(CHANNEL_RECEIVE, 0xecedb83du32);
impl_errno!(CHANNEL_RECEIVE_EMPTY, 0x71b71bbfu32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::CHANNEL_RECEIVE }> for Syscall {
    type Arg0 = Handle<Channel>;
    type Arg1 = VAddr;
    type Arg2 = usize;
    type Arg3 = VAddr;
    async fn syscall(
        env: &Environment,
        (channel, addr, size, ret_size, ..): Self::Args,
    ) -> EffSys<isize> {
        use ChannelMessage::*;
        use ChannelReceiveError::*;
        let message = channel.receive().map_err(|e| match e {
            Empty => Errno::CHANNEL_RECEIVE_EMPTY,
        })?;
        match message {
            Bytes(bytes) => {
                env.handle_side_effect(
                    env.process
                        .space
                        .write_value::<usize>(ret_size, bytes.len()),
                )
                .await?;
                let buffer = &bytes[..usize::min(bytes.len(), size)];
                env.handle_side_effect(env.process.space.write_buffer(addr, buffer))
                    .await?;
                Ok(0)
            }
            Handle(handle) => {
                env.handle_side_effect(
                    env.process
                        .space
                        .write_value::<usize>(ret_size, core::mem::size_of::<HandleID>()),
                )
                .await?;
                let buffer = &env.process.handle_set.push(handle).to_bytes()
                    [..usize::min(4, core::mem::size_of::<HandleID>())];
                env.handle_side_effect(env.process.space.write_buffer(addr, buffer))
                    .await?;
                Ok(1)
            }
        }
    }
}
