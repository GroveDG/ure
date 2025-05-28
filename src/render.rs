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
    collections::HashMap,
    hint::black_box,
    sync::{Arc, atomic::AtomicBool, mpsc::Sender},
    thread,
};

use parking_lot::{Mutex, RwLock};
use wgpu::{Color, Surface, SurfaceConfiguration};

use crate::{app::window::Windows, sys::Components};

use self::_2d::{Commands2D, Render2D, Updates2D};
use self::gpu::GPU;

pub mod _2d;
pub mod gpu;

#[derive(Debug, Default)]
pub struct Updates {
    pub _2d: Updates2D,
}

#[derive(Debug, Default)]
pub struct Commands {
    pub clear: Color,
    pub _2d: Components<Commands2D>,
}

#[derive(Debug, Default)]
pub struct RenderBuffer {
    pub updates: Updates,
    pub commands: Commands,
}

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

pub fn render(
    windows: Arc<RwLock<Windows>>,
    draw: Arc<Mutex<RenderBuffer>>,
    exit: Arc<AtomicBool>,
    parker: Sender<()>,
) {
    let gpu = futures::executor::block_on(GPU::new());
    // Cache previous window sizes to check if a reconfigure is needed.
    let mut surface_sizes = HashMap::new();
    let mut surfaces: Components<Surface> = Default::default();
    let mut render_2d = Render2D::new(&gpu);

    'render: loop {
        if exit.load(std::sync::atomic::Ordering::Relaxed) {
            break 'render;
        }

        // [VITAL] Take Draw Commands
        let mut draw = std::mem::take(&mut *draw.lock());

        // [VITAL] Update Buffers
        render_2d.update(draw.updates._2d, &gpu);

        // Keep surfaces alive to prevent surface drop.
        let windows = windows.read().windows.clone();

        // Collect surfaces for rendering.
        let surface_textures =
            Components::from_iter(windows.iter().map(|(uid, window)| {
                // Create surface for new windows.
                if !surfaces.contains_key(uid) {
                    surfaces.insert(*uid, gpu.instance.create_surface(window.clone()).unwrap());
                }
                let surface = surfaces.get(uid).unwrap();

                let window_size = window.inner_size();
                if surface_sizes
                    .get(uid)
                    .is_none_or(|surface_size| *surface_size != window_size)
                {
                    // [VITAL] Reconfigure Surface
                    // [NOTE] Also performs first time configuration.
                    // Unconfigured surfaces would throw errors.
                    let config = SurfaceConfiguration {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format: SURFACE_FORMAT,
                        view_formats: vec![],
                        alpha_mode: wgpu::CompositeAlphaMode::Auto,
                        width: window_size.width,
                        height: window_size.height,
                        desired_maximum_frame_latency: 1,
                        present_mode: wgpu::PresentMode::Mailbox,
                    };
                    surface.configure(&gpu.device, &config);
                    surface_sizes.insert(*uid, window_size);
                }
                (*uid, surface.get_current_texture().unwrap())
            }));

        // Create GPU command encoder.
        let mut encoder = gpu.device.create_command_encoder(&Default::default());

        for (uid, surface_texture) in surface_textures.iter() {

            // [VITAL] Set-Up Surface Target
            let texture_view = surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor {
                    format: Some(SURFACE_FORMAT),
                    ..Default::default()
                });

            // [VITAL] Begin Render Pass
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // [USEFUL] Clear Surface
                        load: wgpu::LoadOp::Clear(draw.commands.clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // [USEFUL] 2D Rendering
            if let Some(commands) = draw.commands._2d.remove(uid) {
                render_2d.render(&mut render_pass, commands);
            };
        }

        // [VITAL] Prevent Compiler Over-Optimization
        black_box(gpu.queue.submit([encoder.finish()]));

        // [VITAL] Show Surfaces
        for (_, surface_texture) in surface_textures {
            surface_texture.present();
        }

        // [VITAL] Make Sure Comipler Keeps Windows Alive
        black_box(drop(windows));

        // [VITAL] Signal End of Frame
        let _ = parker.send(());
        // [VITAL] Wait for Next Frame
        thread::park();
    }
    let _ = parker.send(());
}
