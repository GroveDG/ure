use std::{
    num::NonZero,
    sync::{Arc, LazyLock},
};

use glam::{Affine2, Vec2};
use rustc_hash::FxHashSet;
use wgpu::{
    BindGroupDescriptor, BindGroupLayout, Buffer, BufferUsages, FragmentState, MultisampleState,
    PipelineCompilationOptions, RenderPipeline, RenderPipelineDescriptor, VertexAttribute,
    VertexBufferLayout, VertexState,
    util::{BufferInitDescriptor, DeviceExt},
    wgt::BufferDescriptor,
};

use crate::{
    data::{Compartment, Data, Element, Resource},
    game::GameComponent,
    gpu::{Color, GPU},
};

pub type MeshHandle2D = Arc<Mesh2D>;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct Vertex2D {
    pub position: Vec2,
    pub uv: Vec2,
    pub color: Color,
}
impl Vertex2D {
    const ATTRIBUTES: &'static [VertexAttribute] = &wgpu::vertex_attr_array![
        // Position
        0 => Float32x2,
        // UV
        1 => Float32x2,
        // Color
        2 => Float32x4,
    ];
    const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: Self::ATTRIBUTES,
    };
}
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance2D {
    pub transform: [f32; 6],
    pub color: Color,
}
impl Instance2D {
    const SIZE: u64 = std::mem::size_of::<Self>() as u64;
    const ATTRIBUTES: &'static [VertexAttribute] = &wgpu::vertex_attr_array![
        // Transform (Mat2)
        3 => Float32x2,
        4 => Float32x2,
        // Translation
        5 => Float32x2,
        // Color
        6 => Float32x4,
    ];
    const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: Self::SIZE,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: Self::ATTRIBUTES,
    };
}

#[derive(Debug)]
pub struct Mesh2D {
    pub vertex: wgpu::Buffer,
    pub index: wgpu::Buffer,
    pub indices: u32,
}

impl Mesh2D {
    pub fn new(vertex: &[Vertex2D], index: &[u16]) -> Self {
        Mesh2D {
            vertex: GPU.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(vertex),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            }),
            index: GPU.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(index),
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            }),
            indices: index.len() as u32,
        }
    }
    pub fn set<'a>(&self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_index_buffer(self.index.slice(..), wgpu::IndexFormat::Uint16);
        pass.set_vertex_buffer(0, self.vertex.slice(..));
    }
}

pub struct Visuals2D {
    keys: FxHashSet<Key>,
    instance_buffers: Vec<wgpu::Buffer>,
    camera_buffer: wgpu::Buffer,
    camera: wgpu::BindGroup,
}
impl Visuals2D {
    pub fn new() -> Self {
        let camera_buffer = GPU.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("camera"),
            contents: bytemuck::cast_slice(&glam::Affine2::IDENTITY.to_cols_array()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        Self {
            keys: Default::default(),
            instance_buffers: Default::default(),
            camera: GPU.device.create_bind_group(&BindGroupDescriptor {
                label: Some("camera"),
                layout: &CAMERA_LAYOUT,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
            }),
            camera_buffer,
        }
    }
    pub fn add(&mut self, element: &mut Element, key: Key) {
        self.keys.insert(key);
        *element.get_mut(GameComponent::InstanceBuffer) =
            Compartment::One(Box::new(GPU.device.create_buffer(&BufferDescriptor {
                label: Some("visuals 2d instance"),
                size: element.len() as u64 * Instance2D::SIZE,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })));
    }
    pub fn render<'a>(&self, data: &mut Data, pass: &mut wgpu::RenderPass<'a>) {
        for key in self.keys.iter().copied() {
            let Some(element) = data.get_mut(key) else {
                continue;
            };
            let Ok(compartments) = element.get_mut_disjoint([
                GameComponent::InstanceBuffer as usize,
                GameComponent::Transform as usize,
                GameComponent::Color as usize,
            ]) else {
                continue;
            };
            let Some(buffer) = compartments[0].cast_one_mut::<Buffer>() else {
                continue;
            };
            let Some(transform) = compartments[1].as_slice::<Affine2>() else {
                continue;
            };
            let Some(color) = compartments[2].as_slice::<Color>() else {
                continue;
            };

            let buffer_len = (buffer.size() / Instance2D::SIZE) as usize;
            let element_len = element.len();
            if buffer_len < element_len {
                self.add(element, key);
            }
            let Some(buffer_size) = NonZero::new(buffer.size()) else {
                continue;
            };
            let Some(view) = GPU.queue.write_buffer_with(buffer, 0, buffer_size) else {
                continue;
            };
            let view = bytemuck::cast_slice_mut::<_, Instance2D>(&mut view);
            for i in 0..element_len {
                view[i] = Instance2D {
                    transform: transform[i].to_cols_array(),
                    color: color[i],
                };
            }
            pass.set_pipeline(&PIPELINE);
            pass.set_bind_group(0, &self.camera, &[]);
            pass.set_vertex_buffer(1, self.instance_buffer.slice(..)); // Instance buffer
            for span in self.spans.iter().copied() {
                let mut buffer_position = data.visual_2d.positions[span] as u32;
                let span = data.get_span(span);
                for mesh in span.mesh.unwrap().iter() {
                    mesh.set(pass);
                    pass.draw_indexed(0..mesh.indices, 0, buffer_position..buffer_position + 1);
                    buffer_position += 1;
                }
            }
        }
    }
    pub fn set_camera(&mut self, transform: &glam::Affine2) {
        GPU.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&transform.to_cols_array()),
        );
    }
}

pub static CAMERA_LAYOUT: LazyLock<BindGroupLayout> = LazyLock::new(|| {
    GPU.device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: None,
        })
});

pub static PIPELINE: LazyLock<RenderPipeline> = LazyLock::new(|| {
    let shader = GPU
        .device
        .create_shader_module(wgpu::include_wgsl!("2d.wgsl"));

    let layout = GPU
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&CAMERA_LAYOUT],
            push_constant_ranges: &[],
        });

    GPU.device
        .create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vertex"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[Vertex2D::LAYOUT, Instance2D::LAYOUT],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fragment"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: super::SURFACE_FORMAT,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
});

pub static QUAD: Resource<Mesh2D> = Resource::new(|| {
    Mesh2D::new(
        &[
            Vertex2D {
                // Top Left
                position: Vec2 { x: -0.5, y: 0.5 },
                uv: Vec2::ZERO,
                color: Color::WHITE,
            },
            Vertex2D {
                // Top Right
                position: Vec2 { x: -0.5, y: -0.5 },
                uv: Vec2::X,
                color: Color::WHITE,
            },
            Vertex2D {
                // Bottom Left
                position: Vec2 { x: 0.5, y: 0.5 },
                uv: Vec2::Y,
                color: Color::WHITE,
            },
            Vertex2D {
                // Bottom Right
                position: Vec2 { x: 0.5, y: -0.5 },
                uv: Vec2::ONE,
                color: Color::WHITE,
            },
        ],
        &[0, 1, 2, 2, 1, 3],
    )
});
