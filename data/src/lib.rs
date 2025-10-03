use std::cell::RefCell;

use slotmap::SlotMap;

use crate::group::Group;

pub mod group;
pub mod resource;

pub type Data<Key> = SlotMap<Key, RefCell<Group>>;