use crate::prelude::*;

impl Codomain for () {
    fn to_return_value(self) -> usize {
        0
    }
}

impl Codomain for usize {
    fn to_return_value(self) -> usize {
        self
    }
}

impl Codomain for isize {
    fn to_return_value(self) -> usize {
        self as usize
    }
}

impl Codomain for VAddr {
    fn to_return_value(self) -> usize {
        self.to_usize()
    }
}
