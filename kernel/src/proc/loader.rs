use crate::prelude::*;
use proc::vmm::UserSpace;
use proc::vmm::*;
use user::objects::memory::Memory;
use user::objects::memory::*;
use zelf::elf::{Elf, ElfType};
use zelf::program::{ParseProgramError, ParseProgramsError};
use zelf::program::{Program, ProgramFlags, ProgramType, Programs};
use zelf::{Class, Data, Version};

#[derive(Debug)]
pub enum LoadError {
    NotFound,
    BadElf,
    BadAbi,
    BadPlatform,
    OutOfMemory,
    NotSupported,
    BadAddress,
    SegmentOfUndersizeAlign,
    SegmentOfZeroSize,
    SegmentOfOverlap,
    SegmentOfBadLayout,
    AlignNotSupported,
    PermissionNotSupported,
}

impl From<ParseProgramsError> for LoadError {
    fn from(_: ParseProgramsError) -> Self {
        Self::BadElf
    }
}

impl From<ParseProgramError> for LoadError {
    fn from(_: ParseProgramError) -> Self {
        Self::BadElf
    }
}

fully!(MemoryCreateError, LoadError;
    OutOfMemory => OutOfMemory,
    UndersizeAlign => SegmentOfUndersizeAlign
);

fully!(AreaMapError, LoadError;
    ZeroSize => SegmentOfZeroSize,
    OutOfRange => NotSupported,
    Overlapping => SegmentOfOverlap,
    BadAddress => BadAddress,
    AlignNotSupported => AlignNotSupported,
    PermissionNotSupported => PermissionNotSupported
);

pub struct Image {
    pub space: Arc<UserSpace>,
    pub pc: VAddr,
    pub tls: Option<ImageTls>,
}

pub struct ImageTls {
    pub layout: MapLayout,
    pub content: Box<[u8]>,
}

pub fn load(name: &str) -> Result<Image, LoadError> {
    use LoadError::*;
    let input = fs::memfs::memfs().read(name).ok_or(NotFound)?;
    let elf = match Elf::parse(input).map_err(|_| BadElf)? {
        Elf::Little64(e) => e,
        _ => return Err(BadPlatform),
    };
    let header = elf.header();
    let ident = header.ident();
    let mut load = Image {
        space: UserSpace::new(),
        pc: VAddr::new(elf.header().entry() as usize),
        tls: None,
    };
    ensure!(ident.class() == Class::Class64, BadPlatform);
    ensure!(ident.data() == Data::Little, BadPlatform);
    ensure!(ident.version() == Version::One, BadPlatform);
    ensure!(ident.os_abi() == 0, BadAbi);
    ensure!(ident.abi_version() == 0, BadAbi);
    ensure!(header.machine() == P::ABI_ELF_ABI, BadPlatform);
    ensure!(header.typa() == ElfType::Exec, BadAbi);
    let programs = Programs::parse(elf)?.ok_or(BadElf)?;
    for index in 0..programs.num() {
        let program = Program::parse(programs, index).unwrap()?;
        let header = program.header();
        match header.typa() {
            ProgramType::Load => {
                let vaddr = VAddr::new(header.vaddr() as usize);
                let size = header.memsz() as usize;
                let align = header.align() as usize;
                let layout = MapLayout::new(size, align).ok_or(SegmentOfBadLayout)?;
                let memory = Memory::create(layout)?;
                let permission = Permission {
                    read: header.flags() & ProgramFlags::READ != 0.into(),
                    write: header.flags() & ProgramFlags::WRITE != 0.into(),
                    execute: header.flags() & ProgramFlags::EXECUTE != 0.into(),
                };
                load.space.root.map(vaddr, memory.clone(), permission)?;
                memory.write(0, program.content());
            }
            ProgramType::Tls => {
                if load.tls.is_some() {
                    return Err(BadAbi);
                }
                let size = header.memsz() as usize;
                let align = header.align() as usize;
                let layout = MapLayout::new(size, align).ok_or(SegmentOfBadLayout)?;
                load.tls = Some(ImageTls {
                    layout,
                    content: program.content().to_vec().into_boxed_slice(),
                });
            }
            _ => return Err(NotSupported),
        }
    }
    Ok(load)
}
