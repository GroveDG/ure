use std::{any::Any, hash::Hash};

use indexmap::IndexSet;

use crate::data::Container;

pub struct BiMap<T: Any + Hash + Eq> {
    inner: IndexSet<T>,
    f: Box<dyn FnMut(&IndexSet<T>) -> T>,
}

impl<T: Any + Hash + Eq> Container for BiMap<T> {
    fn execute(&mut self, commands: &[super::ComponentCommand]) {
        for command in commands.iter().cloned() {
            match command {
                super::ComponentCommand::New { num } => {
                    for _ in 0..num {
                        self.inner.insert((self.f)(&self.inner));
                    }
                }
                super::ComponentCommand::Delete { range } => {
                    for i in range.rev() {
                        self.inner.swap_remove_index(i);
                    }
                },
            }
        }
    }
}
