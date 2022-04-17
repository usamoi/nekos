use crate::prelude::*;
use core::any::Any;
use core::cmp::Ordering;
use core::convert::Infallible;
use core::marker::PhantomData;
use core::ops::{ControlFlow, FromResidual, Try};

pub type Arguments = [usize; 6];

pub type HandleID = usize;

pub trait Object: Any + Send + Sync + ObjectUpcast {}

pub trait ObjectUpcast: Any + Send + Sync {
    fn upcast(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

impl<T: Any + Send + Sync + Sized> ObjectUpcast for T {
    fn upcast(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

#[derive(Deref)]
pub struct Handle<T: ?Sized = dyn Object> {
    pub object: Arc<T>,
}

pub trait HandleUpcast {
    fn upcast(self) -> Handle;
}

impl<T: Object + Sized> HandleUpcast for Handle<T> {
    fn upcast(self) -> Handle {
        Handle {
            object: self.object,
        }
    }
}

impl HandleUpcast for Handle {
    fn upcast(self) -> Handle {
        self
    }
}

impl<T: Object + ?Sized> Handle<T> {
    pub fn new(object: Arc<T>) -> Handle<T> {
        Handle { object }
    }
    pub fn downcast<U: Object>(self) -> Option<Handle<U>> {
        let object = Arc::downcast::<U>(self.object.upcast()).ok()?;
        Some(Handle { object })
    }
}

impl<T: Object + ?Sized> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            object: self.object.clone(),
        }
    }
}

impl<T: Object + ?Sized> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        let lhs = Arc::as_ptr(&self.object);
        let rhs = Arc::as_ptr(&other.object);
        PartialEq::eq(&lhs, &rhs)
    }
}

impl<T: Object + ?Sized> Eq for Handle<T> {}

impl<T: Object + ?Sized> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let lhs = Arc::as_ptr(&self.object);
        let rhs = Arc::as_ptr(&other.object);
        PartialOrd::partial_cmp(&lhs, &rhs)
    }
}

impl<T: Object + ?Sized> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let lhs = Arc::as_ptr(&self.object);
        let rhs = Arc::as_ptr(&other.object);
        Ord::cmp(&lhs, &rhs)
    }
}

pub trait Domain: Sized {
    type Error: DomainError;
    fn from_arguments(env: &Environment, x: usize)
        -> Flow<Self, Either<GeneralError, Self::Error>>;
}

pub trait Codomain: Sized {
    fn to_return_value(self) -> usize;
}

pub struct Effect;

pub enum UserError {
    General(GeneralError),
    Domain { order: u8, code: u8 },
    Syscall { code: u8 },
}

impl From<UserError> for u32 {
    fn from(x: UserError) -> u32 {
        match x {
            UserError::General(e) => 1 << 24 | e as u32,
            UserError::Domain { order, code } => 2 << 24 | (order as u32) << 8 | code as u32,
            UserError::Syscall { code } => 3 << 24 | code as u32,
        }
    }
}

#[repr(u8)]
pub enum GeneralError {
    Internal = 0,
    InvaildSyscall = 1,
    NotSupported = 2,
    NotImplemented = 3,
}

pub trait DomainError {
    fn into_u8(self) -> u8;
}

pub trait SyscallError {
    fn into_u8(self) -> u8;
}

impl SyscallError for ! {
    fn into_u8(self) -> u8 {
        self
    }
}

pub struct Syscall(PhantomData<()>);

#[async_trait::async_trait]
pub trait Syscalls<const CODE: u32> {
    type Domain0: Domain = ();
    type Domain1: Domain = ();
    type Domain2: Domain = ();
    type Domain3: Domain = ();
    type Domain4: Domain = ();
    type Domain5: Domain = ();
    type Codomain: Codomain = ();
    type Error: SyscallError;
    async fn syscall(env: &Environment, args: domain!()) -> codomain!();
}

pub macro impl_syscall($name:ident, $code:literal) {
    impl Syscall {
        pub const $name: u32 = $code;
    }
}

pub macro domain() {
    (
        Self::Domain0,
        Self::Domain1,
        Self::Domain2,
        Self::Domain3,
        Self::Domain4,
        Self::Domain5,
    )
}

pub macro codomain() {
    Flow<Self::Codomain, Either<GeneralError, Self::Error>>
}

pub enum Flow<T, E = !> {
    Ok(T),
    Err(E),
    Eff(Effect),
}

impl<T, E> Try for Flow<T, E> {
    type Output = T;

    type Residual = Flow<!, E>;

    fn from_output(output: Self::Output) -> Self {
        Flow::Ok(output)
    }

    fn branch(self) -> core::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            Flow::Ok(v) => ControlFlow::Continue(v),
            Flow::Err(e) => ControlFlow::Break(Flow::Err(e)),
            Flow::Eff(e) => ControlFlow::Break(Flow::Eff(e)),
        }
    }
}

impl<T, E, F: From<E>> FromResidual<Flow<!, E>> for Flow<T, F> {
    fn from_residual(residual: Flow<!, E>) -> Self {
        match residual {
            Flow::Ok(_infallible) => _infallible,
            Flow::Err(e) => Flow::Err(e.into()),
            Flow::Eff(e) => Flow::Eff(e),
        }
    }
}

impl<T, E, F: From<E>> FromResidual<Result<Infallible, E>> for Flow<T, F> {
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        match residual {
            Result::Ok(_infallible) => match _infallible {},
            Result::Err(e) => Flow::Err(e.into()),
        }
    }
}

impl<T, E> Flow<T, E> {
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Flow<U, E> {
        match self {
            Flow::Ok(t) => Flow::Ok(f(t)),
            Flow::Err(e) => Flow::Err(e),
            Flow::Eff(e) => Flow::Eff(e),
        }
    }
    pub fn map_err<F, O: FnOnce(E) -> F>(self, f: O) -> Flow<T, F> {
        match self {
            Flow::Ok(t) => Flow::Ok(t),
            Flow::Err(e) => Flow::Err(f(e)),
            Flow::Eff(e) => Flow::Eff(e),
        }
    }
    pub fn shift(self) -> Flow<Result<T, E>> {
        match self {
            Flow::Ok(t) => Flow::Ok(Ok(t)),
            Flow::Err(e) => Flow::Ok(Err(e)),
            Flow::Eff(e) => Flow::Eff(e),
        }
    }
}
