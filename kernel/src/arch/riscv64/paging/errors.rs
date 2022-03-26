#[derive(Debug)]
pub enum PageTableMapError {
    InvalidVAddr,
    InvalidPAddr,
    AlignNotSupported,
    PermissionNotSupported,
}

#[derive(Debug)]
pub enum PageTableUnmapError {
    InvalidVAddr,
    AlignNotSupported,
}
