use slotmap::new_key_type;
use spin_sleep::sleep;
use ure::{
    app::{App, AppReceiver, Window, init_windows},
    data, element, extend,
    gpu::{
        Surface, begin_passes, init_encoders, init_surface_sizes, init_surface_textures,
        init_surfaces, present_surfaces, reconfigure_surfaces, submit_encoders,
        two::{Mesh2DHandle, QUAD},
    },
    store::{Data, Element},
};
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
    windows: Element,
    data: Data<GameKey>,
    test_visuals: GameKey,
    receiver: AppReceiver,
}

pub enum Event {
    Exit,
}

new_key_type! {
    pub struct GameKey;
}

impl ure::app::Game for Game {
    fn new(receiver: AppReceiver) -> Self {
        let windows = extend! {
            element!(Window, Surface, PhysicalSize<u32>),
            1,
            {
                windows: Window = { init_windows([WindowAttributes::default()], windows, &receiver); },
                surfaces: Surface = { init_surfaces(windows, surfaces) },
                sizes: PhysicalSize<u32> = { init_surface_sizes(windows, sizes) },
            }
        };
        let mut data = Data::<GameKey>::with_key();
        let test_visuals = data.insert(extend! {
            element!(Mesh2DHandle),
            1,
            {
                meshes: Mesh2DHandle = { for i in 0..meshes.len() { meshes[i].write(QUAD.load()); } },
            }
        });
        Game {
            windows,
            data,
            test_visuals,
            receiver,
        }
    }

    fn run(mut self) {
        let mut frame_start;
        let mut delta = std::time::Duration::ZERO;
        loop {
            frame_start = std::time::Instant::now();
            {
                data! {
                    self.windows,
                    {
                        windows: Window,
                        surfaces: Surface,
                        sizes: PhysicalSize<u32>,
                    } else {
                        panic!()
                    }
                }
                reconfigure_surfaces(windows, surfaces, sizes);
                let surface_textures = init_surface_textures(surfaces);
                {
                    let mut encoders = init_encoders(surfaces.len());
                    {
                        let mut passes =
                            begin_passes(&mut encoders, &surface_textures, wgpu::Color::BLACK);
                        for pass in passes.iter_mut() {}
                    }
                    submit_encoders(encoders);
                }
                present_surfaces(surface_textures);
            }
            sleep(FRAME_TIME.saturating_sub(frame_start.elapsed()));
            delta = frame_start.elapsed();
        }
    }
}
