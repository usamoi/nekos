mod errors;
pub use self::errors::*;
mod karea;
pub use self::karea::*;
mod kmap;
pub use self::kmap::*;
mod kspace;
pub use self::kspace::*;

use crate::prelude::*;
use arch::memory::CONFIG;
use arch::paging::{PageTableToken, Template};
use common::basic::Singleton;

pub static TEMPLATE: Singleton<Template> = Singleton::new();
pub static SPACE: Singleton<KSpace> = Singleton::new();

extern "C" {
    static _text_start: LinkerSymbol;
    static _text_end: LinkerSymbol;
    static _rodata_start: LinkerSymbol;
    static _rodata_end: LinkerSymbol;
    static _debug_start: LinkerSymbol;
    static _debug_end: LinkerSymbol;
    static _tdata_start: LinkerSymbol;
    static _tdata_end: LinkerSymbol;
    static _data_start: LinkerSymbol;
    static _data_end: LinkerSymbol;
    static _bss_start: LinkerSymbol;
    static _bss_end: LinkerSymbol;
    static _uninit_start: LinkerSymbol;
    static _uninit_end: LinkerSymbol;
    static _bump_start: LinkerSymbol;
    static _bump_end: LinkerSymbol;
}

pub unsafe fn init_boot() {
    TEMPLATE.init(Template::new());
    let space = KSpace::new();
    CONFIG.bump_alloc(CONFIG.bump_ptr().align_to(4096) - CONFIG.bump_ptr());
    CONFIG.bump_disable();
    space.phys.map(&space.page_table);
    space.kernel.map(
        _text_start.as_usize(),
        _text_end.as_usize(),
        MapPermission::EO,
    );
    space.kernel.map(
        _rodata_start.as_usize(),
        _rodata_end.as_usize(),
        MapPermission::RO,
    );
    space.kernel.map(
        _debug_start.as_usize(),
        _debug_end.as_usize(),
        MapPermission::RO,
    );
    space.kernel.map(
        _tdata_start.as_usize(),
        _tdata_end.as_usize(),
        MapPermission::RO,
    );
    space.kernel.map(
        _data_start.as_usize(),
        _data_end.as_usize(),
        MapPermission::RW,
    );
    space.kernel.map(
        _bss_start.as_usize(),
        _bss_end.as_usize(),
        MapPermission::RW,
    );
    space.kernel.map(
        _uninit_start.as_usize(),
        _uninit_end.as_usize(),
        MapPermission::RW,
    );
    space.kernel.map(
        _bump_start.as_usize(),
        _bump_start.as_usize() + (CONFIG.bump_ptr() - CONFIG.bump_start()),
        MapPermission::RW,
    );
    SPACE.init(space);
    pt().switch();
}

pub fn pt() -> PageTableToken {
    SPACE.page_table.token()
}

pub unsafe fn init_start() {
    pt().switch();
}
