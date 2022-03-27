use crate::prelude::*;
use core::future::Future;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::task::Poll;
use spin::Once;

pub macro print {
    () => (),
    ($($arg:tt)*) => {{
        let mut s = $crate::arch::stdout::STDOUT.write.lock();
        core::fmt::Write::write_fmt(&mut *s, core::format_args!($($arg)*)).unwrap();
    }}
}

pub macro println {
    () => {{
        let mut s = $crate::arch::stdout::STDOUT.write.lock();
        core::fmt::Write::write_str(&mut *s, "\n").unwrap();
    }},
    ($($arg:tt)*) => {{
        let mut s = $crate::arch::stdout::STDOUT.write.lock();
        core::fmt::Write::write_fmt(&mut *s, core::format_args!($($arg)*)).unwrap();
        core::fmt::Write::write_str(&mut *s, "\n").unwrap();
    }}
}

pub macro dbg {
    () => {
        debug!("[{}:{}]", file!(), line!())
    },
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                debug!("[{}:{}] {} = {:#?}", file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    },
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    }
}

#[derive(Clone, Copy)]
pub struct Id<T, U>(PhantomData<fn(T, U) -> (T, U)>);

impl<T> Id<T, T> {
    pub const fn refl() -> Id<T, T> {
        Id(PhantomData)
    }
}

impl<T, U> Id<T, U> {
    pub fn transport(self, value: T) -> U {
        unsafe {
            let ans = core::mem::transmute_copy(&value);
            core::mem::forget(value);
            ans
        }
    }
    pub const fn commutative(self) -> Id<U, T> {
        Id(PhantomData)
    }
    pub const fn transitive<R>(self, _: Id<U, R>) -> Id<T, R> {
        Id(PhantomData)
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Either<T, U> {
    Left(T),
    Right(U),
}

pub use Either::{Left, Right};

impl<T, U> Either<T, U> {
    pub const fn is_left(&self) -> bool {
        match self {
            Left(_) => true,
            Right(_) => false,
        }
    }
    pub const fn is_right(&self) -> bool {
        match self {
            Left(_) => false,
            Right(_) => true,
        }
    }
    pub const fn unwrap_left(self) -> T
    where
        Self: ~const Drop,
    {
        match self {
            Left(x) => x,
            Right(_) => panic!("called `Either::unwrap_left()` on an `Right` value"),
        }
    }
    pub const fn unwrap_right(self) -> U
    where
        Self: ~const Drop,
    {
        match self {
            Left(_) => panic!("called `Either::unwrap_right()` on an `Left` value"),
            Right(x) => x,
        }
    }
}

pub struct Singleton<T>(Once<T>);

impl<T> Singleton<T> {
    pub const fn new() -> Singleton<T> {
        Singleton(Once::new())
    }
    pub fn init(&self, t: T) {
        self.0.call_once(|| t);
    }
}

impl<T> Deref for Singleton<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.0.get().unwrap()
    }
}

impl<T> DerefMut for Singleton<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0.get_mut().unwrap()
    }
}

pub macro env_match(match env $x: literal { $($y: literal => $z: expr),+ $(,)? }) {{
    use $crate::common::inherit::ConstEq;
    $( if env!($x).const_eq($y) { $z } else )+
    { panic!(concat!("unknown value ", env!($x), " of environment variable ", $x)) }
}}

pub macro env_cast($x: literal) {{
    use $crate::common::inherit::ConstFromStr;
    ConstFromStr::const_from_str(env!($x))
}}

pub macro ensure($cond:expr, $err:expr) {{
    if !$cond {
        return Err($err);
    }
}}

pub trait ErrorOut: Sized {
    type T;
    type U;
    fn try_out<S: TryFrom<Self::U>>(self) -> Result<Result<Self::T, S>, S::Error>;
    fn out<S: TryFrom<Self::U>>(self) -> Result<Self::T, S> {
        self.try_out().const_unwrap()
    }
}

impl<T, U> ErrorOut for Result<T, U> {
    type T = T;
    type U = U;
    fn try_out<S: TryFrom<U>>(self) -> Result<Result<Self::T, S>, S::Error> {
        match self {
            Ok(x) => Ok(Ok(x)),
            Err(e) => e.try_into().map(Err),
        }
    }
}

pub macro fully {
    ($src: ty, $dst: ty; $($variant: ident),+) => {
        impl From<$src> for $dst {
            fn from(src: $src) -> Self {
                match src {
                    $(<$src>::$variant => Self::$variant,)*
                }
            }
        }
    },
    ($src: ty, $dst: ty; $($src_variant: ident => $dst_variant: ident),+) => {
        impl From<$src> for $dst {
            fn from(src: $src) -> Self {
                match src {
                    $(<$src>::$src_variant => Self::$dst_variant,)*
                }
            }
        }
    }
}

pub macro partially {
    ($src: ty, $dst: ty; $($variant: ident),+) => {
        impl TryFrom<$src> for $dst {
            type Error = ();
            fn try_from(src: $src) -> Result<Self, Self::Error> {
                match src {
                    $(<$src>::$variant => Ok(Self::$variant),)*
                    _ => Err(()),
                }
            }
        }
    },
    ($src: ty, $dst: ty; $($src_variant: ident => $dst_variant: ident),+) => {
        impl TryFrom<$src> for $dst {
            type Error = ();
            fn try_from(src: $src) -> Result<Self, Self::Error> {
                match src {
                    $(<$src>::$src_variant => Ok(Self::$dst_variant),)*
                    _ => Err(()),
                }
            }
        }
    }
}

pub struct ThreadLocalRef<T: 'static>(&'static T);

impl<T: 'static> ThreadLocalRef<T> {
    pub unsafe fn new(reference: &T) -> ThreadLocalRef<T> {
        ThreadLocalRef(&*(reference as *const T))
    }
}

impl<T: 'static> !Send for ThreadLocalRef<T> {}

impl<T: 'static> Deref for ThreadLocalRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

pub fn yield_now() -> impl Future<Output = ()> {
    struct Yield(bool);
    impl Future for Yield {
        type Output = ();
        fn poll(mut self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
            if !self.0 {
                self.0 = true;
                cx.waker().wake_by_ref();
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        }
    }
    Yield(false)
}
