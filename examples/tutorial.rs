use std::cell::RefCell;

use glam::Affine2;
use slotmap::new_key_type;
use spin_sleep::sleep;
use ure::{
    app::{
        reconfigure_surfaces, App, AppProxy, WindowReceiver
    },
    gpu::GPU,
};
use ure_data::{ComponentStruct, Data, Group};
use wgpu::CommandEncoderDescriptor;
use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::WindowAttributes};

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
    // test_visuals: GameKey,
    // visuals_2d: Visuals2D<GameKey>,
}

new_key_type! {
    pub struct GameKey;
}

impl ure::app::Game for Game {
    fn new(app: AppProxy, windows: WindowReceiver) -> Self {
        let mut data = Data::<GameKey>::with_key();
        let windows = data.insert({
            let mut group = Group::default();
            group.add_component(ComponentStruct::new(

            ));
            group.add_component(windows);
            group.add_component(WindowIds);
            group.add_component(Surfaces);
            group.add_component(WindowSizes);
            group.new(1);
            RefCell::new(group)
        });
        // let mut visuals_2d = Visuals2D::new();
        // visuals_2d.add(test_visuals);
        Game {
            app,
            windows,
            data,
            // test_visuals,
            // visuals_2d,
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
                let surface_textures = {
                    let mut windows = self.data.get(self.windows).unwrap().borrow_mut();
                    let (windows, sizes, surfaces) = windows
                        .get_components_mut::<(Windows, WindowSizes, Surfaces)>()
                        .unwrap();
                    reconfigure_surfaces(windows, sizes, surfaces)
                };
                let encoder = GPU
                    .device
                    .create_command_encoder(&CommandEncoderDescriptor::default());

                // RENDER PASSES

                GPU.queue.submit([encoder.finish()]);
                for surface_teture in surface_textures {
                    surface_teture.present();
                }
            }
            sleep(FRAME_TIME.saturating_sub(frame_start.elapsed()));
            delta = frame_start.elapsed();
        }
        self.app.exit();
    }
}
