use std::pin::Pin;

use ordered_float::NotNan;
use wgpu::{Buffer, BufferSlice};

use crate::gpu::two::{Mesh2D, Mesh2DHandle};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct RenderPosition(NotNan<f32>);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct RenderLayer(usize);

pub struct RenderCommand<'a> {
    pub instance_slice: BufferSlice<'a>,
    pub mesh: Mesh2DHandle,
}

pub struct SortItem<'a> {
    pub position: RenderPosition,
    pub command: RenderCommand<'a>,
}

pub struct Rendering2D {
    pub render: fn(&Buffer, &[Mesh2DHandle], &[RenderLayer], &[RenderPosition], &mut Vec<Pin<Box<Buffer>>>, &mut [Vec<SortItem>]),
}

// pub const DIFFERENT: Rendering2D = Rendering2D {
//     render: |instances: &Buffer,
//              meshes: &[Mesh2DHandle],
//              layer: &[RenderLayer],
//              position: &[RenderPosition],
//              buffers: &mut Vec<Pin<Box<Buffer>>>,
//              commands: &mut [Vec<SortItem>]| {
//         for i in 0..position.len() {
//             let layer = layer[i].0;
//             commands[layer].push(SortItem {
//                 position: position[i],
//                 command: RenderCommand {
//                     instance_slice: buffer.slice(i as u64..i as u64 + 1),
//                     mesh: meshes[i].clone(),
//                 },
//             });
//         }
//     },
// };
