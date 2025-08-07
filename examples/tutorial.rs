use ure::{
    app::{App, Input, Window, init_windows},
    gpu::{Gpu, init_surfaces},
};

#[repr(usize)]
#[derive(Debug, Clone, Copy)]
pub enum Actions {
    Jump,
}

fn main() {
    let mut app: App<Game> = App::default();
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.run_app(&mut app).unwrap();
}

struct Game {
    gpu_data: ure::gpu::Data,
    // game_data: ure::game::Data,
    gpu: Gpu,
    windows: usize,
    input: Input,
}
impl ure::app::Game for Game {
    fn new(event_loop: &winit::event_loop::ActiveEventLoop, input: Input) -> Self {
        let mut gpu_data = ure::gpu::Data::default();
        let gpu = futures::executor::block_on(Gpu::new());
        let windows = gpu_data.new_span(ure::gpu::SpanMask {
            window: true,
            surface: true,
            ..Default::default()
        });
        gpu_data.grow_span(windows, 1);
        {
            let span = gpu_data.extend_span(windows, 1);
            let window = span.window.unwrap();
            let surface = span.surface.unwrap();
            init_windows(window, event_loop);
            let window: &mut [Window] = unsafe { std::mem::transmute(window) };
            init_surfaces(window, surface, &gpu);
        }
        Game {
            gpu_data,
            gpu,
            windows,
            input,
        }
    }

    fn run(self) {
        loop {
            {
                let span = self.gpu_data.get_span(self.windows);
                let surfaces = span.surface.unwrap();
                let surface_textures = ure::gpu::init_surface_textures(surfaces);
                {
                    let mut encoders = ure::gpu::init_encoders(surfaces.len(), &self.gpu);
                    ure::gpu::clear_surfaces(&mut encoders, &surface_textures, wgpu::Color::BLACK);
                    ure::gpu::submit_encoders(encoders, &self.gpu);
                }
                ure::gpu::present_surfaces(surface_textures);
            }
        }
    }
}
