use std::{
	any::Any,
	cell::{Cell, Ref, RefCell, RefMut},
	collections::HashMap,
};

use nohash_hasher::BuildNoHashHasher;
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

signal!(pub NEW: NewArgs);
signal!(pub DELETE: &[usize]);

#[derive(Debug, Copy, Clone)]
pub struct MissingDependency(pub ComponentId);

impl Group {
	pub fn add_component<C: Component>(&mut self) -> Result<(), MissingDependency>
	where
		C::Container: Default,
	{
		self.add_container::<C>(Default::default())
	}
	pub fn add_container<C: Component>(
		&mut self,
		container: C::Container,
	) -> Result<(), MissingDependency> {
		self.are_depencencies_satisfied(&C::dependencies())?;
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
		self.new_args(NewArgs::new(num));
	}
	pub fn new_with(&mut self, num: usize) -> GroupNew<'_> {
		GroupNew {
			group: self,
			args: NewArgs::new(num),
		}
	}
	fn new_args(&mut self, args: NewArgs) {
		let num = args.num();
		if num == 0 {
			return;
		}
		self.signals.call(&NEW, self, args);
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
	) -> Option<<C::Container as Container>::Ref<'_>> {
		self.components.borrow_component::<C>()
	}
	pub fn borrow_component_mut<C: Component>(
		&'_ self,
	) -> Option<<C::Container as Container>::RefMut<'_>> {
		self.components.borrow_component_mut::<C>()
	}
	pub fn len(&self) -> usize {
		self.len
	}
	pub fn is_empty(&self) -> bool {
		self.len == 0
	}
	pub fn contains_component<C: Component>(&self) -> bool {
		self.components.contains(&C::ID)
	}
	pub fn are_depencencies_satisfied(
		&self,
		dependencies: &[ComponentId],
	) -> Result<(), MissingDependency> {
		for depencency in dependencies {
			if !self.components.contains(&depencency) {
				return Err(MissingDependency(*depencency));
			}
		}
		Ok(())
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

pub struct NewArgs {
	num: usize,
	map: HashMap<ComponentId, Box<dyn Any>, BuildNoHashHasher<ComponentId>>,
}
impl NewArgs {
	pub fn new(num: usize) -> Self {
		Self {
			num,
			map: Default::default(),
		}
	}
	pub fn take<C: Component>(&mut self) -> Option<C::NewArg>
	where
		C::NewArg: 'static,
	{
		Some(*self.map.remove(&C::ID)?.downcast().unwrap())
	}
	pub fn add<C: Component>(&mut self, args: C::NewArg)
	where
		C::NewArg: 'static,
	{
		self.map.insert(C::ID, Box::new(args)); // TODO: Validate Args
	}
	pub fn num(&self) -> usize {
		self.num
	}
}

#[must_use]
pub struct GroupNew<'a> {
	group: &'a mut Group,
	args: NewArgs,
}
impl<'a> GroupNew<'a> {
	pub fn add<C: Component>(mut self, args: C::NewArg) -> Self
	where
		C::NewArg: 'static,
	{
		if !self.group.contains_component::<C>() {
			return self; // TODO: Return Err somewhere
		}
		self.args.add::<C>(args);
		self
	}
	pub fn done(self) {
		self.group.new_args(self.args);
	}
}
