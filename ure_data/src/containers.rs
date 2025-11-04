use std::{any::Any, hash::Hash};

use bitvec::{slice::BitSlice, vec::BitVec};
use indexmap::IndexSet;
use one_or_many::OneOrMany;

pub trait Container: Any {
	type Slice: ?Sized;
	type Item;

	fn as_ref(&self) -> &Self::Slice;
	fn as_mut(&mut self) -> &mut Self::Slice;
	fn delete(&mut self, indices: &[usize]);
}
pub trait Push: Container {
	fn push(&mut self, items: Vec<Self::Item>);
}
pub trait NewDefault: Container {
	fn new_default(&mut self, num: usize);
}
impl<C: Push> NewDefault for C
where
	C::Item: Default,
{
	fn new_default(&mut self, num: usize) {
		let mut items = Vec::with_capacity(num);
		for _ in 0..num {
			items.push(Default::default())
		}
		self.push(items);
	}
}

#[derive(Debug, Default)]
pub struct One<T: 'static>(pub T);
impl<T: 'static> Container for One<T> {
	type Slice = T;
	type Item = T;

	fn as_ref(&self) -> &Self::Slice {
		&self.0
	}
	fn as_mut(&mut self) -> &mut Self::Slice {
		&mut self.0
	}
	fn delete(&mut self, _: &[usize]) {}
}
impl<T: 'static> NewDefault for One<T> {
	fn new_default(&mut self, _: usize) {}
}
impl<T: 'static> Container for Option<T> {
	type Slice = Self;
	type Item = T;

	fn as_ref(&self) -> &Self::Slice {
		self
	}
	fn as_mut(&mut self) -> &mut Self::Slice {
		self
	}
	fn delete(&mut self, _: &[usize]) {}
}
impl<T: 'static> NewDefault for Option<T> {
	fn new_default(&mut self, _: usize) {}
}
impl<T: 'static> Container for Vec<T> {
	type Slice = [T];
	type Item = T;

	fn as_ref(&self) -> &Self::Slice {
		self
	}
	fn as_mut(&mut self) -> &mut Self::Slice {
		self
	}
	fn delete(&mut self, indices: &[usize]) {
		for &index in indices {
			self.swap_remove(index);
		}
	}
}
impl<T: 'static> Push for Vec<T> {
	fn push(&mut self, mut items: Vec<Self::Item>) {
		self.append(&mut items);
	}
}
impl<T: 'static + Hash + Eq> Container for IndexSet<T> {
	type Slice = Self;
	type Item = T;

	fn as_ref(&self) -> &Self::Slice {
		self
	}
	fn as_mut(&mut self) -> &mut Self::Slice {
		self
	}
	fn delete(&mut self, indices: &[usize]) {
		for &index in indices {
			self.swap_remove_index(index);
		}
	}
}
impl<T: 'static + Hash + Eq> Push for IndexSet<T> {
	fn push(&mut self, items: Vec<Self::Item>) {
		for item in items {
			self.insert(item);
		}
	}
}
impl<T: 'static> Container for OneOrMany<T> {
	type Slice = [T];
	type Item = T;

	fn as_ref(&self) -> &Self::Slice {
		self.as_slice()
	}
	fn as_mut(&mut self) -> &mut Self::Slice {
		self.as_mut_slice()
	}
	fn delete(&mut self, indices: &[usize]) {
		if let OneOrMany::Many(items) = self {
			for &index in indices {
				items.swap_remove(index);
			}
		}
	}
}
impl<T: 'static> Push for OneOrMany<T> {
	fn push(&mut self, mut items: Vec<Self::Item>) {
		if let OneOrMany::Many(vec) = self {
			vec.append(&mut items);
		}
	}
}
impl Container for BitVec {
	type Slice = BitSlice;
	type Item = bool;

	fn as_ref(&self) -> &Self::Slice {
		self
	}
	fn as_mut(&mut self) -> &mut Self::Slice {
		self
	}
	fn delete(&mut self, indices: &[usize]) {
		for &index in indices {
			self.swap_remove(index);
		}
	}
}
impl Push for BitVec {
	fn push(&mut self, items: Vec<Self::Item>) {
		for item in items {
			self.push(item);
		}
	}
}
