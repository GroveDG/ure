use std::{
    marker::PhantomData,
    mem::MaybeUninit,
    sync::{
        Arc,
        mpsc::{Receiver, Sender, channel},
    },
};

use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, EventLoopProxy},
    window::WindowAttributes,
};

pub mod input;

pub type Input = Arc<input::Input>;

pub type Window = Arc<winit::window::Window>;

#[derive(Debug, Clone)]
pub enum Event {
    NewWindow(WindowAttributes),
    Exit,
}

pub trait Game: 'static {
    fn new(recv: AppReceiver) -> Self;
    fn run(self);
}
pub fn init_windows(
    attributes: impl IntoIterator<Item = WindowAttributes>,
    windows: &mut [MaybeUninit<Window>],
    receiver: &AppReceiver,
) {
    let mut attributes = attributes.into_iter();
    for i in 0..windows.len() {
        let attributes = attributes.next().unwrap();
        windows[i].write(receiver.new_window(attributes));
    }
}

pub struct AppSender {
    pub window: Sender<winit::window::Window>,
    pub input: Input,
}

pub struct AppReceiver {
    pub window: Receiver<winit::window::Window>,
    pub input: Input,
    pub proxy: EventLoopProxy<Event>,
}
impl AppReceiver {
    pub fn new_window(&self, attrs: WindowAttributes) -> Window {
        self.proxy.send_event(Event::NewWindow(attrs)).unwrap();
        Arc::new(self.window.recv().unwrap())
    }
}

pub struct App<G: Game> {
    game: Option<std::thread::JoinHandle<()>>,
    proxy: EventLoopProxy<Event>,
    sender: AppSender,
    _marker: PhantomData<G>,
}
impl<G: Game> ApplicationHandler<Event> for App<G> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let proxy = self.proxy.clone();
        let (window_send, window_recv) = channel();
        let input: Input = Default::default();

        self.sender = AppSender {
            window: window_send,
            input: input.clone(),
        };
        let app_recv = AppReceiver {
            window: window_recv,
            input,
            proxy,
        };

        self.game = Some(std::thread::spawn(move || {
            let game = G::new(app_recv);
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
        self.sender.input.process_device_event(&device_id, event);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: Event) {
        match event {
            Event::NewWindow(attrs) => {
                if self
                    .sender
                    .window
                    .send(event_loop.create_window(attrs).unwrap())
                    .is_err()
                {
                    event_loop.exit()
                }
            }
            Event::Exit => {
                event_loop.exit();
            }
        }
    }

    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(game) = self.game.take() {
            _ = game.join();
        }
    }
}

impl<G: Game> App<G> {
    pub fn new(proxy: EventLoopProxy<Event>) -> Self {
        Self {
            game: Default::default(),
            proxy,
            sender: AppSender {
                window: channel().0,
                input: Default::default(),
            },
            _marker: Default::default(),
        }
    }
}
