use std::sync::{Arc, Mutex, RwLock};

use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    window::WindowAttributes,
};

use crate::sys::{UID, gpu::GPU, input::Input, window::Windows};

#[derive(Debug)]
#[non_exhaustive]
pub enum UserEvent {
    NewWindow(UID, WindowAttributes),
    Exit,
}

pub struct App<'a> {
    pub gpu: Arc<GPU>,
    pub windows: Arc<RwLock<Windows<'a>>>,
    pub input: Arc<Mutex<Input>>,
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
                let Ok(mut input) = self.input.lock() else {
                    event_loop.exit();
                    return;
                };
                let Ok(windows) = self.windows.read() else {
                    event_loop.exit();
                    return;
                };
                let Some(uid) = windows.get_uid(&window_id) else {
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
                let Ok(mut windows) = self.windows.write() else {
                    event_loop.exit();
                    return;
                };
                let window = Arc::new(
                    event_loop
                        .create_window(attr)
                        .expect("Window creation failed. See winit::event_loop::ActiveEventLoop."),
                );
                windows.insert(uid, window.id(), window.clone(), &self.gpu);
                println!("Configured");
            }
            UserEvent::Exit => {
                event_loop.exit();
                return;
            }
        }
    }
}
