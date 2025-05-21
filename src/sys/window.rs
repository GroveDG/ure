use std::sync::Arc;

use wgpu::{Surface, SurfaceCapabilities, TextureFormat};
use winit::{
    event_loop::{EventLoopClosed, EventLoopProxy},
    window::{Window, WindowAttributes, WindowId},
};

use crate::app::UserEvent;

use super::{BiComponents, Components, UID, delete::Delete, gpu::GPU};

#[derive(Debug, Default)]
pub struct Windows<'a> {
    pub windows: Components<Arc<WindowSurface<'a>>>,
    pub window_ids: BiComponents<WindowId>,
    pub requested: u8,
}

#[derive(Debug)]
pub struct WindowSurface<'a> {
    pub window: Arc<Window>,
    pub surface: Surface<'a>,
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
        self.windows.insert(
            uid,
            Arc::new(WindowSurface {
                window,
                surface,
            }),
        );
        self.window_ids.insert(uid, window_id);
        self.requested = self.requested.saturating_sub(1);
    }
}
impl<'a> Delete for Windows<'a> {
    fn delete(&mut self, uid: &UID) {
        self.window_ids.remove_by_left(uid);
        self.windows.remove(uid);
    }
}
