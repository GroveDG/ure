use wgpu::{VertexAttribute, VertexStepMode};

pub trait VertexBuffer {
    const ATTRIBUTES: &'static [VertexAttribute];
    const STEP: VertexStepMode;
}

pub trait Vertex {}
pub trait Instance: bytemuck::Pod {}

pub fn relocate_attributes(
    attrs: &'static [VertexAttribute],
    location: u32,
) -> Box<[VertexAttribute]> {
    let mut boxed = attrs.to_owned().into_boxed_slice();
    for i in 0..attrs.len() {
        boxed[i].shader_location += location;
    }
    boxed
}

#[macro_export]
macro_rules! vertex {
    ($step:ident $name:ident $($location:literal $i:ident : $t:ty | $format:ident),+ $(,)?) => {
        #[repr(C)]
        #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
        pub struct $name {
            $(
            pub $i: $t
            ),+
        }
        impl $crate::gpu::vertex::VertexBuffer for $name {
            const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &[$(wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::$format,
                    offset: std::mem::offset_of!($name, $i) as u64,
                    shader_location: $location as u32,
                }),+];
            const STEP: wgpu::VertexStepMode = wgpu::VertexStepMode::$step;
        }
        impl $crate::gpu::vertex::$step for $name {}
    };
}

#[macro_export]
macro_rules! vertex_buffers {
    ($name:ident $($i:ident : $t:ty),+ $(,)?) => {
        use $crate::gpu::vertex::VertexBuffer;
        let mut location: u32 = 0;
        $(
        let $i = $crate::gpu::vertex::relocate_attributes(<$t>::ATTRIBUTES, location);
        #[allow(unused_assignments)]
        { location += <$t>::ATTRIBUTES.len() as u32; }
        )+
        let $name = [$(
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<$t>() as u64,
                step_mode: <$t>::STEP,
                attributes: &$i,
            }
        ),+];
    };
}
