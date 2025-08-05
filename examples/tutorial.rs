use ure::{
    app::{App, Window, init_windows},
    data::{Data, SpanMask},
    gpu::{Gpu, init_surfaces},
};

fn main() {
    let mut app: App<Game> = App::default();
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.run_app(&mut app).unwrap();
}

struct Game {
    data: Data,
    gpu: Gpu,
    windows: usize,
}
impl ure::app::Game for Game {
    fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        let mut data = Data::default();
        let gpu = futures::executor::block_on(Gpu::new());
        let windows = data.new_span(SpanMask {
            window: true,
            surface: true,
            ..Default::default()
        });
        data.grow_span(windows, 1);
        {
            let span = data.extend_span(windows, 1);
            let window = span.window.unwrap();
            let surface = span.surface.unwrap();
            init_windows(window, event_loop);
            let window: &mut [Window] = unsafe { std::mem::transmute(window) };
            init_surfaces(window, surface, &gpu);
        }
        Game { data, gpu, windows }
    }

    fn run(self) {
        loop {
            {
                let span = self.data.get_span(self.windows);
                let surfaces = span.surface.unwrap();
                let mut encoders = ure::gpu::init_encoders(surfaces.len(), &self.gpu);
                let surface_textures = ure::gpu::init_surface_textures(surfaces);
                ure::gpu::clear_surfaces(&mut encoders, &surface_textures, wgpu::Color::BLACK);
                ure::gpu::submit_encoders(encoders, &self.gpu);
                ure::gpu::present_surfaces(surface_textures);
            }
        }
    }
}
