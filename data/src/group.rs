use std::{any::Any, collections::VecDeque, ops::Range};

pub use bitvec::{slice::BitSlice, vec::BitVec};
use indexmap::IndexMap;
pub use indexmap::IndexSet;
pub use one_or_many::OneOrMany;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(u64);

pub trait Container: Any {
	type Ref<'a>;
	type Mut<'a>;

	fn new() -> Self;
	fn container_ref(&self) -> Self::Ref<'_>;
	fn container_mut(&mut self) -> Self::Mut<'_>;
}

pub type New<C> = Box<dyn FnMut(&mut C, usize)>;
pub type Delete<C> = Box<dyn FnMut(&mut C, Range<usize>)>;
pub trait Component {
	const ID: ComponentId = ComponentId(const_fnv1a_hash::fnv1a_hash_str_64(Self::IDENT));
	const IDENT: &'static str;
	type Container: Container;
	type Dependencies: ComponentRetrieve;

	fn new(self) -> ComponentBox;
}
pub struct ComponentBox {
	container: Box<dyn Any>,
	dependencies: Vec<ComponentId>,
	new: Box<dyn FnMut(&mut dyn Any, Range<usize>, &Group)>,
	delete: Box<dyn FnMut(&mut dyn Any, Range<usize>)>,
}
impl ComponentBox {
	pub fn new<C: Component>(
		container: Option<C::Container>,
		mut new: impl FnMut(
			&mut C::Container,
			Range<usize>,
			<<<C as Component>::Dependencies as ComponentRetrieve>::Containers as Container>::Ref<
				'_,
			>,
		) + 'static,
		mut delete: impl FnMut(&mut C::Container, Range<usize>) + 'static,
	) -> Self {
		assert!(!C::Dependencies::IDS.contains(&C::ID));
		Self {
			container: Box::new(container.unwrap_or_else(C::Container::new)),
			dependencies: C::Dependencies::IDS.to_vec(),
			new: Box::new(move |c, range, group| {
				(new)(
					c.downcast_mut().unwrap(),
					range,
					group.get_components::<C::Dependencies>().unwrap(),
				)
			}),
			delete: Box::new(move |c, range| (delete)(c.downcast_mut().unwrap(), range)),
		}
	}
	fn downcast_mut<C: Container>(&mut self) -> Option<&mut C> {
		self.container.downcast_mut()
	}
	fn downcast_ref<C: Container>(&self) -> Option<&C> {
		self.container.downcast_ref()
	}
}

type Components = IndexMap<ComponentId, ComponentBox>;

#[derive(Default)]
pub struct Group {
	len: usize,
	components: Components,
	// pub(crate) functions: HashMap<FunctionId>,
}

impl Group {
	pub fn len(&self) -> usize {
		self.len
	}
	pub fn is_empty(&self) -> bool {
		self.len == 0
	}
	pub fn add_component<C: Component>(&mut self, component: C) {
		let mut boxed = component.new();
		(boxed.new)(boxed.container.as_mut(), 0..self.len, &self);
		self.components.insert(C::ID, boxed);
	}
	pub fn get_components<C: ComponentRetrieve>(
		&self,
	) -> Option<<C::Containers as Container>::Ref<'_>> {
		C::retrieve(&self.components)
	}
	pub fn get_components_mut<C: ComponentRetrieve>(
		&mut self,
	) -> Option<<C::Containers as Container>::Mut<'_>> {
		C::retrieve_mut(&mut self.components)
	}
	pub fn remove_component(&mut self, id: &ComponentId) {
		let mut ids = VecDeque::<ComponentId>::from([*id]);
		while let Some(id) = ids.pop_front() {
			let Some(i) = self.components.get_index_of(&id) else {
				return;
			};
			for i in (i..self.components.len()).rev() {
				let (id, component) = self.components.get_index(i).unwrap();
				if component.dependencies.contains(id) {
					ids.push_back(*id);
				}
			}
			self.components.shift_remove(&id);
		}
	}
	pub fn new(&mut self, num: usize) {
		for i in 0..self.components.len() {
			let (id, mut component) = self.components.shift_remove_index(i).unwrap();
			(component.new)(component.container.as_mut(), self.len..num, &self);
			self.components.insert(id, component);
		}
		self.len += num;
	}
	pub fn delete(&mut self, range: Range<usize>) {
		for component in self.components.values_mut() {
			(component.delete)(component.container.as_mut(), range.clone());
		}
		self.len -= range.len();
	}
}

pub trait ComponentRetrieve {
	type Containers: Container;
	const IDS: &'static [ComponentId];

	fn retrieve(components: &Components) -> Option<<Self::Containers as Container>::Ref<'_>>;
	fn retrieve_mut(
		components: &mut Components,
	) -> Option<<Self::Containers as Container>::Mut<'_>>;
}

impl ComponentRetrieve for () {
	type Containers = ();
	const IDS: &'static [ComponentId] = &[];
	fn retrieve(_: &Components) -> Option<<Self::Containers as Container>::Ref<'_>> {
		Some(())
	}
	fn retrieve_mut(_: &mut Components) -> Option<<Self::Containers as Container>::Mut<'_>> {
		Some(())
	}
}
impl<C: Component> ComponentRetrieve for C {
	type Containers = C::Container;
	const IDS: &'static [ComponentId] = &[C::ID];

	fn retrieve(components: &Components) -> Option<<Self::Containers as Container>::Ref<'_>> {
		Some(
			components
				.get(&C::ID)?
				.downcast_ref::<C::Container>()?
				.container_ref(),
		)
	}
	fn retrieve_mut(
		components: &mut Components,
	) -> Option<<Self::Containers as Container>::Mut<'_>> {
		Some(
			components
				.get_mut(&C::ID)?
				.downcast_mut::<C::Container>()?
				.container_mut(),
		)
	}
}

impl Container for () {
	type Ref<'a> = ();
	type Mut<'a> = ();

	fn new() -> Self {}
	fn container_ref(&self) -> Self::Ref<'_> {}
	fn container_mut(&mut self) -> Self::Mut<'_> {}
}
impl<T: 'static> Container for Vec<T> {
	type Ref<'a> = &'a [T];
	type Mut<'a> = &'a mut [T];

	fn new() -> Self {
		Vec::new()
	}
	fn container_ref(&self) -> Self::Ref<'_> {
		self
	}
	fn container_mut(&mut self) -> Self::Mut<'_> {
		self
	}
}
impl<T: 'static> Container for IndexSet<T> {
	type Ref<'a> = &'a IndexSet<T>; // TODO: prevent structural modifications
	type Mut<'a> = &'a mut IndexSet<T>;

	fn new() -> Self {
		IndexSet::new()
	}
	fn container_ref(&self) -> Self::Ref<'_> {
		self
	}
	fn container_mut(&mut self) -> Self::Mut<'_> {
		self
	}
}
impl<T: 'static> Container for OneOrMany<T> {
	type Ref<'a> = &'a [T];
	type Mut<'a> = &'a mut [T];

	fn new() -> Self {
		OneOrMany::None
	}
	fn container_ref(&self) -> Self::Ref<'_> {
		self.as_slice()
	}
	fn container_mut(&mut self) -> Self::Mut<'_> {
		self.as_mut_slice()
	}
}
impl Container for BitVec {
	type Ref<'a> = &'a BitSlice;
	type Mut<'a> = &'a mut BitSlice;

	fn new() -> Self {
		BitVec::new()
	}
	fn container_ref(&self) -> Self::Ref<'_> {
		self
	}
	fn container_mut(&mut self) -> Self::Mut<'_> {
		self
	}
}
// GPU Buffer container defined in the URE GPU crate.

macro_rules! container_tuples {
	($($T:ident),*) => {
#[allow(non_snake_case)]
impl<$($T: Container),*> Container for ($($T),*) {
	type Ref<'a> = ($($T::Ref<'a>),*);
	type Mut<'a> = ($($T::Mut<'a>),*);

	fn new() -> Self {
		panic!("Component tuples are invalid containers.")
	}
	fn container_ref(&self) -> Self::Ref<'_> {
		let ($($T),*) = self;
		($($T.container_ref()),*)
	}
	fn container_mut(&mut self) -> Self::Mut<'_> {
		let ($($T),*) = self;
		($($T.container_mut()),*)
	}
}
#[allow(non_snake_case)]
impl<$($T: Component),*> ComponentRetrieve for ($($T),*) {
	type Containers = ($($T::Container),*);
	const IDS: &'static [ComponentId] = &[$($T::ID),*];

	fn retrieve(components: &Components) -> Option<<Self::Containers as Container>::Ref<'_>> {
		$(
			let $T = components
			.get(&$T::ID)?
			.downcast_ref::<$T::Container>()?
			.container_ref();
		)*
		Some(($($T),*))
	}
	fn retrieve_mut(
		components: &mut Components,
	) -> Option<<Self::Containers as Container>::Mut<'_>> {
		let [$($T),*] = components.get_disjoint_mut([$(&$T::ID),*]);
		$(
			let $T = $T?.downcast_mut::<$T::Container>()?.container_mut();
		)*
		Some(($($T),*))
	}
}
	};
}
container_tuples!(A, B);
container_tuples!(A, B, C);
container_tuples!(A, B, C, D);
container_tuples!(A, B, C, D, E);
container_tuples!(A, B, C, D, E, F);
container_tuples!(A, B, C, D, E, F, G);
container_tuples!(A, B, C, D, E, F, G, H);
container_tuples!(A, B, C, D, E, F, G, H, I);
container_tuples!(A, B, C, D, E, F, G, H, I, J);
