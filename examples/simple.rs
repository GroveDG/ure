use std::sync::Arc;

use ure::{components, compose, comprise, entity};
use winit::event_loop::ActiveEventLoop;

components! {
    Window: Arc<winit::window::Window>,
    Surface: wgpu::Surface<'static>,
    VisualTransform: ure::game::tf::Transform2D,
    VisualColor: color::Rgba8,
    VisualMesh: wgpu::Buffer,
}

entity!(WindowSurface);
impl WindowSurface {
    compose!(Window, Surface);
    comprise!{}
}

entity!(VisualInstance);
impl VisualInstance {
    compose!(VisualTransform, VisualColor, VisualMesh);
    comprise!{}
}

entity!(Game);
impl Game {
    compose!(WindowSurface);
    comprise!{
        transform_1: VisualTransform,
    }

    pub fn init(event_loop: &ActiveEventLoop) -> Data {
        let mut data = Data::init(Game::SIZE);
        // data.init_window(event_loop.create_window(Default::default()).unwrap());
        let game = Game(Offset::ZERO);
        data
    }
}

fn main() {}
