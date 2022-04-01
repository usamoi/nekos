mod buddy;

mod errors;
pub use self::errors::*;
mod line_box;
pub use self::line_box::*;
mod phys_box;
pub use self::phys_box::*;
mod points_box;
pub use self::points_box::*;

use crate::prelude::*;
use buddy::Buddy;
use common::basic::Singleton;
use spin::Mutex;

struct Frames {
    buddy: Mutex<Buddy<'static>>,
}

static FRAMES: Singleton<Frames> = Singleton::new();

pub unsafe fn init_global() {
    use arch::memory::CONFIG;
    let segment = by_points(CONFIG.start(), CONFIG.start() + CONFIG.size()).unwrap();
    let buffer_size = CONFIG.size() / 4096 * 2;
    let buffer_paddr = CONFIG.bump_alloc(buffer_size);
    let buffer_slice =
        core::slice::from_raw_parts_mut(buffer_paddr.to_usize() as *mut i8, buffer_size);
    let buddy_start = segment.start().to_usize().div_ceil(4096);
    let buddy_end = segment.end().map(|x| x.to_usize() >> 12);
    let buddy_segment = Segment::new(buddy_start, buddy_end).unwrap();
    let buddy = Buddy::new(buddy_segment, buffer_slice).unwrap();
    let allocator = Frames {
        buddy: Mutex::new(buddy),
    };
    let reserve_size = CONFIG.bump_ptr() - CONFIG.start();
    FRAMES.init(allocator);
    set(by_size(CONFIG.start(), reserve_size).unwrap(), true);
}

// todo: non-continuous alloc
pub fn alloc(layout: MapLayout) -> Result<PAddr, FramesAllocError> {
    use FramesAllocError::*;
    if layout.size() == 0 {
        return Ok(PAddr::new(layout.align()));
    }
    if layout.align() < 4096 {
        return Err(UndersizeAlign);
    }
    let mut buddy = FRAMES.buddy.lock();
    let paddr = buddy.alloc(layout.size() >> 12).out::<FramesAllocError>()?;
    Ok(PAddr::new(paddr << 12))
}

pub unsafe fn dealloc(paddr: PAddr, layout: MapLayout) {
    if layout.size() == 0 {
        assert_eq!(paddr, PAddr::new(layout.align()));
        return;
    }
    assert!(layout.align() >= 4096);
    assert!(layout.check(paddr.to_usize()));
    let mut buddy = FRAMES.buddy.lock();
    let addr = paddr.to_usize() >> 12;
    let size = layout.size() >> 12;
    buddy.dealloc(addr, size).unwrap();
}

unsafe fn set(segment: Segment<PAddr>, val: bool) {
    assert!(!segment.is_empty());
    assert!(segment.start().to_usize() % 4096 == 0);
    assert!(segment.wrapping_end().to_usize() % 4096 == 0);
    let mut buddy = FRAMES.buddy.lock();
    let start = segment.start().to_usize() >> 12;
    let end = segment.end().map(|x| x.to_usize() >> 12);
    buddy.set(Segment::new(start, end).unwrap(), val).unwrap();
}
