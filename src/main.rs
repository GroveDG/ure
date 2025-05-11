use std::{
    fs::File,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use app::{App, UserEvent};
use cgmath::SquareMatrix;
use ron::ser::PrettyConfig;
use sys::{
    UIDs,
    input::Input,
    tf::{Matrix2D, Space2D},
    tree::Tree,
    window::Windows,
};
use winit::{
    error::EventLoopError,
    event_loop::{EventLoop, EventLoopProxy},
    window::WindowAttributes,
};

mod app;
mod sys;

/// **CORE** Code in URE is flagged to give information
/// to the developer.
/// ***
/// Code is flagged by importance:
/// - `[CORE] What makes URE, URE.`
/// - `[VITAL] Essential to most games.`
/// - `[USEFUL] Commonly used, but may vary.`
/// - `[EXAMPLE] For instructional purposes only.`
///
/// Brackets may be replaced by bolding in docs.
/// ***
/// Docs might have these sections:
///
/// WHY
/// -------------------------------
/// Explains why this must be done.
///
/// Other flags include:
/// - __TEST__ Test features (remove for release).
/// - __TODO__ Mark where work is needed.

fn main() {
    // [VITAL] Initialize App
    let event_loop = EventLoop::with_user_event()
        .build()
        .expect("EventLoop building failed. See winit::event_loop::EventLoopBuilder::build");

    // [VITAL] Initialize App Systems
    let (window_sender, windows) = Windows::new();
    let input = Arc::new(Mutex::new(Input::default()));

    // [VITAL] Initialize Game Loop
    let game_thread = {
        let event_proxy = event_loop.create_proxy();
        let input = Arc::clone(&input);
        thread::spawn(|| game(event_proxy, windows, input))
    };

    // [VITAL] Run App
    if let Err(e) = event_loop.run_app(&mut App {
        game: Some(game_thread),
        windows: window_sender,
        input,
    }) {
        match e {
            EventLoopError::NotSupported(_) => {
                println!("Operation unsupported (unspecified by winit)");
            }
            EventLoopError::Os(os_error) => {
                println!("{}", os_error)
            }
            EventLoopError::RecreationAttempt => {
                println!("Cannot recreate EventLoop")
            }
            EventLoopError::ExitFailure(error_code) => {
                println!("Exit with error code: {}", error_code)
            }
        }
    }
}

// [VITAL] Frame Period (Inverse of FPS)
const FRAME_PERIOD: Duration = Duration::new(0, 0_016_666_667);

fn game(
    event_proxy: EventLoopProxy<UserEvent>,
    mut windows: Windows,
    mut input: Arc<Mutex<Input>>,
) {
    // [CORE] Initialize UID System
    let mut uids = UIDs::new();

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

        {
            let Ok(input) = input.lock() else {
                break 'game;
            };
            if input.close {
                break 'game;
            }
        }

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
