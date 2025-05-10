use std::{sync::mpsc::Sender, thread::JoinHandle};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

use crate::sys::UID;

#[derive(Debug)]
pub enum UserEvent {
    NewWindow(UID, WindowAttributes),
}

pub struct App {
    pub game: JoinHandle<()>,
    pub windows: Sender<(UID, Window)>,
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                // self.game.join();
                event_loop.exit();
            }
            _ => {}
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::NewWindow(uid, attr) => {
                self.windows
                    .send((uid, event_loop.create_window(attr).unwrap()))
                    .unwrap();
            }
        }
    }
}
