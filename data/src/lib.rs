use std::cell::RefCell;

use slotmap::SlotMap;

mod group;
mod resource;
pub use group::{Container, Component, Get, Extract, Method, Group, Components, ComponentRetrieve, One};
pub use resource::Resource;

pub type Data<Key> = SlotMap<Key, RefCell<Group>>;