use std::sync::{mpsc::{channel, Receiver, Sender}, Arc};

use wgpu::Surface;
use winit::{
    event_loop::{EventLoopClosed, EventLoopProxy},
    window::{Window, WindowAttributes},
};

use crate::app::UserEvent;

use super::{Components, UID};

#[derive(Debug, Default)]
pub struct Windows {
    windows: Components<Arc<Window>>,
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
    /// send the resulting window back (see
    /// [Windows::poll]). If you cannot
    /// continue without the window, see
    /// [Windows::await_new].
    pub fn request_new(
        uid: UID,
        attr: WindowAttributes,
        event_proxy: &EventLoopProxy<UserEvent>,
    ) -> Result<(), EventLoopClosed<UserEvent>> {
        event_proxy.send_event(UserEvent::NewWindow(uid, attr))
    }
    // /// Requests a [Window] (see [Windows::request_new])
    // /// and waits for the Window.
    // /// 
    // /// This is particularly useful for start-up
    // /// when having no Windows might close the
    // /// game.
    // /// 
    // /// WHY
    // /// -------------------------------------
    // /// See [Windows::request_new].
    // pub fn await_new(
    //     &mut self,
    //     uid: UID,
    //     attr: WindowAttributes,
    //     event_proxy: &EventLoopProxy<UserEvent>,
    // ) {
    //     Self::request_new(uid, attr, event_proxy).unwrap();
    //     let (uid, window) = self.receiver.recv().unwrap();
    //     self.windows.insert(uid, window);
    // }
    pub fn insert(&mut self, uid: UID, window: Arc<Window>) {
        self.windows.insert(uid, window);
    }
    pub fn get(&self, uid: &UID) -> Option<&Arc<Window>> {
        self.windows.get(uid)
    }
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = (&UID, &Arc<Window>)> {
        self.windows.iter()
    }
}
