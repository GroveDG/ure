use std::collections::HashSet;

use winit::keyboard::Key;

#[derive(Debug, Default, Clone)]
pub struct Input {
    pub pressed: HashSet<Key>,
    pub close: bool,
}