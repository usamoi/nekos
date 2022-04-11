use core::ops::Deref;

#[derive(Debug, Clone, Copy)]
pub struct ThreadLocalRef<T: 'static>(&'static T);

impl<T: 'static> ThreadLocalRef<T> {
    pub unsafe fn new(reference: &T) -> ThreadLocalRef<T> {
        ThreadLocalRef(&*(reference as *const T))
    }
}

impl<T: 'static> !Send for ThreadLocalRef<T> {}
impl<T: 'static> !Sync for ThreadLocalRef<T> {}

impl<T: 'static> Deref for ThreadLocalRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
