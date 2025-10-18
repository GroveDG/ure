use std::{
	cell::OnceCell,
	collections::HashMap,
	marker::PhantomData,
	sync::{
		Arc,
		atomic::AtomicBool,
		mpsc::{Receiver, Sender, channel},
	},
};

use ure_data::{Extract, Get, Method, One, component, method};
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

pub struct WindowSource {
	receiver: Receiver<Window>,
	proxy: EventLoopProxy<Event>,
}
impl WindowSource {
	pub fn new_window(&self, attrs: WindowAttributes) -> Arc<Window> {
		self.proxy.send_event(Event::NewWindow(attrs)).unwrap();
		Arc::new(self.receiver.recv().unwrap())
	}
}

component!(WindowReceiver: One<WindowSource>);
component!(Windows: Vec<Arc<Window>>);
pub fn new_windows(receiver: WindowReceiver) {}
component!(WindowIds: Vec<WindowId>);
component!(WindowSizes: Vec<PhysicalSize<u32>>);
component!(WindowExits: Vec<Arc<AtomicBool>>);
component!(Surfaces: Vec<Surface<'static>>);
component!(SurfaceTextures: Vec<Option<SurfaceTexture>>);

const RECONFIGURE_SURFACES: Method = method!(
	reconfigure_surfaces,
	get (Windows, WindowSizes, Surfaces, SurfaceTextures)
);
pub fn reconfigure_surfaces(
	(windows, sizes, surfaces, textures): Get<(Windows, WindowSizes, Surfaces, SurfaceTextures)>,
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
	fn new(app: AppProxy, windows: Windows, window_close: WindowExits) -> Self;
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
		let (window_close_send, window_close_recv) = channel();
		let input: Input = Default::default();

		self.sender = AppSender {
			window: window_send,
			window_close: window_close_send,
			input: input.clone(),
		};
		let windows = Windows {
			// receiver: window_recv,
			// proxy: proxy.clone(),
		};
		let window_close = WindowExits {
			// receiver: window_close_recv,
		};

		self.game = Some(std::thread::spawn(move || {
			let game = G::new(AppProxy { inner: proxy }, windows, window_close);
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
