mod errors;
pub use self::errors::*;

use crate::prelude::*;

pub struct Pipe {}

impl Pipe {
    pub fn create() -> Arc<Pipe> {
        todo!()
    }
    pub fn read(&self, _buf: &mut [u8]) -> usize {
        todo!()
    }
    pub fn write(&self, _buf: &[u8]) {
        todo!()
    }
}
