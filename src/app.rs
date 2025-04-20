use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

#[derive(Debug)]
pub struct App {
    windows: Arc<Mutex<HashMap<WindowId, Window>>>,
}
impl App {
    pub fn new(windows: Arc<Mutex<HashMap<WindowId, Window>>>) -> Self {
        Self { windows }
    }
    fn new_window(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        self.windows.lock().unwrap().insert(window.id(), window);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.new_window(event_loop);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => println!("redraw"),
            _ => (),
        }
    }
}
