use std::{marker::PhantomData, mem::MaybeUninit, sync::Arc};

use winit::{application::ApplicationHandler, window::WindowAttributes};

use crate::declare_components;

pub mod input;

pub type Input = Arc<input::Input>;

pub type Window = Arc<winit::window::Window>;

pub trait Game: Send + 'static {
    fn new(event_loop: &winit::event_loop::ActiveEventLoop, input: Input) -> Self;
    fn run(self);
}

declare_components! {
    window: crate::app::Window,
    surface: crate::gpu::Surface,
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
    input: Input,
}
impl<G: Game> ApplicationHandler for App<G> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let game = G::new(event_loop, self.input.clone());
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
        _ = (event_loop, window_id);
        match event {
            winit::event::WindowEvent::Resized(_) => todo!(),
            winit::event::WindowEvent::CloseRequested => todo!(),
            _ => {},
        }
    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        self.input.process_device_event(&device_id, event);
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
            input: Default::default(),
        }
    }
}
