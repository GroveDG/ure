use std::cell::{Ref, RefCell, RefMut};

use slotmap::SlotMap;

use crate::{
	component::{Component, Components},
	container::Container,
};

#[derive(Default)]
pub struct Group {
	len: usize,
	components: Components,
}

impl Group {
	// pub fn add_component<C: Container>(
	// 	&mut self,
	// 	component: ComponentContainer<C>,
	// ) -> Result<(), ()> {
	// 	let ComponentContainer::<C> {
	// 		id,
	// 		container,
	// 		len,
	// 		new,
	// 		delete,
	// 	} = component;

	// 	let (new_id, delete_id) = (new.id, delete.id);
	// 	let new_func = new.func;

	// 	self.components
	// 		.insert(id.inner(), UnsafeCell::new(Box::new(container)));
	// 	let new_ok = self.connect(NEW, new).is_ok();
	// 	let delete_ok = self.connect(DELETE, delete).is_ok();
	// 	if new_ok && delete_ok {
	// 		(new_func)(&mut self.components, len..self.len);
	// 		return Ok(());
	// 	}
	// 	if !new_ok {
	// 		self.disconnect(&NEW.inner, &new_id.inner).unwrap();
	// 	}
	// 	if !delete_ok {
	// 		self.disconnect(&DELETE.inner, &delete_id.inner).unwrap();
	// 	}
	// 	self.components.remove(&id.inner());
	// 	Err(())
	// }
	// pub fn remove_component<C: Container>(&mut self, id: &ComponentId<C>) {
	// 	self.components.remove(&id.inner());

	// 	for method in self.dependency.remove(&id.inner()).unwrap() {
	// 		for signal in self.connection.remove(&method).unwrap() {
	// 			self.signals.get_mut(&signal).unwrap().disconnect(&method);
	// 		}
	// 	}
	// }
	// fn connect<Args: Clone>(
	// 	&mut self,
	// 	signal_id: SignalId<Args>,
	// 	method: Method<Args>,
	// ) -> Result<(), ()> {
	// 	let Method::<Args> {
	// 		id,
	// 		func,
	// 		dependencies,
	// 	} = method;

	// 	for component in dependencies {
	// 		if !self.components.contains_key(component) {
	// 			return Err(());
	// 		}
	// 	}
	// 	let Some(signal) = self.signals.get_signal_mut(&signal_id) else {
	// 		return Err(());
	// 	};
	// 	signal.insert(id, func);

	// 	for component in method.dependencies.iter().copied() {
	// 		self.dependency.insert(component, id.inner);
	// 	}
	// 	self.connection.insert(id.inner, signal_id.inner);

	// 	Ok(())
	// }
	// fn disconnect(&mut self, signal: &SignalIdInner, method: &MethodIdInner) -> Result<(), ()> {
	// 	let Some(signal) = self.signals.get_mut(signal) else {
	// 		return Err(());
	// 	};
	// 	signal.disconnect(method);
	// 	Ok(())
	// }
	// pub fn call_signal<Args: Clone>(&mut self, id: &SignalId<Args>, args: Args) -> Result<(), ()> {
	// 	let Some(signal) = self.signals.get_signal(id) else {
	// 		return Err(());
	// 	};
	// 	signal.call(&mut self.components, args);
	// 	Ok(())
	// }
	// pub fn call_method<Args, Return>(
	// 	&mut self,
	// 	method: Method<Args, Return>,
	// 	args: Args,
	// ) -> Return {
	// 	(method.func)(&mut self.components, args)
	// }
	// pub fn new(&mut self, num: usize) {
	// 	self.call_signal(&NEW, self.len..self.len + num).unwrap();
	// 	self.len += num;
	// }
	// pub fn delete(&mut self, num: usize) {
	// 	self.call_signal(&DELETE, self.len..self.len + num).unwrap();
	// 	self.len -= num;
	// }
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
