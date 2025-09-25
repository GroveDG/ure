use std::any::Any;

use crate::data::{ComponentCommand, Container};

pub struct VecData<T: Any> {
    inner: Vec<T>,
    f: Box<dyn FnMut() -> T>,
}

impl<T: Any> Container for VecData<T> {
    fn execute(&mut self, commands: &[super::ComponentCommand]) {
        for command in commands.iter().cloned() {
            match command {
                ComponentCommand::New { num } => {
                    self.inner.resize_with(self.inner.len() + num, self.f.as_mut());
                },
                ComponentCommand::Delete { range } => {
                    for i in range.rev() {
                        self.inner.swap_remove(i);
                    }
                },
            }
        }
    }
}

impl<T: Any + Default> Default for VecData<T> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            f: Box::new(T::default),
        }
    }
}
