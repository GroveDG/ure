use std::cell::{Ref, RefCell, RefMut};

use slotmap::SlotMap;

use crate::{
	components::{Component, ComponentDependency, ComponentId, Components},
	containers::Container,
	method::{FromGroup, Method},
	signal,
	signals::{SignalId, Signals},
};

#[derive(Default)]
pub struct Group {
	len: usize,
	components: Components,
	signals: Signals,
}

signal!(pub NEW usize);
signal!(pub DELETE &[usize]);

#[derive(Debug, Clone)]
pub enum MethodError {
	MissingDependency(ComponentId),
}
#[derive(Debug, Clone)]
pub enum ComponentError {
	MissingDependency(ComponentId),
}

impl Group {
	pub fn add_component<C: Component>(&mut self) -> Result<(), ComponentError>
	where
		C::Container: Default,
	{
		self.add_container::<C>(Default::default())
	}
	pub fn add_container<C: Component>(
		&mut self,
		container: C::Container,
	) -> Result<(), ComponentError> {
		for depencency in C::dependencies() {
			if !self.components.contains(&depencency) {
				return Err(ComponentError::MissingDependency(depencency));
			}
		}
		self.signals.connect(&NEW, C::NEW);
		self.signals.connect(&DELETE, C::DELETE);
		self.components.add::<C>(container);
		Ok(())
	}
	pub fn connect_signal<'a, Args: Clone>(
		&mut self,
		signal_id: &SignalId<Args>,
		method: impl Into<Method<Args>>,
	) {
		self.signals.connect(signal_id, method.into());
	}
	pub fn new(&mut self, num: usize) {
		if num == 0 {
			return;
		}
		self.signals.call(&NEW, self, num);
		self.len += num;
	}
	pub fn delete(&mut self, indices: &[usize]) {
		if indices.len() == 0 {
			return;
		}
		self.signals.call(&DELETE, self, indices);
		self.len -= indices.len();
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
impl ComponentDependency for Len {
	fn dependencies() -> Vec<ComponentId> {
		Vec::new()
	}
}
impl<'a> FromGroup<'a> for Len {
	fn from_group(group: &'a Group) -> Option<Self>
	where
		Self: Sized,
	{
		Some(Self(group.len()))
	}
}
