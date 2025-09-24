use std::{any::Any, collections::HashMap, hash::Hash};

use const_fnv1a_hash::fnv1a_hash_str_64;
use nohash_hasher::BuildNoHashHasher;

pub mod bimap;
pub mod bitvec;
pub mod single;
pub mod vec;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentId {
    inner: u64,
}
impl ComponentId {
    pub const fn new(name: &'static str) -> Self {
        Self {
            inner: fnv1a_hash_str_64(name),
        }
    }
}
impl Hash for ComponentId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.inner);
    }
}
impl nohash_hasher::IsEnabled for ComponentId {}

pub trait Container: Any {
    fn swap_delete(&mut self, indices: &[usize]);
    fn new(&mut self, num: usize);
}

impl dyn Container {
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref()
    }
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        (self as &mut dyn Any).downcast_mut()
    }
}

#[derive(Default)]
pub struct Components {
    data: HashMap<ComponentId, Box<dyn Container>, BuildNoHashHasher<ComponentId>>,
}
impl Components {
    pub fn get(&self, component: &'static ComponentId) -> Option<&dyn Container> {
        Some(self.data.get(component)?.as_ref())
    }
    pub fn get_mut(&mut self, component: &'static ComponentId) -> Option<&mut dyn Container> {
        Some(self.data.get_mut(component)?.as_mut())
    }
    pub fn get_disjoint_mut<const N: usize>(
        &mut self,
        components: [&'static ComponentId; N],
    ) -> [Option<&mut dyn Container>; N] {
        self.data
            .get_disjoint_mut(components)
            .map(|v| Some(v?.as_mut()))
    }
}
