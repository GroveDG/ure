use std::any::Any;

use crate::{
	data::{Component, ComponentId, Components},
	func::{Func, FunctionId, Functions, ImplError, Method},
};

#[derive(Default)]
pub struct Group {
	pub(crate) components: Components,
	pub(crate) functions: Functions,
	method_chain: Vec<&'static Component>,
}

impl Group {
	pub fn impl_function<F: Any + Clone>(&mut self, func: &'static Func<F>) -> Option<ImplError> {
		self.functions.implement(func, &self.components)
	}
	pub fn unimpl_function(&mut self, id: &FunctionId) {
		self.functions.unimplement(id);
	}
	pub fn reimpl(&mut self) {
		self.functions.reimplement(&self.components);
	}
	pub fn add_component<C: Any, New: Any + Clone, Delete: Any + Clone>(
		&mut self,
		component: &'static Component,
		container: C,
	) {
		self.components.add(component.id, container);
		if self.functions.implement(component.new, &self.components).is_some() {
			if self.functions.implement(component.delete, &self.components).is_some() {
				self.method_chain.push(component);
				return;
			}
			self.unimpl_function(&component.new.id);
		};
		self.components.remove(&component.id);
	}
	pub fn remove_component(
		&mut self,
		id: &ComponentId
	) {
		self.components.remove(id);
		self.reimpl();
	}
	pub fn call_method(&mut self, func: &'static Func<impl Fn(&mut dyn Any, &[&dyn Any])>) {
		let Some(f) = self.functions.get(func) else {
			return
		};
		let mut_comp_id = func.components[0];
		let Some(mut mut_comp) = self.components.remove(&mut_comp_id) else {
			return
		};
		'call: {
			let mut comps = Vec::new();
			for id in func.components[1..].iter() {
				let Some(comp) = self.components.get(id) else {
					break 'call;
				};
				comps.push(comp);
			}
			(f)(&mut mut_comp, &comps);
		}
		self.components.insert(mut_comp_id, mut_comp);
	}
	// pub fn execute(&mut self, commands: Vec<ComponentCommand>) {
	//     for (_, component) in self.components.iter_mut() {
	//         component.execute(&commands);
	//     }
	// }
}
