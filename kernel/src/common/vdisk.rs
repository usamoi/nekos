use alloc::collections::BTreeMap;
use spin::Lazy;

static FILES: Lazy<BTreeMap<usize, &'static [u8]>> = Lazy::new(|| {
    let mut files = BTreeMap::new();
    files.insert(
        0,
        &include_bytes!(
            "../../../crates/nekos-initproc/target/riscv64gc-unknown-none-elf/debug/nekos-initproc"
        )[..],
    );
    files
});

pub fn read(index: usize) -> Option<&'static [u8]> {
    FILES.get(&index).cloned()
}
