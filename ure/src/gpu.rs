use std::{marker::PhantomData, sync::OnceLock};

use bitvec::vec::BitVec;
use bytemuck::Pod;
use ure_data::containers::{Container, NewDefault, NewWith};
use wgpu::{
	Adapter, Buffer, BufferUsages, CommandEncoder, Device, DeviceDescriptor, Instance,
	InstanceDescriptor, Queue, RequestAdapterOptions, TextureFormat, wgt::BufferDescriptor,
};

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
}
impl<T: Pod> Container for TypedBuffer<T> {
	type Slice = [T];
	type Item = T;

	fn as_ref(&self) -> &Self::Slice {
		self.inner.get_mapped_range(..)
	}

	fn as_mut(&mut self) -> &mut Self::Slice {
		unsafe { std::mem::transmute(self) }
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