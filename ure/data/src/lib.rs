#![feature(allocator_api)]
#![feature(ptr_metadata)]
#![feature(alloc_layout_extra)]
#![feature(slice_ptr_get)]

mod data;
mod func;
mod resource;
mod group;

pub use data::{
    DataTyped, DataAny, DataBox, DataMut, DataRef, DataGeneric,
    Data, ComponentId,
};
pub use func::{
    Func, Implr, FuncId
};
pub use group::Group;
pub use resource::Resource;
// pub type Data<K> = slotmap::SlotMap<K, Group>;
