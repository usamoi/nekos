use super::*;
use core::fmt::Debug;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use volatile::{ReadOnly, ReadWrite, WriteOnly};

pub struct MMIO {
    regs: &'static mut MMIORegisters,
    #[allow(dead_code)]
    size: usize,
    queue_sel: Option<u32>,
}

impl MMIO {
    pub unsafe fn acknowledge(addr: usize, size: usize) -> VirtIOResult<MMIO> {
        let regs = &mut *(addr as *mut MMIORegisters);
        if regs.magic_value.read() != 0x74726976 {
            return Err(VirtIOError::BadMagic);
        }
        if regs.version.read() != 0x2 {
            return Err(VirtIOError::BadVersion);
        }
        let mut mmio = MMIO {
            regs,
            size,
            queue_sel: None,
        };
        mmio.set_status(DeviceStatus::Acknowledge);
        Ok(mmio)
    }

    pub fn on_initializing(&mut self, driver_features: u64) {
        self.regs.status.write(DeviceStatus::Driver);
        self.set_driver_features(driver_features);
        self.regs.status.write(DeviceStatus::FeaturesOk);
    }

    pub fn on_initialized(&mut self) {
        self.regs.status.write(DeviceStatus::DriverOk);
    }

    pub fn set_status(&mut self, x: DeviceStatus) {
        self.regs.status.write(x);
    }

    pub fn device_id(&self) -> DeviceType {
        self.regs.device_id.read().try_into().unwrap()
    }

    pub fn device_features(&mut self) -> u64 {
        self.regs.device_features_sel.write(0);
        let low = self.regs.device_features.read();
        self.regs.device_features_sel.write(1);
        let high = self.regs.device_features.read();
        ((high as u64) << 32) | low as u64
    }

    pub fn set_driver_features(&mut self, features: u64) {
        self.regs.device_features_sel.write(0);
        self.regs.driver_features.write(features as u32);
        self.regs.device_features_sel.write(1);
        self.regs.driver_features.write((features >> 32) as u32);
    }

    pub fn set_notify(&mut self, queue: u32) {
        self.regs.queue_notify.write(queue);
    }

    pub fn config_generation(&self) -> u32 {
        self.regs.config_generation.read()
    }

    pub fn config(&self) -> *mut u8 {
        unsafe { (self as *const _ as *mut u8).offset(100) }
    }

    fn set_queue_sel(&mut self, x: u32) {
        if let Some(queue_sel) = self.queue_sel {
            if queue_sel == x {
                return;
            }
        }
        self.regs.queue_sel.write(x);
        self.queue_sel = Some(x);
    }

    pub fn queue(&mut self, x: u32) -> MMIOQueue<'_> {
        MMIOQueue { i: x, mmio: self }
    }
}

impl Debug for MMIO {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VirtIOMMIO")
    }
}

pub struct MMIOQueue<'a> {
    i: u32,
    mmio: &'a mut MMIO,
}

impl MMIOQueue<'_> {
    pub fn queue_num_max(&mut self) -> u32 {
        self.mmio.set_queue_sel(self.i);
        self.mmio.regs.queue_num_max.read()
    }

    pub fn set_queue_num(&mut self, num: u32) {
        self.mmio.set_queue_sel(self.i);
        self.mmio.regs.queue_num.write(num);
    }

    pub fn queue_ready(&mut self) -> bool {
        self.mmio.set_queue_sel(self.i);
        let r = self.mmio.regs.queue_ready.read();
        assert!(r == 0 || r == 1);
        r == 1
    }

    pub fn set_queue_ready(&mut self, x: bool) {
        self.mmio.set_queue_sel(self.i);
        self.mmio.regs.queue_ready.write(x as u32);
    }

    pub fn set_descriptor(&mut self, x: u64) {
        self.mmio.set_queue_sel(self.i);
        self.mmio.regs.queue_desc_low.write(x as u32);
        self.mmio.regs.queue_desc_high.write((x >> 32) as u32);
    }

    pub fn set_avail(&mut self, x: u64) {
        self.mmio.set_queue_sel(self.i);
        self.mmio.regs.queue_driver_low.write(x as u32);
        self.mmio.regs.queue_driver_high.write((x >> 32) as u32);
    }

    pub fn set_used(&mut self, x: u64) {
        self.mmio.set_queue_sel(self.i);
        self.mmio.regs.queue_device_low.write(x as u32);
        self.mmio.regs.queue_device_high.write((x >> 32) as u32);
    }
}

#[repr(C)]
pub struct MMIORegisters {
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

static_assertions::assert_eq_size!(MMIORegisters, [u8; 0x100]);

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
