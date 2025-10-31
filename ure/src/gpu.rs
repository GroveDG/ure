use std::{marker::PhantomData, ops::Range, sync::OnceLock};

use bytemuck::Pod;
use wgpu::{
    Adapter, Buffer, BufferUsages, CommandBuffer, CommandEncoder, Device, DeviceDescriptor,
    Instance, InstanceDescriptor, Queue, RequestAdapterOptions, TextureFormat,
    wgt::{BufferDescriptor, CommandEncoderDescriptor},
};

pub static GPU: std::sync::LazyLock<Gpu> =
    std::sync::LazyLock::new(|| futures::executor::block_on(Gpu::new()));
pub static SURFACE_FORMAT: OnceLock<TextureFormat> = OnceLock::new();

pub use color::{AlphaColor, OpaqueColor};

pub type Srgba = AlphaColor<color::Srgb>;
pub type Srgb = OpaqueColor<color::Srgb>;

pub use color::Rgba8;

// pub struct Colors;
// impl Component for Colors {
//     const IDENT: &'static str = "Colors";

//     type Container = Vec<Rgba8>;
// }
// pub struct ColorNewDefaultWhite;
// impl Method for ColorNewDefaultWhite {
//     const IDENT: &'static str = "ColorNewDefaultWhite";

//     type Args = Range<usize>;
//     type Components = Colors;

//     fn call(
//         components: <<Self::Components as ure_data::group::ComponentRetrieve>::Containers as Container>::Mut<'_>,
//         args: Self::Args,
//     ) {
//         for i in args {
//             components
//         }
//     }
// }

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

// pub struct TypedBuffer<T: Pod> {
//     inner: Buffer,
//     len: usize,
//     capacity: usize,
//     _marker: PhantomData<T>,
// }
// impl<T: Pod> TypedBuffer<T> {
//     pub fn extend(&mut self, num: usize) {
//         let len = self.len + num;
//         if len > self.capacity {
//             self.capacity = len.next_power_of_two();
//             let usage = self.inner.usage();
//             let old_buffer = std::mem::replace(
//                 &mut self.inner,
//                 GPU.device.create_buffer(&BufferDescriptor {
//                     label: None,
//                     size: self.capacity as u64,
//                     usage,
//                     mapped_at_creation: false,
//                 }),
//             );
//             let mut cmds = GPU.device.create_command_encoder(&Default::default());
//             cmds.copy_buffer_to_buffer(&old_buffer, 0, &self.inner, 0, None);
//             GPU.queue.submit([cmds.finish()]);
//         }
//         self.len = len;
//     }
//     pub fn delete(&mut self, range: Range<usize>) {
//         let mut cmds = GPU.device.create_command_encoder(&Default::default());
//         let len = (self.len * size_of::<T>()) as u64;
//         let size = (range.len() * size_of::<T>()) as u64;
//         let src_offset = len - size;
//         let dest_offset = (range.start * size_of::<T>()) as u64;
//         {
//             let buffer = self.inner.clone();
//             self.inner.map_async(wgpu::MapMode::Write, .., move |e| {
//                 if e.is_err() {
//                     return;
//                 }
//                 let mut dest = buffer.get_mapped_range_mut(dest_offset..dest_offset + size);
//                 let src = buffer.get_mapped_range(src_offset..src_offset + size);

//                 dest.copy_from_slice(&src);

//                 drop(dest);
//                 drop(src);
//                 drop(buffer);
//             });
//         }
//         cmds.clear_buffer(&self.inner, src_offset, Some(size));
//         GPU.queue.submit([cmds.finish()]);
//     }
//     pub fn new(usage: BufferUsages) -> Self {
//         Self {
//             inner: GPU.device.create_buffer(&BufferDescriptor {
//                 label: None,
//                 size: 0,
//                 usage: BufferUsages::MAP_READ & BufferUsages::MAP_WRITE & usage,
//                 mapped_at_creation: false,
//             }),
//             len: 0,
//             capacity: 0,
//             _marker: PhantomData,
//         }
//     }
// }
// impl<T: Pod> Container for TypedBuffer<T> {
//     type Ref<'a> = &'a Self;
//     type Mut<'a> = &'a mut Self;

//     fn new() -> Self {
//         Self::new(BufferUsages::empty())
//     }

//     fn container_ref(&self) -> Self::Ref<'_> {
//         self
//     }
//     fn container_mut(&mut self) -> Self::Mut<'_> {
//         self
//     }
// }
