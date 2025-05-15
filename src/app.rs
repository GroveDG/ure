use std::{
    sync::{mpsc::Sender, Arc, Mutex},
    thread::{sleep, JoinHandle}, time::Instant,
};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

use crate::{
    FRAME_PERIOD,
    sys::{
        BiComponents, UID,
        gpu::{GPU, surfaces::Surfaces},
        input::Input,
    },
};

#[derive(Debug)]
#[non_exhaustive]
pub enum UserEvent {
    NewWindow(UID, WindowAttributes),
}

pub struct App<'a> {
    pub windows: Sender<(UID, Arc<Window>)>,
    pub window_ids: BiComponents<WindowId>,
    pub surfaces: Surfaces<'a>,
    pub input: Arc<Mutex<Input>>,
    pub gpu: GPU,
}

impl<'a> App<'a> {
    pub fn render(&self, uid: &UID) {
        let mut encoder = self.gpu.device.create_command_encoder(&Default::default());

        let Some(surface) = self.surfaces.get(uid) else {
            return;
        };
        let surface_texture = surface.surface.get_current_texture().unwrap();

        // Set-up surface.
        let capabilities = &surface.capabilities;
        let surface_format = capabilities.formats[0];
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
        self.gpu.queue.submit([encoder.finish()]);

        surface_texture.present();
    }
}

impl<'a> ApplicationHandler<UserEvent> for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                let uid = self.window_ids.get_by_right(&window_id).unwrap();
                self.surfaces.configure(uid, &self.gpu.device);
            }
            WindowEvent::RedrawRequested => {
                let start = Instant::now();
                self.render(self.window_ids.get_by_right(&window_id).unwrap());
                let end = Instant::now();
                println!("GPU {:?}", end - start);
            }
            _ => {}
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::NewWindow(uid, attr) => {
                let window = Arc::new(
                    event_loop
                        .create_window(attr)
                        .expect("Window creation failed. See winit::event_loop::ActiveEventLoop."),
                );
                if self.windows.send((uid, Arc::clone(&window))).is_err() {
                    event_loop.exit();
                }
                self.window_ids.insert(uid, window.id());
                self.surfaces.insert(uid, window, &self.gpu);
                self.surfaces.configure(&uid, &self.gpu.device);
            }
        }
    }
}
