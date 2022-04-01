use crate::prelude::*;
use core::fmt::Debug;
use drivers::virtio::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use volatile::{ReadOnly, ReadWrite, WriteOnly};

pub struct MMIO {
    regs: &'static mut Registers,
    config: &'static mut [u8],
}

impl MMIO {
    pub unsafe fn new(addr: *mut u8, size: usize) -> VirtIOResult<MMIO> {
        use VirtIOError::*;
        let regs = &mut *(addr as *mut Registers);
        let config = core::slice::from_raw_parts_mut(addr, size)
            .get_mut(100..)
            .ok_or(BadConfig)?;
        if regs.magic_value.read() != 0x74726976 {
            return Err(BadMagic);
        }
        if regs.version.read() != 0x2 {
            return Err(BadVersion);
        }
        let mmio = MMIO { regs, config };
        mmio.regs.status.write(DeviceStatus::Acknowledge);
        Ok(mmio)
    }

    pub fn init(&mut self, features: u64) {
        self.regs.status.write(DeviceStatus::Driver);
        self.regs.device_features_sel.write(0);
        self.regs.driver_features.write(features as u32);
        self.regs.device_features_sel.write(1);
        self.regs.driver_features.write((features >> 32) as u32);
        self.regs.status.write(DeviceStatus::FeaturesOk);
        self.regs.status.write(DeviceStatus::DriverOk);
    }

    pub fn device(&self) -> DeviceType {
        self.regs.device_id.read().try_into().unwrap()
    }

    pub fn vendor(&self) -> u32 {
        self.regs.vendor_id.read()
    }

    pub fn features(&mut self) -> u64 {
        self.regs.device_features_sel.write(0);
        let low = self.regs.device_features.read();
        self.regs.device_features_sel.write(1);
        let high = self.regs.device_features.read();
        ((high as u64) << 32) | low as u64
    }

    pub fn status(&mut self) -> DeviceStatus {
        self.regs.status.read()
    }

    pub fn interrupt_status(&mut self) -> u32 {
        self.regs.interrupt_status.read()
    }

    pub fn interrupt_ack(&mut self) -> bool {
        let interrupt = self.regs.interrupt_status.read();
        if interrupt != 0 {
            self.regs.interrupt_ack.write(interrupt);
            true
        } else {
            false
        }
    }

    pub fn config_generation(&self) -> u32 {
        self.regs.config_generation.read()
    }

    pub fn config_data(&mut self) -> &mut [u8] {
        self.config
    }

    pub fn queue_select(&mut self, index: u32) {
        self.regs.queue_sel.write(index);
    }

    pub fn queue_max_size(&self) -> u32 {
        self.regs.queue_num_max.read()
    }

    pub fn queue_init(&mut self, size: u32, desc: u64, avail: u64, used: u64) {
        self.regs.queue_num.write(size);
        self.regs.queue_desc_low.write(desc as u32);
        self.regs.queue_desc_high.write((desc >> 32) as u32);
        self.regs.queue_driver_low.write(avail as u32);
        self.regs.queue_driver_high.write((avail >> 32) as u32);
        self.regs.queue_device_low.write(used as u32);
        self.regs.queue_device_high.write((used >> 32) as u32);
    }

    pub fn queue_wake(&mut self) {
        self.regs.queue_ready.write(1);
    }

    pub fn queue_notify(&mut self, value: u32) {
        self.regs.queue_notify.write(value);
    }
}

impl Debug for MMIO {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VirtIOMMIO")
    }
}

#[repr(C)]
pub struct Registers {
    magic_value: ReadOnly<u32>,
    version: ReadOnly<u32>,
    device_id: ReadOnly<u32>,
    vendor_id: ReadOnly<u32>,
    device_features: ReadOnly<u32>,
    device_features_sel: WriteOnly<u32>,
    _e1: [u32; 2],
    driver_features: WriteOnly<u32>,
    driver_features_sel: WriteOnly<u32>,
    _e2: [u32; 2],
    queue_sel: WriteOnly<u32>,
    queue_num_max: ReadOnly<u32>,
    queue_num: WriteOnly<u32>,
    _e3: [u32; 1],
    _s4: [u32; 1],
    queue_ready: ReadWrite<u32>,
    _e4: [u32; 2],
    queue_notify: WriteOnly<u32>,
    _e5: [u32; 3],
    interrupt_status: ReadOnly<u32>,
    interrupt_ack: WriteOnly<u32>,
    _e6: [u32; 2],
    status: ReadWrite<DeviceStatus>,
    _e7: [u32; 3],
    queue_desc_low: WriteOnly<u32>,
    queue_desc_high: WriteOnly<u32>,
    _e8: [u32; 2],
    queue_driver_low: WriteOnly<u32>,
    queue_driver_high: WriteOnly<u32>,
    _e9: [u32; 2],
    queue_device_low: WriteOnly<u32>,
    queue_device_high: WriteOnly<u32>,
    _ea: [u32; 2],
    _fb: [u32; 4],
    _fc: [u32; 4],
    _fd: [u32; 4],
    _fe: [u32; 4],
    _sf: [u32; 3],
    config_generation: ReadOnly<u32>,
}

static_assertions::assert_eq_size!(Registers, [u8; 0x100]);

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum DeviceStatus {
    Acknowledge = 1,
    Driver = 2,
    Failed = 128,
    FeaturesOk = 8,
    DriverOk = 4,
    DeviceNeedsReset = 64,
}
