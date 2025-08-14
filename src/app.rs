use std::{marker::PhantomData, mem::MaybeUninit, sync::Arc};

use winit::{application::ApplicationHandler, dpi::PhysicalSize, event_loop::{ActiveEventLoop, EventLoopProxy}, window::WindowAttributes};

use crate::{declare_components, gpu::Surface};

pub mod input;

pub type Input = Arc<input::Input>;

pub type Window = Arc<winit::window::Window>;

declare_components! {
    window: Window,
    surface: Surface,
    window_size: PhysicalSize<u32>,
}

pub trait Game: Send + 'static {
    type Event;

    fn new(event_loop: &ActiveEventLoop, proxy: EventLoopProxy<Self::Event>, input: Input) -> Self;
    fn run(self);
    fn event(event_loop: &ActiveEventLoop, event: Self::Event);
}
pub fn init_windows<'a>(
    windows: &'a mut [MaybeUninit<Window>],
    event_loop: &ActiveEventLoop,
) -> &'a mut [Window] {
    for w in windows.iter_mut() {
        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default())
                .unwrap(),
        );
        w.write(window);
    }
    unsafe { std::mem::transmute(windows) }
}

pub struct App<G: Game> {
    game: Option<std::thread::JoinHandle<()>>,
    proxy: EventLoopProxy<G::Event>,
    _marker: PhantomData<G>,
    input: Input,
}
impl<G: Game> ApplicationHandler<G::Event> for App<G> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let game = G::new(event_loop,self.proxy.clone(), self.input.clone());
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
            // winit::event::WindowEvent::Resized(_) => todo!(),
            // winit::event::WindowEvent::CloseRequested => todo!(),
            _ => {}
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
    
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: G::Event) {
        G::event(event_loop, event);
    }

    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(game) = self.game.take() {
            _ = game.join();
        }
    }
}

impl<G: Game> App<G> {
    pub fn new(proxy: EventLoopProxy<G::Event>) -> Self {
        Self {
            game: Default::default(),
            proxy,
            _marker: Default::default(),
            input: Default::default(),
        }
    }
}
