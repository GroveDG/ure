use std::ops::Range;

use glam::Vec2;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, Buffer, BufferUsages, Device,
    FilterMode, FragmentState, MultisampleState, PipelineCompilationOptions, Queue, RenderPass,
    RenderPipeline, RenderPipelineDescriptor, VertexAttribute, VertexBufferLayout, VertexState,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    game::{SURFACE_FORMAT, tf::Matrix2D},
    render::Matrix2DGPU,
    sys::{Components, Uid, UIDs, delete::Delete},
};

use super::Color;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct Vertex2D {
    pub position: Vec2,
    pub color: Color,
    pub uv: Vec2,
}
impl Vertex2D {
    const ATTRIBUTES: &'static [VertexAttribute] = &wgpu::vertex_attr_array![
        // Position
        0 => Float32x2,
        // Color
        1 => Float32x4,
        // UV
        2 => Float32x2,
    ];
    const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: Self::ATTRIBUTES,
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh2D {
    pub vertex: Vec<Vertex2D>,
    pub index: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Instances2D {
    pub interp: FilterMode,
    pub instances: Vec<Instance2D>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance2D {
    pub tf: Matrix2D,
    pub color: Color,
}
impl Instance2D {
    const ATTRIBUTES: &'static [VertexAttribute] = &wgpu::vertex_attr_array![
        // Transform
        3 => Float32x3,
        4 => Float32x3,
        5 => Float32x3,
        // Color
        6 => Float32x4,
    ];
    const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as u64,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: Self::ATTRIBUTES,
    };
}

pub struct Draw2D {
    meshes: Components<(Buffer, Buffer)>,
    instances: Components<Buffer>,
    cameras: Components<(BindGroup, Buffer)>,
    pipeline: RenderPipeline,
    camera_layout: BindGroupLayout,
}
impl Draw2D {
    pub fn new(device: &Device) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("2d.wgsl"));

        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&camera_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
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
                    format: SURFACE_FORMAT,
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
        });
        Self {
            meshes: Default::default(),
            instances: Default::default(),
            cameras: Default::default(),
            pipeline,
            camera_layout,
        }
    }
    pub fn primitives(&mut self, uids: &mut UIDs, device: &Device, queue: &Queue) -> (Uid,) {
        let quad = uids.add();
        let mut update = self.update(device, queue);
        update.mesh(
            quad,
            Mesh2D {
                vertex: vec![
                    Vertex2D {
                        // Top Left
                        position: Vec2 { x: -0.5, y: 0.5 },
                        color: Color::WHITE,
                        uv: Vec2::ZERO,
                    },
                    Vertex2D {
                        // Top Right
                        position: Vec2 { x: -0.5, y: -0.5 },
                        color: Color::WHITE,
                        uv: Vec2::X,
                    },
                    Vertex2D {
                        // Bottom Left
                        position: Vec2 { x: 0.5, y: 0.5 },
                        color: Color::WHITE,
                        uv: Vec2::Y,
                    },
                    Vertex2D {
                        // Bottom Right
                        position: Vec2 { x: 0.5, y: -0.5 },
                        color: Color::WHITE,
                        uv: Vec2::ONE,
                    },
                ],
                index: vec![0, 1, 2, 2, 1, 3],
            },
        );
        (quad,)
    }
    pub fn update<'a>(&'a mut self, device: &'a Device, queue: &'a Queue) -> Draw2DUpdate<'a> {
        Draw2DUpdate::new(self, device, queue)
    }
    pub fn pass<'a, 'b>(&'a self, pass: &'a mut RenderPass<'b>) -> Draw2DPass<'a, 'b> {
        pass.set_pipeline(&self.pipeline);
        Draw2DPass::new(self, pass)
    }
}

pub struct Draw2DUpdate<'a> {
    draw_2d: &'a mut Draw2D,
    device: &'a Device,
    queue: &'a Queue,
}
impl<'a> Draw2DUpdate<'a> {
    pub fn new(draw_2d: &'a mut Draw2D, device: &'a Device, queue: &'a Queue) -> Self {
        Self {
            draw_2d,
            device,
            queue,
        }
    }
    pub fn camera(&mut self, camera: Uid, tf: Matrix2D) {
        // Add padding to Mat3x3 https://github.com/gfx-rs/wgpu-rs/issues/36
        let tfgpu = Matrix2DGPU::from(tf);
        let bytes = bytemuck::cast_slice(&tfgpu.inner);
        if let Some((_, buffer)) = self.draw_2d.cameras.get(&camera) {
            self.queue.write_buffer(buffer, 0, bytes);
        } else {
            let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytes,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });
            let bind = self.device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &self.draw_2d.camera_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });
            self.draw_2d.cameras.insert(camera, (bind, buffer));
        }
    }
    pub fn mesh(&mut self, uid: Uid, mesh: Mesh2D) {
        let vertex_bytes = bytemuck::cast_slice(&mesh.vertex);
        let index_bytes = bytemuck::cast_slice(&mesh.index);
        if let Some((vertex, index)) = self.draw_2d.meshes.get(&uid) {
            self.queue.write_buffer(vertex, 0, vertex_bytes);
            self.queue.write_buffer(index, 0, index_bytes);
        } else {
            let vertex = self.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: vertex_bytes,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });
            let index = self.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: vertex_bytes,
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            });
            self.draw_2d.meshes.insert(uid, (vertex, index));
        }
    }
    pub fn instances(&mut self, uid: Uid, instances: Vec<Instance2D>) {
        let bytes = bytemuck::cast_slice(&instances);
        if let Some(buffer) = self.draw_2d.instances.get(&uid) {
            self.queue.write_buffer(buffer, 0, bytes);
        } else {
            let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytes,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });
            self.draw_2d.instances.insert(uid, buffer);
        }
    }
}

pub struct Draw2DPass<'a, 'b> {
    draw_2d: &'a Draw2D,
    pass: &'a mut RenderPass<'b>,
    index_slice: Range<u32>,
    instances_slice: Range<u32>,
}
impl<'a, 'b> Draw2DPass<'a, 'b> {
    pub fn new(draw_2d: &'a Draw2D, pass: &'a mut RenderPass<'b>) -> Self {
        Self {
            draw_2d,
            pass,
            index_slice: 0..0,
            instances_slice: 0..0,
        }
    }
    pub fn camera(&mut self, camera: Uid) {
        self.pass
            .set_bind_group(0, &self.draw_2d.cameras.get(&camera).unwrap().0, &[]);
    }
    pub fn mesh(&mut self, uid: Uid) {
        let (vertex, index) = self.draw_2d.meshes.get(&uid).unwrap();

        self.pass
            .set_index_buffer(index.slice(..), wgpu::IndexFormat::Uint16);
        self.pass.set_vertex_buffer(0, vertex.slice(..));
        self.index_slice = 0..(index.size() / 2) as u32;
    }
    pub fn instances(&mut self, uid: Uid) {
        let instances = self.draw_2d.instances.get(&uid).unwrap();

        self.pass.set_vertex_buffer(1, instances.slice(..));
        self.instances_slice = 0..(instances.size() / (size_of::<Instance2D>() as u64)) as u32;
    }
    pub fn draw(&mut self) {
        self.pass
            .draw_indexed(self.index_slice.clone(), 0, self.instances_slice.clone());
    }
}

impl Delete for Draw2D {
    fn delete(&mut self, uid: &Uid) {
        self.cameras.remove(uid);
        self.meshes.remove(uid);
        self.instances.remove(uid);
    }
}
