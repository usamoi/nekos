use crate::prelude::*;
use crossbeam::atomic::AtomicCell;

pub struct Config {
    start: AtomicCell<Option<PAddr>>,
    size: AtomicCell<Option<usize>>,
    bump_start: AtomicCell<Option<PAddr>>,
    bump_ptr: AtomicCell<Option<PAddr>>,
    bump_status: AtomicCell<bool>,
}

impl Config {
    const fn new() -> Config {
        Config {
            start: AtomicCell::new(None),
            size: AtomicCell::new(None),
            bump_start: AtomicCell::new(None),
            bump_ptr: AtomicCell::new(None),
            bump_status: AtomicCell::new(true),
        }
    }
    pub fn set_start(&self, start: PAddr) {
        let ans = self.start.swap(Some(start));
        assert!(ans.is_none(), "the value can be set only once");
    }
    pub fn set_size(&self, size: usize) {
        let ans = self.size.swap(Some(size));
        assert!(ans.is_none(), "the value can be set only once");
    }
    pub fn set_bump(&self, x: PAddr) {
        let ans_rs = self.bump_start.swap(Some(x));
        assert!(ans_rs.is_none(), "the value can be set only once");
        let ans_re = self.bump_ptr.swap(Some(x));
        assert!(ans_re.is_none(), "the value can be set only once");
    }
    pub fn get_start(&self) -> Option<PAddr> {
        self.start.load()
    }
    pub fn get_size(&self) -> Option<usize> {
        self.size.load()
    }
    pub fn get_end(&self) -> Option<PAddr> {
        Some(self.get_start()? + self.get_size()?)
    }
    pub fn get_bump_start(&self) -> Option<PAddr> {
        self.bump_start.load()
    }
    pub fn get_bump_ptr(&self) -> Option<PAddr> {
        self.bump_ptr.load()
    }
    pub fn start(&self) -> PAddr {
        self.get_start().expect("the value remains uninitialized")
    }
    pub fn size(&self) -> usize {
        self.get_size().expect("the value remains uninitialized")
    }
    pub fn end(&self) -> PAddr {
        self.get_end().expect("the value remains uninitialized")
    }
    pub fn bump_start(&self) -> PAddr {
        self.get_bump_start()
            .expect("the value remains uninitialized")
    }
    pub fn bump_ptr(&self) -> PAddr {
        self.get_bump_ptr()
            .expect("the value remains uninitialized")
    }
    pub fn bump_alloc(&self, size: usize) -> PAddr {
        loop {
            let x = self.bump_ptr();
            if x > self.end() {
                panic!("bump memory overflow");
            }
            if !self.bump_status.load() {
                panic!("cannot allocate bump memory");
            }
            if self
                .bump_ptr
                .compare_exchange(Some(x), Some(x + size))
                .is_ok()
            {
                break x;
            }
        }
    }
    pub fn bump_disable(&self) {
        self.bump_status
            .compare_exchange(true, false)
            .expect("the value can be set only once");
    }
}

pub static CONFIG: Config = Config::new();
