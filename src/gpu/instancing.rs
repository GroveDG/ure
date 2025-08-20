use std::{fmt::Debug, marker::PhantomData, num::NonZero};

use glam::Affine2;
use wgpu::{Buffer, BufferDescriptor, BufferUsages};

use crate::{
    data,
    gpu::{Color, GPU, two::Instance2D, vertex::Instance},
    store::Element,
};

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct InstanceBuffer<I: Instance> {
    pub inner: Buffer,
    _marker: PhantomData<I>,
}
impl<I: Instance> InstanceBuffer<I> {
    pub fn new(buffer: Buffer) -> Self {
        Self {
            inner: buffer,
            _marker: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Instancing<I: Instance> {
    pub instance: fn(&Element),
    _marker: PhantomData<I>,
}

impl<I: Instance> Instancing<I> {
    pub fn new(f: fn(&Element)) -> Self {
        Self {
            instance: f,
            _marker: Default::default(),
        }
    }
    pub fn new_buffer(len: usize) -> InstanceBuffer<I> {
        InstanceBuffer::new(GPU.device.create_buffer(&BufferDescriptor {
            label: Some("visuals 2d instance"),
            size: (len * std::mem::size_of::<I>()) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }))
    }
}

// pub fn different<I: Instance>(element: &Element) {
//     data! {
//         element
//         {
//             buffer: InstanceBuffer<I>,
//             transform: Affine2,
//             color: Color,
//         } else {
//             return;
//         }
//     }
//     let buffer = &mut buffer[0];

//     // Resize Buffer
//     let buffer_len = (buffer.inner.size() / std::mem::size_of::<I>() as u64) as usize;
//     let element_len = transform.len();
//     if buffer_len < element_len {
//         *buffer = Instancing::<I>::new_buffer(element_len);
//     }

//     // Fill Buffer
//     let Some(buffer_size) = NonZero::new(buffer.inner.size()) else {
//         return;
//     };
//     let Some(mut view) = GPU.queue.write_buffer_with(buffer, 0, buffer_size) else {
//         return;
//     };
//     let view = bytemuck::cast_slice_mut::<_, Instance2D>(&mut view);
//     (instancing.fill)(&transform, &color, view);
// }
