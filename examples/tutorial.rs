use ure::{
    app::{App, Input, Window, init_windows},
    gpu::init_surfaces,
};

#[repr(usize)]
#[derive(Debug, Clone, Copy)]
pub enum Actions {
    Jump,
}

const FRAME_TIME: std::time::Duration = std::time::Duration::from_nanos(0_166_666_667);

fn main() {
    let mut app: App<Game> = App::default();
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.run_app(&mut app).unwrap();
}

struct Game {
    window_data: ure::app::Data,
    game_data: ure::game::Data,
    windows: usize,
    input: Input,
    test_visuals: usize,
    visuals_2d: ure::gpu::two::Visuals2D,
}
impl ure::app::Game for Game {
    fn new(event_loop: &winit::event_loop::ActiveEventLoop, input: Input) -> Self {
        let mut window_data = ure::app::Data::default();
        let windows = window_data.new_span(ure::app::SpanMask {
            window: true,
            surface: true,
            ..Default::default()
        });
        window_data.grow_span(windows, 1);
        {
            let span = window_data.extend_span(windows, 1);
            let window = span.window.unwrap();
            let surface = span.surface.unwrap();
            init_windows(window, event_loop);
            let window: &mut [Window] = unsafe { std::mem::transmute(window) };
            init_surfaces(window, surface);
        }
        let mut game_data = ure::game::Data::default();
        let test_visuals = game_data.new_span(ure::game::SpanMask {
            visual_2d: true,
            mesh: true,
            ..Default::default()
        });
        game_data.grow_span(test_visuals, 1);
        {
            let span = game_data.extend_span(test_visuals, 1);
            span.visual_2d
                .unwrap()
                .fill(std::mem::MaybeUninit::new(ure::gpu::two::Instance2D {
                    transform: glam::Affine2::IDENTITY.to_cols_array(),
                    color: ure::gpu::Color::WHITE,
                }));
            span.mesh
                .unwrap()
                .fill_with(|| std::mem::MaybeUninit::new(ure::gpu::two::QUAD.load()));
        }
        let visuals_2d =
            ure::gpu::two::Visuals2D::new(vec![test_visuals], game_data.visual_2d.elements.len());
        Game {
            window_data,
            game_data,
            windows,
            input,
            test_visuals,
            visuals_2d,
        }
    }

    fn run(self) {
        let mut frame_start;
        let mut delta = std::time::Duration::ZERO;
        loop {
            frame_start = std::time::Instant::now();
            {
                let span = self.window_data.get_span(self.windows);
                let surfaces = span.surface.unwrap();
                let surface_textures = ure::gpu::init_surface_textures(surfaces);
                {
                    let mut encoders = ure::gpu::init_encoders(surfaces.len());
                    {
                        let mut passes = ure::gpu::begin_passes(
                            &mut encoders,
                            &surface_textures,
                            wgpu::Color::BLACK,
                        );
                        for pass in passes.iter_mut() {
                            self.visuals_2d.render(&self.game_data, pass);
                        }
                    }
                    ure::gpu::submit_encoders(encoders);
                }
                ure::gpu::present_surfaces(surface_textures);
            }
            spin_sleep::sleep(FRAME_TIME.saturating_sub(frame_start.elapsed()));
            delta = frame_start.elapsed();
        }
    }
}
