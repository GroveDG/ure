use std::sync::{mpsc::{Receiver, Sender}, Arc};

use winit::window::Window;

use crate::sys::gpu::DrawCommand;

pub struct GameBank {
    pub windows: Receiver<Arc<Window>>,
    pub draw: Sender<DrawCommand>,
}

pub struct AppBank {
    pub windows: Sender<Arc<Window>>,
    pub draw: Receiver<DrawCommand>,
}