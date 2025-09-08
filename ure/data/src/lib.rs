#![feature(allocator_api)]
#![feature(ptr_metadata)]
#![feature(alloc_layout_extra)]

mod resource;
mod component;
// mod group;

pub use component::{ComponentsBox, ComponentsAny, Components, Elements};
pub use resource::Resource;
// pub use group::Group;
// pub type Data<K> = slotmap::SlotMap<K, Group>;