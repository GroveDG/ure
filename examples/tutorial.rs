use ure::{
    app::{App, Input},
    gpu::two::Visuals2D,
};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};

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
    window_data: ure::app::Data,
    game_data: ure::game::Data,
    windows: usize,
    input: Input,
    test_visuals: usize,
    visuals_2d: ure::gpu::two::Visuals2D,
}

pub enum Event {
    Exit,
}

impl ure::app::Game for Game {
    type Event = Event;

    fn new(event_loop: &ActiveEventLoop, proxy: EventLoopProxy<Self::Event>, input: Input) -> Self {
        let mut window_data = ure::app::Data::default();
        let windows = window_data.init_span(ure::gpu::MASK, 1, |mut span| {
            ure::gpu::init_windows_and_surfaces(&mut span, event_loop)
        });
        window_data.get_span(windows).window.unwrap()[0].set_title("URE");
        let mut game_data = ure::game::Data::default();
        let test_visuals = game_data.init_span(Visuals2D::MASK, 1, |span| {
            span.visual_2d
                .unwrap()
                .fill(std::mem::MaybeUninit::new(ure::gpu::two::Instance2D {
                    transform: glam::Affine2::IDENTITY.to_cols_array(),
                    color: ure::gpu::Color::WHITE,
                }));
            span.mesh
                .unwrap()
                .fill_with(|| std::mem::MaybeUninit::new(ure::gpu::two::QUAD.load()));
        });
        let visuals_2d = Visuals2D::new(vec![test_visuals], game_data.visual_2d.elements.len());
        Game {
            window_data,
            game_data,
            windows,
            input,
            test_visuals,
            visuals_2d,
        }
    }

    fn run(mut self) {
        let mut frame_start;
        let mut delta = std::time::Duration::ZERO;
        loop {
            frame_start = std::time::Instant::now();
            {
                let span = self.window_data.get_mut_span(self.windows);
                let surfaces = span.surface.unwrap();
                ure::gpu::reconfigure_surfaces(
                    surfaces,
                    span.window.unwrap(),
                    span.window_size.unwrap(),
                );
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

    fn event(event_loop: &ActiveEventLoop, event: Self::Event) {
        match event {
            Event::Exit => event_loop.exit(),
        }
    }
}
