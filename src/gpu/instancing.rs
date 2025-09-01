use std::{fmt::Debug, marker::PhantomData, mem::MaybeUninit};

use wgpu::{Buffer, BufferDescriptor, BufferSlice, BufferUsages};

use crate::{
    gpu::{GPU, vertex::Instance},
    store::Element,
};

#[derive(Debug, Clone)]
pub struct InstanceBuffer<I: Instance> {
    length: usize,
    capacity: usize,
    buffer: Buffer,
    _marker: PhantomData<I>,
}
impl<I: Instance> InstanceBuffer<I> {
    pub fn new() -> Self {
        Self {
            length: 0,
            capacity: 0,
            buffer: GPU.device.create_buffer(&BufferDescriptor {
                label: Some("visuals 2d instance"),
                size: (0 * std::mem::size_of::<I>()) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            _marker: Default::default(),
        }
    }
    pub fn extend(&mut self, add: usize, init: impl FnOnce(&mut [MaybeUninit<I>])) {
        let length = self.length + add;
        if self.capacity < length {
            let capacity: usize = length.next_power_of_two();
            let buffer = GPU.device.create_buffer(&BufferDescriptor {
                label: Some("visuals 2d instance"),
                size: (capacity * std::mem::size_of::<I>()) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: true,
            });

            {
                let mut new = buffer.get_mapped_range_mut(..);
                let new_slice: &mut [I] = bytemuck::cast_slice_mut(&mut new);

                if self.length > 0 {
                    let mut current = self.buffer.get_mapped_range_mut(..);
                    let current_slice: &mut [I] = bytemuck::cast_slice_mut(&mut current);
                    new_slice[0..self.length].copy_from_slice(&current_slice[0..self.length]);
                }

                (init)(unsafe { std::mem::transmute(&mut new_slice[self.length..length]) });
            }

            self.buffer = buffer;
            self.capacity = capacity;
        } else {
            let mut view = self
                .buffer
                .get_mapped_range_mut((self.length as u64)..(length as u64));
            let slice: &mut [I] = bytemuck::cast_slice_mut(&mut view);

            (init)(unsafe { std::mem::transmute(slice) });
        }
        self.buffer.unmap();
        self.length = length;
    }
    pub fn len(&self) -> usize {
        self.length
    }
    pub fn slice<'a>(&'a self) -> BufferSlice<'a> {
        self.buffer.slice(0..(self.length * std::mem::size_of::<I>()) as u64)
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
