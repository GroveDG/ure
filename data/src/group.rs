use std::any::Any;

use crate::{
	data::{Component, Components},
	func::{Func, FunctionId, Functions},
};

#[derive(Default)]
pub struct Group {
	pub(crate) components: Components,
	pub(crate) functions: Functions,
}

impl Group {
	pub fn impl_function<F: Any + Clone>(&mut self, func: &'static Func<F>) -> Option<()> {
		self.functions.implement(func, &self.components)
	}
	pub fn unimpl_function(&mut self, id: &FunctionId) {
		self.functions.unimplement(id);
	}
	pub fn add_component<C: Any, New: Any + Clone, Delete: Any + Clone>(
		&mut self,
		component: &Component<New, Delete>,
		container: C,
	) {
		self.components.add(component.id, container);
		if self.functions.implement(component.new, &self.components).is_some() {
			if self.functions.implement(component.delete, &self.components).is_some() {
				return;
			}
			self.unimpl_function(&component.new.id);
		};
		self.components.remove(&component.id);
	}
	// pub fn execute(&mut self, commands: Vec<ComponentCommand>) {
	//     for (_, component) in self.components.iter_mut() {
	//         component.execute(&commands);
	//     }
	// }
}
