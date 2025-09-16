use std::any::Any;

use crate::{data::Component, func::Functions, Data, DataSpecific, Func};

#[derive(Default)]
pub struct Group {
    pub(crate) data: Data,
    pub(crate) funcs: Functions,
}

impl Group {
    pub fn add_function(&mut self, func: &'static Func) {
        self.funcs.add(&self.data, func);
    }
    pub fn call_function(&mut self) {
        self.funcs.ge
    }
    pub fn add_component<T: Any, D: DataSpecific<T>>(&mut self, comp: Component) {
        self.add_function(comp.new);
        self.data.insert(comp.id, Box::new(D::new_data()));
        self.
    }
}
