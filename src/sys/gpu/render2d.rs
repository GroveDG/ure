use std::collections::{HashMap, VecDeque};

use cgmath::Vector2;
use wgpu::{
    Color, FragmentState, MultisampleState, PipelineCompilationOptions, RenderPipeline, RenderPipelineDescriptor, TextureFormat, VertexState
};

use crate::{render::SURFACE_FORMAT, sys::{tf::Matrix2D, UID}};

use super::{GPU, Pixels};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub pos: Vector2<Pixels>,
    pub size: Vector2<Pixels>,
}
impl Default for Rect {
    fn default() -> Self {
        Self {
            pos: Vector2 { x: 0, y: 0 },
            size: Vector2 { x: 0, y: 0 },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Geometry {
    Mesh(UID),
    Quad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fill {
    Texture(UID),
    Blank,
}

#[derive(Debug, Clone, Copy)]
pub struct Instance(Matrix2D, Geometry, Fill, Color);
impl Instance {
    pub fn diff(&self, other: &Self) -> Diff2D {
        Diff2D(
            diff(&self.0, &other.0),
            diff(&self.1, &other.1),
            diff(&self.2, &other.2),
            diff(&self.3, &other.3),
        )
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Diff2D(
    Option<Matrix2D>,
    Option<Geometry>,
    Option<Fill>,
    Option<Color>,
);
pub fn diff<T: PartialEq + Clone>(lhs: &T, rhs: &T) -> Option<T> {
    if lhs == rhs { None } else { Some(rhs.clone()) }
}

pub type Commands2D = Vec<(UID, Diff2D)>;

#[derive(Debug, Default)]
pub struct Draw2D {
    previous: HashMap<UID, Instance>,
    state: HashMap<UID, Instance>,
    diff: Commands2D,
}

impl Draw2D {
    pub fn draw(&mut self, uid: UID, instance: Instance) {
        let diff = if let Some(previous) = self.previous.remove_entry(&uid) {
            let diff = previous.1.diff(&instance);
            self.state.insert(previous.0, instance);
            diff
        } else {
            Diff2D(
                Some(instance.0),
                Some(instance.1),
                Some(instance.2),
                Some(instance.3),
            )
        };
        self.diff.push((uid, diff));
    }
    pub fn finish(&mut self) -> Commands2D {
        self.previous.clear();
        std::mem::swap(&mut self.previous, &mut self.state);
        std::mem::take(&mut self.diff)
    }
}

pub struct Render2D {
    pub pipeline: RenderPipeline,
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
                        buffers: &[],
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
        }
    }
}
