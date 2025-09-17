use std::ops::RangeBounds;

use crate::{
    data::Data,
    func::{Func, Functions},
};

#[derive(Default)]
pub struct Group {
    pub(crate) data: Data,
    pub(crate) funcs: Functions,
}

impl Group {
    pub fn add_function(&mut self, func: &'static Func) {
        self.funcs.add(&self.data, func);
    }
    pub fn call(&mut self, func: &'static Func, range: impl RangeBounds<usize>) {
        let Some(range) = self.data.validate_range(range) else {
            return;
        };
        let Some(func) = self.funcs.get(func) else {
            return;
        };
        (func)(&mut self.data, range)
    }
}