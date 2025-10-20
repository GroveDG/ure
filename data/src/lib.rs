use std::cell::RefCell;

use slotmap::SlotMap;

mod group;
mod resource;
pub use group::{
    ComponentStruct, ComponentId, Components, Container, Group, Method, MethodId, One, Signal, SignalId,
};
pub use resource::Resource;
pub extern crate mident;

pub type Data<Key> = SlotMap<Key, RefCell<Group>>;
