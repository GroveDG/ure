use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::{sleep, spawn},
    time::{Duration, Instant},
};

use app::App;
use sys::{UIDs, tf::Space2D, tree::Tree};
use winit::{event_loop::EventLoop, window::{Window, WindowId}};

mod app;
mod sys;

// const FRAME_TIME: Duration = Duration::new(0, 0_016_666_667);
const FRAME_TIME: Duration = Duration::new(0, 0_166_666_667);

fn main() {
    let event_loop = EventLoop::builder().build().unwrap();
    let windows = Arc::new(Mutex::new(HashMap::new()));
    let mut app = App::new(windows.clone());
    spawn(|| game(windows));
    event_loop.run_app(&mut app);
}

fn game(windows: Arc<Mutex<HashMap<WindowId, Window>>>) {
    let mut uids: UIDs = UIDs::new().expect("UID RNG failed to intitialize");
    let mut tree: Tree = Tree::new(uids.new_uid());
    let mut transform: Space2D = Default::default();

    let mut prev_frame_start = Instant::now();
    loop {
        let start = Instant::now();
        let delta = start - prev_frame_start;

        let end = Instant::now();
        let cpu_time = end - start;
        let remaining = FRAME_TIME.saturating_sub(cpu_time);
        sleep(remaining);

        prev_frame_start = start;
        println!("{:?}", delta);

        // Redraw all windows
        for window in windows.lock().unwrap().values() {
            window.request_redraw();
        }
    }
}
