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
    ops::Range,
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
    thread,
};

use _2d::Draw2D;
use wgpu::{
    Buffer, BufferUsages, Color, RenderPipeline, Surface, SurfaceConfiguration, SurfaceTexture,
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
    let gpu = futures::executor::block_on(GPU::new());

    let mut surfaces: Components<(Surface, Arc<Window>)> = Default::default();
    let mut buffers: Components<Buffer> = Default::default();
    let pipelines: Vec<RenderPipeline> = vec![Draw2D::pipeline(&gpu)];

    'render: loop {
        let mut command: RenderCommand;
        let mut surface_textures: Components<SurfaceTexture> = Default::default();

        loop {
            command = commands.recv().unwrap();

            match command {
                RenderCommand::Buffer(uid, data, usage) => {
                    let buffer = buffers.insert(
                        uid,
                        gpu.device.create_buffer_init(&BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(&data),
                            usage: usage,
                        }),
                    );
                    if let Some(buffer) = buffer {
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
                            buffer.destroy();
                        }
                    }
                }
                RenderCommand::Delete(uid) => {
                    if let Some((surface, window)) = surfaces.remove(&uid) {
                        // Specify drop order specifically so
                        // Surface doesn't exist without window.
                        // IDK if this is actually important,
                        // but WGPU is picky about when you
                        // close the window.
                        drop(surface);
                        drop(window);
                    }
                    buffers.remove(&uid).map(|buffer| buffer.destroy());
                }
                RenderCommand::Window(window, uid) => {
                    let size = window.inner_size();
                    // Create surface for new windows.
                    if !surfaces.contains_key(&uid) {
                        surfaces.insert(
                            uid,
                            (gpu.instance.create_surface(window.clone()).unwrap(), window),
                        );
                    }
                    let (surface, _) = surfaces.get(&uid).unwrap();
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
                    // [BUG] On X11, when rapidly and repeatedly resizing
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
                    surface_textures.insert(uid, surface.get_current_texture().unwrap());
                }
                _ => break,
            }
        }

        let mut encoder = gpu.device.create_command_encoder(&Default::default());

        'surfaces: loop {
            let uid = loop {
                match command {
                    RenderCommand::Pass(uid) => break uid,
                    RenderCommand::Quit | RenderCommand::Submit => break 'surfaces,
                    _ => command = commands.recv().unwrap()
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
                        load: wgpu::LoadOp::Clear(Color::BLUE),
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
                black_box(gpu.queue.submit([encoder.finish()]));

                for (_, surface_texture) in surface_textures.drain() {
                    surface_texture.present();
                }

                // [VITAL] Signal End of Frame
                let _ = parker.send(());
                // [VITAL] Wait for Next Frame
                thread::park();
            }
            RenderCommand::Quit => break 'render,
            _ => {}
        }
    }
}
