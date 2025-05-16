use std::{
    ops::DerefMut, sync::{Arc, Barrier, Mutex, RwLock}, time::{Duration, Instant}
};

use crate::{app::UserEvent, sys::delete::DeleteQueue};
use crate::
    sys::{
        UIDs,
        gui::Layout,
        input::Input,
        tf::{Matrix2D, Space2D},
        tree::Tree,
        window::Windows,
    }
;
use cgmath::SquareMatrix;
use spin_sleep::SpinSleeper;
use winit::event_loop::EventLoopProxy;

// [VITAL] Frame Period (Inverse of FPS)
const FRAME_PERIOD: Duration = Duration::new(0, 0_016_660_000);

pub fn game(
    event_proxy: EventLoopProxy<UserEvent>,
    windows: Arc<RwLock<Windows>>,
    input: Arc<Mutex<Input>>,
    render_sync: Arc<Barrier>,
) {
    // [CORE] Initialize UID System
    let mut uids = UIDs::new();

    // [VITAL] Initialize Delete System
    let mut delete = DeleteQueue::default();

    // [USEFUL] Initialize UI Systems
    let mut layout = Layout::default();

    // [USEFUL] Initialize Game Systems
    let mut tree = Tree::default();
    let mut space = Space2D::default();

    // [EXAMPLE] Init Root
    let root = uids.add();
    tree.insert(root, None);
    space.insert(root, Matrix2D::identity());
    {
        let mut windows = windows.write().unwrap();
        if windows
            .request_new(root, Default::default(), &event_proxy)
            .is_err()
        {
            return;
        }
    }

    // [VITAL] Frame Timing
    let frame_timer = SpinSleeper::default();
    let mut last_start = Instant::now(); // Last frame start

    // [VITAL] Game Loop
    'game: loop {
        // [VITAL] Time Frame
        let start = Instant::now();
        let delta = start - last_start;

        delete.start_frame();
        delete.apply(&mut uids);

        // [VITAL] Acquire Input State
        let input_state = {
            let Ok(input) = input.lock() else {
                break 'game;
            };
            input.clone()
        };

        // [USEFUL] Delete Window on Close
        {
            let mut windows = windows.write().unwrap();
            delete.apply(windows.deref_mut());
            for window in input_state.close {
                delete.delete(windows.deref_mut(), window);
            }
        }
        // [USEFUL] Exit if No Windows
        {
            let windows = windows.read().unwrap();
            if windows.is_empty() {
                let _ = event_proxy.send_event(UserEvent::Exit);
                break 'game;
            }
        }

        // [VITAL] Wait for Render
        render_sync.wait();

        // [USEFUL] GUI Layout
        #[cfg(feature = "GUI")]
        {
            delete.apply(&mut layout);
            layout.run();
        }

        // ========================================================
        // END OF FRAME
        // ========================================================

        // [VITAL] Time Frame
        let end = Instant::now();
        let cpu_time = end - start;
        println!("CPU {:?}", cpu_time);

        // [VITAL] Delay Update
        let remaining = FRAME_PERIOD.saturating_sub(cpu_time);
        frame_timer.sleep(remaining); // Sleep slightly overshoots frame period

        // [VITAL] Store Start of Last Frame
        last_start = start;
    }
}
