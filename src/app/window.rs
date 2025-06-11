use std::sync::{Arc, mpsc::Receiver};

use wgpu::{Device, Instance, Surface, wgt::SurfaceConfiguration};
use winit::{
    dpi::PhysicalSize,
    event_loop::{EventLoopClosed, EventLoopProxy},
    window::{Window, WindowAttributes},
};

use crate::{
    game::SURFACE_FORMAT,
    sys::{Components, UID, delete::Delete},
};

use super::UserEvent;

#[derive(Debug)]
pub struct Windows<'a> {
    pub windows: Components<(Arc<Window>, Surface<'a>)>,
    pub requested: u8,
    event_proxy: EventLoopProxy<UserEvent>,
    window_recv: Receiver<(UID, Window)>,
    sizes: Components<PhysicalSize<u32>>,
}

impl<'a> Windows<'a> {
    pub fn new(
        event_proxy: EventLoopProxy<UserEvent>,
        window_recv: Receiver<(UID, Window)>,
    ) -> Self {
        Self {
            windows: Default::default(),
            requested: Default::default(),
            event_proxy,
            window_recv,
            sizes: Default::default(),
        }
    }
    /// Request a new window component to be
    /// added to the entity.
    ///
    /// WHY
    /// -------------------------------------
    /// Windows are safely and soley created
    /// by [winit]. We must send an event to
    /// prompt winit to make a window. It will
    /// send the resulting window back.
    pub fn request(
        &mut self,
        uid: UID,
        attr: WindowAttributes,
    ) -> Result<(), EventLoopClosed<UserEvent>> {
        self.requested += 1;
        self.event_proxy.send_event(UserEvent::NewWindow(uid, attr))
    }
    pub fn receive(&mut self, instance: &Instance, device: &Device) {
        for (uid, window) in self.window_recv.try_iter() {
            self.requested -= 1;
            let window = Arc::new(window);
            let size = window.inner_size();
            let surface = instance.create_surface(window.clone()).unwrap();
            surface.configure(
                device,
                &SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: SURFACE_FORMAT,
                    view_formats: vec![],
                    alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    width: size.width,
                    height: size.height,
                    desired_maximum_frame_latency: 2,
                    present_mode: wgpu::PresentMode::Mailbox,
                },
            );
            self.windows.insert(uid, (window, surface));
        }
    }
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty() && self.requested == 0
    }
}
impl<'a> Delete for Windows<'a> {
    fn delete(&mut self, uid: &UID) {
        self.windows.remove(uid);
        self.sizes.remove(uid);
    }
}
