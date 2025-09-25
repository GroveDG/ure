use bitvec::vec::BitVec;

use crate::data::Container;

pub struct BitData {
    inner: BitVec,
    f: Box<dyn FnMut() -> bool>,
}

impl Container for BitData {
    fn execute(&mut self, commands: &[super::ComponentCommand]) {
        for command in commands.iter().cloned() {
            match command {
                super::ComponentCommand::New { num } => self
                    .inner
                    .resize_with(self.inner.len() + num, |_| (self.f)()),
                super::ComponentCommand::Delete { range } => {
                    for i in range.rev() {
                        self.inner.swap_remove(i);
                    }
                },
            }
        }
    }
}
