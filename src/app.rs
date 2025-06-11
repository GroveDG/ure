use std::sync::{mpsc::Sender, Arc};

use parking_lot::Mutex;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

use crate::sys::{BiComponents, Uid};

use self::input::Input;

pub mod input;
pub mod window;

#[derive(Debug)]
#[non_exhaustive]
pub enum UserEvent {
    NewWindow(Uid, Box<WindowAttributes>),
    Exit,
}

pub struct App {
    pub window_ids: BiComponents<WindowId>,
    pub window_sndr: Sender<(Uid, Window)>,
    pub input: Arc<Mutex<Input>>,
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                let mut input = self.input.lock();
                let Some(uid) = self.window_ids.get_by_right(&window_id) else {
                    return;
                };
                input.close.insert(*uid);
            }
            _ => {}
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::NewWindow(uid, attr) => {
                let window = event_loop
                    .create_window(*attr)
                    .expect("Window creation failed. See winit::event_loop::ActiveEventLoop.");
                self.window_ids.insert(uid, window.id());
                let _ = self.window_sndr.send((uid, window));
            }
            UserEvent::Exit => {
                event_loop.exit();
            }
        }
    }
}
