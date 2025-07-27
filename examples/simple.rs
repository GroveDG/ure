use ure::{components, compose, comprise, entity};
use winit::event_loop::ActiveEventLoop;

components! {
    Window: winit::window::Window,
    Surface: wgpu::Surface<'static>,
    Transform: ure::game::tf::Transform2D,
}

entity!(Game);
impl Game {
    compose!(Window, Surface);
    comprise!{
        transform_1: Transform,
    }

    pub fn init(event_loop: &ActiveEventLoop) -> Data {
        let mut data = Data::init(Game::SIZE);
        data.init_window(event_loop.create_window(Default::default()).unwrap());
        let game = Game(Offset::ZERO);
        data
    }
}

fn main() {}
