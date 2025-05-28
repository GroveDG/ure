//! **CORE** Code in URE is flagged to give information
//! to the developer.
//! ***
//! Code is flagged by importance:
//! - `[CORE] What makes URE, URE.`
//! - `[VITAL] Essential to most games.`
//! - `[USEFUL] Commonly used, but may vary.`
//! - `[TRIVIAL] Minor expected features.`
//! - `[EXAMPLE] For instructional purposes only.`
//!
//! Brackets may be replaced by bolding in docs.
//! ***
//! Docs might have these sections:
//!
//! WHY
//! -------------------------------
//! Explains why this must be done.
//!
//! Other flags include:
//! - __TEST__ Test features (remove for release).
//! - __TODO__ Mark where work is needed.

use std::{
    sync::{Arc, atomic::AtomicBool, mpsc::channel},
    thread::{self},
};

use parking_lot::{Mutex, RwLock};
use winit::{error::EventLoopError, event_loop::EventLoop};

use crate::render::RenderBuffer;
use crate::app::{App, input::Input, window::Windows};

mod app;
mod game;
mod render;
mod sys;
mod timing;



fn main() {
    // [VITAL] Initialize App
    let event_loop = EventLoop::with_user_event()
        .build()
        .expect("EventLoop building failed. See winit::event_loop::EventLoopBuilder::build");

    // [VITAL] Initialize Shared Systems
    let windows = Arc::new(RwLock::new(Windows::default()));
    let input = Arc::new(Mutex::new(Input::default()));

    // [VITAL] Initialize Thread Communication
    let draw_buffer = Arc::new(Mutex::new(RenderBuffer::default()));
    let draw_render = Arc::new(Mutex::new(RenderBuffer::default()));
    let quit = Arc::new(AtomicBool::new(false));
    let (parker, parked) = channel();

    // [VITAL] Initialize Render Thread
    let render = {
        let windows = windows.clone();
        let draw = draw_render.clone();
        let quit = quit.clone();
        let parker = parker.clone();
        thread::spawn(|| render::render(windows, draw, quit, parker))
    };

    // [VITAL] Initialize Game Thread
    let game = {
        let event_proxy = event_loop.create_proxy();
        let input = Arc::clone(&input);
        let windows = windows.clone();
        let draw = draw_buffer.clone();
        let quit = quit.clone();
        thread::spawn(|| game::game(event_proxy, windows, input, draw, quit, parker))
    };

    // [VITAL] Time Frames in Game and Render Threads
    let timing = {
        let event_proxy = event_loop.create_proxy();
        let quit = quit.clone();

        thread::spawn(move || {
            timing::timing(
                parked,
                game,
                render,
                quit,
                event_proxy,
                draw_buffer,
                draw_render,
            )
        })
    };

    // [VITAL] Run App
    let mut app = App {
        windows,
        input,
    };
    if let Err(e) = event_loop.run_app(&mut app) {
        // [TRIVIAL] Expose Errors
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

    // [VITAL] Cleanup Threads
    // Request quit.
    quit.store(true, std::sync::atomic::Ordering::Relaxed);
    // Resume timing thread if waiting.
    timing.thread().unpark();

    let _ = timing.join();
}
