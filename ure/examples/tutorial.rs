use std::cell::RefCell;

use slotmap::new_key_type;
use spin_sleep::sleep;
use ure::{
	app::{App, AppProxy, WindowSystem, Windows},
	gpu::GPU,
	two::Visuals2D,
};
use ure_data::group::{Data, Group};
use wgpu::CommandEncoderDescriptor;
use winit::{event_loop::EventLoop, window::WindowAttributes};

#[repr(usize)]
#[derive(Debug, Clone, Copy)]
pub enum Actions {
	Jump,
}

const FRAME_TIME: std::time::Duration = std::time::Duration::from_nanos(0_016_666_667);

fn main() {
	let event_loop = EventLoop::with_user_event().build().unwrap();
	let mut app: App<Game> = App::new(event_loop.create_proxy());
	event_loop.run_app(&mut app).unwrap();
}

struct Game {
	app_proxy: AppProxy,
	data: Data<GameKey>,
	window_system: WindowSystem<GameKey>,
	visuals_2d: Visuals2D<GameKey>,
}

new_key_type! {
	pub struct GameKey;
}

impl ure::app::Game for Game {
	fn new(app_proxy: AppProxy) -> Self {
		let mut data = Data::<GameKey>::with_key();

		let mut window_system = WindowSystem::new(app_proxy.clone());

		let windows = data.insert(RefCell::new(Group::default()));
		window_system.add(&data, windows);

		data.get(windows)
			.unwrap()
			.borrow_mut()
			.new(1)
			.with::<Windows>(vec![WindowAttributes::default().with_title("URE")])
			.done();
		window_system.inspect_capabilities(&data);

		let format = window_system.surface_format().unwrap();
		let mut visuals_2d = Visuals2D::new(format);

		let rects = data.insert(RefCell::new(Group::default()));
		visuals_2d.add(&data, rects);

		Game {
			app_proxy,
			window_system,
			visuals_2d,
			data,
		}
	}

	fn run(self) {
		let mut frame_start;
		let mut delta = std::time::Duration::ZERO;
		'game: loop {
			frame_start = std::time::Instant::now();

			// ================================ INPUT ================================
			if self.window_system.close(&self.data) {
				break 'game;
			}

			// ============================== RENDERING ==============================
			self.window_system.reconfigure(&self.data);
			let mut encoder = GPU
				.device
				.create_command_encoder(&CommandEncoderDescriptor::default());

			// RENDER PASSES
			{
				let pass = self.visuals_2d.begin_pass(&mut encoder, view);
				self.visuals_2d.render(&self.data, &mut pass);
			}

			GPU.queue.submit([encoder.finish()]);
			self.window_system.present(&self.data);

			// =============================== TIMING ================================
			sleep(FRAME_TIME.saturating_sub(frame_start.elapsed()));
			delta = frame_start.elapsed();
		}
		self.app_proxy.exit();
	}
}
