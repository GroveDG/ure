use std::{marker::PhantomData, mem::MaybeUninit, sync::Arc};

use winit::{application::ApplicationHandler, window::WindowAttributes};

pub type Window = Arc<winit::window::Window>;

pub trait Game: Send + 'static {
    fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self;
    fn run(self);
}

pub fn init_windows(
    windows: &mut [MaybeUninit<Window>],
    event_loop: &winit::event_loop::ActiveEventLoop,
) {
    for w in windows.iter_mut() {
        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default())
                .unwrap(),
        );
        w.write(window);
    }
}

pub struct App<G: Game> {
    game: Option<std::thread::JoinHandle<()>>,
    _marker: PhantomData<G>,
}
impl<G: Game> ApplicationHandler for App<G> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let game = G::new(event_loop);
        self.game = Some(std::thread::spawn(move || {
            game.run();
        }));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        // TODO
    }

    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(game) = self.game.take() {
            _ = game.join();
        }
    }
}

impl<G: Game> Default for App<G> {
    fn default() -> Self {
        Self {
            game: Default::default(),
            _marker: Default::default(),
        }
    }
}
