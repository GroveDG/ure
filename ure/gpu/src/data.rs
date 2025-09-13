use std::{any::TypeId, marker::PhantomData};

use bytemuck::Pod;
use ure_data::{DataAny, DataGeneric, DataMut, DataRef, DataTyped};
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
}
impl<T: Sized + 'static + Pod> DataAny for BufferTyped<T> {
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
impl<T: Pod> DataTyped<T> for BufferTyped<T> {
    type View<'a> = BufferViewTyped<'a, T>;
    type ViewMut<'a> = BufferViewMutTyped<'a, T>;

    fn view<'a>(&'a self) -> Self::View<'a> {
        BufferViewTyped {
            inner: self.initialized().get_mapped_range(),
            _marker: Default::default(),
        }
    }
    fn view_mut<'a>(&'a mut self) -> Self::ViewMut<'a> {
        BufferViewMutTyped {
            inner: self.initialized().get_mapped_range_mut(),
            _marker: Default::default(),
        }
    }
}

pub struct BufferViewTyped<'a, T: Pod> {
    inner: BufferView<'a>,
    _marker: PhantomData<T>,
}
impl<'a, T: Pod> BufferViewTyped<'a, T> {
    pub fn get(&'a self, index: usize) -> Option<&'a T> {
        bytemuck::cast_slice(&self.inner).get(index)
    }
}
impl<'a, T: Pod> AsRef<[T]> for BufferViewTyped<'a, T> {
    fn as_ref(&self) -> &[T] {
        bytemuck::cast_slice(&self.inner)
    }
}
impl<'a, T: Pod> DataRef<'a, T> for BufferViewTyped<'a, T> {
    fn read(&'a self, index: usize) -> Option<&'a T> {
        self.get(index)
    }
}

pub struct BufferViewMutTyped<'a, T> {
    inner: BufferViewMut<'a>,
    _marker: PhantomData<T>,
}
impl<'a, T: Pod> BufferViewMutTyped<'a, T> {
    pub fn get_mut(&'a mut self, index: usize) -> Option<&'a mut T> {
        bytemuck::cast_slice_mut(&mut self.inner).get_mut(index)
    }
}
impl<'a, T: Pod> AsMut<[T]> for BufferViewMutTyped<'a, T> {
    fn as_mut(&mut self) -> &mut [T] {
        bytemuck::cast_slice_mut(&mut self.inner)
    }
}
impl<'a, T: Pod> DataMut<'a, T> for BufferViewMutTyped<'a, T> {
    fn write(&'a mut self, index: usize, value: T) {
        if let Some(i) = self.get_mut(index) {
            *i = value;
        }
    }
}
