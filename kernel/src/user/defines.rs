use crate::prelude::*;
use common::basic::Is;
use core::marker::PhantomData;
use core::num::NonZeroU32;

pub type Arguments = [usize; 6];

pub trait Parameter: Sized {
    const N: usize = 1;

    fn from_arguments(env: &Environment, xs: [usize; Self::N]) -> EffSys<Self>;
}

#[must_use]
pub struct Errno(NonZeroU32);

impl Errno {
    pub const fn new<const CODE: NonZeroU32>() -> Self
    where
        Errno: Errnos<{ CODE }>,
    {
        Self(CODE)
    }
    pub const fn into_raw(&self) -> NonZeroU32 {
        self.0
    }
}

pub trait Errnos<const CODE: NonZeroU32> {}

pub macro impl_errno($name:ident, $code:literal) {
    impl Errno {
        pub const $name: Errno = Errno::new::<{ ::core::num::NonZeroU32::new($code).unwrap() }>();
    }
    impl Errnos<{ ::core::num::NonZeroU32::new($code).unwrap() }> for Errno {}
}

impl_errno!(GENERAL_INTERNAL, 0xa9244d1cu32);
impl_errno!(GENERAL_INVAILD_SYSCALL, 0x7f06733du32);
impl_errno!(GENERAL_NOT_SUPPORTED, 0xc2966069u32);

pub struct Syscall(PhantomData<()>);

pub type Args<T, const CODE: u32> = (
    <T as Syscalls<CODE>>::Arg0,
    <T as Syscalls<CODE>>::Arg1,
    <T as Syscalls<CODE>>::Arg2,
    <T as Syscalls<CODE>>::Arg3,
    <T as Syscalls<CODE>>::Arg4,
    <T as Syscalls<CODE>>::Arg5,
);

#[async_trait::async_trait]
pub trait Syscalls<const CODE: u32> {
    type Arg0: Parameter = ();
    type Arg1: Parameter = ();
    type Arg2: Parameter = ();
    type Arg3: Parameter = ();
    type Arg4: Parameter = ();
    type Arg5: Parameter = ();
    type Args: Is<Args<Self, CODE>> = Args<Self, CODE>;
    async fn syscall(env: &Environment, args: Self::Args) -> EffSys<isize>;
}

pub macro impl_syscall($name:ident, $code:literal) {
    impl Syscall {
        pub const $name: u32 = $code;
    }
}
