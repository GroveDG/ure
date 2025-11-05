use std::{any::Any, hash::Hash};

pub use bitvec::{slice::BitSlice, vec::BitVec};
pub use indexmap::IndexSet;
pub use one_or_many::OneOrMany;

pub trait Container: Any {
	type Slice: ?Sized;
	type Item;

	fn as_ref(&self) -> &Self::Slice;
	fn as_mut(&mut self) -> &mut Self::Slice;
	fn delete(&mut self, indices: &[usize]);
}
pub trait NewDefault: Container {
	fn new_default(&mut self, num: usize);
}
pub trait NewWith: Container {
	type Args;

	fn new_with(&mut self, args: Self::Args);
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
impl<T: 'static> NewWith for One<T> {
	type Args = ();

	fn new_with(&mut self, _: Self::Args) {}
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
impl<T: 'static> NewWith for Option<T> {
	type Args = ();

	fn new_with(&mut self, _: Self::Args) {}
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
impl<T: 'static + Default> NewDefault for Vec<T> {
	fn new_default(&mut self, num: usize) {
		self.reserve(num);
		for _ in 0..num {
			self.push(Default::default());
		}
	}
}
impl<T: 'static> NewWith for Vec<T> {
	type Args = Vec<T>;

	fn new_with(&mut self, mut args: Self::Args) {
		self.append(&mut args);
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
impl<T: 'static + Hash + Eq> NewWith for IndexSet<T> {
	type Args = IndexSet<T>;

	fn new_with(&mut self, mut args: Self::Args) {
		self.append(&mut args);
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
		if let OneOrMany::Many(vec) = self {
			vec.delete(indices);
		}
	}
}
impl<T: 'static + Default> NewDefault for OneOrMany<T> {
	fn new_default(&mut self, num: usize) {
		if let OneOrMany::Many(vec) = self {
			vec.new_default(num);
		}
	}
}
impl<T: 'static> NewWith for OneOrMany<T> {
	type Args = Vec<T>;

	fn new_with(&mut self, mut args: Self::Args) {
		if let OneOrMany::Many(vec) = self {
			vec.append(&mut args);
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
impl NewDefault for BitVec {
	fn new_default(&mut self, num: usize) {
		self.reserve(num);
		for _ in 0..num {
			self.push(Default::default());
		}
	}
}
impl NewWith for BitVec {
	type Args = BitVec;

	fn new_with(&mut self, mut args: Self::Args) {
		self.append(&mut args);
	}
}
