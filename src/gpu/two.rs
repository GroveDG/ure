use std::{
    any::Any,
    sync::{Arc, LazyLock},
};

use glam::Vec2;
use rustc_hash::FxHashSet;
use slotmap::Key;
use wgpu::{
    BindGroupDescriptor, BindGroupLayout, BufferUsages, FragmentState, MultisampleState,
    PipelineCompilationOptions, PipelineLayout, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderModule, VertexState,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    gpu::{
        Color, GPU, SURFACE_FORMAT,
        instancing::InstanceBuffer,
        rendering::{RenderLayer, Rendering},
    },
    store::{Data, Resource},
    vertex, vertex_buffers,
};

pub type Mesh2DHandle = Arc<Mesh2D>;

vertex! {
    Vertex Vertex2D
    0 position: Vec2 | Float32x2,
    1 uv: Vec2 | Float32x2,
    2 color: Color | Float32x4,
}
vertex! {
    Instance Instance2D
    0 col_0: Vec2 | Float32x2,
    1 col_1: Vec2 | Float32x2,
    2 position: Vec2 | Float32x2,
    3 color: Color | Float32x4,
}

#[derive(Debug, Clone)]
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

pub struct Visuals2D<K: Key> {
    keys: FxHashSet<K>,
    camera_buffer: wgpu::Buffer,
    camera: wgpu::BindGroup,
}
impl<K: Key> Visuals2D<K> {
    pub fn new() -> Self {
        let camera_buffer = GPU.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("camera"),
            contents: bytemuck::cast_slice(&glam::Affine2::IDENTITY.to_cols_array()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        Self {
            keys: Default::default(),
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
    pub fn add(&mut self, key: K) {
        self.keys.insert(key);
    }
    pub fn render(&self, data: &Data<K>, pass: &mut RenderPass) {
        let mut elements = Vec::with_capacity(self.keys.len());
        pass.set_bind_group(0, &self.camera, &[]);
        for key in self.keys.iter().copied() {
            let Some(element) = data.get(key) else {
                continue;
            };
            let Some(rendering) = element.get_func::<Rendering>() else {
                continue;
            };
            let layers = (rendering.layer)(element);
            for layer in layers {
                elements.push((layer, rendering, element));
            }
        }

        elements.sort_by_key(|e| e.0);

        for (layer, rendering, element) in elements {
            (rendering.render)(element, *layer, pass);
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

pub const DEFAULT_2D: &'static dyn Any = &Rendering {
    layer: |element| {
        let Some(layers) = element.get::<RenderLayer>() else {
            return &[RenderLayer(0)];
        };
        layers
    },
    render: |element, layer, pass| {
        let Some(instances) = element.get::<InstanceBuffer<Instance2D>>() else {
            return;
        };
        let Some(mesh) = element.get::<Mesh2DHandle>() else {
            return;
        };

        pass.set_pipeline(&PIPELINE);

        let instances = &instances[0];
        let mesh = &mesh[0];
        pass.set_vertex_buffer(0, mesh.vertex.slice(..));
        pass.set_index_buffer(mesh.index.slice(..), wgpu::IndexFormat::Uint16);
        pass.set_vertex_buffer(1, instances.slice());

        pass.draw_indexed(0..mesh.indices, 0, 0..instances.len() as u32);
    },
};

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

pub static PIPELINE_LAYOUT: LazyLock<PipelineLayout> = LazyLock::new(|| {
    GPU.device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&CAMERA_LAYOUT],
            push_constant_ranges: &[],
        })
});

pub static SHADER: LazyLock<ShaderModule> = LazyLock::new(|| {
    GPU.device
        .create_shader_module(wgpu::include_wgsl!("2d.wgsl"))
});

pub static PIPELINE: LazyLock<RenderPipeline> = LazyLock::new(|| {
    vertex_buffers! {
        buffers
        vertex: Vertex2D,
        instance: Instance2D
    };

    GPU.device
        .create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&PIPELINE_LAYOUT),
            vertex: VertexState {
                module: &SHADER,
                entry_point: Some("vertex"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &buffers,
            },
            fragment: Some(FragmentState {
                module: &SHADER,
                entry_point: Some("fragment"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: *SURFACE_FORMAT.get().unwrap(),
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
