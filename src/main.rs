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
    ops::DerefMut,
    sync::{Arc, atomic::AtomicBool, mpsc::channel},
    thread::{self, park},
    time::Duration,
};

use parking_lot::{Condvar, Mutex, RwLock};

use app::App;
use render::DrawBuffer;
use spin_sleep::SpinSleeper;
use sys::{gpu::GPU, input::Input, window::Windows};
use winit::{error::EventLoopError, event_loop::EventLoop};

mod app;
mod game;
mod render;
mod sys;

fn main() {
    // [VITAL] Initialize App
    let event_loop = EventLoop::with_user_event()
        .build()
        .expect("EventLoop building failed. See winit::event_loop::EventLoopBuilder::build");

    // [VITAL] Initialize Shared Systems
    let windows = Arc::new(RwLock::new(Windows::default()));
    let input = Arc::new(Mutex::new(Input::default()));
    let gpu = Arc::new(futures::executor::block_on(GPU::new()));

    // [VITAL] Initialize Thread Communication
    let draw_buffer = Arc::new(Mutex::new(DrawBuffer::default()));
    let draw_render = Arc::new(Mutex::new(DrawBuffer::default()));
    let quit = Arc::new(AtomicBool::new(false));
    let (parker, parked) = channel();

    // [VITAL] Initialize Render Thread
    let render = {
        let gpu = gpu.clone();
        let windows = windows.clone();
        let draw = draw_render.clone();
        let quit = quit.clone();
        let parker = parker.clone();
        thread::spawn(|| render::render(gpu, windows, draw, quit, parker))
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

    let timing = {
        let timer = SpinSleeper::default();
        let event_proxy = event_loop.create_proxy();
        let quit = quit.clone();

        // [VITAL] Frame Period (Inverse of FPS)
        const FRAME_PERIOD: Duration = Duration::new(0, 0_016_660_000);

        thread::spawn(move || {
            loop {
                timer.sleep(FRAME_PERIOD);
                for _ in 0..2 {
                    let _ = parked.recv();
                    if game.is_finished() || render.is_finished() {
                        quit.store(true, std::sync::atomic::Ordering::Relaxed);
                        game.thread().unpark();
                        render.thread().unpark();
                        let _ = game.join();
                        println!("game joined");
                        let _ = render.join();
                        println!("render joined");
                        let _ = event_proxy.send_event(app::UserEvent::Exit);
                        return;
                    }
                }
                // [VITAL] Swap Draw Buffers
                {
                    let mut draw_buffer = draw_buffer.lock();
                    let mut draw_render = draw_render.lock();
                    std::mem::swap(draw_buffer.deref_mut(), draw_render.deref_mut());
                }
                game.thread().unpark();
                render.thread().unpark();
            }
        })
    };

    // [VITAL] Run App
    let mut app = App {
        gpu,
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

    // [VITAL] Cleanup
    // Prevent detatched thread.
    // (See std::thread::Thread)
    quit.store(true, std::sync::atomic::Ordering::Relaxed);
    let _ = timing.join();
}
