use std::{
    fs::File,
    thread,
    time::{Duration, Instant},
};

use app::{App, UserEvent};
use cgmath::SquareMatrix;
use ron::ser::PrettyConfig;
use sys::{
    UIDs,
    tf::{Matrix2D, Space2D},
    tree::Tree,
    window::Windows,
};
use winit::{
    event_loop::{EventLoop, EventLoopProxy},
    window::{Window, WindowAttributes},
};

mod app;
mod call;
mod sys;

/*
    Code is flagged by importance to inform
    you about deleting or modifying it.

    [CORE] What makes URE, URE.
    [VITAL] Essential to most games.
    [USEFUL] Commonly used, but may vary.
    [EXAMPLE] For instructional purposes only.

    Other flags include...
    [TEST] Test features (remove for release).
    [TODO] Mark where work is needed.
*/

fn main() {
    // [VITAL] Initialize App
    let event_loop = EventLoop::with_user_event().build().unwrap();

    // [VITAL] Initialize App Systems
    let (window_sender, windows) = Windows::new();

    let game_thread = {
        let event_proxy = event_loop.create_proxy();
        thread::spawn(|| game(event_proxy, windows))
    };
    event_loop
        .run_app(&mut App {
            game: game_thread,
            windows: window_sender,
        })
        .unwrap();
}

// [VITAL] Frame Period (Inverse of FPS)
const FRAME_PERIOD: Duration = Duration::new(0, 0_016_666_667);

fn game(event_proxy: EventLoopProxy<UserEvent>, mut windows: Windows) {
    // [CORE] Initialize UID System
    let mut uids = UIDs::new().unwrap();

    // [USEFUL] Initialize Game Systems
    let mut tree = Tree::default();
    let mut space = Space2D::default();

    // [EXAMPLE] Init Root
    let root = uids.add();
    tree.insert(root, None);
    tree.insert(uids.add(), Some(root));
    tree.insert(uids.add(), Some(root));
    tree.insert(uids.add(), None);
    space.insert(root, Matrix2D::identity());
    windows.await_new(root, WindowAttributes::default(), &event_proxy);

    // [TEST] Write Tree
    {
        let file = File::options()
            .create(true)
            .write(true)
            .open("tree.ure")
            .unwrap();
        ron::Options::default()
            .to_io_writer_pretty(
                file,
                &tree,
                PrettyConfig::default().indentor("\t").struct_names(true),
            )
            .unwrap();
    }

    // [VITAL] Game Loop
    let mut last_start = Instant::now(); // Last frame start
    'game: loop {
        // [VITAL] Time Frame
        let start = Instant::now();
        let delta = start - last_start;

        // [USEFUL] GUI Layout & Rendering
        // {
        //     layout.run();
        //     if let Some(window) = windows.get_mut(&root) {
        //         for node in layout.render_order() {
        //             box_renderer.render(node, window, &layout);
        //         }
        //     }
        // }

        // [VITAL] Receive New Windows, Quit if Empty
        {
            windows.poll();
            if windows.is_empty() {
                break 'game;
            }
        }

        // [VITAL] Time Frame
        let end = Instant::now();
        let cpu_time = end - start;
        println!("{:?}", cpu_time);

        // [VITAL] Delay Update
        let remaining = FRAME_PERIOD.saturating_sub(cpu_time);
        thread::sleep(remaining); // Sleep slightly overshoots frame period

        last_start = start;
    }
}
