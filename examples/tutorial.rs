use glam::Affine2;
use slotmap::new_key_type;
use spin_sleep::sleep;
use ure::{
    app::{init_windows, App, AppReceiver, Window},
    data, element,
    gpu::{
        begin_passes, init_encoders, init_surface_sizes, init_surface_textures, init_surfaces, instancing::InstanceBuffer, present_surfaces, reconfigure_surfaces, rendering::Rendering, submit_encoders, two::{Instance2D, Mesh2DHandle, Visuals2D, DEFAULT_2D, QUAD}, Color, Surface
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
    visuals_2d: Visuals2D<GameKey>,
    receiver: AppReceiver,
}

new_key_type! {
    pub struct GameKey;
}

impl ure::app::Game for Game {
    fn new(receiver: AppReceiver) -> Self {
        let windows = element! {
            len: 1,
            {

            }
            {
                windows: many Window { init_windows([WindowAttributes::default()], windows, &receiver); },
                surfaces: many Surface { init_surfaces(windows, surfaces) },
                sizes: many PhysicalSize<u32> { init_surface_sizes(windows, sizes) },
            }
        };
        let mut data = Data::<GameKey>::with_key();
        let test_visuals = data.insert(element! {
            len: 1,
            {
                Rendering: DEFAULT_2D,
            }
            {
                transforms: many Affine2 { for i in 0..len { transforms[i].write(Default::default()); } },
                color: many Color { for i in 0..len { color[i].write(Color::WHITE); } },
                instances: one InstanceBuffer<Instance2D> { instances.extend(len, |instances| {
                    for i in 0..len {
                        let transform = transforms[i];
                        let matrix = transform.matrix2;
                        let translation = transform.translation;
                        let color = color[i];
                        instances[i].write(Instance2D {
                            col_0: matrix.x_axis,
                            col_1: matrix.y_axis,
                            position: translation,
                            color: color
                        });
                    }
                }) },
                meshes: many Mesh2DHandle { for i in 0..len { meshes[i].write(QUAD.load()); } },
            }
        });
        let mut visuals_2d = Visuals2D::new();
        visuals_2d.add(test_visuals);
        Game {
            windows,
            data,
            test_visuals,
            visuals_2d,
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
                        for pass in passes.iter_mut() {
                            self.visuals_2d.render(&self.data, pass);
                        }
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
