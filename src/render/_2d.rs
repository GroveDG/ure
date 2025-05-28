use wgpu::{
    Buffer, BufferUsages, FragmentState, MultisampleState, PipelineCompilationOptions,
    RenderPass, RenderPipeline, RenderPipelineDescriptor, VertexAttribute, VertexBufferLayout,
    VertexState,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::game::tf::Matrix2D;
use crate::sys::{Components, Entities, UID};

use super::{
    SURFACE_FORMAT,
    gpu::{GPU, update_buffer},
};

#[derive(Debug, Default)]
pub struct Draw2D {
    instances: Entities,
    meshes: Entities,
    updates: Updates2D,
    commands: Components<Commands2D>,
}
impl Draw2D {
    pub fn add_instance(&mut self, uid: UID, instances: Vec<Instance2D>) {
        self.instances.insert(uid);
        self.updates.instances.insert(uid, Some(instances));
    }
    pub fn add_mesh(&mut self, uid: UID, mesh: Mesh2D) {
        self.meshes.insert(uid);
        self.updates.meshes.insert(uid, Some(mesh));
    }
    pub fn remove_instance(&mut self, uid: UID) {
        self.instances.remove(&uid);
        self.updates.instances.insert(uid, None);
    }
    pub fn remove_mesh(&mut self, uid: UID) {
        self.meshes.remove(&uid);
        self.updates.meshes.insert(uid, None);
    }
    pub fn draw(&mut self, window: &UID, instance: UID, mesh: UID) {
        self.commands.get_mut(window).unwrap().push((instance, mesh));
    }
    pub fn finish(&mut self) -> (Updates2D, Components<Commands2D>) {
        (
            std::mem::take(&mut self.updates),
            std::mem::take(&mut self.commands),
        )
    }
}

#[derive(Debug, Default)]
pub struct Updates2D {
    instances: Components<Option<Vec<Instance2D>>>,
    meshes: Components<Option<Mesh2D>>,
}

pub type Commands2D = Vec<(
    UID, // Mesh
    UID, // Instance
)>;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct Vertex2D {
    pub position: [f32; 2],
    pub color: [f32; 3],
    pub uv: [f32; 2],
}
impl Vertex2D {
    const ATTRIBUTES: &'static [VertexAttribute] = &wgpu::vertex_attr_array![
        // Position
        0 => Float32x2,
        // Color
        1 => Float32x3,
        // UV
        2 => Float32x2,
    ];
    const DESCRIPTOR: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: Self::ATTRIBUTES,
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh2D {
    pub vertex: Vec<Vertex2D>,
    pub index: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Instance2D {
    tf: Matrix2D,
    color: [f32; 3],
}
impl Instance2D {
    const ATTRIBUTES: &'static [VertexAttribute] = &wgpu::vertex_attr_array![
        // Transform
        3 => Float32x3,
        4 => Float32x3,
        5 => Float32x3,
        // Color
        6 => Float32x3,
    ];
    const DESCRIPTOR: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as u64,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: Self::ATTRIBUTES,
    };
}

pub struct Mesh2DGPU {
    pub vertex: Buffer,
    pub index: Buffer,
}
impl Drop for Mesh2DGPU {
    fn drop(&mut self) {
        self.vertex.destroy();
        self.index.destroy();
    }
}
pub struct Instances2DGPU {
    pub buffer: Buffer,
}
impl Drop for Instances2DGPU {
    fn drop(&mut self) {
        self.buffer.destroy();
    }
}
impl Instances2DGPU {
    pub fn len(&self) -> u64 {
        self.buffer.size() / size_of::<Self>() as u64
    }
}

pub struct Render2D {
    pub pipeline: RenderPipeline,
    pub meshes: Components<Mesh2DGPU>,
    pub instances: Components<Instances2DGPU>,
}
impl Render2D {
    pub fn new(gpu: &GPU) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::include_wgsl!("2d.wgsl"));

        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        Self {
            pipeline: gpu
                .device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&layout),
                    vertex: VertexState {
                        module: &shader,
                        entry_point: Some("vertex"),
                        compilation_options: PipelineCompilationOptions::default(),
                        buffers: &[Vertex2D::DESCRIPTOR],
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
                }),
            meshes: Default::default(),
            instances: Default::default(),
        }
    }
    pub fn update(&mut self, commands: Updates2D, gpu: &GPU) {
        for (uid, mesh) in commands.meshes {
            let Some(mesh) = mesh else {
                self.meshes.remove(&uid);
                continue;
            };
            if let Some(mesh_gpu) = self.meshes.get(&uid) {
                update_buffer(mesh_gpu.vertex.clone(), mesh.vertex, &gpu.device);
                update_buffer(mesh_gpu.index.clone(), mesh.index, &gpu.device);
            } else {
                let mesh = {
                    Mesh2DGPU {
                        vertex: gpu.device.create_buffer_init(&BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(&mesh.vertex),
                            usage: BufferUsages::VERTEX,
                        }),
                        index: gpu.device.create_buffer_init(&BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(&mesh.index),
                            usage: BufferUsages::INDEX,
                        }),
                    }
                };
                self.meshes.insert(uid, mesh);
            }
        }
    }
    pub fn render(&mut self, render_pass: &mut RenderPass, commands: Commands2D) {
        render_pass.set_pipeline(&self.pipeline);
        for (mesh, instance) in commands.iter() {
            let Some(mesh) = self.meshes.get(mesh) else {
                continue;
            };
            let Some(instances) = self.instances.get(instance) else {
                continue;
            };
            render_pass.set_vertex_buffer(0, mesh.vertex.slice(..));
            render_pass.set_vertex_buffer(1, instances.buffer.slice(..));
            render_pass.set_index_buffer(mesh.index.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..mesh.index.size() as u32, 0, 0..instances.len() as u32);
        }
    }
}
