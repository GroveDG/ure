use std::{
    alloc::{Allocator, Global, Layout},
    any::{Any, TypeId},
    collections::HashMap,
    hash::Hash,
    mem::MaybeUninit,
    ops::{Deref, DerefMut, Range},
    ptr::{NonNull, drop_in_place},
    slice,
};

use bimap::BiHashMap;
use const_fnv1a_hash::fnv1a_hash_str_64;
use nohash_hasher::BuildNoHashHasher;

// use crate::Group;

type Untyped = NonNull<u8>;

type StableId = u64;

// #[derive(Debug, Clone, Copy)]
// pub struct Component {
//     name: &'static str,
//     id: StableId,
//     type_id: TypeId,
//     layout: Layout,
//     drop: unsafe fn(Untyped),
//     init: unsafe fn(&Group, &mut [u8]),
// }
// impl Component {
//     pub const fn new<T: Sized + Any>(
//         name: &'static str,
//         init: fn(&Group, &mut [MaybeUninit<T>]),
//     ) -> Component {
//         Component {
//             name,
//             id: fnv1a_hash_str_64(name),
//             type_id: TypeId::of::<T>(),
//             layout: Layout::new::<T>().pad_to_align(),
//             drop: Self::drop_in_place_untyped::<T>,
//             init: unsafe { std::mem::transmute(init) }, // This is so undefined no one knows what it'll do
//         }
//     }
//     pub const fn get_name(&self) -> &'static str {
//         self.name
//     }
//     unsafe fn drop_in_place_untyped<T: Sized>(ptr: Untyped) {
//         unsafe {
//             drop_in_place(ptr.as_ptr() as *mut T);
//         }
//     }
// }
// type DynComponent = (Untyped, &'static Component);

pub struct ComponentsBox {
    any: Box<dyn ComponentsAny>,
    typed: NonNull<()>,
}
impl ComponentsBox {
    pub fn cast<'a, T: Sized + 'static>(&'a self) -> Option<&'a dyn Components<T>> {
        let type_id = self.inner_type();
        if type_id != TypeId::of::<T>() {
            return None;
        }
        let ptr = self.any.as_ref() as *const dyn ComponentsAny;
        let typed: *const dyn Components<T> =
            unsafe { std::ptr::from_raw_parts(ptr as *const (), std::mem::transmute(self.typed)) };
        unsafe { typed.as_ref() }
    }
    pub fn cast_mut<'a, T: Sized + 'static>(&'a mut self) -> Option<&'a mut dyn Components<T>> {
        let type_id = self.inner_type();
        if type_id != TypeId::of::<T>() {
            return None;
        }
        let ptr = self.any.as_mut() as *mut dyn ComponentsAny;
        let typed: *mut dyn Components<T> =
            unsafe { std::ptr::from_raw_parts_mut(ptr as *mut (), std::mem::transmute(self.typed)) };
        unsafe { typed.as_mut() }
    }
}
impl Deref for ComponentsBox {
    type Target = dyn ComponentsAny;

    fn deref(&self) -> &Self::Target {
        self.any.as_ref()
    }
}
impl DerefMut for ComponentsBox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.any.as_mut()
    }
}

pub trait ComponentsAny {
    fn inner_type(&self) -> TypeId;
    fn reserve(&mut self, additional: usize);
}

pub trait Components<T: Sized + 'static>: Any + ComponentsAny {
    fn direct<'a>(&'a self) -> Option<&'a dyn DirectComponents<T>> {
        None
    }
    fn direct_mut<'a>(&'a mut self) -> Option<&'a mut dyn DirectComponents<T>> {
        None
    }
    fn indirect<'a>(&'a self) -> &'a dyn IndirectComponents<T>
    where
        Self: Sized + IndirectComponents<T>,
    {
        self
    }
    fn indirect_mut<'a>(&'a mut self) -> &'a mut dyn IndirectComponents<T>
    where
        Self: Sized + IndirectComponents<T>,
    {
        self
    }
}
pub trait DirectComponents<T> {
    fn as_slice<'a>(&'a self) -> &'a dyn AsRef<[MaybeUninit<T>]>;
    fn as_slice_mut<'a>(&'a mut self) -> &'a mut dyn AsMut<[MaybeUninit<T>]>;
}
pub trait IndirectComponents<T> {
    fn read<'a>(&'a self, index: usize) -> &'a T;
    fn write(&mut self, index: usize, value: T);
}

// ============================================================================
//                                 Boxed Slice
// ============================================================================
impl<T: Sized + 'static> ComponentsAny for Box<[MaybeUninit<T>]> {
    fn inner_type(&self) -> TypeId {
        TypeId::of::<T>()
    }
    fn reserve(&mut self, additional: usize) {
        // Length is equivalent to Vec capacity here
        let capacity = self.len() + additional;
        *self = Box::new_uninit_slice_in(capacity, *Box::allocator(&self));
    }
}
impl<T: Sized + 'static> Components<T> for Box<[MaybeUninit<T>]> {
    fn direct<'a>(&'a self) -> Option<&'a dyn DirectComponents<T>> {
        Some(self)
    }
    fn direct_mut<'a>(&'a mut self) -> Option<&'a mut dyn DirectComponents<T>> {
        Some(self)
    }
}
impl<T: Sized + 'static> DirectComponents<T> for Box<[MaybeUninit<T>]> {
    fn as_slice<'a>(&'a self) -> &'a dyn AsRef<[MaybeUninit<T>]> {
        self
    }
    fn as_slice_mut<'a>(&'a mut self) -> &'a mut dyn AsMut<[MaybeUninit<T>]> {
        self
    }
}
impl<T: Sized + 'static> IndirectComponents<T> for Box<[MaybeUninit<T>]> {
    fn read<'a>(&'a self, index: usize) -> &'a T {
        unsafe { self[index].assume_init_ref() }
    }
    fn write(&mut self, index: usize, value: T) {
        self[index].write(value);
    }
}

// ============================================================================
//                                    Bimap
// ============================================================================
impl<T: Sized + 'static + Hash + Eq> ComponentsAny for BiHashMap<usize, T> {
    fn inner_type(&self) -> TypeId {
        TypeId::of::<T>()
    }
    fn reserve(&mut self, additional: usize) {
        self.reserve(additional);
    }
}
impl<T: Sized + 'static + Hash + Eq> Components<T> for BiHashMap<usize, T> {}
impl<T: Sized + 'static + Hash + Eq> IndirectComponents<T> for BiHashMap<usize, T> {
    fn read<'a>(&'a self, index: usize) -> &'a T {
        self.get_by_left(&index).unwrap()
    }
    fn write(&mut self, index: usize, value: T) {
        self.insert(index, value);
    }
}

pub struct Elements {
    components: HashMap<StableId, ComponentsBox>,
    len: usize,
    capacity: usize,
}
impl Elements {
    fn insert<T: Sized + 'static>(&mut self, id: StableId, components: Box<dyn Components<T>>) {
        let ptr = components.as_ref() as *const dyn Components<T>;
        let typed = unsafe { std::mem::transmute(std::ptr::metadata(ptr)) };
        let any: Box<dyn ComponentsAny> = components;
        let boxed = ComponentsBox { any, typed };
        self.components.insert(id, boxed);
    }
    pub(crate) fn reserve(&mut self, additional: usize) {
        for value in self.components.values_mut() {
            value.reserve(additional);
        }
    }
    pub fn get<'a, T: Sized + 'static>(&'a self, id: &StableId) -> Option<&'a dyn Components<T>> {
        let boxed = self.components.get(id)?;
        boxed.cast::<T>()
    }
    pub fn get_mut<'a, T: Sized + 'static>(&'a mut self, id: &StableId) -> Option<&'a mut dyn Components<T>> {
        let boxed = self.components.get_mut(id)?;
        boxed.cast_mut::<T>()
    }
    pub fn get_mut_disjoint<'a, const N: usize>(&'a mut self, ids: [&StableId; N]) -> [Option<&'a mut ComponentsBox>; N] {
        self.components.get_disjoint_mut(ids)
    }
}
