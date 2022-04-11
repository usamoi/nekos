use core::cell::UnsafeCell;
use core::lazy::OnceCell;
use core::ops::Deref;
use spin::Once as SyncOnceCell;

pub struct SingletonCell<T>(SyncOnceCell<T>);

impl<T> SingletonCell<T> {
    pub const fn new() -> SingletonCell<T> {
        SingletonCell(SyncOnceCell::new())
    }
    pub fn initialize(&self, t: T) {
        self.0.call_once(|| t);
    }
    pub fn maybe(&self) -> Option<&T> {
        self.0.get()
    }
}

impl<T> Deref for SingletonCell<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.0.get().unwrap()
    }
}

pub struct LocalSingletonCell<T>(OnceCell<T>);

impl<T> LocalSingletonCell<T> {
    pub const fn new() -> LocalSingletonCell<T> {
        LocalSingletonCell(OnceCell::new())
    }
    pub fn initialize(&self, t: T) {
        self.0.set(t).ok().expect("already initialized");
    }
    pub fn maybe(&self) -> Option<&T> {
        self.0.get()
    }
}

impl<T> Deref for LocalSingletonCell<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.0.get().unwrap()
    }
}

// Voliate cell's associated functions are safe
// because there are no safe ways to create a reference to a voliate cell

#[derive(Debug)]
#[repr(transparent)]
pub struct VolCell<T: Copy>(UnsafeCell<T>);

impl<T: Copy> VolCell<T> {
    pub fn write(&self, x: T) {
        unsafe { self.0.get().write_volatile(x) }
    }
    pub fn read(&self) -> T {
        unsafe { self.0.get().read_volatile() }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct VolWCell<T: Copy>(UnsafeCell<T>);

impl<T: Copy> VolWCell<T> {
    pub fn write(&self, x: T) {
        unsafe { self.0.get().write_volatile(x) }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct VolRCell<T: Copy>(UnsafeCell<T>);

impl<T: Copy> VolRCell<T> {
    pub fn read(&self) -> T {
        unsafe { self.0.get().read_volatile() }
    }
}
