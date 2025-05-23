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

use parking_lot::{Condvar, Mutex, RwLock};
use std::{
    collections::HashMap,
    hint::black_box,
    ops::DerefMut,
    sync::{Arc, atomic::AtomicBool, mpsc::Sender},
    thread,
    time::Instant,
};
use wgpu::{Color, SurfaceConfiguration};

use crate::sys::{
    gpu::{
        GPU,
        render2d::{Commands2D, Render2D},
    },
    window::Windows,
};

#[derive(Debug, Default)]
pub struct DrawBuffer {
    pub exit: bool,
    pub clear: Color,
    pub commands_2d: Commands2D,
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
    gpu: Arc<GPU>,
    windows: Arc<RwLock<Windows>>,
    draw: Arc<Mutex<DrawBuffer>>,
    exit: Arc<AtomicBool>,
    parker: Sender<()>,
) {
    let mut surface_sizes = HashMap::new();
    let render_2d = Render2D::new(&gpu);

    'render: loop {
        if exit.load(std::sync::atomic::Ordering::Relaxed) {
            break 'render;
        }

        let start = Instant::now();

        // Keep windows alive to prevent surface drop.
        let mut surface_arc = Vec::new();
        let mut surface_textures = Vec::new();
        {
            let windows = windows.read();

            for (uid, surface) in windows.windows.iter() {
                surface_arc.push(surface.clone());
                let surface_size = surface_sizes.get(uid);
                let window_size = surface.window.inner_size();
                if surface_size.is_none_or(|surface_size| *surface_size != window_size) {
                    let config = SurfaceConfiguration {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format: SURFACE_FORMAT,
                        view_formats: vec![],
                        alpha_mode: wgpu::CompositeAlphaMode::Auto,
                        width: window_size.width,
                        height: window_size.height,
                        desired_maximum_frame_latency: 2,
                        present_mode: wgpu::PresentMode::AutoVsync,
                    };
                    surface.surface.configure(&gpu.device, &config);
                    surface_sizes.insert(*uid, window_size);
                }
                surface_textures.push(surface.surface.get_current_texture().unwrap());
            }
        }

        let mut encoder = gpu.device.create_command_encoder(&Default::default());

        for surface_texture in surface_textures.iter() {
            // Set-up surface.
            let texture_view = surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor {
                    format: Some(SURFACE_FORMAT),
                    ..Default::default()
                });

            // [USEFUL] 2D Rendering
            {
                let draw = draw.lock();
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(draw.clear),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                render_pass.set_pipeline(&render_2d.pipeline);
                render_pass.draw(0..3, 0..1);
            }
        }

        black_box(gpu.queue.submit([encoder.finish()]));

        for surface_texture in surface_textures {
            surface_texture.present();
        }

        black_box(drop(surface_arc));

        println!("GPU {:?}", start.elapsed());

        let _ = parker.send(());
        thread::park();
    }
    let _ = parker.send(());
}
