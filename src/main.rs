use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use sdl2::{event::Event, pixels::Color};
use sys::{
    UIDs,
    sdl::{Events, Windows},
    tf::Space2D,
    tree::Tree,
};

mod sys;

const FRAME_PERIOD: Duration = Duration::new(0, 0_016_666_667);

fn main() {
    // Initialize UID system
    let mut uids: UIDs = UIDs::new().expect("UID RNG failed to intitialize");

    // Initialize SDL
    let sdl = sdl2::init().unwrap();
    let mut windows = Windows::new(&sdl).unwrap();
    let mut events = Events::new(&sdl).unwrap();

    // Initialize game systems
    let mut tree: Tree = Tree::new(uids.new_uid());
    let mut transform: Space2D = Default::default();

    // Create window
    windows.new_window(&mut uids, "Window", 640, 480);

    // Store previous frame start for timing
    let mut last_start = Instant::now();
    'game: loop {
        // Calculate frame timing
        let start = Instant::now();
        let delta = start - last_start;

        // Poll events
        for event in events.poll() {
            match event {
                Event::Quit { .. } => break 'game,
                _ => {}
            }
        }

        // Draw frame
        // windows.request_redraws();

        // Calculate CPU time spent
        let end = Instant::now();
        let cpu_time = end - start;
        
        // Minimize updates
        let remaining = FRAME_PERIOD.saturating_sub(cpu_time);
        sleep(remaining); // Sleep slightly overshoots frame period

        last_start = start;
        // println!("{:?}", cpu_time);
    }
}
