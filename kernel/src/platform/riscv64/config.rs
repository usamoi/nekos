use crate::prelude::*;

pub struct Config {}

impl Config {
    pub fn global(&self) -> Segment<VAddr> {
        Segment::new(VAddr::new(0xFFFFFFFFC0000000), None).unwrap()
    }
    pub fn user(&self) -> Segment<VAddr> {
        by_points(
            VAddr::new(0x0000000000000000),
            VAddr::new(0x0000004000000000),
        )
        .unwrap()
    }
    pub fn phys(&self) -> Segment<VAddr> {
        by_points(
            VAddr::new(0x0000000000000000),
            VAddr::new(0x0000004000000000),
        )
        .unwrap()
    }
    pub fn kernel(&self) -> Segment<VAddr> {
        by_points(
            VAddr::new(0xFFFFFFC000000000),
            VAddr::new(0xFFFFFFC040000000),
        )
        .unwrap()
    }
    pub fn heap(&self) -> Segment<VAddr> {
        by_points(
            VAddr::new(0xFFFFFFC040000000),
            VAddr::new(0xFFFFFFFFC0000000),
        )
        .unwrap()
    }
}

pub static CONFIG: Config = Config {};
