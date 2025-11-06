use std::sync::{Arc, LazyLock};

use color::palette::css::WHITE;
use glam::{Affine2, Vec2};
use ure_data::{
	component,
	components::{CompMut, CompRef, ContMut},
	containers::OneOrMany,
	group::{Data, NewArgs},
	resource::Resource,
};
use wgpu::{
	BindGroupDescriptor, BindGroupLayout, BufferUsages, FragmentState, MultisampleState,
	PipelineCompilationOptions, RenderPipeline, RenderPipelineDescriptor, VertexAttribute,
	VertexBufferLayout, VertexState,
	util::{BufferInitDescriptor, DeviceExt},
};

use crate::gpu::{GPU, Rgba8, Srgba, TypedBuffer};

pub type MeshHandle2D = Arc<Mesh2D>;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct Vertex2D {
	pub position: Vec2,
	pub uv: Vec2,
	pub color: Rgba8,
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

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance2D {
	pub transform: [f32; 6],
	pub color: Rgba8,
}
impl Default for Instance2D {
	fn default() -> Self {
		Self {
			transform: Affine2::IDENTITY.to_cols_array(),
			color: WHITE.to_rgba8(),
		}
	}
}
impl Instance2D {
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
		array_stride: std::mem::size_of::<Self>() as u64,
		step_mode: wgpu::VertexStepMode::Instance,
		attributes: Self::ATTRIBUTES,
	};
}

component!(pub Colors: Vec<Srgba>, new_colors as fn(_, _), Vec<Srgba>);
pub fn new_colors(ContMut(mut colors): ContMut<Colors>, args: &NewArgs) {
	if let Some(mut new_colors) = args.take::<Colors>() {
		colors.append(&mut new_colors);
	} else {
		colors.append(&mut vec![WHITE; args.num()]);
	}
}
component!(pub Transforms2D: Vec<Affine2>);
component!(pub Instances2D: TypedBuffer<Instance2D>);
pub fn update_instances_2d(
	CompMut(mut instances): CompMut<Instances2D>,
	CompRef(transforms): CompRef<Transforms2D>,
	colors: Option<CompRef<Colors>>,
	args: &NewArgs,
) {
	instances.
}
component!(pub Meshes2D: OneOrMany<Arc<Mesh2D>>, new_meshes_2d as fn(_, _), Vec<Arc<Mesh2D>>);
pub fn new_meshes_2d(ContMut(mut meshes): ContMut<Meshes2D>, args: &NewArgs) {
	let OneOrMany::Many(vec) = &mut *meshes else {
		return;
	};
	let empty = EMPTY.load();
	for _ in 0..args.num() {
		vec.push(empty.clone());
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
		.create_shader_module(wgpu::include_wgsl!("two/2d.wgsl"));

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
					format: todo!(),
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

pub static EMPTY: Resource<Mesh2D> = Resource::new(|| Mesh2D::new(&[], &[]));
pub static QUAD: Resource<Mesh2D> = Resource::new(|| {
	Mesh2D::new(
		&[
			Vertex2D {
				// Top Left
				position: Vec2 { x: -0.5, y: 0.5 },
				uv: Vec2::ZERO,
				color: Srgba::WHITE.to_rgba8(),
			},
			Vertex2D {
				// Top Right
				position: Vec2 { x: -0.5, y: -0.5 },
				uv: Vec2::X,
				color: Srgba::WHITE.to_rgba8(),
			},
			Vertex2D {
				// Bottom Left
				position: Vec2 { x: 0.5, y: 0.5 },
				uv: Vec2::Y,
				color: Srgba::WHITE.to_rgba8(),
			},
			Vertex2D {
				// Bottom Right
				position: Vec2 { x: 0.5, y: -0.5 },
				uv: Vec2::ONE,
				color: Srgba::WHITE.to_rgba8(),
			},
		],
		&[0, 1, 2, 2, 1, 3],
	)
});

pub struct Visuals2D<Key: slotmap::Key> {
	keys: Vec<Key>,
	camera_buffer: wgpu::Buffer,
	camera: wgpu::BindGroup,
}
impl<Key: slotmap::Key> Visuals2D<Key> {
	pub fn new() -> Self {
		let camera_buffer = GPU.device.create_buffer_init(&BufferInitDescriptor {
			label: Some("camera"),
			contents: bytemuck::cast_slice(&glam::Affine2::IDENTITY.to_cols_array()),
			usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
		});
		let camera = GPU.device.create_bind_group(&BindGroupDescriptor {
			label: Some("camera"),
			layout: &CAMERA_LAYOUT,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: camera_buffer.as_entire_binding(),
			}],
		});
		Self {
			keys: Vec::new(),
			camera,
			camera_buffer,
		}
	}
	// pub fn render<'a>(&self, data: &mut Data<Key>, pass: &mut wgpu::RenderPass<'a>) {
	// 	for key in self.keys {
	// 		let Some(group) = data.get(key) else {
	// 			continue;
	// 		};
	// 		let mut group = group.get_mut();
	// 		let Some((mut instances, meshes)) =
	// 			group.get_components_mut::<(Instances2D, Meshes2D)>()
	// 		else {
	// 			continue;
	// 		};
	// 	}
	// 	GPU.queue.write_buffer(
	// 		&self.instance_buffer,
	// 		0,
	// 		bytemuck::cast_slice(unsafe {
	// 			std::mem::transmute::<&[std::mem::MaybeUninit<Instance2D>], &[Instance2D]>(
	// 				&data.visual_2d.elements,
	// 			)
	// 		}),
	// 	);
	// 	pass.set_pipeline(&PIPELINE);
	// 	pass.set_bind_group(0, &self.camera, &[]);
	// 	pass.set_vertex_buffer(1, self.instance_buffer.slice(..)); // Instance buffer
	// 	for span in self.spans.iter().copied() {
	// 		let mut buffer_position = data.visual_2d.positions[span] as u32;
	// 		let span = data.get_span(span);
	// 		for mesh in span.mesh.unwrap().iter() {
	// 			mesh.set(pass);
	// 			pass.draw_indexed(0..mesh.indices, 0, buffer_position..buffer_position + 1);
	// 			buffer_position += 1;
	// 		}
	// 	}
	// }
}
