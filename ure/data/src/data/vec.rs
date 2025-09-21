use std::any::Any;

use crate::data::Container;

pub struct VecData<T: Any> {
    inner: Vec<T>,
    f: fn() -> T,
}

impl<T: Any> Container for VecData<T> {
    fn swap_delete(&mut self, indices: &[usize]) {
        for &index in indices {
            self.inner.swap_remove(index);
        }
    }
    fn new(&mut self, num: usize) {
        self.inner.resize_with(self.inner.len() + num, &self.f);
    }
}

impl<T: Any + Default> Default for VecData<T> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            f: T::default,
        }
    }
}
