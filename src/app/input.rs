use std::collections::HashSet;

use winit::keyboard::Key;

use super::Uid;



#[derive(Debug, Default, Clone)]
pub struct Input {
    pub pressed: HashSet<Key>,
    pub close: HashSet<Uid>
}