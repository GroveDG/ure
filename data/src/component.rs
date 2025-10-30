use std::{
	any::Any,
	cell::{self, RefCell},
	collections::HashMap,
};

use nohash_hasher::BuildNoHashHasher;

use crate::{container::Container, group::Group, method::FromGroup};

pub trait Component {
	const ID: u64;
	type Container: Container;
}

pub const fn component_id(name: &str) -> u64 {
	const_fnv1a_hash::fnv1a_hash_str_64(name)
}

#[macro_export]
macro_rules! component {
	($v:vis $name:ident, $container:ty) => {
$v struct $name;
impl $crate::Component for $name {
	const ID: u64 = $crate::component_id(stringify!($name));
	type Container = $container;
}
	};
}

pub struct ContainerRef<'a, C: Component>(pub cell::Ref<'a, C::Container>);
impl<'a, C: Component> FromGroup<'a> for ContainerRef<'a, C> {
	fn from_group(group: &'a Group) -> Option<Self>
	where
		Self: 'a + Sized,
	{
		Some(Self(group.borrow_container::<C>()?))
	}
}
pub struct ContainerMut<'a, C: Component>(pub cell::RefMut<'a, C::Container>);
impl<'a, C: Component> FromGroup<'a> for ContainerMut<'a, C> {
	fn from_group(group: &'a Group) -> Option<Self>
	where
		Self: 'a + Sized,
	{
		Some(Self(group.borrow_container_mut::<C>()?))
	}
}
pub struct CRef<'a, C: Component>(pub cell::Ref<'a, <C::Container as Container>::Slice>);
impl<'a, C: Component> FromGroup<'a> for CRef<'a, C> {
	fn from_group(group: &'a Group) -> Option<Self>
	where
		Self: 'a + Sized,
	{
		Some(Self(group.borrow_component::<C>()?))
	}
}
pub struct CMut<'a, C: Component>(pub cell::RefMut<'a, <C::Container as Container>::Slice>);
impl<'a, C: Component> FromGroup<'a> for CMut<'a, C> {
	fn from_group(group: &'a Group) -> Option<Self>
	where
		Self: 'a + Sized,
	{
		Some(Self(group.borrow_component_mut::<C>()?))
	}
}

#[derive(Debug, Default)]
pub struct Components {
	inner: HashMap<u64, RefCell<Box<dyn Any>>, BuildNoHashHasher<u64>>,
}
impl Components {
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
	pub fn borrow_component<C: Component>(&'_ self) -> Option<cell::Ref<'_, <C::Container as Container>::Slice>> {
		cell::Ref::filter_map(self.inner.get(&C::ID)?.try_borrow().ok()?, |c| {
			Some(c.downcast_ref::<C::Container>()?.as_ref())
		})
		.ok()
	}
	pub fn borrow_component_mut<C: Component>(&'_ self) -> Option<cell::RefMut<'_, <C::Container as Container>::Slice>> {
		cell::RefMut::filter_map(self.inner.get(&C::ID)?.try_borrow_mut().ok()?, |c| {
			Some(c.downcast_mut::<C::Container>()?.as_mut())
		})
		.ok()
	}
}
