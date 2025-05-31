use glam::Vec2;
use wgpu::{FilterMode, FragmentState, MultisampleState, PipelineCompilationOptions, RenderPipeline, RenderPipelineDescriptor, VertexAttribute, VertexBufferLayout, VertexState};

use crate::game::tf::Matrix2D;

use super::{gpu::{Color, GPU}, SURFACE_FORMAT};

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
    const DESCRIPTOR: VertexBufferLayout<'static> = VertexBufferLayout {
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
    const DESCRIPTOR: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as u64,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: Self::ATTRIBUTES,
    };
}

pub struct Render2D {
    
}
impl Render2D {
    pub fn pipeline(gpu: &GPU) -> RenderPipeline {
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

        gpu.device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: Some("vertex"),
                    compilation_options: PipelineCompilationOptions::default(),
                    buffers: &[Vertex2D::DESCRIPTOR, Instance2D::DESCRIPTOR],
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
            })
    }
    // pub fn update(&mut self, commands: Updates2D, gpu: &GPU) {
    //     for (uid, mesh) in commands.meshes {
    //         let Some(mesh) = mesh else {
    //             self.meshes.remove(&uid);
    //             continue;
    //         };
    //         if let Some(mesh_gpu) = self.meshes.get(&uid) {
    //             update_buffer(mesh_gpu.vertex.clone(), mesh.vertex, &gpu.device);
    //             update_buffer(mesh_gpu.index.clone(), mesh.index, &gpu.device);
    //         } else {
    //             let mesh = {
    //                 Mesh2DGPU {
    //                     vertex: gpu.device.create_buffer_init(&BufferInitDescriptor {
    //                         label: None,
    //                         contents: bytemuck::cast_slice(&mesh.vertex),
    //                         usage: BufferUsages::VERTEX,
    //                     }),
    //                     index: gpu.device.create_buffer_init(&BufferInitDescriptor {
    //                         label: None,
    //                         contents: bytemuck::cast_slice(&mesh.index),
    //                         usage: BufferUsages::INDEX,
    //                     }),
    //                 }
    //             };
    //             self.meshes.insert(uid, mesh);
    //         }
    //     }
    //     for (uid, instances) in commands.instances {
    //         let Some(instances) = instances else {
    //             self.instances.remove(&uid);
    //             continue;
    //         };
    //         if let Some(instances_gpu) = self.instances.get(&uid) {
    //             update_buffer(instances_gpu.buffer.clone(), instances, &gpu.device);
    //         } else {
    //             let instances = {
    //                 Instances2DGPU {
    //                     buffer: gpu.device.create_buffer_init(&BufferInitDescriptor {
    //                         label: None,
    //                         contents: bytemuck::cast_slice(&instances),
    //                         usage: BufferUsages::VERTEX,
    //                     }),
    //                 }
    //             };
    //             self.instances.insert(uid, instances);
    //         }
    //     }
    // }
    // pub fn render(&mut self, render_pass: &mut RenderPass, commands: Commands2D) {
    //     render_pass.set_pipeline(&self.pipeline);
    //     for (mesh, instance) in commands.iter() {
    //         let Some(mesh) = self.meshes.get(mesh) else {
    //             continue;
    //         };
    //         let Some(instances) = self.instances.get(instance) else {
    //             continue;
    //         };
    //         render_pass.set_vertex_buffer(0, mesh.vertex.slice(..));
    //         render_pass.set_vertex_buffer(1, instances.buffer.slice(..));
    //         render_pass.set_index_buffer(mesh.index.slice(..), wgpu::IndexFormat::Uint16);
    //         render_pass.draw_indexed(
    //             0..mesh.index.size() as u32 / 2,
    //             0,
    //             0..instances.len() as u32,
    //         );
    //     }
    // }
}
