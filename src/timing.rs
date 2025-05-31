use std::{
    sync::mpsc::{Receiver, Sender},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use spin_sleep::SpinSleeper;
use winit::event_loop::EventLoopProxy;

use crate::{app::UserEvent, render::RenderCommand};



// [VITAL] Frame Period (Inverse of FPS)
const FRAME_PERIOD: Duration = Duration::new(0, 0_016_660_000);

pub fn timing(
    frame: Receiver<()>,
    game: JoinHandle<()>,
    render: JoinHandle<()>,
    render_sndr: Sender<RenderCommand>,
    event_proxy: EventLoopProxy<UserEvent>,
) {
    let timer = SpinSleeper::default();
    loop {
        let start = Instant::now();
        for _ in 0..2 {
            let _ = frame.recv();
        }

        // If either thread quit...
        if game.is_finished() {
            game.join();
            render_sndr.send(RenderCommand::Quit);
            render.join();
            // Request app thread quit.
            let _ = event_proxy.send_event(UserEvent::Exit);
            // Wait for wake up in main().
            return;
        }

        // Resume threads.
        game.thread().unpark();

        // Calculate Remaining Time in Frame
        let elapsed = start.elapsed();
        let remaining = FRAME_PERIOD.saturating_sub(elapsed);
        // [VITAL] Sleep For Remaining Time
        timer.sleep(remaining);
    }
}
