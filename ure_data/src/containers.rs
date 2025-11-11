use std::{
	any::Any,
	cell::{Ref, RefMut},
	hash::Hash,
};

pub use bitvec::{slice::BitSlice, vec::BitVec};
pub use indexmap::IndexSet;
pub use one_or_many::OneOrMany;

pub trait Container: Any {
	type Ref<'a>;
	type RefMut<'a>;

	fn as_ref<'a>(cont: Ref<'a, Self>) -> Self::Ref<'a>;
	fn as_mut<'a>(cont: RefMut<'a, Self>) -> Self::RefMut<'a>;
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
	type Ref<'a> = Ref<'a, T>;
	type RefMut<'a> = RefMut<'a, T>;

	fn as_ref<'a>(cont: Ref<'a, Self>) -> Self::Ref<'a> {
		Ref::map(cont, |c| &c.0)
	}
	fn as_mut<'a>(cont: RefMut<'a, Self>) -> Self::RefMut<'a> {
		RefMut::map(cont, |c| &mut c.0)
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
	type Ref<'a> = Option<Ref<'a, T>>;
	type RefMut<'a> = Option<RefMut<'a, T>>;

	fn as_ref<'a>(cont: Ref<'a, Self>) -> Self::Ref<'a> {
		if cont.is_some() {
			Some(Ref::map(cont, |c| c.as_ref().unwrap()))
		} else {
			None
		}
	}
	fn as_mut<'a>(cont: RefMut<'a, Self>) -> Self::RefMut<'a> {
		if cont.is_some() {
			Some(RefMut::map(cont, |c| c.as_mut().unwrap()))
		} else {
			None
		}
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
	type Ref<'a> = Ref<'a, [T]>;
	type RefMut<'a> = RefMut<'a, [T]>;

	fn as_ref<'a>(cont: Ref<'a, Self>) -> Self::Ref<'a> {
		Ref::map(cont, |c| c.as_slice())
	}
	fn as_mut<'a>(cont: RefMut<'a, Self>) -> Self::RefMut<'a> {
		RefMut::map(cont, |c| c.as_mut_slice())
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

	fn new_with(&mut self, args: Self::Args) {
		self.extend(args);
	}
}
impl<T: 'static + Hash + Eq> Container for IndexSet<T> {
	type Ref<'a> = Ref<'a, Self>;
	type RefMut<'a> = RefMut<'a, Self>;

	fn as_ref<'a>(cont: Ref<'a, Self>) -> Self::Ref<'a> {
		cont
	}
	fn as_mut<'a>(cont: RefMut<'a, Self>) -> Self::RefMut<'a> {
		cont
	}
	fn delete(&mut self, indices: &[usize]) {
		for &index in indices {
			self.swap_remove_index(index);
		}
	}
}
impl<T: 'static + Hash + Eq> NewWith for IndexSet<T> {
	type Args = IndexSet<T>;

	fn new_with(&mut self, args: Self::Args) {
		self.extend(args);
	}
}
#[derive(Debug, Default)]
pub enum RefOrSlice<'a, T> {
	Ref(Ref<'a, T>),
	Slice(Ref<'a, [T]>),
	#[default]
	None,
}
#[derive(Debug, Default)]
pub enum RefOrSliceMut<'a, T> {
	Ref(RefMut<'a, T>),
	Slice(RefMut<'a, [T]>),
	#[default]
	None,
}
impl<T: 'static> Container for OneOrMany<T> {
	type Ref<'a> = RefOrSlice<'a, T>;
	type RefMut<'a> = RefOrSliceMut<'a, T>;

	fn as_ref<'a>(cont: Ref<'a, Self>) -> Self::Ref<'a> {
		if cont.is_many() {
			RefOrSlice::Slice(Ref::map(cont, |c| match c {
				OneOrMany::Many(items) => items.as_slice(),
				_ => panic!(),
			}))
		} else if cont.is_one() {
			RefOrSlice::Ref(Ref::map(cont, |c| match c {
				OneOrMany::One(item) => item.as_ref(),
				_ => panic!(),
			}))
		} else {
			RefOrSlice::None
		}
	}
	fn as_mut<'a>(cont: RefMut<'a, Self>) -> Self::RefMut<'a> {
		if cont.is_many() {
			RefOrSliceMut::Slice(RefMut::map(cont, |c| match c {
				OneOrMany::Many(items) => items.as_mut_slice(),
				_ => panic!(),
			}))
		} else if cont.is_one() {
			RefOrSliceMut::Ref(RefMut::map(cont, |c| match c {
				OneOrMany::One(item) => item.as_mut(),
				_ => panic!(),
			}))
		} else {
			RefOrSliceMut::None
		}
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

	fn new_with(&mut self, args: Self::Args) {
		if let OneOrMany::Many(vec) = self {
			vec.extend(args);
		}
	}
}
impl Container for BitVec {
	type Ref<'a> = Ref<'a, BitSlice>;
	type RefMut<'a> = RefMut<'a, BitSlice>;

	fn as_ref<'a>(cont: Ref<'a, Self>) -> Self::Ref<'a> {
		Ref::map(cont, |c| c.as_bitslice())
	}
	fn as_mut<'a>(cont: RefMut<'a, Self>) -> Self::RefMut<'a> {
		RefMut::map(cont, |c| c.as_mut_bitslice())
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

	fn new_with(&mut self, args: Self::Args) {
		self.extend(args);
	}
}
