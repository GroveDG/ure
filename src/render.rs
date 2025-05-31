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
    hint::black_box,
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
    thread,
};

use _2d::Render2D;
use wgpu::{
    Buffer, Color, RenderPipeline, Surface, SurfaceConfiguration, SurfaceTexture,
    util::{BufferInitDescriptor, DeviceExt},
};
use winit::window::Window;

use crate::sys::{Components, UID};

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

pub enum RenderCommand {
    // Update
    Buffer(UID, Vec<u8>),
    Delete(UID),
    Window(Arc<Window>, UID),
    // Draw
    Pass(UID),
    Pipeline(UID),
    Vertex(u32, UID),
    Index(UID),
    Submit,

    Quit,
}

pub fn render(commands: Receiver<RenderCommand>, parker: &Sender<()>) {
    let gpu = futures::executor::block_on(GPU::new());

    let mut surfaces: Components<Surface> = Default::default();
    let mut windows: Vec<Arc<Window>> = Default::default();
    let mut buffers: Components<Buffer> = Default::default();
    let mut pipelines: Components<RenderPipeline> = Default::default();

    let pipeline_2d = Render2D::pipeline(&gpu);

    'render: loop {
        let mut command: RenderCommand;
        let mut surface_textures: Components<SurfaceTexture> = Default::default();

        loop {
            command = commands.recv().unwrap();

            match command {
                RenderCommand::Buffer(uid, data) => {
                    let Some(buffer) = buffers.get(&uid) else {
                        continue 'render;
                    };
                    if buffer.size() == data.len() as u64 {
                        let capture = buffer.clone();
                        buffer.map_async(wgpu::MapMode::Write, .., move |result| {
                            if result.is_err() {
                                return;
                            }
                            let mut view = capture.get_mapped_range_mut(..);
                            view.copy_from_slice(&data);
                            drop(view);
                            capture.unmap();
                        });
                    } else {
                        let usage = buffer.usage();
                        buffer.destroy();
                        gpu.device.create_buffer_init(&BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(&data),
                            usage,
                        });
                    }
                }
                RenderCommand::Delete(uid) => {
                    surfaces.remove(&uid);
                    buffers.remove(&uid).map(|buffer| buffer.destroy());
                    pipelines.remove(&uid);
                }

                RenderCommand::Window(window, uid) => {
                    // Create surface for new windows.
                    if !surfaces.contains_key(&uid) {
                        surfaces.insert(uid, gpu.instance.create_surface(window.clone()).unwrap());
                    }
                    let surface = surfaces.get(&uid).unwrap();
                    // [VITAL] Reconfigure Surface
                    // [NOTE] Also performs first time configuration.
                    // Unconfigured surfaces would throw errors.
                    let size = window.inner_size();
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
                    // [BUG] When rapidly and repeatedly resizing
                    // (far more than is realistic) the mutex that
                    // configure uses becomes locked elsewhere and
                    // this thread becomes unresponsive.
                    //
                    // This may also be happening in get_current_texture
                    // since they both use the same mutex.
                    //
                    // This could also be in some other mutex within
                    // WGPU since it uses a lot of mutexes.
                    surface.configure(&gpu.device, &config);
                    windows.push(window);
                    surface_textures.insert(uid, surface.get_current_texture().unwrap());
                }
                _ => break,
            }
        }

        let mut encoder = gpu.device.create_command_encoder(&Default::default());

        loop {
            let mut pass = match command {
                RenderCommand::Pass(uid) => {
                    let view = surface_textures.get(&uid).unwrap().texture.create_view(
                        &wgpu::TextureViewDescriptor {
                            format: Some(SURFACE_FORMAT),
                            ..Default::default()
                        },
                    );

                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                    })
                }
                _ => break,
            };

            loop {
                match command {
                    RenderCommand::Pipeline(uid) => pass.set_pipeline(pipelines.get(&uid).unwrap()),
                    RenderCommand::Vertex(slot, uid) => {
                        pass.set_vertex_buffer(slot, buffers.get(&uid).unwrap().slice(..))
                    }
                    RenderCommand::Index(uid) => pass.set_index_buffer(
                        buffers.get(&uid).unwrap().slice(..),
                        wgpu::IndexFormat::Uint16,
                    ),
                    _ => break,
                }
            }

            command = commands.recv().unwrap();
        }

        match command {
            RenderCommand::Submit => {
                black_box(gpu.queue.submit([encoder.finish()]));

                for (_, surface_texture) in surface_textures.drain() {
                    surface_texture.present();
                }

                black_box(windows.clear());

                // [VITAL] Signal End of Frame
                let _ = parker.send(());
                // [VITAL] Wait for Next Frame
                thread::park();
            }
            RenderCommand::Quit => break 'render,
            _ => {}
        }
    }
    let _ = parker.send(());
}
