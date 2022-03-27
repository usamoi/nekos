use crate::prelude::*;
use core::alloc::Layout;

extern "C" {
    static _tdata_start: LinkerSymbol;
    static _tdata_end: LinkerSymbol;
    static _tbss_start: LinkerSymbol;
    static _tbss_end: LinkerSymbol;
}

pub unsafe fn init_local() {
    let tdata_size = _tdata_end.as_vaddr() - _tdata_start.as_vaddr();
    let tbss_size = _tbss_end.as_vaddr() - _tbss_start.as_vaddr();
    let tdata = core::slice::from_raw_parts(_tdata_start.as_ptr::<u8>(), tdata_size);
    let tls_size = tdata_size + tbss_size;
    let tls_layout = Layout::from_size_align(tls_size, 4096).unwrap();
    let tls = alloc::alloc::alloc(tls_layout);
    assert!(!tls.is_null());
    core::slice::from_raw_parts_mut(tls, tdata.len()).copy_from_slice(tdata);
    core::slice::from_raw_parts_mut(tls.add(tdata.len()), tbss_size).fill(0);
    arch::abi::set_thread_pointer(tls as usize);
}
