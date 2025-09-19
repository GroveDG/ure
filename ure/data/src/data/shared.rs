use std::any::{Any, TypeId};

use crate::data::{DataAny, DataSlice, DataSpecific};

pub struct Shared<T: Any> {
    inner: Option<T>,
}

impl<T: Any> DataAny for Shared<T> {
    fn inner_type(&self) -> std::any::TypeId {
        TypeId::of::<T>()
    }
    fn reserve(&mut self, _: usize) {}
}

impl<T: Any> DataSpecific for Shared<T> {
    type Inner = T;
    type Slice = Self;

    fn slice_ref<'a: 'b, 'b>(&'a self) -> (super::Mooring<'a>, &'b Self::Slice) {
        (None, self)
    }

    fn slice_mut<'a: 'b, 'b>(&'a mut self) -> (super::Mooring<'a>, &'b mut Self::Slice) {
        (None, self)
    }

    fn new_data() -> Self {
        Self { inner: None }
    }
}

impl<T: Any> DataSlice<T> for Shared<T> {
    fn get_data<'a>(&'a self, _: super::ValidIndex<'a>) -> &'a T {
        self.inner.as_ref().unwrap()
    }

    fn set_data<'a>(&'a mut self, _: super::ValidIndex<'a>, value: T) {
        self.inner.replace(value);
    }
}
