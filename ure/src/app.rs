use std::{
	collections::HashMap,
	marker::PhantomData,
	sync::{
		Arc,
		atomic::AtomicBool,
		mpsc::{Receiver, Sender, channel},
	},
};

use ure_data::{
	component,
	components::{CompMut, CompRef, ContMut},
	containers::One,
	group::Len,
};
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

component!(pub WindowSource: One<WindowReceiver>);
component!(pub Windows: Vec<Arc<Window>>, new_windows as fn(_, _, _));
pub fn new_windows(
	ContMut(mut windows): ContMut<Windows>,
	CompRef(window_source): CompRef<WindowSource>,
	new: usize,
) {
	for _ in 0..new {
		windows.push(window_source.new_window(Default::default()));
	}
}
component!(pub WindowExits: Vec<Arc<AtomicBool>>, new_window_exits as fn(_, _, _));
pub fn new_window_exits(
	ContMut(mut window_exits): ContMut<WindowExits>,
	CompRef(window_source): CompRef<WindowSource>,
	new: usize,
) {
	for _ in 0..new {
		window_exits.push(window_source.recv_exit());
	}
}
component!(pub WindowIds: Vec<WindowId>, new_window_ids as fn(_, _, _, _));
pub fn new_window_ids(
	Len(len): Len,
	ContMut(mut window_ids): ContMut<WindowIds>,
	CompRef(windows): CompRef<Windows>,
	new: usize,
) {
	for i in len..len + new {
		window_ids.push(windows[i].id());
	}
}
component!(pub WindowSizes: Vec<PhysicalSize<u32>>);
component!(pub Surfaces: Vec<Surface<'static>>, new_surfaces as fn(_, _, _, _));
pub fn new_surfaces(
	Len(len): Len,
	ContMut(mut surfaces): ContMut<Surfaces>,
	CompRef(windows): CompRef<Windows>,
	new: usize,
) {
	for i in len..len + new {
		surfaces.push(GPU.instance.create_surface(windows[i].clone()).unwrap());
	}
}
component!(pub SurfaceTextures: Vec<Option<SurfaceTexture>>);

pub fn close_windows(CompRef(window_exits): CompRef<WindowExits>, _: ()) -> Vec<usize> {
	let mut delete = Vec::new();
	for (i, c) in window_exits.iter().enumerate() {
		if c.load(std::sync::atomic::Ordering::Relaxed) {
			delete.push(i);
		}
	}
	delete
}
pub fn reconfigure_surfaces(
	Len(len): Len,
	CompRef((windows, surfaces)): CompRef<(Windows, Surfaces)>,
	CompMut((mut sizes, mut textures)): CompMut<(WindowSizes, SurfaceTextures)>,
	_: (),
) {
	for i in 0..len {
		let window_size = windows[i].inner_size();
		if sizes[i] != window_size {
			sizes[i] = window_size;
			surfaces[i].configure(
				&GPU.device,
				&surfaces[i]
					.get_default_config(&GPU.adapter, window_size.width, window_size.height)
					.unwrap(),
			);
		}
		textures[i] = surfaces[i].get_current_texture().ok();
	}
}
pub fn present_surfaces(CompMut(mut textures): CompMut<SurfaceTextures>, _: ()) {
	for texture in textures.iter_mut() {
		if let Some(texture) = texture.take() {
			texture.present();
		}
	}
}

pub struct WindowSystem {
	
}

#[derive(Debug, Clone)]
pub enum Event {
	NewWindow(WindowAttributes),
	Exit,
}

pub trait Game: 'static {
	fn new(app: AppProxy, window_source: One<WindowReceiver>) -> Self;
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
		let window_receiver = WindowReceiver {
			windows: window_recv,
			exits: exits_recv,
			proxy: proxy.clone(),
		};
		let window_source = One(window_receiver);

		self.game = Some(std::thread::spawn(move || {
			let game = G::new(AppProxy { inner: proxy }, window_source);
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
