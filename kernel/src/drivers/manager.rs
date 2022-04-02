use crate::prelude::*;
use drivers::virtio::DeviceType;
use spin::Mutex;

#[derive(Debug, Clone)]
pub struct Record {
    pub addr: PAddr,
    pub size: usize,
    pub int: Vec<usize>,
}

static RECORDS: Mutex<Vec<Record>> = Mutex::new(Vec::new());

pub fn register(addr: PAddr, size: usize, int: Vec<usize>) {
    RECORDS.lock().push(Record { addr, size, int });
}

pub fn init_global() {
    let records = RECORDS.lock().clone();
    for record in records {
        let addr = record.addr.to_usize() as *mut u8;
        let size = record.size;
        let mmio = unsafe { drivers::virtio::mmio::MMIO::new(addr, size) };
        match mmio {
            Ok(mmio) => match mmio.device() {
                DeviceType::Invalid => (),
                DeviceType::Block => {
                    info!("find a block device");
                }
                fallback => {
                    warn!("no driver for the MMIO device {:?}", fallback);
                }
            },
            Err(e) => warn!("failed to acknowledge the device, reason = {:?}", e),
        }
    }
}
