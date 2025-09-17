use std::{
    any::{Any, TypeId},
    collections::HashMap,
    ops::{Deref, DerefMut, Range, RangeBounds},
    ptr::NonNull,
};

mod index;
pub use index::{ValidIndex, ValidRange};

// Containers
pub mod bimap;
pub mod bitvec;
pub mod vec;

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
    pub fn downcast_ref<'a, T: Any, D: DataSpecific<Inner = T>>(&'a self) -> Option<&'a D> {
        (self as &dyn Any).downcast_ref()
    }
    pub fn downcast_mut<'a, T: Any, D: DataSpecific<Inner = T>>(&'a mut self) -> Option<&'a mut D> {
        (self as &mut dyn Any).downcast_mut()
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
    pub fn slice_ref<'a, T: Any>(&'a self) -> Option<&'a dyn DataSlice<T>> {
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
        Some(typed.generic())
    }
    pub fn slice_mut<'a, T: Any>(&'a mut self) -> Option<&'a mut dyn DataSlice<T>> {
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
        Some(typed.generic_mut())
    }
    pub fn downcast_ref<'a, D: DataSpecific<Inner = T>, T: Any>(&'a self) -> Option<&'a D::Slice> {
        let any: &dyn Any = self.any.as_ref();
        Some(any.downcast_ref::<D>()?.slice_ref())
    }
    pub fn downcast_mut<'a, D: DataSpecific<Inner = T>, T: Any>(
        &'a mut self,
    ) -> Option<&'a mut D::Slice> {
        let any: &mut dyn Any = self.any.as_mut();
        Some(any.downcast_mut::<D>()?.slice_mut())
    }
}

pub trait DataGeneric<T: Any>: DataAny {
    fn generic<'a>(&'a self) -> &'a dyn DataSlice<T>;
    fn generic_mut<'a>(&'a mut self) -> &'a mut dyn DataSlice<T>;
}
impl<T: Any, S: DataSpecific<Inner = T> + DataAny> DataGeneric<T> for S {
    fn generic<'a>(&'a self) -> &'a dyn DataSlice<T> {
        self.slice_ref()
    }
    fn generic_mut<'a>(&'a mut self) -> &'a mut dyn DataSlice<T> {
        self.slice_mut()
    }
}

impl<T> dyn DataSlice<T> {
    pub fn downcast_ref<'a, D: DataSpecific<Inner = T>>(&'a self) -> Option<&'a D::Slice> {
        (self as &dyn Any).downcast_ref()
    }
    pub fn downcast_mut<'a, D: DataSpecific<Inner = T>>(&'a mut self) -> Option<&'a mut D::Slice> {
        (self as &mut dyn Any).downcast_mut()
    }
}

pub trait DataSpecific: DataAny {
    type Inner: Any;
    type Slice: DataSlice<Self::Inner>;

    fn slice_ref<'a>(&'a self) -> &'a Self::Slice;
    fn slice_mut<'a>(&'a mut self) -> &'a mut Self::Slice;
    fn new_data() -> Self;
}

pub trait DataSlice<T: Any>: Any {
    fn get_data<'a>(&'a self, index: ValidIndex) -> &'a T;
    fn set_data(&mut self, index: ValidIndex, value: T);
}

pub type ComponentId = u64;
pub type New<T> = fn(&Data, &mut dyn DataSlice<T>);

#[derive(Debug)]
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
    pub(crate) fn validate_range(
        &self,
        range: impl RangeBounds<usize>,
    ) -> Option<ValidRange<'static>> {
        let start = match range.start_bound() {
            std::ops::Bound::Included(i) => *i,
            std::ops::Bound::Excluded(i) => *i + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.start_bound() {
            std::ops::Bound::Included(i) => *i + 1,
            std::ops::Bound::Excluded(i) => *i,
            std::ops::Bound::Unbounded => self.len,
        };
        if end > self.len || start > end {
            return None;
        }
        Some(ValidRange {
            inner: Range { start, end },
            _marker: std::marker::PhantomData,
        })
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
