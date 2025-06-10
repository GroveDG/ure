use std::sync::mpsc::Sender;

use glam::Vec2;
use wgpu::{
    BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, Buffer, BufferUsages, FilterMode,
    FragmentState, MultisampleState, PipelineCompilationOptions, RenderPipeline,
    RenderPipelineDescriptor, VertexAttribute, VertexBufferLayout, VertexState, util::DeviceExt,
};

use crate::{
    game::tf::Matrix2D,
    render::{gpu::Matrix2DGPU, BindResource},
    sys::{delete::Delete, Components, UIDs, UID},
};

use super::{
    RenderCommand, SURFACE_FORMAT,
    gpu::{Color, GPU},
};

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
    render_sndr: Sender<RenderCommand>,
    meshes: Components<(UID, UID)>,
    cameras: Components<Matrix2D>,
}
impl Draw2D {
    pub fn pipeline(gpu: &GPU) -> (RenderPipeline, BindGroupLayout) {
        let shader = gpu
            .device
            .create_shader_module(wgpu::include_wgsl!("2d.wgsl"));

        let camera_layout = gpu
            .device
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
            });

        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&camera_layout],
                push_constant_ranges: &[],
            });

        let pipeline = gpu
            .device
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
        (pipeline, camera_layout)
    }
    pub fn new(render_sndr: Sender<RenderCommand>) -> Self {
        Self {
            render_sndr,
            meshes: Default::default(),
            cameras: Default::default(),
        }
    }
    pub fn primitives(&mut self, uids: &mut UIDs) -> (UID,) {
        let quad = uids.add();
        self.update_mesh(
            quad,
            uids,
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
    pub fn update_camera(&mut self, camera: UID, tf: Matrix2D) {
        // Add padding to Mat3x3 https://github.com/gfx-rs/wgpu-rs/issues/36
        let _ = self.render_sndr.send(RenderCommand::Buffer(
            camera,
            bytemuck::cast_slice(&Matrix2DGPU::from(tf).inner).to_vec(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        ));
        let _ = self.render_sndr.send(RenderCommand::Bind(
            camera,
            super::BindLayout::_2DCam,
            vec![(0, BindResource::Buffer(camera))],
        ));
    }
    pub fn update_mesh(&mut self, uid: UID, uids: &mut UIDs, mesh: Mesh2D) {
        if !self.meshes.contains_key(&uid) {
            let vertex = uids.add();
            let index = uids.add();

            self.meshes.insert(uid, (vertex, index));
        }

        let (vertex, index) = self.meshes.get(&uid).copied().unwrap();

        let _ = self.render_sndr.send(RenderCommand::Buffer(
            vertex,
            bytemuck::cast_slice(&mesh.vertex).to_vec(),
            BufferUsages::VERTEX | BufferUsages::COPY_DST,
        ));
        let _ = self.render_sndr.send(RenderCommand::Buffer(
            index,
            bytemuck::cast_slice(&mesh.index).to_vec(),
            BufferUsages::INDEX | BufferUsages::COPY_DST,
        ));
    }
    pub fn update_instances(&self, uid: UID, instances: Vec<Instance2D>) {
        let _ = self.render_sndr.send(RenderCommand::Buffer(
            uid,
            bytemuck::cast_slice(&instances).to_vec(),
            BufferUsages::VERTEX | BufferUsages::COPY_DST,
        ));
    }
    pub fn start(&self) {
        let _ = self
            .render_sndr
            .send(RenderCommand::Pipeline(super::Pipelines::_2D));
    }
    pub fn camera(&self, camera: UID) {
        let _ = self.render_sndr.send(RenderCommand::Bound(0, camera));
    }
    pub fn mesh(&self, uid: UID) {
        let (vertex, index) = self.meshes.get(&uid).copied().unwrap();
        let _ = self
            .render_sndr
            .send(RenderCommand::Vertex(0, vertex, None));
        let _ = self.render_sndr.send(RenderCommand::Index(index));
    }
    pub fn instances(&self, uid: UID) {
        let _ = self.render_sndr.send(RenderCommand::Vertex(
            1,
            uid,
            Some(size_of::<Instance2D>() as u64),
        ));
    }
    pub fn draw(&self) {
        let _ = self.render_sndr.send(RenderCommand::Draw);
    }
}

impl Delete for Draw2D {
    fn delete(&mut self, uid: &UID) {
        let Some((vertex, index)) = self.meshes.get(uid).copied() else {
            return;
        };
        let _ = self.render_sndr.send(RenderCommand::Delete(vertex));
        let _ = self.render_sndr.send(RenderCommand::Delete(index));
    }
}
