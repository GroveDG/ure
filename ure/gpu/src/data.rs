use std::{any::TypeId, marker::PhantomData, ops::{Deref, DerefMut}};

use bytemuck::Pod;
use ure_data::data::{vec::UnboundSlice, DataAny, DataSpecific};
use wgpu::{Buffer, BufferDescriptor, BufferSlice, BufferUsages, BufferView, BufferViewMut};

use crate::GPU;

pub struct BufferTyped<T: Pod> {
    inner: Buffer,
    len: usize,
    label: Option<&'static str>,
    usage: BufferUsages,
    _marker: PhantomData<T>,
}
impl<T: Pod> BufferTyped<T> {
    pub fn new(capacity: usize, label: Option<&'static str>, usage: BufferUsages) -> Self {
        let usage = usage & BufferUsages::MAP_READ & BufferUsages::MAP_WRITE;
        Self {
            inner: GPU.device.create_buffer(&BufferDescriptor {
                label,
                size: (capacity * size_of::<T>()) as u64,
                usage,
                mapped_at_creation: false,
            }),
            len: 0,
            label,
            usage,
            _marker: Default::default(),
        }
    }
    pub fn capacity(&self) -> usize {
        self.inner.size() as usize / size_of::<T>()
    }
    pub fn size_with(len: usize) -> u64 {
        (size_of::<T>() * len) as u64
    }
    pub fn len_of(size: u64) -> usize {
        size as usize * size_of::<T>()
    }
    pub fn initialized<'a>(&'a self) -> BufferSlice<'a> {
        self.inner.slice(..Self::size_with(self.len))
    }
    pub fn view<'a>(&'a self) -> BufferViewTyped<'a, T> {
        BufferViewTyped {
            view: self.initialized().get_mapped_range(),
            _marker: Default::default()
        }
    }
    pub fn view_mut<'a>(&'a mut self) -> BufferViewMutTyped<'a, T> {
        BufferViewMutTyped {
            view: self.initialized().get_mapped_range_mut(),
            _marker: Default::default()
        }
    }
}

impl<T: Pod> DataAny for BufferTyped<T> {
    fn inner_type(&self) -> std::any::TypeId {
        TypeId::of::<T>()
    }
    fn reserve(&mut self, additional: usize) {
        let old_capacity = self.capacity();
        let new_capacity = old_capacity + additional;
        let new_buffer = GPU.device.create_buffer(&BufferDescriptor {
            label: self.label,
            size: Self::size_with(new_capacity),
            usage: self.usage,
            mapped_at_creation: true,
        });
        {
            self.inner.map_async(mode, bounds, callback);
            let old_view = self.inner.get_mapped_range(..);
            let mut new_view = new_buffer.get_mapped_range_mut(..);
            let old_slice: &[T] = bytemuck::cast_slice(&old_view);
            let new_slice: &mut [T] = bytemuck::cast_slice_mut(&mut new_view);
            new_slice[..old_capacity].copy_from_slice(old_slice);
        }
        self.inner.unmap();
        new_buffer.unmap();
        self.inner = new_buffer;
    }
}
impl<T: Pod> DataSpecific for BufferTyped<T> {
    type Inner = T;
    type Slice = UnboundSlice<T>;

    fn slice_ref<'a: 'b, 'b>(&'a self) -> (ure_data::data::Mooring<'a>, &'b Self::Slice) {
        let view = self.view();
        (Some(Box::new(view)), UnboundSlice::from_slice(&view))
    }
    fn slice_mut<'a: 'b, 'b>(&'a mut self) -> (ure_data::data::Mooring<'a>, &'b mut Self::Slice) {
        todo!()
    }
    fn new_data() -> Self {
        todo!()
    }
}

// Slice
// ================================================================
#[repr(transparent)]
pub struct BufferViewTyped<'a, T> {
    view: BufferView<'a>,
    _marker: PhantomData<T>,
}
impl<'a, T: Pod> Deref for BufferViewTyped<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        bytemuck::cast_slice(&self.view)
    }
}
#[repr(transparent)]
pub struct BufferViewMutTyped<'a, T> {
    view: BufferViewMut<'a>,
    _marker: PhantomData<T>,
}
impl<'a, T: Pod> Deref for BufferViewMutTyped<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        bytemuck::cast_slice(&self.view)
    }
}
impl<'a, T: Pod> DerefMut for BufferViewMutTyped<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        bytemuck::cast_slice_mut(&mut self.view)
    }
}