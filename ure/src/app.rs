use std::{
	collections::HashMap,
	marker::PhantomData,
	sync::{Arc, atomic::AtomicBool},
};

use ure_data::{
	component,
	components::{CompMut, CompRef, ContMut},
	containers::One,
	group::{Data, Len},
	method::MethodTrait,
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
#[derive(Debug, Clone)]
pub struct AppProxy(EventLoopProxy<Event>);
impl AppProxy {
	pub fn new_windows(&self, attrs: Vec<WindowAttributes>) -> Vec<Window> {
		let (send, recv) = oneshot::channel();
		self.0.send_event(Event::NewWindow(attrs, send)).unwrap();
		recv.recv().unwrap()
	}
	pub fn recv_exits(&self, ids: Vec<WindowId>) -> Vec<Arc<AtomicBool>> {
		let (send, recv) = oneshot::channel();
		self.0.send_event(Event::RecvExits(ids, send)).unwrap();
		recv.recv().unwrap()
	}
	pub fn exit(&self) {
		self.0.send_event(Event::Exit).unwrap();
	}
}

component!(pub Proxy: One<AppProxy>);
component!(pub Windows: Vec<Arc<Window>>, new_windows as fn(_, _, _));
pub fn new_windows(
	ContMut(mut windows): ContMut<Windows>,
	CompRef(app_proxy): CompRef<Proxy>,
	new: usize,
) {
	windows.append(
		&mut app_proxy
			.new_windows(vec![Default::default(); new])
			.into_iter()
			.map(|w| Arc::new(w))
			.collect(),
	);
}
component!(pub WindowExits: Vec<Arc<AtomicBool>>, new_window_exits as fn(_, _, _, _));
pub fn new_window_exits(
	Len(len): Len,
	ContMut(mut window_exits): ContMut<WindowExits>,
	CompRef((windows, app_proxy)): CompRef<(Windows, Proxy)>,
	new: usize,
) {
	window_exits.append(
		&mut app_proxy.recv_exits(windows[len..len + new].iter().map(|w| w.id()).collect()),
	);
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

#[derive(Debug, Default)]
pub struct WindowSystem<Key: slotmap::Key> {
	keys: Vec<Key>,
}
impl<Key: slotmap::Key> WindowSystem<Key> {
	pub fn add(&mut self, data: &Data<Key>, key: Key, app_proxy: AppProxy) {
		let Some(group) = data.get(key) else {
			return;
		};
		let mut group = group.borrow_mut();
		group
			.add_container::<Proxy>(One(app_proxy.clone()))
			.unwrap();
		group.add_component::<Windows>().unwrap();
		group.add_component::<WindowExits>().unwrap();
		group.add_component::<WindowSizes>().unwrap();
		group.add_component::<Surfaces>().unwrap();
		group.add_component::<SurfaceTextures>().unwrap();
		self.keys.push(key)
	}
	pub fn close(&self, data: &Data<Key>) -> bool {
		let mut all_closed = true;
		for key in self.keys.iter().copied() {
			if let Some(group) = data.get(key) {
				let mut group = group.borrow_mut();
				if let Some(delete) = (close_windows as fn(_, _) -> Vec<usize>).call_method(&group, ()) {
					group.delete(&delete);
					all_closed &= group.is_empty();
				}
			}
		}
		all_closed
	}
	pub fn reconfigure(&self, data: &Data<Key>) {
		for key in self.keys.iter().copied() {
			if let Some(group) = data.get(key) {
				let group = group.borrow();
				(reconfigure_surfaces as fn(_, _, _, _)).call_method(&group, ());
			}
		}
	}
	pub fn present(&self, data: &Data<Key>) {
		for key in self.keys.iter().copied() {
			if let Some(group) = data.get(key) {
				let group = group.borrow();
				(present_surfaces as fn(_, _)).call_method(&group, ());
			}
		}
	}
}

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

#[derive(Debug)]
pub enum Event {
	NewWindow(Vec<WindowAttributes>, oneshot::Sender<Vec<Window>>),
	RecvExits(Vec<WindowId>, oneshot::Sender<Vec<Arc<AtomicBool>>>),
	Exit,
}

pub trait Game: 'static {
	fn new(proxy: AppProxy) -> Self;
	fn run(self);
}

pub struct InputReceiver {
	pub input: Input,
}

struct AppWindow {
	closed: Arc<AtomicBool>,
}

pub struct App<G: Game> {
	game: Option<std::thread::JoinHandle<()>>,
	proxy: EventLoopProxy<Event>,
	input: Input,
	windows: HashMap<WindowId, AppWindow>,
	_marker: PhantomData<G>,
}
impl<G: Game> ApplicationHandler<Event> for App<G> {
	fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		let proxy = AppProxy(self.proxy.clone());

		self.game = Some(std::thread::spawn(move || {
			let game = G::new(proxy);
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
		self.input.process_device_event(&device_id, event);
	}

	fn user_event(&mut self, event_loop: &ActiveEventLoop, event: Event) {
		match event {
			Event::NewWindow(attrs, sender) => {
				let windows: Vec<Window> = attrs
					.into_iter()
					.map(|attr| event_loop.create_window(attr).unwrap())
					.collect();
				for window in windows.iter() {
					self.windows.insert(
						window.id(),
						AppWindow {
							closed: Arc::new(AtomicBool::new(false)),
						},
					);
				}
				let _ = sender.send(windows);
			}
			Event::RecvExits(ids, sender) => {
				_ = sender.send(
					ids.into_iter()
						.map(|id| self.windows.get(&id).unwrap().closed.clone())
						.collect(),
				)
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
			input: Default::default(),
			windows: Default::default(),
			_marker: Default::default(),
		}
	}
}
