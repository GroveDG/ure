use std::sync::mpsc::{Receiver, Sender, channel};

use winit::{
    event_loop::{EventLoopClosed, EventLoopProxy},
    window::{Window, WindowAttributes},
};

use crate::app::UserEvent;

use super::{Components, UID};

pub struct Windows {
    windows: Components<Window>,
    receiver: Receiver<(UID, Window)>,
}

impl Windows {
    pub fn new() -> (Sender<(UID, Window)>, Self) {
        let (sender, receiver) = channel();
        (
            sender,
            Self {
                windows: Default::default(),
                receiver,
            },
        )
    }
    pub fn request_new(
        uid: UID,
        attr: WindowAttributes,
        event_proxy: &EventLoopProxy<UserEvent>,
    ) -> Result<(), EventLoopClosed<UserEvent>> {
        event_proxy.send_event(UserEvent::NewWindow(uid, attr))
    }
    pub fn await_new(
        &mut self,
        uid: UID,
        attr: WindowAttributes,
        event_proxy: &EventLoopProxy<UserEvent>,
    ) {
        Self::request_new(uid, attr, event_proxy).unwrap();
        let (uid, window) = self.receiver.recv().unwrap();
        self.windows.insert(uid, window);
    }
    pub fn poll(&mut self) {
        for (uid, window) in self.receiver.try_iter() {
            self.windows.insert(uid, window);
        }
    }
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }
}
