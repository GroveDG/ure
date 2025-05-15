use std::{
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, channel},
    },
    thread,
    time::{Duration, Instant},
};

use app::{App, UserEvent};
use cgmath::SquareMatrix;
use sys::{
    UID, UIDs,
    gpu::GPU,
    gui::Layout,
    input::Input,
    tf::{Matrix2D, Space2D},
    tree::Tree,
    window::Windows,
};
use winit::{
    error::EventLoopError,
    event_loop::{EventLoop, EventLoopProxy},
    window::{Window, WindowAttributes},
};

mod app;
mod sys;
mod bank;

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
    let (window_sender, window_receiver) = channel();
    let input = Arc::new(Mutex::new(Input::default()));
    let gpu = futures::executor::block_on(GPU::new());

    // [VITAL] Initialize Game Loop
    let game_thread = {
        let event_proxy = event_loop.create_proxy();
        let input = Arc::clone(&input);
        thread::spawn(|| game(event_proxy, window_receiver, input))
    };

    // [VITAL] Run App
    let mut app = App {
        windows: window_sender,
        window_ids: Default::default(),
        input,
        surfaces: Default::default(),
        gpu
    };
    if let Err(e) = event_loop.run_app(&mut app) {
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

    // [VITAL] Cleanup

    // Inform game thread of close.
    app.input.lock().unwrap().close = true;
    // Wait for game to stop.
    let mut i = 0;
    while !game_thread.is_finished() && i < 10 {
        thread::sleep(FRAME_PERIOD);
        i += 1;
    }
    // Prevent detatched thread.
    // (See std::thread::Thread)
    let _ = game_thread.join(); // Ignore panic.
}

// [VITAL] Frame Period (Inverse of FPS)
const FRAME_PERIOD: Duration = Duration::new(0, 0_016_666_667);

fn game(
    event_proxy: EventLoopProxy<UserEvent>,
    window_receiver: Receiver<(UID, Arc<Window>)>,
    input: Arc<Mutex<Input>>,
) {
    // [CORE] Initialize UID System
    let mut uids = UIDs::new();

    // [USEFUL] Initialize Graphics Systems
    let mut windows = Windows::default();

    // [USEFUL] Initialize UI Systems
    let mut layout = Layout::default();

    // [USEFUL] Initialize Game Systems
    let mut tree = Tree::default();
    let mut space = Space2D::default();

    // [EXAMPLE] Init Root
    let root = uids.add();
    tree.insert(root, None);
    space.insert(root, Matrix2D::identity());
    if Windows::request_new(root, WindowAttributes::default(), &event_proxy).is_err() {
        return;
    }

    // [VITAL] Game Loop
    let mut last_start = Instant::now(); // Last frame start
    'game: loop {
        // [VITAL] Time Frame
        let start = Instant::now();
        let delta = start - last_start;

        // [VITAL] Acquire Input State
        let input_state = {
            let Ok(input) = input.lock() else {
                break 'game;
            };
            input.clone()
        };
        // [VITAL] Close Game If Requested
        if input_state.close {
            break 'game;
        }

        // [VITAL] Receive New Windows
        {
            for (uid, window) in window_receiver.try_iter() {
                let window = window;
                windows.insert(uid, window);
            }
        }

        // [USEFUL] GUI Layout
        #[cfg(feature = "GUI")]
        {
            layout.run();
        }

        // ========================================================
        // END OF FRAME
        // ========================================================

        // [VITAL] Request Redraws
        for (_, window) in windows.iter() {
            window.request_redraw();
        }

        // [VITAL] Time Frame
        let end = Instant::now();
        let cpu_time = end - start;
        println!("CPU {:?}", cpu_time);

        // [VITAL] Delay Update
        let remaining = FRAME_PERIOD.saturating_sub(cpu_time);
        thread::sleep(remaining); // Sleep slightly overshoots frame period

        last_start = start;
    }
}
