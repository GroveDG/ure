use std::sync::{
    Arc, Weak,
    mpsc::{Receiver, Sender, channel},
};

use wgpu::{Device, Surface, SurfaceCapabilities, SurfaceConfiguration};
use winit::{
    dpi::PhysicalSize,
    event_loop::{EventLoopClosed, EventLoopProxy},
    window::{Window, WindowAttributes, WindowId},
};

use crate::app::UserEvent;

use super::{BiComponents, Components, UID, gpu::GPU};

#[derive(Debug, Default)]
pub struct Windows<'a> {
    windows: Components<WindowSurface<'a>>,
    window_ids: BiComponents<WindowId>,
    requested: u8,
}

#[derive(Debug)]
pub struct WindowSurface<'a> {
    pub window: Arc<Window>,
    pub surface: Surface<'a>,
    pub capabilities: SurfaceCapabilities,
}

impl<'a> Windows<'a> {
    /// Request a new window component to be
    /// added to the entity.
    ///
    /// WHY
    /// -------------------------------------
    /// Windows are safely and soley created
    /// by [winit]. We must send an event to
    /// prompt winit to make a window. It will
    /// send the resulting window back.
    pub fn request_new(
        &mut self,
        uid: UID,
        attr: WindowAttributes,
        event_proxy: &EventLoopProxy<UserEvent>,
    ) -> Result<(), EventLoopClosed<UserEvent>> {
        self.requested += 1;
        event_proxy.send_event(UserEvent::NewWindow(uid, attr))
    }
    pub fn insert(&mut self, uid: UID, window_id: WindowId, window: Arc<Window>, gpu: &GPU) {
        let surface = gpu.instance.create_surface(window.clone()).unwrap();
        let capabilities = surface.get_capabilities(&gpu.adapter);
        self.windows.insert(
            uid,
            WindowSurface {
                window,
                surface,
                capabilities,
            },
        );
        self.window_ids.insert(uid, window_id);
        self.requested = self.requested.saturating_sub(1);
    }
    pub fn get(&self, uid: &UID) -> Option<&WindowSurface> {
        self.windows.get(uid)
    }
    pub fn is_empty(&self) -> bool {
        self.requested == 0 && self.windows.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = (&UID, &WindowSurface)> {
        self.windows.iter()
    }
    pub fn values(&self) -> impl Iterator<Item = &WindowSurface> {
        self.windows.values()
    }
    pub fn get_uid(&self, window_id: &WindowId) -> Option<&UID> {
        self.window_ids.get_by_right(window_id)
    }
    pub fn remove(&mut self, uid: &UID) -> Option<WindowSurface> {
        self.window_ids.remove_by_left(uid);
        self.windows.remove(uid)
    }
}
