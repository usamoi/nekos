use crate::prelude::*;
use alloc::collections::BTreeMap;
use drivers::virtio::mmio::MMIO;
use drivers::virtio::queue::VirtQueue;
use drivers::virtio::DeviceType;
use mem::dma::{DmaAllocator, DmaBox};
use volatile::Volatile;

pub const QUEUE: u32 = 0;

#[derive(Debug)]
pub enum BlkError {
    NotABlk,
    NotSupported,
    BadConfig,
}

enum BlkSave {
    Read(
        DmaBox<BlkRequestHeader>,
        DmaBox<BlkStatus>,
        DmaBox<[u8; 512]>,
    ),
    Write(
        DmaBox<BlkRequestHeader>,
        DmaBox<BlkStatus>,
        DmaBox<[u8; 512]>,
    ),
}

#[derive(Debug)]
pub enum BlkReturn {
    Read(BlkStatus, DmaBox<[u8; 512]>),
    Write(BlkStatus, DmaBox<[u8; 512]>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlkToken(u16);

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum BlkStatus {
    Ok = 0,
    IOErr = 1,
    Unsupported = 2,
}

pub struct Blk {
    mmio: MMIO,
    queue: VirtQueue,
    saves: BTreeMap<BlkToken, BlkSave>,
}

impl Blk {
    pub fn new(mut mmio: MMIO) -> Result<Self, BlkError> {
        if mmio.device() != DeviceType::Block {
            return Err(BlkError::NotABlk);
        }
        let config_raw = mmio.config_data();
        if config_raw.len() < core::mem::size_of::<BlkConfig>() {
            return Err(BlkError::BadConfig);
        }
        let config = unsafe { &mut *(mmio.config_data().as_mut_ptr() as *mut BlkConfig) };
        mmio.init_ack();
        mmio.init_driver();
        mmio.init_features_ok(0);
        dbg!(config.blk_size.read());
        dbg!(config.capacity.read());
        let queue = VirtQueue::new(&mut mmio, QUEUE, 16).unwrap();
        mmio.init_driver_ok();
        Ok(Blk {
            mmio,
            queue,
            saves: BTreeMap::new(),
        })
    }

    pub fn interrupt_ack(&mut self) {
        self.mmio.interrupt_ack();
    }

    pub fn read(&mut self, sector: u64, buffer: DmaBox<[u8; 512]>) -> Result<BlkToken, BlkError> {
        self.mmio.queue_lock(QUEUE);
        let request = Box::new_in(
            BlkRequestHeader::new(BlkRequestType::In, sector),
            DmaAllocator,
        );
        let status = Box::new_in(BlkStatus::Ok, DmaAllocator);
        let idx = self
            .queue
            .push(
                &[request.as_raw_dma_ref()],
                &[buffer.as_raw_dma_mut(), status.as_raw_dma_mut()],
            )
            .unwrap();
        let token = BlkToken(idx);
        self.saves
            .insert(token, BlkSave::Read(request, status, buffer));
        self.mmio.queue_unlock(QUEUE);
        self.mmio.queue_notify(QUEUE);
        Ok(token)
    }

    pub fn write(&mut self, sector: u64, buffer: DmaBox<[u8; 512]>) -> Result<BlkToken, BlkError> {
        self.mmio.queue_lock(QUEUE);
        let request = Box::new_in(
            BlkRequestHeader::new(BlkRequestType::Out, sector),
            DmaAllocator,
        );
        let status = Box::new_in(BlkStatus::Ok, DmaAllocator);
        let idx = self
            .queue
            .push(
                &[request.as_raw_dma_ref(), buffer.as_raw_dma_ref()],
                &[status.as_raw_dma_mut()],
            )
            .unwrap();
        let token = BlkToken(idx);
        self.saves
            .insert(token, BlkSave::Write(request, status, buffer));
        self.mmio.queue_unlock(QUEUE);
        self.mmio.queue_notify(QUEUE);
        Ok(token)
    }

    pub fn poll(&mut self) -> Option<(BlkToken, BlkReturn)> {
        self.mmio.queue_lock(QUEUE);
        let token = self.queue.pop().map(|(id, _)| BlkToken(id))?;
        let save = self.saves.remove(&token).unwrap();
        let ret = match save {
            BlkSave::Read(_, status, buffer) => BlkReturn::Read(*status, buffer),
            BlkSave::Write(_, status, buffer) => BlkReturn::Write(*status, buffer),
        };
        self.mmio.queue_unlock(QUEUE);
        Some((token, ret))
    }
}

#[repr(u32)]
#[derive(Debug)]
enum BlkRequestType {
    In = 0,
    Out = 1,
    #[allow(dead_code)]
    Flush = 4,
    #[allow(dead_code)]
    Discard = 11,
    #[allow(dead_code)]
    WriteZeroes = 13,
}

#[repr(C)]
#[derive(Debug)]
struct BlkRequestHeader {
    typa: BlkRequestType,
    _reserved: u32,
    sector: u64,
}

impl BlkRequestHeader {
    fn new(typa: BlkRequestType, sector: u64) -> Self {
        Self {
            typa,
            sector,
            _reserved: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
struct BlkGeometry {
    cylinders: Volatile<u16>,
    heads: Volatile<u8>,
    sectors: Volatile<u8>,
}

#[repr(C)]
#[derive(Debug)]
struct BlkTopology {
    physical_block_exp: Volatile<u8>,
    alignment_offset: Volatile<u8>,
    min_io_size: Volatile<u16>,
    opt_io_size: Volatile<u32>,
}

#[repr(C)]
#[derive(Debug)]
struct BlkConfig {
    capacity: Volatile<u64>,
    size_max: Volatile<u32>,
    seg_max: Volatile<u32>,
    geometry: BlkGeometry,
    blk_size: Volatile<u32>,
    topology: BlkTopology,
    writeback: Volatile<u8>,
    _reserved_0: [u8; 3],
    max_discard_sectors: Volatile<u32>,
    max_discard_seg: Volatile<u32>,
    discard_sector_alignment: Volatile<u32>,
    max_write_zeroes_sectors: Volatile<u32>,
    max_write_zeroes_seg: Volatile<u32>,
    write_zeroes_may_unmap: Volatile<u8>,
    _reserved_1: [u8; 3],
}
