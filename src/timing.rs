use std::{
    sync::{Arc, atomic::AtomicBool, mpsc::Receiver},
    thread::{JoinHandle, park},
    time::{Duration, Instant},
};

use parking_lot::Mutex;
use spin_sleep::SpinSleeper;
use winit::event_loop::EventLoopProxy;

use crate::{app::UserEvent, render::RenderBuffer};



// [VITAL] Frame Period (Inverse of FPS)
const FRAME_PERIOD: Duration = Duration::new(0, 0_016_660_000);

pub fn timing(
    parked: Receiver<()>,
    game: JoinHandle<()>,
    render: JoinHandle<()>,
    quit: Arc<AtomicBool>,
    event_proxy: EventLoopProxy<UserEvent>,
    draw_buffer: Arc<Mutex<RenderBuffer>>,
    draw_render: Arc<Mutex<RenderBuffer>>,
) {
    let timer = SpinSleeper::default();
    loop {
        let start = Instant::now();
        for _ in 0..2 {
            let _ = parked.recv();
        }

        // If either thread quit...
        if game.is_finished() || render.is_finished() {
            // Request app thread quit.
            let _ = event_proxy.send_event(UserEvent::Exit);
            // Wait for wake up in main().
            park();
        }

        // [VITAL] Swap Draw Buffers
        {
            let mut draw_buffer = draw_buffer.lock();
            let mut draw_render = draw_render.lock();
            std::mem::swap(&mut *draw_buffer, &mut *draw_render);
        }

        // Resume threads.
        game.thread().unpark();
        render.thread().unpark();

        // If quit requested...
        if quit.load(std::sync::atomic::Ordering::Relaxed) {
            // Wait for both threads.
            let _ = game.join();
            println!("game joined");
            let _ = render.join();
            println!("render joined");
            // Stop thread.
            return;
        }

        // Calculate Remaining Time in Frame
        let elapsed = start.elapsed();
        let remaining = FRAME_PERIOD.saturating_sub(elapsed);
        // [VITAL] Sleep For Remaining Time
        timer.sleep(remaining);
    }
}
