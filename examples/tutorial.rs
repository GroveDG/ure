use std::cell::RefCell;

use slotmap::new_key_type;
use spin_sleep::sleep;
use ure::{
    app::{
        App, AppProxy, PRESENT_SURFACES, RECONFIGURE_SURFACES, SURFACE_TEXTURES, SURFACES,
        WINDOW_EXITS, WINDOW_IDS, WINDOW_SIZES, WINDOWS, WindowReceiver,
    },
    gpu::GPU,
};
use ure_data::{ComponentContainer, Data, Group, One};
use wgpu::CommandEncoderDescriptor;
use winit::event_loop::EventLoop;

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
    app: AppProxy,
    data: Data<GameKey>,
    windows: GameKey,
}

new_key_type! {
    pub struct GameKey;
}

impl ure::app::Game for Game {
    fn new(app: AppProxy, windows: ComponentContainer<One<WindowReceiver>>) -> Self {
        let mut data = Data::<GameKey>::with_key();
        let windows = data.insert({
            let mut group = Group::default();
            group.add_component(windows);
            group.add_component(WINDOWS.new());
            group.add_component(WINDOW_EXITS.new());
            group.add_component(WINDOW_IDS.new());
            group.add_component(WINDOW_SIZES.new());
            group.add_component(SURFACES.new());
            group.add_component(SURFACE_TEXTURES.new());
            group.new(1);
            RefCell::new(group)
        });
        Game {
            app,
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
                let close = windows.get_components::<WindowClose>().unwrap();
                let mut delete = Vec::new();
                for (i, c) in close.iter().enumerate() {
                    if c.load(std::sync::atomic::Ordering::Relaxed) {
                        delete.push(i);
                    }
                }
                for i in delete {
                    windows.delete(i..i + 1);
                }
                if windows.is_empty() {
                    break 'game;
                }
            }

            // RENDERING
            {
                if let Some(windows) = self.data.get(self.windows) {
                    let windows = windows.get_mut();
                    windows.call_method(RECONFIGURE_SURFACES, ());

                    let encoder = GPU
                        .device
                        .create_command_encoder(&CommandEncoderDescriptor::default());

                    // RENDER PASSES

                    GPU.queue.submit([encoder.finish()]);

                    windows.call_method(PRESENT_SURFACES, ());
                }
            }
            sleep(FRAME_TIME.saturating_sub(frame_start.elapsed()));
            delta = frame_start.elapsed();
        }
        self.app.exit();
    }
}
