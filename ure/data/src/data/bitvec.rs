use bitvec::vec::BitVec;

use crate::data::Container;

#[derive(Debug, Default)]
pub struct BitData {
    inner: BitVec,
    default: bool,
}

impl Container for BitData {
    fn swap_delete(&mut self, indices: &[usize]) {
        for &index in indices {
            self.inner.swap_remove(index);
        }
    }
    fn new(&mut self, num: usize) {
        self.inner.resize(self.inner.len() + num, self.default);
    }
}
