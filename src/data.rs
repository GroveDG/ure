use std::{
    any::Any,
    fmt::Debug,
    slice::GetDisjointMutError,
    sync::{Arc, Weak},
};

use bitvec::slice::BitSlice;
use parking_lot::Mutex;
use slotmap::{DefaultKey, SlotMap};

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

pub type Data = SlotMap<DefaultKey, Element>;

#[derive(Debug)]
pub struct Element {
    length: usize,
    compartments: Vec<Compartment>,
}
impl Element {
    #[inline]
    pub fn len(&self) -> usize {
        self.length
    }
    #[inline]
    pub fn get<'a>(&'a self, index: usize) -> &'a Compartment {
        &self.compartments[index]
    }
    #[inline]
    pub fn get_mut<'a>(&'a mut self, index: usize) -> &'a mut Compartment {
        &mut self.compartments[index]
    }
    #[inline]
    pub fn get_mut_disjoint<'a, const N: usize>(
        &'a mut self,
        indices: [usize; N],
    ) -> Result<[&'a mut Compartment; N], GetDisjointMutError> {
        self.compartments.get_disjoint_mut(indices)
    }
}

#[derive(Debug)]
pub enum Compartment {
    None,
    One(Box<dyn Any>),
    Many(Box<dyn Any>),
}
impl Default for Compartment {
    fn default() -> Self {
        Self::None
    }
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
    pub fn as_slice<'a, T: 'static>(&'a self) -> Option<&'a [T]> {
        match self {
            Self::None => Default::default(),
            Self::One(t) => t.as_ref().downcast_ref::<T>().map(std::slice::from_ref),
            Self::Many(t) => t.as_ref().downcast_ref::<Vec<T>>().map(Vec::as_slice),
        }
    }
    pub fn as_slice_mut<'a, T: 'static>(&'a mut self) -> Option<&'a mut [T]> {
        match self {
            Self::None => Default::default(),
            Self::One(t) => t.as_mut().downcast_mut::<T>().map(std::slice::from_mut),
            Self::Many(t) => t.as_mut().downcast_mut::<Vec<T>>().map(Vec::as_mut_slice),
        }
    }
}
