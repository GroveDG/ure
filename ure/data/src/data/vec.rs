use std::{
    any::{Any, TypeId},
    ops::{Index, IndexMut},
};

use crate::data::{DataAny, DataSlice, DataSpecific, ValidIndex};

impl<T: Any> DataAny for Vec<T> {
    fn inner_type(&self) -> TypeId {
        TypeId::of::<T>()
    }
    fn reserve(&mut self, additional: usize) {
        self.reserve(additional);
    }
}
impl<T: Any> DataSpecific for Vec<T> {
    type Inner = T;
    type Slice = UnboundSlice<T>;

    fn slice_ref<'a>(&'a self) -> &'a Self::Slice {
        UnboundSlice::from_slice(self)
    }
    fn slice_mut<'a>(&'a mut self) -> &'a mut Self::Slice {
        UnboundSlice::from_slice_mut(self)
    }
    fn new_data() -> Self {
        Self::new()
    }
}

// Slice struct
// ------------------------------------------------------
#[repr(transparent)]
pub struct UnboundSlice<T> {
    first: T,
}
impl<T> UnboundSlice<T> {
    pub fn from_slice(slice: &[T]) -> &Self {
        unsafe { (slice.as_ptr() as *const UnboundSlice<T>).as_ref_unchecked() }
    }
    pub fn from_slice_mut(slice: &mut [T]) -> &mut Self {
        unsafe { (slice.as_ptr() as *mut UnboundSlice<T>).as_mut_unchecked() }
    }
}
impl<T> Index<usize> for UnboundSlice<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe {
            let ptr = std::mem::transmute::<&Self, &T>(self) as *const T;
            ptr.add(index * size_of::<T>()).as_ref_unchecked()
        }
    }
}
impl<T> IndexMut<usize> for UnboundSlice<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe {
            let ptr = std::mem::transmute::<&mut Self, &mut T>(self) as *mut T;
            ptr.add(index * size_of::<T>()).as_mut_unchecked()
        }
    }
}

// Slice impl
// ------------------------------------------------------
impl<T: Any> DataSlice<T> for UnboundSlice<T> {
    fn get_data<'a>(&'a self, index: ValidIndex<'a>) -> &'a T {
        &self[index.into()]
    }
    fn set_data<'a>(&'a mut self, index: ValidIndex<'a>, value: T) {
        self[index.into()] = value;
    }
}
