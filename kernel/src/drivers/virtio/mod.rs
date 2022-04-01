pub mod mmio;
pub mod queue;

use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u32)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
pub enum DeviceType {
    Invalid = 0,
    Network = 1,
    Block = 2,
    Console = 3,
    EntropySource = 4,
    MemoryBallooningTrad = 5,
    IoMemory = 6,
    Rpmsg = 7,
    ScsiHost = 8,
    Transport9P = 9,
    Mac80211Wlan = 10,
    RprocSerial = 11,
    VirtIoCaif = 12,
    MemoryBalloon = 13,
    Gpu = 16,
    TimerClock = 17,
    Input = 18,
    Socket = 19,
    Crypto = 20,
    SignalDistroModule = 21,
    Pstore = 22,
    Iommu = 23,
    Memory = 24,
}

#[derive(Debug, Eq, PartialEq)]
pub enum VirtIOError {
    BadConfig,
    BadMagic,
    BadVersion,
}

pub type VirtIOResult<T = ()> = Result<T, VirtIOError>;
