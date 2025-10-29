use std::{any::Any, hash::Hash, ops::Range};

use bitvec::{slice::BitSlice, vec::BitVec};
use indexmap::IndexSet;
use one_or_many::OneOrMany;

pub trait Container: Any {
    type Ref<'a>;
    type Mut<'a>;
    type Item;

    fn as_ref(&self) -> Self::Ref<'_>;
    fn as_mut(&mut self) -> Self::Mut<'_>;
    fn delete(&mut self, range: Range<usize>);
    fn push(&mut self, item: Self::Item);
}
pub trait ContainerDefault: Container {
    fn new(&mut self, range: Range<usize>);
}
impl Container for () {
    type Ref<'a> = ();
    type Mut<'a> = ();
    type Item = ();

    fn as_ref(&self) -> Self::Ref<'_> {}
    fn as_mut(&mut self) -> Self::Mut<'_> {}
    fn delete(&mut self, _: Range<usize>) {}
    fn push(&mut self, _: Self::Item) {}
}
impl ContainerDefault for () {
    fn new(&mut self, _: Range<usize>) {}
}
#[derive(Debug, Default)]
pub struct One<T: 'static>(pub T);
impl<T: 'static> Container for One<T> {
    type Ref<'a> = &'a T;
    type Mut<'a> = &'a mut T;
    type Item = T;

    fn as_ref(&self) -> Self::Ref<'_> {
        &self.0
    }
    fn as_mut(&mut self) -> Self::Mut<'_> {
        &mut self.0
    }
    fn delete(&mut self, _: Range<usize>) {}
    fn push(&mut self, _: Self::Item) {}
}
impl<T: 'static> ContainerDefault for One<T> {
    fn new(&mut self, _: Range<usize>) {}
}
impl<T: 'static> Container for Option<T> {
    type Ref<'a> = Option<&'a T>;
    type Mut<'a> = Option<&'a mut T>;
    type Item = T;

    fn as_ref(&self) -> Self::Ref<'_> {
        self.as_ref()
    }
    fn as_mut(&mut self) -> Self::Mut<'_> {
        self.as_mut()
    }
    fn delete(&mut self, _: Range<usize>) {}
    fn push(&mut self, _: Self::Item) {}
}
impl<T: 'static> ContainerDefault for Option<T> {
    fn new(&mut self, _: Range<usize>) {}
}
impl<T: 'static> Container for Vec<T> {
    type Ref<'a> = &'a [T];
    type Mut<'a> = &'a mut [T];
    type Item = T;

    fn as_ref(&self) -> Self::Ref<'_> {
        self
    }
    fn as_mut(&mut self) -> Self::Mut<'_> {
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
impl<T: 'static + Default> ContainerDefault for Vec<T> {
    fn new(&mut self, range: Range<usize>) {
        for _ in range {
            self.push(Default::default());
        }
    }
}
impl<T: 'static + Hash + Eq> Container for IndexSet<T> {
    type Ref<'a> = &'a IndexSet<T>; // TODO: prevent structural modifications
    type Mut<'a> = &'a mut IndexSet<T>;
    type Item = T;

    fn as_ref(&self) -> Self::Ref<'_> {
        self
    }
    fn as_mut(&mut self) -> Self::Mut<'_> {
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
impl<T: 'static + Default + Hash + Eq> ContainerDefault for IndexSet<T> {
    fn new(&mut self, range: Range<usize>) {
        for _ in range {
            self.insert(Default::default());
        }
    }
}
impl<T: 'static> Container for OneOrMany<T> {
    type Ref<'a> = &'a [T];
    type Mut<'a> = &'a mut [T];
    type Item = T;

    fn as_ref(&self) -> Self::Ref<'_> {
        self.as_slice()
    }
    fn as_mut(&mut self) -> Self::Mut<'_> {
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
impl<T: 'static + Default> ContainerDefault for OneOrMany<T> {
    fn new(&mut self, range: Range<usize>) {
        let OneOrMany::Many(items) = self else {
            return;
        };
        for _ in range {
            items.push(Default::default());
        }
    }
}
impl Container for BitVec {
    type Ref<'a> = &'a BitSlice;
    type Mut<'a> = &'a mut BitSlice;
    type Item = bool;

    fn as_ref(&self) -> Self::Ref<'_> {
        self
    }
    fn as_mut(&mut self) -> Self::Mut<'_> {
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
impl ContainerDefault for BitVec {
    fn new(&mut self, range: Range<usize>) {
        for _ in range {
            self.push(Default::default());
        }
    }
}
