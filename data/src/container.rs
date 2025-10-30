use std::{any::Any, hash::Hash, ops::Range};

use bitvec::{slice::BitSlice, vec::BitVec};
use indexmap::IndexSet;
use one_or_many::OneOrMany;

pub trait Container: Any {
    type Slice: ?Sized;
    type Item;

    fn as_ref(&self) -> &Self::Slice;
    fn as_mut(&mut self) -> &mut Self::Slice;
    fn delete(&mut self, range: Range<usize>);
    fn push(&mut self, item: Self::Item);
}
#[derive(Debug, Default)]
pub struct One<T: 'static>(pub T);
impl<T: 'static> Container for One<T> {
    type Slice = T;
    type Item = T;

    fn as_ref(&self) -> &Self::Slice {
        &self.0
    }
    fn as_mut(&mut self) -> &mut Self::Slice {
        &mut self.0
    }
    fn delete(&mut self, _: Range<usize>) {}
    fn push(&mut self, _: Self::Item) {}
}
impl<T: 'static> Container for Option<T> {
    type Slice = Self;
    type Item = T;

    fn as_ref(&self) -> &Self::Slice {
        self
    }
    fn as_mut(&mut self) -> &mut Self::Slice {
        self
    }
    fn delete(&mut self, _: Range<usize>) {}
    fn push(&mut self, _: Self::Item) {}
}
impl<T: 'static> Container for Vec<T> {
    type Slice = [T];
    type Item = T;

    fn as_ref(&self) -> &Self::Slice {
        self
    }
    fn as_mut(&mut self) -> &mut Self::Slice {
        self
    }
    fn delete(&mut self, range: Range<usize>) {
        for i in range {
            self.swap_remove(i);
        }
    }
    fn push(&mut self, item: Self::Item) {
        self.push(item);
    }
}
impl<T: 'static + Hash + Eq> Container for IndexSet<T> {
    type Slice = Self;
    type Item = T;

    fn as_ref(&self) -> &Self::Slice {
        self
    }
    fn as_mut(&mut self) -> &mut Self::Slice {
        self
    }
    fn delete(&mut self, range: Range<usize>) {
        for i in range {
            self.swap_remove_index(i);
        }
    }
    fn push(&mut self, item: Self::Item) {
        self.insert(item);
    }
}
impl<T: 'static> Container for OneOrMany<T> {
    type Slice = [T];
    type Item = T;

    fn as_ref(&self) -> &Self::Slice {
        self.as_slice()
    }
    fn as_mut(&mut self) -> &mut Self::Slice {
        self.as_mut_slice()
    }
    fn delete(&mut self, range: Range<usize>) {
        if let OneOrMany::Many(items) = self {
            for i in range {
                items.swap_remove(i);
            }
        }
    }
    fn push(&mut self, item: Self::Item) {
        if let OneOrMany::Many(items) = self {
            items.push(item);
        }
    }
}
impl Container for BitVec {
    type Slice = BitSlice;
    type Item = bool;

    fn as_ref(&self) -> &Self::Slice {
        self
    }
    fn as_mut(&mut self) -> &mut Self::Slice {
        self
    }
    fn delete(&mut self, range: Range<usize>) {
        for i in range {
            self.swap_remove(i);
        }
    }
    fn push(&mut self, item: Self::Item) {
        self.push(item);
    }
}