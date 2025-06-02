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
    sync::{mpsc::channel, Arc},
    thread::{self}, time::{Duration, Instant},
};

use parking_lot::Mutex;
use spin_sleep::sleep;
use winit::{error::EventLoopError, event_loop::EventLoop};

use crate::app::{App, input::Input};

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
    let (window_sndr, window_recv) = channel();
    let input = Arc::new(Mutex::new(Input::default()));

    // [VITAL] Initialize Thread Communication
    let (render_sndr, render_recv) = channel();
    let (parker, parked) = channel();

    // [VITAL] Initialize Render Thread
    let render = {
        let parker = parker.clone();
        thread::Builder::new()
            .name("render".to_string())
            .spawn(move || {
                render::render(render_recv, &parker);
                let _ = parker.send(());
                println!("RENDER THREAD QUIT {:?}", Instant::now());
            })
            .unwrap()
    };

    // [VITAL] Initialize Game Thread
    let game = {
        let event_proxy = event_loop.create_proxy();
        let input = Arc::clone(&input);
        let render_sndr = render_sndr.clone();

        thread::Builder::new()
            .name("game".to_string())
            .spawn(move || {
                game::game(event_proxy, window_recv, input, render_sndr, &parker);
                let _ = parker.send(());
                println!("GAME THREAD QUIT {:?}", Instant::now());
            })
            .unwrap()
    };

    // [VITAL] Time Frames in Game and Render Threads
    let timing = {
        let event_proxy = event_loop.create_proxy();

        thread::spawn(move || timing::timing(parked, game, render, render_sndr, event_proxy))
    };

    // [VITAL] Run App
    let mut app = App {
        window_ids: Default::default(),
        window_sndr,
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

    println!("APP THREAD QUIT {:?}", Instant::now());

    let _ = timing.join();

    println!("TIMING THREAD JOIN {:?}", Instant::now());
}
