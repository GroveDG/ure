use std::cell::{Ref, RefMut};
use std::ops::{Deref, DerefMut};
use std::{marker::PhantomData, sync::OnceLock};

use bitvec::slice::BitSlice;
use bitvec::vec::BitVec;
use bytemuck::Pod;
use ure_data::containers::{Container, NewDefault, NewWith};
use wgpu::{
	Adapter, Buffer, BufferUsages, CommandEncoder, Device, DeviceDescriptor, Instance,
	InstanceDescriptor, Queue, RequestAdapterOptions, TextureFormat, wgt::BufferDescriptor,
};
use wgpu::{BufferView, BufferViewMut};

pub static GPU: std::sync::LazyLock<Gpu> =
	std::sync::LazyLock::new(|| futures::executor::block_on(Gpu::new()));
pub static SURFACE_FORMAT: OnceLock<TextureFormat> = OnceLock::new();

pub use color::{AlphaColor, OpaqueColor};

pub type Srgba = AlphaColor<color::Srgb>;
pub type Srgb = OpaqueColor<color::Srgb>;

pub use color::Rgba8;

pub struct Gpu {
	pub instance: Instance,
	pub adapter: Adapter,
	pub device: Device,
	pub queue: Queue,
}
impl Gpu {
	pub async fn new() -> Self {
		let instance = Instance::new(&InstanceDescriptor::from_env_or_default());
		let adapter = instance
			.request_adapter(&RequestAdapterOptions::default())
			.await
			.unwrap();
		let (device, queue) = adapter
			.request_device(&DeviceDescriptor::default())
			.await
			.unwrap();
		Self {
			instance,
			adapter,
			device,
			queue,
		}
	}
}

pub struct TypedBuffer<T: Pod> {
	inner: Buffer,
	len: usize,
	up_len: usize,
	capacity: usize,
	diff: BitVec,
	_marker: PhantomData<T>,
}
impl<T: Pod> Default for TypedBuffer<T> {
	fn default() -> Self {
		Self::new(BufferUsages::empty())
	}
}
impl<T: Pod> TypedBuffer<T> {
	pub fn new(usage: BufferUsages) -> Self {
		Self {
			inner: GPU.device.create_buffer(&BufferDescriptor {
				label: None,
				size: 0,
				usage: BufferUsages::MAP_READ & BufferUsages::MAP_WRITE & usage,
				mapped_at_creation: false,
			}),
			len: 0,
			up_len: 0,
			capacity: 0,
			diff: BitVec::new(),
			_marker: PhantomData,
		}
	}
	pub fn update_size(&mut self, encoder: &mut CommandEncoder) {
		let up_capacity = self.up_len.next_power_of_two();
		if self.capacity >= up_capacity {
			return;
		}

		let new_buffer = GPU.device.create_buffer(&BufferDescriptor {
			label: None,
			size: (up_capacity * size_of::<T>()) as u64,
			usage: self.inner.usage(),
			mapped_at_creation: false,
		});
		encoder.copy_buffer_to_buffer(&self.inner, 0, &new_buffer, 0, self.inner.size());

		self.inner = new_buffer;
	}
	pub fn buffer(&self) -> &Buffer {
		&self.inner
	}
	pub fn len(&self) -> usize {
		self.len
	}
}
impl<T: Pod> Container for TypedBuffer<T> {
	type Ref<'a> = (TypedBufferView<T>, Ref<'a, BitSlice>);
	type RefMut<'a> = (TypedBufferViewMut<T>, RefMut<'a, BitSlice>);

	fn as_ref<'a>(cont: std::cell::Ref<'a, Self>) -> Self::Ref<'a> {
		(
			TypedBufferView {
				buffer: cont.inner.clone(),
				inner: cont.inner.get_mapped_range(..),
				_marker: PhantomData::<T>,
			},
			Ref::map(cont, |c| c.diff.as_bitslice()),
		)
	}
	fn as_mut<'a>(cont: std::cell::RefMut<'a, Self>) -> Self::RefMut<'a> {
		(
			TypedBufferViewMut {
				buffer: cont.inner.clone(),
				inner: cont.inner.get_mapped_range_mut(..),
				_marker: PhantomData::<T>,
			},
			RefMut::map(cont, |c| c.diff.as_mut_bitslice()),
		)
	}
	fn delete(&mut self, indices: &[usize]) {
		self.diff.reserve(indices.len());
		for &index in indices {
			self.diff.set(index, true);
		}
		self.up_len -= indices.len();
		self.diff.truncate(self.up_len);
	}
}
impl<T: Pod + Default> NewDefault for TypedBuffer<T> {
	fn new_default(&mut self, num: usize) {
		self.up_len += num;
		self.diff.resize(self.up_len, true);
	}
}
impl<T: Pod + Default> NewWith for TypedBuffer<T> {
	type Args = ();

	fn new_with(&mut self, _: Self::Args) {
		panic!("Do not call 'new_with' on TypedBuffers.")
	}
}

pub struct TypedBufferView<T: Pod> {
	inner: BufferView,
	buffer: Buffer,
	_marker: PhantomData<T>,
}
impl<T: Pod> Deref for TypedBufferView<T> {
	type Target = [T];

	fn deref(&self) -> &Self::Target {
		bytemuck::cast_slice(&self.inner)
	}
}
impl<T: Pod> Drop for TypedBufferView<T> {
	fn drop(&mut self) {
		self.buffer.unmap();
	}
}

pub struct TypedBufferViewMut<T: Pod> {
	inner: BufferViewMut,
	buffer: Buffer,
	_marker: PhantomData<T>,
}
impl<T: Pod> Deref for TypedBufferViewMut<T> {
	type Target = [T];

	fn deref(&self) -> &Self::Target {
		bytemuck::cast_slice(&self.inner)
	}
}
impl<T: Pod> DerefMut for TypedBufferViewMut<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		bytemuck::cast_slice_mut(&mut self.inner)
	}
}
impl<T: Pod> Drop for TypedBufferViewMut<T> {
	fn drop(&mut self) {
		self.buffer.unmap();
	}
}
