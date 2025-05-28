use std::sync::Arc;

use winit::{
    event_loop::{EventLoopClosed, EventLoopProxy},
    window::{Window, WindowAttributes, WindowId},
};

use crate::sys::{BiComponents, Components, UID, delete::Delete};

use super::UserEvent;

#[derive(Debug, Default)]
pub struct Windows {
    pub windows: Components<Arc<Window>>,
    pub window_ids: BiComponents<WindowId>,
    pub requested: u8,
}

impl Windows {
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
    pub fn insert(&mut self, uid: UID, window_id: WindowId, window: Window) {
        self.windows.insert(
            uid,
            Arc::new(window),
        );
        self.window_ids.insert(uid, window_id);
        self.requested = self.requested.saturating_sub(1);
    }
}
impl Delete for Windows {
    fn delete(&mut self, uid: &UID) {
        self.window_ids.remove_by_left(uid);
        self.windows.remove(uid);
    }
}
