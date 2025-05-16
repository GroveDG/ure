use std::{
    collections::HashMap,
    sync::{Arc, Barrier, Condvar, Mutex, RwLock},
    time::Instant,
};

use wgpu::SurfaceConfiguration;

use crate::sys::{gpu::GPU, window::Windows};

#[derive(Debug, Clone, Copy)]
pub enum RenderStatus {
    Wait,
    Render,
    Break,
}

pub fn render(
    frame_barrier: Arc<Barrier>,
    gpu: Arc<GPU>,
    windows: Arc<RwLock<Windows>>,
) {
    let mut surface_sizes = HashMap::new();

    'render: loop {
        frame_barrier.wait();

        let start = Instant::now();

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
                ))
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
