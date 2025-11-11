use std::{
	any::Any,
	cell::{self, RefCell},
	collections::HashMap,
	error::Error,
	fmt::Display,
};

use nohash_hasher::BuildNoHashHasher;

use crate::{
	containers::{Container, NewDefault},
	glob::{ContMut, GlobItemRef},
	group::Group,
	util::all_the_tuples,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(u64);
impl nohash_hasher::IsEnabled for ComponentId {}
impl ComponentId {
	pub const fn new(path: &str, name: &str) -> Self {
		let path_hash = const_fnv1a_hash::fnv1a_hash_str_64(path);
		let name_hash = const_fnv1a_hash::fnv1a_hash_str_64(name);
		Self(crate::util::hash_combine(path_hash, name_hash))
	}
}

pub trait ComponentDependency {
	fn dependencies() -> Vec<ComponentId>;
	fn method_dependencies(&self) -> Vec<ComponentId> {
		Self::dependencies()
	}
}

pub trait Component: Sized + ComponentDependency + 'static {
	const ID: ComponentId;
	type Container: Container;

	type NewArg: Sized + 'static;
	fn new(glob: GlobItemRef<'_>, args: &mut NewArgs) -> Result<(), Box<dyn Error>>;
	fn delete(glob: GlobItemRef<'_>, indices: &mut &[usize]) -> Result<(), Box<dyn Error>>;
}

pub struct NewArgs {
	len: usize,
	args: HashMap<ComponentId, Box<dyn Any>, BuildNoHashHasher<ComponentId>>,
}
impl NewArgs {
	pub const fn len(&self) -> usize {
		self.len
	}
	pub fn take<C: Component>(&mut self) -> Option<C::NewArg> {
		Some(*self.args.remove(&C::ID)?.downcast().unwrap())
	}
}

#[macro_export]
macro_rules! component {
	($v:vis $name:ident: $container:ty) => {
$v struct $name;
impl $crate::components::ComponentDependency for $name {
	fn dependencies() -> Vec<$crate::components::ComponentId> {
		Vec::new()
	}
}
impl $crate::components::Component for $name {
	const ID: $crate::components::ComponentId = $crate::components::ComponentId::new(std::module_path!(), stringify!($name));
	type Container = $container;

	type NewArg = ();

	fn new<'a>(glob: $crate::glob::GlobItemRef<'a>, args: &mut $crate::components::NewArgs) -> Result<(), Box<dyn std::error::Error>> {
		$crate::components::new_default::<Self>.call_method(glob, args)
	}
	fn delete(glob: $crate::glob::GlobItemRef<'_>, indices: &mut &[usize]) -> Result<(), Box<dyn std::error::Error>> {
		$crate::components::delete_default::<Self>.call_method(glob, indices)
	}
}
	};
	($v:vis $name:ident: $container:ty, $new:expr $(, $new_arg:ty)?) => {
$v struct $name;
impl $crate::components::ComponentDependency for $name {
	fn dependencies() -> Vec<$crate::components::ComponentId> {
		<&dyn $crate::method::MethodTrait<_, _, _>>::method_dependencies(&(&$new as &dyn $crate::method::MethodTrait<_, _, _>))
	}
}
impl $crate::components::Component for $name {
	const ID: $crate::components::ComponentId = $crate::components::ComponentId::new(std::module_path!(), stringify!($name));
	type Container = $container;

	type NewArg = ($($new_arg)?);
	fn new<'a>(glob: $crate::glob::GlobItemRef<'a>, args: &mut $crate::components::NewArgs) -> Result<(), Box<dyn std::error::Error>> {
		($new).call_method(glob, args)
	}
	fn delete(glob: $crate::glob::GlobItemRef<'_>, indices: &mut &[usize]) -> Result<(), Box<dyn std::error::Error>> {
		$crate::components::delete_default::<Self>.call_method(glob, indices)
	}
}
	};
}

pub fn new_default<C: Component>(ContMut(mut c): ContMut<C>, args: &mut NewArgs)
where
	for<'a> C: ComponentGroup<ContainersRefMut<'a> = std::cell::RefMut<'a, C::Container>>,
	C::Container: NewDefault,
{
	c.new_default(args.len());
}

pub fn delete_default<C: ComponentGroup + Component>(
	ContMut(mut c): ContMut<C>,
	indices: &mut &[usize],
) where
	for<'a> C: ComponentGroup<ContainersRefMut<'a> = std::cell::RefMut<'a, C::Container>>,
{
	c.delete(indices);
}

#[derive(Debug, Default)]
pub struct Components {
	inner: HashMap<ComponentId, RefCell<Box<dyn Any>>, BuildNoHashHasher<ComponentId>>,
}
impl Components {
	pub fn add<C: Component>(&mut self, container: C::Container) {
		self.inner.insert(C::ID, RefCell::new(Box::new(container)));
	}
	pub fn borrow_container<C: Component>(&'_ self) -> Option<std::cell::Ref<'_, C::Container>> {
		Some(
			cell::Ref::filter_map(self.inner.get(&C::ID)?.borrow(), |c| {
				Some(c.downcast_ref::<C::Container>()?)
			})
			.unwrap(),
		)
	}
	pub fn borrow_container_mut<C: Component>(
		&'_ self,
	) -> Option<std::cell::RefMut<'_, C::Container>> {
		Some(
			cell::RefMut::filter_map(self.inner.get(&C::ID)?.borrow_mut(), |c| {
				Some(c.downcast_mut::<C::Container>()?)
			})
			.unwrap(),
		)
	}
	pub fn borrow_component<C: Component>(
		&'_ self,
	) -> Option<<C::Container as Container>::Ref<'_>> {
		Some(<C::Container as Container>::as_ref(
			cell::Ref::filter_map(self.inner.get(&C::ID)?.borrow(), |c| {
				Some(c.downcast_ref::<C::Container>()?)
			})
			.unwrap(),
		))
	}
	pub fn borrow_component_mut<C: Component>(
		&'_ self,
	) -> Option<<C::Container as Container>::RefMut<'_>> {
		Some(<C::Container as Container>::as_mut(
			cell::RefMut::filter_map(self.inner.get(&C::ID)?.borrow_mut(), |c| {
				Some(c.downcast_mut::<C::Container>()?)
			})
			.unwrap(),
		))
	}
	pub fn contains(&self, id: &ComponentId) -> bool {
		self.inner.contains_key(id)
	}
}

#[derive(Debug, Copy, Clone)]
pub struct MissingDependency(pub ComponentId);
impl Display for MissingDependency {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.0)
	}
}
impl Error for MissingDependency {}

pub trait ComponentGroup {
	const IDS: &'static [ComponentId];

	type ContainersRef<'a>;
	fn borrow_containers(group: &Group) -> Result<Self::ContainersRef<'_>, MissingDependency>;
	type ContainersRefMut<'a>;
	fn borrow_containers_mut(
		group: &Group,
	) -> Result<Self::ContainersRefMut<'_>, MissingDependency>;
	type ComponentsRef<'a>;
	fn borrow_components(group: &Group) -> Result<Self::ComponentsRef<'_>, MissingDependency>;
	type ComponentsRefMut<'a>;
	fn borrow_components_mut(
		group: &Group,
	) -> Result<Self::ComponentsRefMut<'_>, MissingDependency>;
}

macro_rules! impl_component_group {
	($($C:ident),*) => {
#[allow(unused_parens, unused_variables)]
impl<$($C: Component),*> ComponentGroup for ($($C),*) {
	const IDS: &'static [ComponentId] = &[
		$(<$C as Component>::ID),*
	];

	type ContainersRef<'a> = (
		$(std::cell::Ref<'a, <$C as Component>::Container>),*
	);
	fn borrow_containers(group: &Group) -> Result<Self::ContainersRef<'_>, MissingDependency> {
		Ok(($( group.borrow_container::<$C>().ok_or(MissingDependency(<$C as Component>::ID))? ),*))
	}
	type ContainersRefMut<'a> = (
		$(std::cell::RefMut<'a, <$C as Component>::Container>),*
	);
	fn borrow_containers_mut(group: &Group) -> Result<Self::ContainersRefMut<'_>, MissingDependency> {
		Ok(($( group.borrow_container_mut::<$C>().ok_or(MissingDependency(<$C as Component>::ID))? ),*))
	}
	type ComponentsRef<'a> = (
		$(<<$C as Component>::Container as Container>::Ref<'a>),*
	);
	fn borrow_components(group: &Group) -> Result<Self::ComponentsRef<'_>, MissingDependency> {
		Ok(($( group.borrow_component::<$C>().ok_or(MissingDependency(<$C as Component>::ID))? ),*))
	}
	type ComponentsRefMut<'a> = (
		$(<<$C as Component>::Container as Container>::RefMut<'a>),*
	);
	fn borrow_components_mut(group: &Group) -> Result<Self::ComponentsRefMut<'_>, MissingDependency> {
		Ok(($( group.borrow_component_mut::<$C>().ok_or(MissingDependency(<$C as Component>::ID))? ),*))
	}
}
	};
}
all_the_tuples!(impl_component_group);
