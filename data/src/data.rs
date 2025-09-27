use std::{any::Any, collections::HashMap, hash::Hash, ops::Range};

use const_fnv1a_hash::fnv1a_hash_str_64;
use nohash_hasher::BuildNoHashHasher;

use crate::func::Func;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentId {
	inner: u64,
}
impl ComponentId {
	pub const fn new(name: &str) -> Self {
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

#[derive(Debug, Clone)]
pub enum ComponentCommand {
	/// Creates `num` new elements.
	New { num: usize },
	/// Deletes all elements in `range`.
	///
	/// Deletes are always `swap_remove` with ordering within the swapped chunk preserved.
	/// To implement this with the `swap_remove` function, reverse the iterator with `range.rev()`.
	Delete { range: Range<usize> },
}

pub struct Component<New: Any, Delete: Any> {
	pub(crate) id: ComponentId,
	pub(crate) new: &'static Func<New>,
	pub(crate) delete: &'static Func<Delete>,
}
impl<New: Any, Delete: Any> Component<New, Delete> {
	pub const fn new(name: &str, new: &'static Func<New>, delete: &'static Func<Delete>) -> Self {
		Self {
			id: ComponentId::new(name),
			new,
			delete,
		}
	}
}

#[derive(Default)]
pub struct Components {
	components: HashMap<ComponentId, Box<dyn Any>, BuildNoHashHasher<ComponentId>>,
}
impl Components {
	pub fn get(&self, id: &ComponentId) -> Option<&dyn Any> {
		Some(self.components.get(id)?.as_ref())
	}
	pub fn get_mut(&mut self, id: &ComponentId) -> Option<&mut dyn Any> {
		Some(self.components.get_mut(id)?.as_mut())
	}
	pub fn get_disjoint_mut<const N: usize>(
		&mut self,
		ids: [&ComponentId; N],
	) -> [Option<&mut dyn Any>; N] {
		self.components
			.get_disjoint_mut(ids)
			.map(|v| Some(v?.as_mut()))
	}
	pub fn add<C: Any>(&mut self, id: ComponentId, component: C) {
		self.components.insert(id, Box::new(component));
	}
	pub fn remove(&mut self, id: &ComponentId) {
		self.components.remove(id);
	}
}
