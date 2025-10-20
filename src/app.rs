use std::{
    collections::HashMap,
    marker::PhantomData,
    ops::Range,
    sync::{
        Arc,
        atomic::AtomicBool,
        mpsc::{Receiver, Sender, channel},
    },
};

use ure_data::{One, component, method, new};
use wgpu::{Surface, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event_loop::{ActiveEventLoop, EventLoopProxy},
    window::{Window, WindowAttributes, WindowId},
};

use crate::gpu::GPU;

pub mod input;

pub type Input = Arc<input::Input>;

pub struct WindowReceiver {
    windows: Receiver<Window>,
    exits: Receiver<Arc<AtomicBool>>,
    proxy: EventLoopProxy<Event>,
}
impl WindowReceiver {
    pub fn new_window(&self, attrs: WindowAttributes) -> Arc<Window> {
        self.proxy.send_event(Event::NewWindow(attrs)).unwrap();
        Arc::new(self.windows.recv().unwrap())
    }
    pub fn recv_exit(&self) -> Arc<AtomicBool> {
        self.exits.recv().unwrap()
    }
}

component!(WINDOW_SOURCE: One<WindowReceiver>);
component!(WINDOWS: Vec<Arc<Window>>);
component!(WINDOW_EXITS: Vec<Arc<AtomicBool>>);
component!(WINDOW_IDS: Vec<WindowId>);
component!(WINDOW_SIZES: Vec<PhysicalSize<u32>>);
component!(SURFACES: Vec<Surface<'static>>);
component!(SURFACE_TEXTURES: Vec<Option<SurfaceTexture>>);

new!(pub new_windows WINDOWS (WINDOW_SOURCE));
pub fn new_windows(windows: &mut Vec<Arc<Window>>, receiver: &WindowReceiver, i: usize) {
    windows.push(receiver.new_window(WindowAttributes::default()));
}
new!(pub new_window_exits WINDOW_EXITS (WINDOW_SOURCE));
pub fn new_window_exits(
    window_exits: &mut Vec<Arc<AtomicBool>>,
    receiver: &WindowReceiver,
    i: usize,
) {
    window_exits.push(receiver.recv_exit());
}
new!(pub new_window_ids WINDOW_IDS (WINDOWS));
pub fn new_window_ids(window_ids: &mut Vec<WindowId>, windows: &[Arc<Window>], i: usize) {
    window_ids.push(windows[i].id());
}
new!(pub new_window_sizes WINDOW_SIZES (WINDOWS));
pub fn new_window_sizes(
    window_sizes: &mut Vec<PhysicalSize<u32>>,
    windows: &[Arc<Window>],
    i: usize,
) {
    window_sizes.push(windows[i].inner_size());
}

new!(pub new_surfaces SURFACES (WINDOWS));
pub fn new_surfaces(surfaces: &mut Vec<Surface<'static>>, windows: &[Arc<Window>], i: usize) {
    surfaces.push(GPU.instance.create_surface(windows[i].clone()).unwrap())
}

method!(pub reconfigure_surfaces () (WINDOWS, WINDOW_SIZES, SURFACES, SURFACE_TEXTURES));
pub fn reconfigure_surfaces(
    windows: &[Arc<Window>],
    sizes: &mut [PhysicalSize<u32>],
    surfaces: &[Surface<'static>],
    textures: &mut [Option<SurfaceTexture>],
) {
    for i in 0..surfaces.len() {
        let size = windows[i].inner_size();
        if sizes[i] != size {
            sizes[i] = size;
            surfaces[i].configure(
                &GPU.device,
                &surfaces[i]
                    .get_default_config(&GPU.adapter, size.width, size.height)
                    .unwrap(),
            );
        }
        textures[i] = surfaces[i].get_current_texture().ok();
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    NewWindow(WindowAttributes),
    Exit,
}

pub trait Game: 'static {
    fn new(app: AppProxy, windows: WindowReceiver) -> Self;
    fn run(self);
}

pub struct AppSender {
    pub window: Sender<winit::window::Window>,
    pub window_close: Sender<Arc<AtomicBool>>,
    pub input: Input,
}

pub struct InputReceiver {
    pub input: Input,
}

struct AppWindow {
    closed: Arc<AtomicBool>,
}

pub struct AppProxy {
    inner: EventLoopProxy<Event>,
}
impl AppProxy {
    pub fn exit(&self) {
        self.inner.send_event(Event::Exit).unwrap();
    }
}

pub struct App<G: Game> {
    game: Option<std::thread::JoinHandle<()>>,
    proxy: EventLoopProxy<Event>,
    sender: AppSender,
    windows: HashMap<WindowId, AppWindow>,
    _marker: PhantomData<G>,
}
impl<G: Game> ApplicationHandler<Event> for App<G> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let proxy = self.proxy.clone();
        let (window_send, window_recv) = channel();
        let (window_close_send, exits_recv) = channel();
        let input: Input = Default::default();

        self.sender = AppSender {
            window: window_send,
            window_close: window_close_send,
            input: input.clone(),
        };
        let windows = WindowReceiver {
            windows: window_recv,
            exits: exits_recv,
            proxy: proxy.clone(),
        };

        self.game = Some(std::thread::spawn(move || {
            let game = G::new(AppProxy { inner: proxy }, windows);
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
            winit::event::WindowEvent::CloseRequested => self
                .windows
                .get(&window_id)
                .unwrap()
                .closed
                .store(true, std::sync::atomic::Ordering::Relaxed),
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
                let window = event_loop.create_window(attrs).unwrap();
                let close = Arc::new(AtomicBool::new(false));
                _ = self.sender.window_close.send(close.clone());
                self.windows
                    .insert(window.id(), AppWindow { closed: close });
                _ = self.sender.window.send(window);
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
                window_close: channel().0,
                input: Default::default(),
            },
            windows: Default::default(),
            _marker: Default::default(),
        }
    }
}
