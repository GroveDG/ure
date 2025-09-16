use std::{
    any::{Any, TypeId},
    borrow::{Borrow, BorrowMut},
    collections::HashMap,
    hash::{BuildHasher, Hash, Hasher, RandomState},
    ops::{Deref, DerefMut, Index, IndexMut, Range},
    ptr::NonNull,
};

use bitvec::{slice::BitSlice, vec::BitVec};
use hashbrown::HashTable;
use indexmap::IndexSet;

pub trait DataAny: Any {
    fn inner_type(&self) -> TypeId;
    fn reserve(&mut self, additional: usize);
}
impl dyn DataAny {
    pub fn is<D: DataGeneric<T>, T: Any>(&self) -> bool {
        (self as &dyn Any).is::<D>()
    }
    pub fn inner_is<T: Any>(&self) -> bool {
        self.inner_type() == TypeId::of::<T>()
    }
}

type GenericVtable = NonNull<()>;

pub struct DataBox {
    any: Box<dyn DataAny>,
    generic: GenericVtable,
}
impl Deref for DataBox {
    type Target = dyn DataAny;

    fn deref(&self) -> &Self::Target {
        self.any.as_ref()
    }
}
impl DerefMut for DataBox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.any.as_mut()
    }
}

impl DataBox {
    pub fn slice<'a, T: Any>(&'a self, range: Range<usize>) -> Option<Box<dyn DataSlice<T> + 'a>> {
        if self.inner_type() != TypeId::of::<T>() {
            return None;
        }
        let ptr = self.any.as_ref() as *const dyn DataAny;
        let typed: &dyn DataGeneric<T> = unsafe {
            std::ptr::from_raw_parts::<dyn DataGeneric<T>>(
                ptr as *const (),
                std::mem::transmute(self.generic),
            )
            .as_ref()?
        };
        Some(typed.boxed(range))
    }
    pub fn slice_mut<'a, T: Any>(
        &'a mut self,
        range: Range<usize>,
    ) -> Option<Box<dyn DataSliceMut<T> + 'a>> {
        if self.inner_type() != TypeId::of::<T>() {
            return None;
        }
        let ptr = self.any.as_mut() as *mut dyn DataAny;
        let typed: &mut dyn DataGeneric<T> = unsafe {
            std::ptr::from_raw_parts_mut::<dyn DataGeneric<T>>(
                ptr as *mut (),
                std::mem::transmute(self.generic),
            )
            .as_mut()?
        };
        Some(typed.boxed_mut(range))
    }
    pub fn downcast_ref<'a, D: DataSpecific<Inner = T>, T: Any>(
        &'a self,
        range: Range<usize>,
    ) -> Option<D::View<'a>> {
        let any: &dyn Any = self.any.as_ref();
        Some(any.downcast_ref::<D>()?.view(range))
    }
    pub fn downcast_mut<'a, D: DataSpecific<Inner = T>, T: Any>(
        &'a mut self,
        range: Range<usize>,
    ) -> Option<D::ViewMut<'a>> {
        let any: &mut dyn Any = self.any.as_mut();
        Some(any.downcast_mut::<D>()?.view_mut(range))
    }
}

pub trait DataGeneric<T: Any>: DataAny {
    fn boxed<'a>(&'a self, range: Range<usize>) -> Box<dyn DataSlice<T> + 'a>;
    fn boxed_mut<'a>(&'a mut self, range: Range<usize>) -> Box<dyn DataSliceMut<T> + 'a>;
}
impl<T: Any, S: DataSpecific<Inner = T> + DataAny> DataGeneric<T> for S {
    fn boxed<'a>(&'a self, range: Range<usize>) -> Box<dyn DataSlice<T> + 'a> {
        Box::new(self.view(range))
    }
    fn boxed_mut<'a>(&'a mut self, range: Range<usize>) -> Box<dyn DataSliceMut<T> + 'a> {
        Box::new(self.view_mut(range))
    }
}

pub trait DataSpecific: DataAny {
    type Inner: Any;
    type View<'a>: DataSlice<Self::Inner>;
    type ViewMut<'a>: DataSliceMut<Self::Inner>;

    fn view<'a>(&'a self, range: Range<usize>) -> Self::View<'a>;
    fn view_mut<'a>(&'a mut self, range: Range<usize>) -> Self::ViewMut<'a>;
    fn new_data() -> Self;
}

pub trait DataSlice<T: Any> {
    fn get_data<'a>(&'a self, index: usize) -> &'a T;
}
pub trait DataSliceMut<T: Any>: DataSlice<T> {
    fn set_data(&mut self, index: usize, value: T);
}

// ============================================================================
//                                     Vec
// ============================================================================
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
    type View<'a> = &'a [T];
    type ViewMut<'a> = &'a mut [T];

    fn view<'a>(&'a self, range: Range<usize>) -> &'a [T] {
        &self[range]
    }
    fn view_mut<'a>(&'a mut self, range: Range<usize>) -> &'a mut [T] {
        &mut self[range]
    }
    fn new_data() -> Self {
        Self::new()
    }
}

// Slice impl
// ------------------------------------------------------
impl<'s, T: Any> DataSlice<T> for &'s [T] {
    fn get_data<'a>(&'a self, index: usize) -> &'a T {
        &self[index]
    }
}
impl<'a, T: Any> DataSliceMut<T> for &'a mut [T] {
    fn set_data(&mut self, index: usize, value: T) {
        self[index] = value;
    }
}
impl<'s, T: Any> DataSlice<T> for &'s mut [T] {
    fn get_data<'a>(&'a self, index: usize) -> &'a T {
        &self[index]
    }
}

// ============================================================================
//                                   BitVec
// ============================================================================
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
    type View<'a> = &'a BitSlice;
    type ViewMut<'a> = &'a mut BitSlice;

    fn view<'a>(&'a self, range: Range<usize>) -> &'a BitSlice {
        &self[range]
    }
    fn view_mut<'a>(&'a mut self, range: Range<usize>) -> &'a mut BitSlice {
        &mut self[range]
    }
    fn new_data() -> Self {
        Self::new()
    }
}

// Slice impl
// ------------------------------------------------------
impl<'s> DataSlice<bool> for &'s BitSlice {
    fn get_data<'a>(&'a self, index: usize) -> &'a bool {
        &self[index]
    }
}
impl<'a> DataSliceMut<bool> for &'a mut BitSlice {
    fn set_data(&mut self, index: usize, value: bool) {
        self.set(index, value);
    }
}
impl<'s> DataSlice<bool> for &'s mut BitSlice {
    fn get_data<'a>(&'a self, index: usize) -> &'a bool {
        &self[index]
    }
}

// ============================================================================
//                                    Bimap
// ============================================================================
impl<T: Any + Hash + Eq> DataAny for IndexSet<T> {
    fn inner_type(&self) -> TypeId {
        TypeId::of::<T>()
    }
    fn reserve(&mut self, additional: usize) {
        self.reserve(additional);
    }
}
impl<T: Any + Hash + Eq> DataSpecific for IndexSet<T> {
    type Inner = T;
    type View<'a> = &'a IndexSet<T>;
    type ViewMut<'a> = BiMapSliceMut<'a, T>;

    fn view<'a>(&'a self, range: Range<usize>) -> Self::View<'a> {
        BiMapSlice {
            inner: self,
            start: range.start,
            len: range.len(),
        }
    }
    fn view_mut<'a>(&'a mut self, range: Range<usize>) -> Self::ViewMut<'a> {
        BiMapSliceMut {
            inner: self,
            start: range.start,
            len: range.len(),
        }
    }
    fn new_data() -> Self {
        Self::new()
    }
}
// Slice struct
// ------------------------------------------------------

// Slice impl
// ------------------------------------------------------
impl<'s, T: Any + Hash + Eq> DataSlice<T> for BiMapSlice<'s, T> {
    fn get_data<'a>(&'a self, index: usize) -> &'a T {
        &self.inner.vec[index]
    }
}
impl<'s, T: Any + Hash + Eq> DataSliceMut<T> for BiMapSliceMut<'s, T> {
    fn set_data(&mut self, index: usize, value: T) {
        self.set(value);
    }
}
impl<'s, T: Any + Hash + Eq> DataSlice<T> for BiMapSliceMut<'s, T> {
    fn get_data<'a>(&'a self, index: usize) -> &'a T {
        &self.inner.vec[index]
    }
}

// ============================================================================

pub type ComponentId = u64;
pub type New<T> = fn(&Data, &mut dyn DataSliceMut<T>);

pub struct Component {
    pub(crate) name: &'static str,
    pub(crate) inner_type: TypeId,
    pub(crate) id: ComponentId,
}
impl Component {
    pub const fn new<T: Any>(name: &'static str) -> Self {
        Self {
            name,
            inner_type: TypeId::of::<T>(),
            id: const_fnv1a_hash::fnv1a_hash_str_64(name),
        }
    }
}

#[derive(Default)]
pub struct Data {
    data: HashMap<ComponentId, DataBox>,
    len: usize,
    cap: usize,
}
impl Data {
    pub(crate) fn insert<T: Any>(&mut self, id: ComponentId, data: Box<dyn DataGeneric<T>>) {
        let ptr = data.as_ref() as *const dyn DataGeneric<T>;
        let typed = unsafe { std::mem::transmute(std::ptr::metadata(ptr)) };
        let any: Box<dyn DataAny> = data;
        let boxed = DataBox {
            any,
            generic: typed,
        };
        self.data.insert(id, boxed);
    }
    pub(crate) fn reserve(&mut self, additional: usize) {
        for value in self.data.values_mut() {
            value.reserve(additional);
        }
        self.cap += additional;
    }
    pub fn get<'a>(&'a self, id: &ComponentId) -> Option<&'a DataBox> {
        self.data.get(id)
    }
    pub fn get_mut<'a>(&'a mut self, id: &ComponentId) -> Option<&'a mut DataBox> {
        self.data.get_mut(id)
    }
    pub fn get_mut_disjoint<'a, const N: usize>(
        &'a mut self,
        ids: [&ComponentId; N],
    ) -> [Option<&'a mut DataBox>; N] {
        self.data.get_disjoint_mut(ids)
    }
}
