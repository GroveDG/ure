use std::{
	any::{Any, TypeId},
	cell::{self, RefCell},
	collections::HashMap,
};

use nohash_hasher::BuildNoHashHasher;

use crate::{
	containers::{Container, NewDefault, NewWith},
	group::{Group, NewArgs},
	method::{FromGroup, Method},
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
}

pub trait Component: Sized {
	const ID: ComponentId;
	type Container: Container;

	const NEW: Method<&NewArgs>;
	const DELETE: Method<&[usize]>;

	type NewArg;
}
impl<C: Component> ComponentDependency for C {
	fn dependencies() -> Vec<ComponentId> {
		C::NEW
			.dependencies()
			.into_iter()
			.chain(C::DELETE.dependencies().into_iter())
			.filter(|&c| c != C::ID)
			.collect()
	}
}

#[macro_export]
macro_rules! component {
	($v:vis $name:ident: $container:ty) => {
$v struct $name;
impl $crate::components::Component for $name {
	const ID: $crate::components::ComponentId = $crate::components::ComponentId::new(std::module_path!(), stringify!($name));
	type Container = $container;

	const NEW: $crate::method::Method<&$crate::group::NewArgs> = $crate::method::Method::new($crate::components::default_new::<Self, $container> as fn(_, _));
	const DELETE: $crate::method::Method<&[usize]> = $crate::method::Method::new($crate::components::default_delete::<Self> as fn(_, _));

	type NewArg = <Self::Container as $crate::containers::NewWith>::Args;
}
	};
	($v:vis $name:ident: $container:ty, $new:expr, $new_arg:ty) => {
$v struct $name;
impl $crate::components::Component for $name {
	const ID: $crate::components::ComponentId = $crate::components::ComponentId::new(std::module_path!(), stringify!($name));
	type Container = $container;

	const NEW: $crate::method::Method<&$crate::group::NewArgs> = $crate::method::Method::new($new);
	const DELETE: $crate::method::Method<&[usize]> = $crate::method::Method::new($crate::components::default_delete::<Self> as fn(_, _));

	type NewArg = $new_arg;
}
	};
}

pub fn default_new<C, Cont>(ContMut(mut container): ContMut<C>, args: &NewArgs)
where
	Cont: Container,
	Cont: NewWith + NewDefault,
	C: Component<Container = Cont, NewArg = <Cont as NewWith>::Args>
{
	assert_eq!(
		TypeId::of::<C::NewArg>(),
		TypeId::of::<<C::Container as NewWith>::Args>()
	);
	if let Some(args) = args.take::<C>() {
		container.new_with(unsafe { std::mem::transmute(args) });
	} else {
		container.new_default(args.num())
	}
}

pub fn default_delete<C: Component>(ContMut(mut container): ContMut<C>, indices: &[usize]) {
	container.delete(indices);
}

#[derive(Debug, Default)]
pub struct Components {
	inner: HashMap<ComponentId, RefCell<Box<dyn Any>>, BuildNoHashHasher<ComponentId>>,
}
impl Components {
	pub fn add<C: Component>(&mut self, container: C::Container) {
		self.inner.insert(C::ID, RefCell::new(Box::new(container)));
	}
	pub fn borrow_container<C: Component>(&'_ self) -> Option<cell::Ref<'_, C::Container>> {
		cell::Ref::filter_map(self.inner.get(&C::ID)?.try_borrow().ok()?, |c| {
			Some(c.downcast_ref::<C::Container>()?)
		})
		.ok()
	}
	pub fn borrow_container_mut<C: Component>(&'_ self) -> Option<cell::RefMut<'_, C::Container>> {
		cell::RefMut::filter_map(self.inner.get(&C::ID)?.try_borrow_mut().ok()?, |c| {
			Some(c.downcast_mut::<C::Container>()?)
		})
		.ok()
	}
	pub fn borrow_component<C: Component>(
		&'_ self,
	) -> Option<cell::Ref<'_, <C::Container as Container>::Slice>> {
		cell::Ref::filter_map(self.inner.get(&C::ID)?.try_borrow().ok()?, |c| {
			Some(c.downcast_ref::<C::Container>()?.as_ref())
		})
		.ok()
	}
	pub fn borrow_component_mut<C: Component>(
		&'_ self,
	) -> Option<cell::RefMut<'_, <C::Container as Container>::Slice>> {
		cell::RefMut::filter_map(self.inner.get(&C::ID)?.try_borrow_mut().ok()?, |c| {
			Some(c.downcast_mut::<C::Container>()?.as_mut())
		})
		.ok()
	}
	pub fn contains(&self, id: &ComponentId) -> bool {
		self.inner.contains_key(id)
	}
}

pub trait ComponentGroup {
	const IDS: &'static [ComponentId];
	type ContainersRef<'a>;
	fn borrow_containers(group: &Group) -> Option<Self::ContainersRef<'_>>;
	type ContainersRefMut<'a>;
	fn borrow_containers_mut(group: &Group) -> Option<Self::ContainersRefMut<'_>>;
	type ComponentsRef<'a>;
	fn borrow_components(group: &Group) -> Option<Self::ComponentsRef<'_>>;
	type ComponentsRefMut<'a>;
	fn borrow_components_mut(group: &Group) -> Option<Self::ComponentsRefMut<'_>>;
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
	fn borrow_containers(group: &Group) -> Option<Self::ContainersRef<'_>> {
		Some(($( group.borrow_container::<$C>()? ),*))
	}
	type ContainersRefMut<'a> = (
		$(std::cell::RefMut<'a, <$C as Component>::Container>),*
	);
	fn borrow_containers_mut(group: &Group) -> Option<Self::ContainersRefMut<'_>> {
		Some(($( group.borrow_container_mut::<$C>()? ),*))
	}
	type ComponentsRef<'a> = (
		$(std::cell::Ref<'a, <<$C as Component>::Container as Container>::Slice>),*
	);
	fn borrow_components(group: &Group) -> Option<Self::ComponentsRef<'_>> {
		Some(($( group.borrow_component::<$C>()? ),*))
	}
	type ComponentsRefMut<'a> = (
		$(std::cell::RefMut<'a, <<$C as Component>::Container as Container>::Slice>),*
	);
	fn borrow_components_mut(group: &Group) -> Option<Self::ComponentsRefMut<'_>> {
		Some(($( group.borrow_component_mut::<$C>()? ),*))
	}
}
	};
}
all_the_tuples!(impl_component_group);

pub struct ContRef<'a, C: ComponentGroup>(pub C::ContainersRef<'a>);
impl<'a, C: ComponentGroup> ComponentDependency for ContRef<'a, C> {
	fn dependencies() -> Vec<ComponentId> {
		C::IDS.to_vec()
	}
}
impl<'a, C: ComponentGroup> FromGroup<'a> for ContRef<'a, C> {
	fn from_group(group: &'a Group) -> Option<Self>
	where
		Self: Sized,
	{
		Some(Self(C::borrow_containers(group)?))
	}
}
pub struct ContMut<'a, C: ComponentGroup>(pub C::ContainersRefMut<'a>);
impl<'a, C: ComponentGroup> ComponentDependency for ContMut<'a, C> {
	fn dependencies() -> Vec<ComponentId> {
		C::IDS.to_vec()
	}
}
impl<'a, C: ComponentGroup> FromGroup<'a> for ContMut<'a, C> {
	fn from_group(group: &'a Group) -> Option<Self>
	where
		Self: Sized,
	{
		Some(Self(C::borrow_containers_mut(group)?))
	}
}
pub struct CompRef<'a, C: ComponentGroup>(pub C::ComponentsRef<'a>);
impl<'a, C: ComponentGroup> ComponentDependency for CompRef<'a, C> {
	fn dependencies() -> Vec<ComponentId> {
		C::IDS.to_vec()
	}
}
impl<'a, C: ComponentGroup> FromGroup<'a> for CompRef<'a, C> {
	fn from_group(group: &'a Group) -> Option<Self>
	where
		Self: Sized,
	{
		Some(Self(C::borrow_components(group)?))
	}
}
impl<'a, C: ComponentGroup> ComponentDependency for CompMut<'a, C> {
	fn dependencies() -> Vec<ComponentId> {
		C::IDS.to_vec()
	}
}
pub struct CompMut<'a, C: ComponentGroup>(pub C::ComponentsRefMut<'a>);
impl<'a, C: ComponentGroup> FromGroup<'a> for CompMut<'a, C> {
	fn from_group(group: &'a Group) -> Option<Self>
	where
		Self: Sized,
	{
		Some(Self(C::borrow_components_mut(group)?))
	}
}
impl<'a, C: 'a + FromGroup<'a>> ComponentDependency for Option<C> {
	fn dependencies() -> Vec<ComponentId> {
		Vec::new()
	}
}
impl<'a, C: 'a + FromGroup<'a>> FromGroup<'a> for Option<C> {
	fn from_group(group: &'a Group) -> Option<Self>
	where
		Self: Sized,
	{
		Some(C::from_group(group))
	}
}
