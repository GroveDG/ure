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
    hint::black_box, ops::Range, pin::Pin, sync::{
        mpsc::{Receiver, Sender}, Arc
    }, thread
};

use _2d::Draw2D;
use wgpu::{
    Buffer, BufferUsages, Color, RenderPipeline, Surface, SurfaceConfiguration, SurfaceTexture,
    util::{BufferInitDescriptor, DeviceExt},
};
use winit::window::Window;

use crate::{render::gpu::BlockingFuture, sys::{Components, UID}};

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
pub enum RenderCommand {
    // Update
    Buffer(UID, Vec<u8>, BufferUsages),
    Delete(UID),
    Window(Arc<Window>, UID),
    // Draw
    Pass(UID),
    Pipeline(Pipelines),
    Vertex(u32, UID, Option<u64>),
    Index(UID),
    Draw,
    Submit,

    Quit,
}

#[derive(Debug)]
#[repr(usize)]
pub enum Pipelines {
    _2D = 0,
}

pub fn render(commands: Receiver<RenderCommand>, parker: &Sender<()>) {
    let gpu = GPU::new().block();

    let mut surfaces: Components<(Surface, Arc<Window>)> = Default::default();
    let mut buffers: Components<Buffer> = Default::default();
    let pipelines: Vec<RenderPipeline> = vec![Draw2D::pipeline(&gpu)];

    let mut surface_textures: Components<SurfaceTexture> = Default::default();

    'render: loop {
        let mut command: RenderCommand;

        loop {
            command = commands.recv().unwrap();

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
                    // [VITAL] Reconfigure Surface
                    // [NOTE] Also performs first time configuration.
                    // Unconfigured surfaces would throw errors.
                    let config = SurfaceConfiguration {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format: SURFACE_FORMAT,
                        view_formats: vec![],
                        alpha_mode: wgpu::CompositeAlphaMode::Auto,
                        width: size.width,
                        height: size.height,
                        desired_maximum_frame_latency: 1,
                        present_mode: wgpu::PresentMode::Mailbox,
                    };
                    surface.configure(&gpu.device, &config);
                    surfaces.insert(uid, (surface, window));
                }
                _ => break,
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
                command = commands.recv().unwrap();

                match command {
                    RenderCommand::Pipeline(i) => pass.set_pipeline(&pipelines[i as usize]),
                    RenderCommand::Vertex(slot, uid, instance_size) => {
                        let vertex = buffers.get(&uid).unwrap();
                        if let Some(instance_size) = instance_size {
                            instances = 0..(vertex.size() / instance_size) as u32
                        }
                        pass.set_vertex_buffer(slot, vertex.slice(..))
                    }
                    RenderCommand::Index(uid) => {
                        let index = buffers.get(&uid).unwrap();
                        indices = 0..(index.size() / 2) as u32; // div by 2 because indices are u16
                        pass.set_index_buffer(index.slice(..), wgpu::IndexFormat::Uint16)
                    }
                    RenderCommand::Draw => pass.draw_indexed(indices.clone(), 0, instances.clone()),
                    _ => break,
                }
            }
        }

        match command {
            RenderCommand::Submit => {
                // [VITAL] Signal End of Frame
                let _ = parker.send(());

                black_box(gpu.queue.submit([encoder.finish()]));

                for (_, surface_texture) in surface_textures.drain() {
                    surface_texture.present();
                }
            }
            RenderCommand::Quit => break 'render,
            _ => {}
        }
    }
}
