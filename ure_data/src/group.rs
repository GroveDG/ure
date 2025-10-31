use std::{
	cell::{Ref, RefCell, RefMut},
	ops::Range,
};

use slotmap::SlotMap;

use crate::{
	components::{Component, Components, ContMut},
	container::Container,
	method::FromGroup, signal::Signals,
};

#[derive(Default)]
pub struct Group {
	len: usize,
	components: Components,
	signals: Signals,
}

impl Group {
	pub fn add_component<C: Component>(&mut self)
	where
		C::Container: Default,
	{
		self.components.add::<C>(Default::default());
	}
	pub fn add_container<C: Component>(&mut self, container: C::Container) {
		self.components.add::<C>(container);
	}
	pub fn borrow_container<C: Component>(&'_ self) -> Option<Ref<'_, C::Container>> {
		self.components.borrow_container::<C>()
	}
	pub fn borrow_container_mut<C: Component>(&'_ self) -> Option<RefMut<'_, C::Container>> {
		self.components.borrow_container_mut::<C>()
	}
	pub fn borrow_component<C: Component>(
		&'_ self,
	) -> Option<Ref<'_, <C::Container as Container>::Slice>> {
		self.components.borrow_component::<C>()
	}
	pub fn borrow_component_mut<C: Component>(
		&'_ self,
	) -> Option<RefMut<'_, <C::Container as Container>::Slice>> {
		self.components.borrow_component_mut::<C>()
	}
	pub fn len(&self) -> usize {
		self.len
	}
	pub fn is_empty(&self) -> bool {
		self.len == 0
	}
}

pub type Data<Key> = SlotMap<Key, RefCell<Group>>;

pub struct Len(pub usize);
impl<'a> FromGroup<'a> for Len {
	fn from_group(group: &'a Group) -> Option<Self>
	where
		Self: Sized,
	{
		Some(Self(group.len()))
	}
}
