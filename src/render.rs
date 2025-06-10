//! Beware all ye who enter here!
//!
//! GPU programming is complex, nuanced, and laden
//! decades of difficult optimization. This version
//! of URE, and likely many others, use [WGPU][wgpu]
//! which is the best, and only, GPU Rust module.
//!
//! WGPU is a compelling library once the pieces
//! click into place, but is still very tricky and
//! prone to unexpected crashes in multi-threaded
//! environments. Please see [this amazing tutorial]
//! [https://sotrh.github.io/learn-wgpu/].

use std::{
    ops::Range,
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
};

use _2d::Draw2D;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt}, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, Buffer, BufferUsages, Color, RenderPass, RenderPipeline, Surface, SurfaceConfiguration, SurfaceTexture
};
use winit::window::Window;

use crate::{
    render::gpu::BlockingFuture,
    sys::{Components, UID},
};

use self::gpu::GPU;

pub mod _2d;
pub mod gpu;

// The only guarenteed color formats
//
// WGPU wants you to manage this per surface,
// but render pipelines and shaders (which are
// created before any windows) must know the
// format they're outputting to.
//
// This constant approach procludes things
// like HDR rendering and output.
// pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;
pub const SURFACE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

#[derive(Debug)]
pub enum BindResource {
    Buffer(UID),
}

#[derive(Debug)]
pub enum RenderCommand {
    // Update
    Buffer(UID, Vec<u8>, BufferUsages),
    Delete(UID),
    Window(Arc<Window>, UID),
    Bind(UID, BindLayout, Vec<(u32, BindResource)>),
    // Draw
    Pass(UID),
    Pipeline(Pipelines),
    Vertex(u32, UID, Option<u64>),
    Index(UID),
    Bound(u32, UID),
    Draw,
    Submit,

    Quit,
}

#[derive(Debug)]
#[repr(usize)]
pub enum Pipelines {
    _2D = 0,
}

#[derive(Debug)]
#[repr(usize)]
pub enum BindLayout {
    _2DCam = 0,
}

pub fn render(commands: Receiver<RenderCommand>, parker: &Sender<()>) {
    let gpu = GPU::new().block();

    let mut surfaces: Components<(Surface, Arc<Window>)> = Default::default();
    let mut buffers: Components<Buffer> = Default::default();
    let mut bind_groups: Components<BindGroup> = Default::default();
    let (pipelines, bind_layouts) = {
        let (_2d, _2d_camera) = Draw2D::pipeline(&gpu);
        ([_2d], [_2d_camera])
    };

    let mut surface_textures: Components<SurfaceTexture> = Default::default();

    'render: loop {
        let mut command: RenderCommand;

        loop {
            if let Some(c) = update(
                commands.recv().unwrap(),
                &gpu,
                &mut buffers,
                &mut surfaces,
                &mut bind_groups,
                &bind_layouts
            ) {
                command = c;
                break;
            }
        }

        for (uid, (surface, _)) in surfaces.iter() {
            if let Ok(surface_texture) = surface.get_current_texture() {
                surface_textures.insert(*uid, surface_texture);
            }
        }

        let mut encoder = gpu.device.create_command_encoder(&Default::default());

        'surfaces: loop {
            let uid = loop {
                match command {
                    RenderCommand::Pass(uid) => break uid,
                    RenderCommand::Quit | RenderCommand::Submit => break 'surfaces,
                    _ => command = commands.recv().unwrap(),
                }
            };

            // If Window was not commanded, next surface.
            let Some(surface_texture) = surface_textures.get(&uid) else {
                command = commands.recv().unwrap();
                continue 'surfaces;
            };
            let view = surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor {
                    format: Some(SURFACE_FORMAT),
                    ..Default::default()
                });

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // [USEFUL] Clear Surface
                        load: wgpu::LoadOp::Clear(Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Dummy defaults to avoid an obnoxious Option
            let mut indices: Range<u32> = 0..0;
            let mut instances: Range<u32> = 0..0;

            loop {
                if let Some(c) = draw(
                    commands.recv().unwrap(),
                    &mut pass,
                    &buffers,
                    &bind_groups,
                    &pipelines,
                    &mut indices,
                    &mut instances,
                ) {
                    command = c;
                    break;
                }
            }
        }
        
        match command {
            RenderCommand::Submit => {
                // [VITAL] Signal End of Frame
                let _ = parker.send(());

                gpu.queue.submit([encoder.finish()]);

                for (_, surface_texture) in surface_textures.drain() {
                    surface_texture.present();
                }
            }
            RenderCommand::Quit => break 'render,
            _ => {}
        }
    }
}

fn update(
    command: RenderCommand,
    gpu: &GPU,
    buffers: &mut Components<Buffer>,
    surfaces: &mut Components<(Surface, Arc<Window>)>,
    bind_groups: &mut Components<BindGroup>,
    bind_layouts: &[BindGroupLayout]
) -> Option<RenderCommand> {
    match command {
        RenderCommand::Buffer(uid, data, usage) => {
            let buffer = buffers.get_mut(&uid);
            let recreate = buffer.is_none_or(|buffer| {
                if buffer.size() == data.len() as u64 {
                    gpu.queue.write_buffer(&buffer, 0, &data);
                    false
                } else {
                    buffer.destroy();
                    true
                }
            });
            if recreate {
                buffers.insert(
                    uid,
                    gpu.device.create_buffer_init(&BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&data),
                        usage,
                    }),
                );
            }
        }
        RenderCommand::Delete(uid) => {
            surfaces.remove(&uid);
            buffers.remove(&uid);
        }
        RenderCommand::Window(window, uid) => {
            let surface = gpu.instance.create_surface(window.clone()).unwrap();
            let size = window.inner_size();
            // [VITAL] (Re)configure Surface
            // [NOTE] Also performs first time configuration.
            // Unconfigured surfaces would throw errors.
            let config = SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: SURFACE_FORMAT,
                view_formats: vec![],
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                width: size.width,
                height: size.height,
                desired_maximum_frame_latency: 2,
                present_mode: wgpu::PresentMode::Mailbox,
            };
            surface.configure(&gpu.device, &config);
            surfaces.insert(uid, (surface, window));
        }
        RenderCommand::Bind(uid, layout, entries) => {
            bind_groups.remove(&uid);
            bind_groups.insert(
                uid,
                gpu.device.create_bind_group(&BindGroupDescriptor {
                    label: None,
                    layout: &bind_layouts[layout as usize],
                    entries: &entries
                        .into_iter()
                        .map(|(slot, entry)| BindGroupEntry {
                            binding: slot,
                            resource: match entry {
                                BindResource::Buffer(uid) => {
                                    buffers.get(&uid).unwrap().as_entire_binding()
                                }
                            },
                        })
                        .collect::<Vec<_>>(),
                }),
            );
        }
        _ => return Some(command),
    }
    None
}

fn draw(
    command: RenderCommand,
    pass: &mut RenderPass,
    buffers: &Components<Buffer>,
    bind_groups: &Components<BindGroup>,
    pipelines: &[RenderPipeline],
    indices: &mut Range<u32>,
    instances: &mut Range<u32>,
) -> Option<RenderCommand> {
    match command {
        RenderCommand::Pipeline(i) => pass.set_pipeline(&pipelines[i as usize]),
        RenderCommand::Vertex(slot, uid, instance_size) => {
            let vertex = buffers.get(&uid).unwrap();
            if let Some(instance_size) = instance_size {
                *instances = 0..(vertex.size() / instance_size) as u32
            }
            pass.set_vertex_buffer(slot, vertex.slice(..))
        }
        RenderCommand::Index(uid) => {
            let index = buffers.get(&uid).unwrap();
            *indices = 0..(index.size() / (size_of::<u16>() as u64)) as u32;
            pass.set_index_buffer(index.slice(..), wgpu::IndexFormat::Uint16)
        }
        RenderCommand::Bound(slot, uid) => {
            pass.set_bind_group(slot, bind_groups.get(&uid).unwrap(), &[]);
        }
        RenderCommand::Draw => pass.draw_indexed(indices.clone(), 0, instances.clone()),
        _ => return Some(command),
    }
    None
}
