use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::Hash,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use bimap::BiHashMap;
use bitvec::vec::BitVec;

use crate::Func;

pub trait DataAny: Any {
    fn inner_type(&self) -> TypeId;
    fn reserve(&mut self, additional: usize);
}

type TypedVtable = NonNull<()>;

pub struct DataBox {
    any: Box<dyn DataAny>,
    typed: TypedVtable,
}
impl DataBox {
    pub fn cast_ref<'a, T: 'static>(&'a self) -> Option<&'a dyn DataGeneric<T>> {
        if self.inner_type() != TypeId::of::<T>() {
            return None;
        }
        let ptr = self.any.as_ref() as *const dyn DataAny;
        let typed: *const dyn DataGeneric<T> =
            unsafe { std::ptr::from_raw_parts(ptr as *const (), std::mem::transmute(self.typed)) };
        unsafe { typed.as_ref() }
    }
    pub fn cast_mut<'a, T: 'static>(&'a mut self) -> Option<&'a mut dyn DataGeneric<T>> {
        if self.inner_type() != TypeId::of::<T>() {
            return None;
        }
        let ptr = self.any.as_mut() as *mut dyn DataAny;
        let typed: *mut dyn DataGeneric<T> = unsafe {
            std::ptr::from_raw_parts_mut(ptr as *mut (), std::mem::transmute(self.typed))
        };
        unsafe { typed.as_mut() }
    }
    pub fn downcast_ref<'a, T: 'static, C: DataTyped<T>>(&'a self) -> Option<C::View<'a>> {
        let any: &dyn Any = self.any.as_ref();
        Some(any.downcast_ref::<C>()?.view())
    }
    pub fn downcast_mut<'a, T: 'static, C: DataTyped<T>>(&'a mut self) -> Option<C::ViewMut<'a>> {
        let any: &mut dyn Any = self.any.as_mut();
        Some(any.downcast_mut::<C>()?.view_mut())
    }
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

pub trait DataTyped<T: 'static>: 'static + DataAny {
    type View<'a>: DataRef<'a, T> + 'a
    where
        Self: 'a;
    type ViewMut<'a>: DataMut<'a, T> + 'a
    where
        Self: 'a;

    fn view<'a>(&'a self) -> Self::View<'a>;
    fn view_mut<'a>(&'a mut self) -> Self::ViewMut<'a>;
}
pub trait DataGeneric<T: 'static>: DataAny {
    fn boxed<'a>(&'a self) -> Box<dyn DataRef<'a, T> + 'a>;
    fn boxed_mut<'a>(&'a mut self) -> Box<dyn DataMut<'a, T> + 'a>;
}
impl<T: 'static, S: DataTyped<T> + DataAny> DataGeneric<T> for S {
    fn boxed<'a>(&'a self) -> Box<dyn DataRef<'a, T> + 'a> {
        Box::new(self.view())
    }
    fn boxed_mut<'a>(&'a mut self) -> Box<dyn DataMut<'a, T> + 'a> {
        Box::new(self.view_mut())
    }
}
pub trait DataRef<'a, T: 'static> {
    fn read(&'a self, index: usize) -> Option<&'a T>;
}
pub trait DataMut<'a, T: 'static> {
    fn write(&'a mut self, index: usize, value: T);
}

// ============================================================================
//                                     Vec
// ============================================================================
impl<T: 'static> DataAny for Vec<T> {
    fn inner_type(&self) -> TypeId {
        TypeId::of::<T>()
    }
    fn reserve(&mut self, additional: usize) {
        self.reserve(additional);
    }
}
impl<T: 'static> DataTyped<T> for Vec<T> {
    type View<'a> = &'a [T];
    type ViewMut<'a> = &'a mut [T];

    fn view<'a>(&'a self) -> Self::View<'a> {
        &self[..]
    }
    fn view_mut<'a>(&'a mut self) -> Self::ViewMut<'a> {
        &mut self[..]
    }
}
impl<'a, T: 'static> DataRef<'a, T> for &'a [T] {
    fn read(&'a self, index: usize) -> Option<&'a T> {
        self.get(index)
    }
}
impl<'a, T: 'static> DataMut<'a, T> for &'a mut [T] {
    fn write(&'a mut self, index: usize, value: T) {
        self[index] = value;
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
impl DataTyped<bool> for BitVec {
    type View<'a> = &'a BitVec;
    type ViewMut<'a> = &'a mut BitVec;

    fn view<'a>(&'a self) -> Self::View<'a> {
        self
    }
    fn view_mut<'a>(&'a mut self) -> Self::ViewMut<'a> {
        self
    }
}
impl<'a> DataRef<'a, bool> for &'a BitVec {
    fn read(&'a self, index: usize) -> Option<&'a bool> {
        Some(&self[index])
    }
}
impl<'a> DataMut<'a, bool> for &'a mut BitVec {
    fn write(&'a mut self, index: usize, value: bool) {
        self.set(index, value)
    }
}

// ============================================================================
//                                    Bimap
// ============================================================================
impl<T: 'static + Hash + Eq> DataAny for BiHashMap<usize, T> {
    fn inner_type(&self) -> TypeId {
        TypeId::of::<T>()
    }
    fn reserve(&mut self, additional: usize) {
        self.reserve(additional);
    }
}
impl<T: 'static + Hash + Eq> DataTyped<T> for BiHashMap<usize, T> {
    type View<'a> = &'a Self;
    type ViewMut<'a> = &'a mut Self;

    fn view<'a>(&'a self) -> Self::View<'a> {
        self
    }
    fn view_mut<'a>(&'a mut self) -> Self::ViewMut<'a> {
        self
    }
}
impl<'a, T: 'static + Hash + Eq> DataRef<'a, T> for &'a BiHashMap<usize, T> {
    fn read(&'a self, index: usize) -> Option<&'a T> {
        self.get_by_left(&index)
    }
}
impl<'a, T: 'static + Hash + Eq> DataMut<'a, T> for &'a mut BiHashMap<usize, T> {
    fn write(&'a mut self, index: usize, value: T) {
        let overwrite = self.insert(index, value);
        assert!(overwrite.did_overwrite());
    }
}

pub type ComponentId = u64;
pub type New<T> = fn(&Data, &mut dyn DataMut<T>);

pub struct Component {
    pub(crate) id: ComponentId,
    pub(crate) new: &'static Func,
}

#[derive(Default)]
pub struct Data {
    data: HashMap<ComponentId, DataBox>,
    len: usize,
    cap: usize,
}
impl Data {
    pub(crate) fn insert<T: 'static>(&mut self, id: ComponentId, data: Box<dyn DataGeneric<T>>) {
        let ptr = data.as_ref() as *const dyn DataGeneric<T>;
        let typed = unsafe { std::mem::transmute(std::ptr::metadata(ptr)) };
        let any: Box<dyn DataAny> = data;
        let boxed = DataBox { any, typed };
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
