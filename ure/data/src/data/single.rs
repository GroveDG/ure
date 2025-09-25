use std::any::Any;

use crate::data::Container;

#[repr(transparent)]
#[derive(Default)]
pub struct Single<T: Any + Default> {
    inner: T,
}

impl<T: Any + Default> Container for Single<T> {
    fn execute(&mut self, _: &[super::ComponentCommand]) {}
}
