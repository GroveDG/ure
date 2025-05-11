use std::{
    sync::{Arc, Mutex, mpsc::Sender},
    thread::{JoinHandle, sleep},
    time::Duration,
};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

use crate::{
    FRAME_PERIOD,
    sys::{UID, input::Input},
};

#[derive(Debug)]
pub enum UserEvent {
    NewWindow(UID, WindowAttributes),
}

pub struct App {
    /// WHY
    /// ----------------------------
    /// This is an Option because
    /// it needs to be taken to join.
    /// See [App::user_event].
    pub game: Option<JoinHandle<()>>,
    pub windows: Sender<(UID, Window)>,
    pub input: Arc<Mutex<Input>>,
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
                // Take ownership of game thread.
                let Some(game) = self.game.take() else {
                    // If a past close attempt has happened,
                    // we must still be closing.
                    return;
                };
                // Inform game thread of close.
                {
                    let Ok(mut input) = self.input.lock() else {
                        event_loop.exit();
                        return;
                    };
                    input.close = true;
                }
                // Wait for game to stop.
                let mut i = 0;
                while !game.is_finished() && i < 10 {
                    sleep(FRAME_PERIOD);
                    i += 1;
                }
                // Prevent detatched thread.
                // (See std::thread::Thread)
                let _ = game.join(); // Ignore panic.
                // Exit.
                event_loop.exit();
            }
            _ => {}
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::NewWindow(uid, attr) => {
                let window = event_loop
                    .create_window(attr)
                    .expect("Window creation failed. See winit::event_loop::ActiveEventLoop.");
                if self.windows.send((uid, window)).is_err() {
                    event_loop.exit();
                }
            }
        }
    }
}
