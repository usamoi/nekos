use crate::prelude::*;
use core::any::Any;
use core::cmp::Ordering;
use core::marker::PhantomData;
use core::num::NonZeroU32;

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

impl<T: Object + ?Sized> Handle<T> {
    pub fn new(object: Arc<T>) -> Handle<T> {
        Handle { object }
    }
    pub fn downcast<U: Object>(self) -> Option<Handle<U>> {
        let object = Arc::downcast::<U>(self.object.upcast()).ok()?;
        Some(Handle { object })
    }
}

impl<T: HandleUpcast> Handle<T> {
    pub fn upcast(self) -> Handle {
        HandleUpcast::upcast(self)
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
        PartialEq::eq(&Arc::as_ptr(&self.object), &Arc::as_ptr(&other.object))
    }
}

impl<T: Object + ?Sized> Eq for Handle<T> {}

impl<T: Object + ?Sized> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&Arc::as_ptr(&self.object), &Arc::as_ptr(&other.object))
    }
}

impl<T: Object + ?Sized> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        Ord::cmp(&Arc::as_ptr(&self.object), &Arc::as_ptr(&other.object))
    }
}

pub trait HandleUpcast: Object {
    fn upcast(this: Handle<Self>) -> Handle;
}

impl<T: Object + Sized> HandleUpcast for T {
    fn upcast(this: Handle<Self>) -> Handle {
        Handle {
            object: this.object,
        }
    }
}

impl HandleUpcast for dyn Object {
    fn upcast(this: Handle<Self>) -> Handle {
        this
    }
}

pub type Arguments = [usize; 6];

pub trait Domain: Sized {
    fn from_arguments(env: &Environment, x: usize) -> EffSys<Self>;
}

pub trait Codomain: Sized {
    fn to_return_value(self) -> usize;
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

#[async_trait::async_trait]
pub trait Syscalls<const CODE: u32> {
    type Domain0: Domain = ();
    type Domain1: Domain = ();
    type Domain2: Domain = ();
    type Domain3: Domain = ();
    type Domain4: Domain = ();
    type Domain5: Domain = ();
    type Codomain: Codomain = ();
    async fn syscall(env: &Environment, args: syscall_domain!()) -> EffSys<Self::Codomain>;
}

pub macro impl_syscall($name:ident, $code:literal) {
    impl Syscall {
        pub const $name: u32 = $code;
    }
}

pub macro syscall_domain() {
    (
        Self::Domain0,
        Self::Domain1,
        Self::Domain2,
        Self::Domain3,
        Self::Domain4,
        Self::Domain5,
    )
}
