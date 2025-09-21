use std::{any::Any, hash::Hash};

use indexmap::IndexSet;

use crate::data::Container;


pub struct BiMap<T: Any + Hash + Eq> {
    inner: IndexSet<T>,
    f: fn(&Self) -> T,
}

impl<T: Any + Hash + Eq> Container for BiMap<T> {
    fn swap_delete(&mut self, indices: &[usize]) {
        for &index in indices {
            self.inner.swap_remove_index(index);
        }
    }
    fn new(&mut self, num: usize) {
        for _ in 0..num {
            let t = (self.f)(&self);
            self.inner.insert(t);
        }
    }
}