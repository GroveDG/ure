use std::sync::{Arc, mpsc::Receiver};

use winit::{
    event_loop::{EventLoopClosed, EventLoopProxy},
    window::{Window, WindowAttributes},
};

use crate::sys::{Components, UID, delete::Delete};

use super::UserEvent;

#[derive(Debug)]
pub struct Windows {
    pub windows: Components<Arc<Window>>,
    pub requested: u8,
    event_proxy: EventLoopProxy<UserEvent>,
    window_recv: Receiver<(UID, Window)>,
}

impl Windows {
    pub fn new(
        event_proxy: EventLoopProxy<UserEvent>,
        window_recv: Receiver<(UID, Window)>,
    ) -> Self {
        Self {
            windows: Default::default(),
            requested: Default::default(),
            event_proxy,
            window_recv,
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
    pub fn request_new(
        &mut self,
        uid: UID,
        attr: WindowAttributes,
        event_proxy: &EventLoopProxy<UserEvent>,
    ) -> Result<(), EventLoopClosed<UserEvent>> {
        self.requested += 1;
        event_proxy.send_event(UserEvent::NewWindow(uid, attr))
    }
    pub fn receive(&mut self) {
        for (uid, window) in self.window_recv.try_iter() {
            self.requested -= 1;
            self.windows.insert(uid, Arc::new(window));
        }
    }
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty() && self.requested == 0
    }
}
impl Delete for Windows {
    fn delete(&mut self, uid: &UID) {
        self.windows.remove(uid);
    }
}
