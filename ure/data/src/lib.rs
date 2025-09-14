#![feature(allocator_api)]
#![feature(ptr_metadata)]
#![feature(alloc_layout_extra)]
#![feature(slice_ptr_get)]
#![feature(const_trait_impl)]
#![feature(const_cmp)]

mod data;
mod func;
mod group;
mod resource;

pub use data::{
    Component, ComponentId, Data, DataAny, DataBox, DataGeneric, DataMut, DataRef, DataSpecific,
};
pub use func::{Impl, FuncId, Func, Implr, ImplError};
pub use group::Group;
pub use resource::Resource;
// pub type Data<K> = slotmap::SlotMap<K, Group>;
