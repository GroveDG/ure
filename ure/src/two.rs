use std::sync::{Arc, LazyLock};

use color::palette::css::WHITE;
use glam::{Affine2, Vec2};
use itertools::izip;
use ure_data::{
	component,
	components::NewArgs,
	containers::OneOrMany,
	glob::{CompMut, CompRef, ContMut, ContRef, Len},
	group::Data,
	method::MethodTrait,
	resource::Resource,
};
use wgpu::{
	BindGroupDescriptor, BindGroupLayout, BufferUsages, CommandEncoder, FragmentState,
	MultisampleState, PipelineCompilationOptions, PipelineLayout, RenderPass, RenderPassDescriptor,
	RenderPipeline, RenderPipelineDescriptor, ShaderModule, TextureFormat, TextureView,
	VertexAttribute, VertexBufferLayout, VertexState,
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

component!(pub Colors: Vec<Srgba>, new_colors, Vec<Srgba>);
pub fn new_colors(ContMut(mut colors): ContMut<Colors>, args: &mut NewArgs) {
	if let Some(new_colors) = args.take::<Colors>() {
		colors.extend(new_colors);
	} else {
		colors.extend(vec![WHITE; args.len()]);
	}
}
component!(pub Transforms2D: Vec<Affine2>);
component!(pub Instances2D: TypedBuffer<Instance2D>);
pub fn update_instances_2d(
	CompMut((mut instances, mut diff)): CompMut<Instances2D>,
	CompRef((transforms, colors)): CompRef<(Transforms2D, Colors)>,
	_: &mut (),
) {
	for (diff, transform, color, instance) in izip!(
		diff.iter(),
		transforms.iter(),
		colors.iter(),
		instances.iter_mut(),
	) {
		if !diff {
			continue;
		}
		instance.transform = transform.to_cols_array();
		instance.color = color.to_rgba8();
	}
	diff.fill(false);
}
pub fn draw_instances_2d(
	Len(len): Len,
	ContRef(instances): ContRef<Instances2D>,
	CompRef(meshes): CompRef<Meshes2D>,
	pass: &mut RenderPass<'_>,
) {
	pass.set_vertex_buffer(1, instances.buffer().slice(..));
	match meshes {
		ure_data::containers::RefOrSlice::Ref(mesh) => {
			mesh.set(pass);
			pass.draw_indexed(0..mesh.indices, 0, 0..instances.len() as u32);
		}
		ure_data::containers::RefOrSlice::Slice(meshes) => {
			for i in 0..len {
				let mesh = &meshes[i];
				mesh.set(pass);
				pass.draw_indexed(0..mesh.indices, 0, i as u32..i as u32 + 1);
			}
		}
		ure_data::containers::RefOrSlice::None => {}
	}
}
pub fn draw_glob_instances_2d(
	ContRef(instances): ContRef<Instances2D>,
	CompRef(meshes): CompRef<Meshes2D>,
	(pass, indices): &mut (RenderPass<'_>, &[usize]),
) {
	pass.set_vertex_buffer(1, instances.buffer().slice(..));
	match meshes {
		ure_data::containers::RefOrSlice::Ref(mesh) => {
			mesh.set(pass);
			for i in indices.iter().copied() {
				pass.draw_indexed(0..mesh.indices, 0, i as u32..i as u32 + 1);
			}
		}
		ure_data::containers::RefOrSlice::Slice(meshes) => {
			for i in indices.iter().copied() {
				let mesh = &meshes[i];
				mesh.set(pass);
				pass.draw_indexed(0..mesh.indices, 0, i as u32..i as u32 + 1);
			}
		}
		ure_data::containers::RefOrSlice::None => {}
	}
}
component!(pub Meshes2D: OneOrMany<Arc<Mesh2D>>, new_meshes_2d, Vec<Arc<Mesh2D>>);
pub fn new_meshes_2d(ContMut(mut meshes): ContMut<Meshes2D>, args: &mut NewArgs) {
	let OneOrMany::Many(vec) = &mut *meshes else {
		return;
	};
	if let Some(args) = args.take::<Meshes2D>() {
		vec.extend(args);
	}
	let empty = EMPTY.load();
	for _ in 0..args.len() {
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

pub static SHADER: LazyLock<ShaderModule> = LazyLock::new(|| {
	GPU.device
		.create_shader_module(wgpu::include_wgsl!("two/2d.wgsl"))
});
pub static LAYOUT: LazyLock<PipelineLayout> = LazyLock::new(|| {
	GPU.device
		.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: None,
			bind_group_layouts: &[&CAMERA_LAYOUT],
			push_constant_ranges: &[],
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
	pipeline: RenderPipeline,
	camera_buffer: wgpu::Buffer,
	camera: wgpu::BindGroup,
}
impl<Key: slotmap::Key> Visuals2D<Key> {
	pub fn new(format: TextureFormat) -> Self {
		let pipeline = GPU
			.device
			.create_render_pipeline(&RenderPipelineDescriptor {
				label: None,
				layout: Some(&LAYOUT),
				vertex: VertexState {
					module: &SHADER,
					entry_point: Some("vertex"),
					compilation_options: PipelineCompilationOptions::default(),
					buffers: &[Vertex2D::LAYOUT, Instance2D::LAYOUT],
				},
				fragment: Some(FragmentState {
					module: &SHADER,
					entry_point: Some("fragment"),
					compilation_options: PipelineCompilationOptions::default(),
					targets: &[Some(wgpu::ColorTargetState {
						format,
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
			pipeline,
			camera,
			camera_buffer,
		}
	}
	pub fn add(&mut self, data: &Data<Key>, key: Key) {
		let Some(group) = data.get(key) else {
			return;
		};
		let mut group = group.borrow_mut();
		group.add_component::<Transforms2D>().unwrap();
		group.add_component::<Colors>().unwrap();
		group.add_component::<Instances2D>().unwrap();
		group.add_component::<Meshes2D>().unwrap();
		self.keys.push(key);
	}
	pub fn begin_pass<'a>(
		&self,
		encoder: &'a mut CommandEncoder,
		view: &TextureView,
	) -> RenderPass<'a> {
		encoder.begin_render_pass(&RenderPassDescriptor {
			label: Some("Visuals 2D"),
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view,
				depth_slice: None,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(wgpu::Color {
						r: 0.0,
						g: 0.0,
						b: 0.0,
						a: 1.0,
					}),
					store: wgpu::StoreOp::Store,
				},
			})],
			depth_stencil_attachment: None,
			timestamp_writes: None,
			occlusion_query_set: None,
		})
	}
	pub fn render<'a>(&self, data: &Data<Key>, pass: &mut wgpu::RenderPass<'a>) {
		pass.set_pipeline(&self.pipeline);
		pass.set_bind_group(0, &self.camera, &[]);
		for key in self.keys.iter() {
			let Some(group) = data.get(*key) else {
				continue;
			};

			group.borrow().call_method(update_instances_2d, &mut ());
			group.borrow().call_method(draw_instances_2d, pass);
		}
	}
}
