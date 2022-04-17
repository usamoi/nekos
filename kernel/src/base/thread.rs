use core::ops::Deref;

#[derive(Debug, Clone, Copy)]
pub struct ThreadLocalRef<T: ?Sized + 'static>(&'static T);

impl<T: ?Sized + 'static> ThreadLocalRef<T> {
    pub unsafe fn new(reference: &T) -> ThreadLocalRef<T> {
        ThreadLocalRef(&*(reference as *const T))
    }
}

impl<T: ?Sized + 'static> !Send for ThreadLocalRef<T> {}
impl<T: ?Sized + 'static> !Sync for ThreadLocalRef<T> {}

impl<T: ?Sized + 'static> Deref for ThreadLocalRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
