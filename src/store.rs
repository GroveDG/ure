use std::{
    any::{Any, TypeId},
    fmt::Debug,
    sync::{Arc, Weak},
};

use any_vec::{AnyVec, AnyVecMut, AnyVecRef, mem::Heap};
use parking_lot::Mutex;
use rustc_hash::FxHashMap;
use slotmap::SlotMap;

pub struct Resource<T, F = fn() -> T> {
    weak: Mutex<Weak<T>>,
    f: F,
}
impl<T, F: Fn() -> T> Resource<T, F> {
    pub const fn new(f: F) -> Self {
        Self {
            weak: Mutex::new(Weak::new()),
            f,
        }
    }
    pub fn load(&'static self) -> Arc<T> {
        let mut lock = self.weak.lock();
        if let Some(arc) = lock.upgrade() {
            return arc;
        }
        let arc = Arc::new((self.f)());
        *lock = Arc::downgrade(&arc);
        arc
    }
}

pub type Data<K> = SlotMap<K, Element>;

#[macro_export]
macro_rules! element {
    ( $($t:ty),* $(,)? ) => {
        $crate::store::Element::new(
            rustc_hash::FxHashMap::default(),
            rustc_hash::FxHashMap::from_iter([
                $(
                (std::any::TypeId::of::<$t>(), $crate::store::Compartment::Many(any_vec::AnyVec::new::<$t>())),
                )*
            ])
        )
    };
}

// #[macro_export]
// macro_rules! element {
//     ( { $($f:ty : $func:expr),* $(,)? } { $($t:ty : $component:expr),* $(,)? } ) => {
//         {
//             let mut len: Option<usize> = None;
//             $crate::store::Element::new (
//                 FxHashMap::from_iter([
//                     $(
//                     (std::any::TypeId::of::<$f>(), $func),
//                     )*
//                 ]),
//                 FxHashMap::from_iter([
//                     $({
//                     let component: Box<dyn std::any::Any> = Box::new($component);
//                     let component = if component.is::<$t>() {
//                         Component::One(component)
//                     } else {
//                         Component::Many(any_vec::AnyVec::from(component))
//                     };
//                     (std::any::TypeId::of::<$t>(), component)
//                     },)*
//                 ])
//             )
//         }
//     };
// }

#[macro_export]
macro_rules! extend {
    ( $element:expr, $amount:expr, { $($component:ident : $t:ty = $init:block),+ $(,)? } ) => {
        {
            let mut element = $element;
            let len = element.len().unwrap();
            let amount = $amount;
            let components = [
                $(&std::any::TypeId::of::<$t>(),)+
            ];
            
            // Assert that we have all the components
            assert_eq!(components.len(), element.bredth());

            // Get the components
            let mut components = element.get_mut_disjoint(components);

            // Assert all components were present
            for component in components.iter() {
                assert!(component.is_some());
            }
            {
                let mut components = components.iter_mut();

                #[allow(unused_variables)]
                {$(
                let $component = unsafe { components.next().unwrap_unchecked().as_mut().unwrap_unchecked() };
                let $component: &mut [$t] = match $component {
                    $crate::store::Compartment::One(one) => {
                        one.as_mut().downcast_mut::<$t>().map(std::slice::from_mut).unwrap()
                    },
                    $crate::store::Compartment::Many(many) => {
                        many.reserve(amount);
                        let $component: &mut [std::mem::MaybeUninit<$t>] = unsafe {
                            std::mem::transmute(many.spare_bytes_mut())
                        };
                        let $component = &mut $component[..amount];
                        $init
                        unsafe { std::mem::transmute($component) }
                    },
                };
                )+}
            }

            for component in components { unsafe {
                let $crate::store::Compartment::Many(many) = component.unwrap_unchecked() else {
                    continue;
                };
                many.set_len(len + amount);
            }}
            unsafe { element.set_len(len + amount); }
            element
        }
    };
}

#[macro_export]
macro_rules! func {
    ( $element:expr, { $($func:ident : $f:ty),+ $(,)? } else $else:block ) => {
        $(
        let Some($func) = $element.get_func::<$f>() else $else;
        ),+
    };
}

#[macro_export]
macro_rules! data {
    ($element:expr, { $($component:ident : $t:ty),+ $(,)? } else $else:block) => {
        let mut components = $element.get_mut_disjoint([
            $(
            &std::any::TypeId::of::<$t>(),
            )*
        ]).into_iter();
        $(
        let Some($component) = unsafe { components.next().unwrap_unchecked() }.and_then(|c| c.as_slice_mut::<$t>()) else $else;
        )*
    };
}

#[derive(Debug)]
pub struct Element {
    len: Option<usize>,
    functions: FxHashMap<TypeId, &'static dyn Any>,
    components: FxHashMap<TypeId, Compartment>,
}
impl Element {
    pub fn new(
        functions: FxHashMap<TypeId, &'static dyn Any>,
        components: FxHashMap<TypeId, Compartment>,
    ) -> Self {
        let mut len: Option<usize> = None;
        for compartment in components.values() {
            let Compartment::Many(v) = compartment else {
                continue;
            };
            if let Some(len) = len {
                assert!(len == v.len())
            } else {
                len = Some(v.len());
            }
        }
        Self {
            len,
            components,
            functions,
        }
    }
    #[inline]
    pub fn len(&self) -> Option<usize> {
        return self.len;
    }
    pub unsafe fn set_len(&mut self, len: usize) {
        self.len = Some(len);
    } 
    #[inline]
    pub fn bredth(&self) -> usize {
        self.components.len()
    }
    pub fn get<'a, T: Any + 'static>(&'a self) -> Option<&'a [T]> {
        let component = self.components.get(&TypeId::of::<T>())?;
        component.as_slice()
    }
    pub fn get_mut<'a, T: Any + 'static>(&'a mut self) -> Option<&'a mut [T]> {
        let component = self.components.get_mut(&TypeId::of::<T>())?;
        component.as_slice_mut()
    }
    pub fn get_mut_compartment<'a, T: Any + 'static>(&'a mut self) -> Option<&'a mut Compartment> {
        self.components.get_mut(&TypeId::of::<T>())
    }
    pub fn get_mut_disjoint<'a, const N: usize>(
        &'a mut self,
        ids: [&TypeId; N],
    ) -> [Option<&'a mut Compartment>; N] {
        self.components.get_disjoint_mut(ids)
    }
    pub fn insert<'a, T: Any + 'static>(&'a mut self, value: Compartment) -> Option<Compartment> {
        if let Compartment::Many(v) = &value {
            if let Some(len) = self.len {
                assert_eq!(len, v.len())
            } else {
                self.len = Some(v.len());
            }
        }
        self.components.insert(TypeId::of::<T>(), value)
    }
    pub fn get_func<T: Any + 'static>(&self) -> Option<&'static T> {
        let func = self.functions.get(&TypeId::of::<T>()).copied()?;
        func.downcast_ref()
    }
    pub fn implement<'a, T: Any + 'static>(
        &mut self,
        value: &'static T,
    ) -> Option<&'static dyn Any> {
        self.functions.insert(TypeId::of::<T>(), value)
    }
    pub fn extend(&mut self, mut other: Element) {
        for (ty, compartment) in self.components.iter_mut() {
            let Compartment::Many(v) = compartment else {
                continue;
            };
            let Some(Compartment::Many(mut o)) = other.components.remove(ty) else {
                continue;
            };
            v.reserve(o.len());
            for value in o.drain(..) {
                // SAFETY: We already checked types since TypeIds are keys.
                unsafe {
                    v.push_unchecked(value);
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum Compartment {
    One(Box<dyn Any>),
    Many(AnyVec),
}
impl Compartment {
    pub fn cast_one<'a, T: 'static>(&'a self) -> Option<&'a T> {
        let Compartment::One(value) = self else {
            return None;
        };
        value.downcast_ref()
    }
    pub fn cast_one_mut<'a, T: 'static>(&'a mut self) -> Option<&'a mut T> {
        let Compartment::One(value) = self else {
            return None;
        };
        value.downcast_mut()
    }
    pub fn cast_many<'a, T: 'static>(&'a self) -> Option<AnyVecRef<'a, T, Heap>> {
        let Compartment::Many(value) = self else {
            return None;
        };
        value.downcast_ref()
    }
    pub fn cast_many_mut<'a, T: 'static>(&'a mut self) -> Option<AnyVecMut<'a, T, Heap>> {
        let Compartment::Many(value) = self else {
            return None;
        };
        value.downcast_mut()
    }
    pub fn as_slice<'a, T: 'static>(&'a self) -> Option<&'a [T]> {
        match self {
            Self::One(t) => t.as_ref().downcast_ref::<T>().map(std::slice::from_ref),
            Self::Many(t) => t.downcast_ref::<T>().map(|v| v.as_slice()),
        }
    }
    pub fn as_slice_mut<'a, T: 'static>(&'a mut self) -> Option<&'a mut [T]> {
        match self {
            Self::One(t) => t.as_mut().downcast_mut::<T>().map(std::slice::from_mut),
            Self::Many(t) => t.downcast_mut::<T>().map(|mut v| v.as_mut_slice()),
        }
    }
}
