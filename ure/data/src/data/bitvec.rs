use std::{any::TypeId, marker::PhantomData, ops::Index};

use bitvec::{
    order::{BitOrder, Lsb0},
    ptr::BitPtr,
    slice::BitSlice,
    store::BitStore,
    vec::BitVec,
};

use crate::data::{ValidIndex, DataAny, DataSlice, DataSpecific};

impl DataAny for BitVec {
    fn inner_type(&self) -> TypeId {
        TypeId::of::<bool>()
    }
    fn reserve(&mut self, additional: usize) {
        self.reserve(additional);
    }
}
impl DataSpecific for BitVec {
    type Inner = bool;
    type Slice = UnboundBitSlice;

    fn slice_ref<'a>(&'a self) -> &'a Self::Slice {
        UnboundBitSlice::from_slice(self)
    }
    fn slice_mut<'a>(&'a mut self) -> &'a mut Self::Slice {
        UnboundBitSlice::from_slice_mut(self)
    }
    fn new_data() -> Self {
        Self::new()
    }
}

// Slice struct
// ------------------------------------------------------
#[repr(transparent)]
pub struct UnboundBitSlice<T = usize, O = Lsb0>
where
    T: BitStore,
    O: BitOrder,
{
    first: T,
    _order: PhantomData<O>,
}
impl<T, O> UnboundBitSlice<T, O>
where
    T: BitStore,
    O: BitOrder,
{
    pub fn from_slice(slice: &BitSlice<T, O>) -> &Self {
        unsafe { (slice.as_bitptr().pointer() as *const UnboundBitSlice<T, O>).as_ref_unchecked() }
    }
    pub fn from_slice_mut(slice: &mut BitSlice) -> &mut Self {
        unsafe {
            (slice.as_mut_bitptr().pointer() as *mut UnboundBitSlice<T, O>).as_mut_unchecked()
        }
    }
    pub fn set(&mut self, index: usize, value: bool) {
        unsafe {
            let ptr: BitPtr<bitvec::ptr::Mut, T, O> = BitPtr::from_mut(std::mem::transmute(self));
            ptr.add(index).write(value);
        }
    }
}
impl<T, O> Index<usize> for UnboundBitSlice<T, O>
where
    T: BitStore,
    O: BitOrder,
{
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe {
            let ptr: BitPtr<bitvec::ptr::Const, T, O> = BitPtr::from_ref(std::mem::transmute(self));
            if ptr.add(index).read() { &true } else { &false }
        }
    }
}

// Slice impl
// ------------------------------------------------------
impl DataSlice<bool> for UnboundBitSlice {
    fn get_data<'a>(&'a self, index: ValidIndex<'a>) -> &'a bool {
        &self[index.into()]
    }
    fn set_data<'a>(&'a mut self, index: ValidIndex<'a>, value: bool) {
        self.set(index.into(), value);
    }
}
