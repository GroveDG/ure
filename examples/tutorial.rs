use ure::{
    app::{App, init_windows},
    data::{Data, Span},
    extend_span, get_span,
    gpu::{Gpu, init_surfaces},
    new_span,
};

fn main() {
    let mut app: App<Game> = App::default();
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.run_app(&mut app).unwrap();
}

struct Game {
    data: Data,
    gpu: Gpu,
    windows: Span,
}
impl ure::app::Game for Game {
    fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        let mut data = Data::default();
        let gpu = futures::executor::block_on(Gpu::new());
        let mut windows = new_span!(data, 1, window, surface);
        {
            extend_span!(data, windows, 1, window);
            init_windows(window, event_loop);
            get_span!(data, windows, window);
            extend_span!(data, windows, 1, surface);
            init_surfaces(window, surface, &gpu);
        }
        Game {
            data: Data::default(),
            gpu,
            windows,
        }
    }

    fn run(self) {
        todo!()
    }
}
