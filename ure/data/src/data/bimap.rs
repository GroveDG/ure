use std::{any::{Any, TypeId}, fmt::Debug, hash::Hash};

use indexmap::IndexSet;

use crate::data::{DataAny, DataSlice, DataSpecific, Mooring};

impl<T: Any + Hash + Eq + Debug> DataAny for IndexSet<T> {
    fn inner_type(&self) -> TypeId {
        TypeId::of::<T>()
    }
    fn reserve(&mut self, additional: usize) {
        self.reserve(additional);
    }
}
impl<T: Any + Hash + Eq + Debug> DataSpecific for IndexSet<T> {
    type Inner = T;
    type Slice = BimapSlice<T>;

    fn slice_ref<'a: 'b, 'b>(&'a self) -> (Mooring<'a>, &'b Self::Slice) {
        (None, BimapSlice::from_bimap(self))
    }
    fn slice_mut<'a: 'b, 'b>(&'a mut self) -> (Mooring<'a>, &'b mut Self::Slice) {
        (None, BimapSlice::from_bimap_mut(self))
    }
    fn new_data() -> Self {
        Self::new()
    }
}
// Slice struct
// ------------------------------------------------------
#[repr(transparent)]
pub struct BimapSlice<T> {
    inner: IndexSet<T>
}
impl<T: Any> BimapSlice<T> {
    fn from_bimap<'a>(inner: &'a IndexSet<T>) -> &'a Self {
        unsafe { std::mem::transmute(inner) }
    }
    fn from_bimap_mut<'a>(inner: &'a mut IndexSet<T>) -> &'a mut Self {
        unsafe { std::mem::transmute(inner) }
    }
}

// Slice impl
// ------------------------------------------------------
impl<T: Any + Hash + Eq + Debug> DataSlice<T> for BimapSlice<T> {
    fn get_data<'a>(&'a self, index: super::ValidIndex<'a>) -> &'a T {
        self.inner.get_index(index.inner()).unwrap()
    }

    fn set_data<'a>(&'a mut self, index: super::ValidIndex<'a>, value: T) {
        self.inner.replace_index(index.inner(), value).unwrap();
    }
}