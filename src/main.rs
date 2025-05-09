use std::{
    fs::File,
    io::BufWriter,
    sync::{Arc, Mutex},
    thread::sleep,
    time::{Duration, Instant},
};

use call::Call;
use cgmath::SquareMatrix;
use ron::ser::{PrettyConfig, to_string_pretty};
use sdl2::event::Event;
use serde::Serialize;
use sys::{
    UID, UIDs,
    sdl::{Events, Windows},
    tf::{Matrix2D, Space2D},
    tree::Tree,
};

mod call;
mod sys;

const FRAME_PERIOD: Duration = Duration::new(0, 0_016_666_667);

fn main() {
    // Initialize UID system
    let mut uids = Arc::new(Mutex::new(UIDs::new().unwrap()));

    // Initialize SDL systems
    let sdl = sdl2::init().unwrap();
    let windows = Arc::new(Mutex::new(Windows::new(&sdl).unwrap()));
    let mut events = Events::new(&sdl).unwrap();

    // Initialize game systems
    let root = uids.lock().unwrap().add();
    let tree = Arc::new(Mutex::new(Tree::default()));
    let space = Arc::new(Mutex::new(Space2D::default()));

    // Initialize mass calls
    // If you call these functions while one of them is locked,
    // the application will become deadlocked and freeze.
    let mut delete = {
        let uids = Arc::clone(&uids);
        let tree = Arc::clone(&tree);
        let space = Arc::clone(&space);
        let windows = Arc::clone(&windows);
        Call::new(Box::new(move |batch: &mut Vec<UID>| {
            // Lock before call for performance.
            let mut uids = uids.lock().unwrap();
            let mut tree = tree.lock().unwrap();
            let mut space = space.lock().unwrap();
            let mut windows = windows.lock().unwrap();
            // Call
            for args in batch.drain(..) {
                uids.delete(&args);
                tree.delete(&args); // Let tree add children to batch
                space.delete(&args);
                windows.delete(&args);
            }
        }))
    };

    // Init root
    tree.lock().unwrap().insert(root, None);
    tree.lock()
        .unwrap()
        .insert(uids.lock().unwrap().add(), Some(root));
    tree.lock()
        .unwrap()
        .insert(uids.lock().unwrap().add(), Some(root));
    tree.lock()
        .unwrap()
        .insert(uids.lock().unwrap().add(), None);
    space.lock().unwrap().insert(root, Matrix2D::identity());
    windows.lock().unwrap().insert(root, "Window", 640, 480);

    {
        let file = File::options()
            .create(true)
            .write(true)
            .open("tree.ure")
            .unwrap();
        ron::Options::default().to_io_writer_pretty(
            file,
            tree.as_ref(),
            PrettyConfig::default().indentor("\t").struct_names(true),
        ).unwrap();
    }

    // Store the start time of the previous frame
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

        //

        // Windowing system
        {
            let mut windows = windows.lock().unwrap();
            // If no windows, quit.
            if windows.is_empty() {
                break 'game;
            }
            // Draw frame
            windows.clear();
            windows.present();
        }

        // Calculate CPU time spent
        let end = Instant::now();
        let cpu_time = end - start;

        // Regulate updates
        let remaining = FRAME_PERIOD.saturating_sub(cpu_time);
        sleep(remaining); // Sleep slightly overshoots frame period

        last_start = start;
    }
}
