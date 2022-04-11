use alloc::collections::BTreeMap;
use spin::Lazy;

pub struct MemFs {
    data: BTreeMap<&'static str, &'static [u8]>,
}

impl MemFs {
    pub fn new() -> Self {
        let mut data = BTreeMap::new();
        data.insert(
            "initproc",
            include_bytes!(env!("memfs_initproc")).as_slice(),
        );
        Self { data }
    }
    pub fn read(&self, index: &str) -> Option<&[u8]> {
        self.data.get(index).cloned()
    }
}

pub fn memfs() -> &'static MemFs {
    static MEMFS: Lazy<MemFs> = Lazy::new(MemFs::new);
    &MEMFS
}
