use std::cell::RefCell;

use slotmap::new_key_type;
use spin_sleep::sleep;
use ure::{
	app::{
		App, AppProxy, Proxy, SurfaceTextures, Surfaces, WindowExits, WindowSizes, Windows,
		close_windows, present_surfaces, reconfigure_surfaces,
	},
	gpu::GPU,
};
use ure_data::{
	containers::One,
	group::{Data, Group},
	method::MethodTrait,
};
use wgpu::CommandEncoderDescriptor;
use winit::event_loop::{EventLoop, EventLoopProxy};

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
	windows: GameKey,
}

new_key_type! {
	pub struct GameKey;
}

impl ure::app::Game for Game {
	fn new(app_proxy: AppProxy) -> Self {
		let mut data = Data::<GameKey>::with_key();
		let windows = data.insert({
			let mut group = Group::default();
			group
				.add_container::<Proxy>(One(app_proxy.clone()))
				.unwrap();
			group.add_component::<Windows>().unwrap();
			group.add_component::<WindowExits>().unwrap();
			group.add_component::<WindowSizes>().unwrap();
			group.add_component::<Surfaces>().unwrap();
			group.add_component::<SurfaceTextures>().unwrap();
			group.new(1);
			RefCell::new(group)
		});
		Game {
			app_proxy,
			windows,
			data,
		}
	}

	fn run(mut self) {
		let mut frame_start;
		let mut delta = std::time::Duration::ZERO;
		'game: loop {
			frame_start = std::time::Instant::now();

			{
				let mut windows = self.data.get(self.windows).unwrap().borrow_mut();
				let delete = (close_windows as fn(_, _) -> _)
					.call_method(&windows, ())
					.unwrap();
				for i in delete {
					windows.delete(i);
				}
				if windows.is_empty() {
					break 'game;
				}
			}

			// RENDERING
			{
				if let Some(windows) = self.data.get(self.windows) {
					let windows = windows.borrow_mut();
					(reconfigure_surfaces as fn(_, _, _, _)).call_method(&windows, ());

					let encoder = GPU
						.device
						.create_command_encoder(&CommandEncoderDescriptor::default());

					// RENDER PASSES

					GPU.queue.submit([encoder.finish()]);

					(present_surfaces as fn(_, _)).call_method(&windows, ());
				}
			}
			sleep(FRAME_TIME.saturating_sub(frame_start.elapsed()));
			delta = frame_start.elapsed();
		}
		self.app_proxy.exit();
	}
}
