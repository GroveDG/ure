use crate::{Component, Components};

pub struct Group {
    pub(crate) components: Components,
}

impl Group {
    pub fn add_component(&mut self, c: &'static Component) {
        Components::add(self, c);
    }
}