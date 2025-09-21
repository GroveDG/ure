use crate::{data::Components, func::Interface};

#[derive(Default)]
pub struct Group {
    pub(crate) components: Components,
}

impl Group {
    pub fn impl_interface<'a, I: Interface<'a>>(&'a mut self) -> Option<()> {
        let intr = I::implement(&self.components)?;
        (intr)(&mut self.components);
        Some(())
    }
}
