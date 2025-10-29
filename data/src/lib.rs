use std::cell::RefCell;

use slotmap::SlotMap;

mod component;
mod container;
mod group;
mod resource;
mod system;
pub use component::{Component, ComponentContainer, ComponentId, ComponentIdInner, Components};
pub use container::{Container, ContainerDefault, One};
pub use group::{Group, Method, MethodId, Signal, SignalId};
pub use resource::Resource;
pub extern crate mident;

pub type Data<Key> = SlotMap<Key, RefCell<Group>>;
