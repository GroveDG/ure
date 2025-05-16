use std::{
    sync::{Arc, Barrier, Condvar, Mutex, RwLock},
    thread,
};

use app::App;
use render::render;
use sys::{gpu::GPU, input::Input, window::Windows};
use winit::{error::EventLoopError, event_loop::EventLoop};

mod app;
mod game;
mod render;
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

    // [VITAL] Initialize Shared Systems
    let windows = Arc::new(RwLock::new(Windows::default()));
    let input = Arc::new(Mutex::new(Input::default()));
    let gpu = Arc::new(futures::executor::block_on(GPU::new()));

    // [VITAL] Initialize Render Thread
    let frame_barrier = Arc::new(Barrier::new(2));
    let render_thread = {
        let gpu = gpu.clone();
        let windows = windows.clone();
        let frame_barrier = frame_barrier.clone();
        thread::spawn(|| render(frame_barrier, gpu, windows))
    };

    // [VITAL] Initialize Game Thread
    let game_thread = {
        let event_proxy = event_loop.create_proxy();
        let input = Arc::clone(&input);
        let windows = windows.clone();
        thread::spawn(|| game::game(event_proxy, windows, input, frame_barrier))
    };

    // [VITAL] Run App
    let mut app = App {
        gpu,
        windows,
        input,
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
    // Prevent detatched thread.
    // (See std::thread::Thread)
    let _ = game_thread.join(); // Ignore panic.
    let _ = render_thread.join(); // Ignore panic.
}
