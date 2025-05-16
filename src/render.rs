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
    collections::{HashMap, VecDeque},
    sync::{Arc, Barrier, Condvar, Mutex, RwLock, mpsc::Receiver},
    time::Instant,
};

use wgpu::{Color, SurfaceConfiguration};

/// A queue-able abstract form of rendering.
///
/// Renderers receive these commands over threaded
/// channels. These commands are more abstract than
/// GPU render commands. Commands contain either
/// simple [Copy]-able structs or [UID][super::UID]s
/// to reference larger resources which may need to
/// be loaded in.
///
/// This allows a seperation of the nitty-gritty GPU
/// instructions and the developer's intentions and
/// systems. It also allows GPU communication to be
/// moved off thread without blocking or jeapordizing
/// frame-by-frame updates.
#[derive(Debug)]
pub enum DrawCommand {
    Clear(Color),

    Submit,
}
#[derive(Debug, Default)]
pub struct DrawBuffer {
    pub exit: bool,
    commands: VecDeque<DrawCommand>,
}

use crate::sys::{gpu::GPU, window::Windows};

pub fn render(
    gpu: Arc<GPU>,
    windows: Arc<RwLock<Windows>>,
    draw_commands: Arc<(Mutex<DrawBuffer>, Condvar)>,
) {
    let mut surface_sizes = HashMap::new();

    'render: loop {
        let Ok(draw_commands) = draw_commands.1.wait(draw_commands.0.lock().unwrap()) else {
            break 'render;
        };

        let start = Instant::now();

        if draw_commands.exit {
            break 'render;
        }

        // Keep windows alive to prevent surface drop.
        let mut windows_arc = Vec::new();
        let mut surface_textures = Vec::new();
        {
            let Ok(surfaces) = windows.read() else {
                break 'render;
            };

            for (uid, surface) in surfaces.iter() {
                let surface_size = surface_sizes.get(uid);
                let window_size = surface.window.inner_size();
                if surface_size.is_none_or(|surface_size| *surface_size != window_size) {
                    let config = SurfaceConfiguration {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format: surface.capabilities.formats[0],
                        view_formats: vec![surface.capabilities.formats[0].add_srgb_suffix()],
                        alpha_mode: wgpu::CompositeAlphaMode::Auto,
                        width: window_size.width,
                        height: window_size.height,
                        desired_maximum_frame_latency: 2,
                        present_mode: wgpu::PresentMode::AutoVsync,
                    };
                    surface.surface.configure(&gpu.device, &config);
                    surface_sizes.insert(*uid, window_size);
                }
                surface_textures.push((
                    surface.capabilities.formats[0],
                    surface.surface.get_current_texture().unwrap(),
                ));
                windows_arc.push(surface.window.clone());
            }
        }

        let mut encoder = gpu.device.create_command_encoder(&Default::default());

        for (surface_format, surface_texture) in surface_textures.iter() {
            // Set-up surface.
            let texture_view = surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor {
                    format: Some(surface_format.add_srgb_suffix()),
                    ..Default::default()
                });

            // [USEFUL] 2D Rendering
            {
                let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                // // [USEFUL] GUI Rendering
                // #[cfg(feature = "GUI")]
                // for node in layout.render_order() {
                //     box_renderer.render(node, window, &layout);
                // }
            }
        }

        gpu.queue.submit([encoder.finish()]);

        for (_, surface_texture) in surface_textures {
            surface_texture.present();
        }

        let end = Instant::now();

        // println!("GPU {:?}", end - start);
    }
}
