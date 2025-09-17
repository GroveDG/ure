#![feature(allocator_api)]
#![feature(ptr_metadata)]
#![feature(alloc_layout_extra)]
#![feature(slice_ptr_get)]
#![feature(const_trait_impl)]
#![feature(const_cmp)]
#![feature(ptr_as_ref_unchecked)]
#![allow(refining_impl_trait)]

pub mod data;
pub mod func;
pub mod group;
pub mod resource;

// pub type Data<K> = slotmap::SlotMap<K, Group>;
