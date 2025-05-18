use std::{
    ops::DerefMut,
    sync::{Arc, Condvar, Mutex, RwLock},
    time::{Duration, Instant},
};

use crate::sys::{
    UIDs,
    gui::Layout,
    input::Input,
    tf::{Matrix2D, Space2D},
    tree::Tree,
    window::Windows,
};
use crate::{app::UserEvent, render::DrawBuffer, sys::delete::DeleteQueue};
use cgmath::SquareMatrix;
use spin_sleep::SpinSleeper;
use winit::event_loop::EventLoopProxy;

// [VITAL] Frame Period (Inverse of FPS)
const FRAME_PERIOD: Duration = Duration::new(0, 0_016_660_000);

pub fn game(
    event_proxy: EventLoopProxy<UserEvent>,
    windows: Arc<RwLock<Windows>>,
    input: Arc<Mutex<Input>>,
    draw_commands: Arc<(Mutex<DrawBuffer>, Condvar)>,
) {
    // [CORE] Initialize UID System
    let mut uids = UIDs::new();

    // [VITAL] Initialize Delete System
    let mut delete = DeleteQueue::default();

    // [VITAL] Initialize Draw Buffer
    let mut draw_buffer = DrawBuffer::default();

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

        // ========================================================
        // PRE-FRAME
        // ========================================================

        delete.start_frame();
        delete.apply(&mut uids);

        // [VITAL] Acquire Input State
        let input_state = {
            let Ok(input) = input.lock() else {
                break 'game;
            };
            input.clone()
        };

        // ========================================================
        // GAME LOGIC
        // ========================================================

        // [USEFUL] Delete Window on Close
        {
            // [USEFUL] Prevent write lock when no deletes are needed.
            let needs_delete = {
                let Ok(windows) = windows.read() else {
                    break 'game;
                };
                windows.windows.keys().any(|uid| delete.contains(uid))
            };
            if needs_delete || !input_state.close.is_empty() {
                let mut windows = windows.write().unwrap();
                delete.apply(windows.deref_mut());
                for window in input_state.close {
                    delete.delete(windows.deref_mut(), window);
                }
            }
        }
        // [USEFUL] Exit if No Windows
        {
            let windows = windows.read().unwrap();
            if windows.windows.is_empty() && windows.requested == 0 {
                println!("Close");
                break 'game;
            }
        }

        // [USEFUL] GUI Layout
        #[cfg(feature = "GUI")]
        {
            delete.apply(&mut layout);
            layout.run();
        }

        println!("CPU {:?}", start.elapsed());

        // [VITAL] Send Draw Commands to Render
        swap_draw_buffer(&draw_commands, &mut draw_buffer);

        // ========================================================
        // END OF FRAME
        // ========================================================

        // [VITAL] Time Frame
        let cpu_time = start.elapsed();
        // println!("CPU {:?}", cpu_time);

        // [VITAL] Delay Update
        let remaining = FRAME_PERIOD.saturating_sub(cpu_time);
        frame_timer.sleep(remaining); // Sleep slightly overshoots frame period

        // [VITAL] Store Start of Last Frame
        last_start = start;
    }

    let _ = event_proxy.send_event(UserEvent::Exit);
    draw_buffer.exit = true;
    swap_draw_buffer(&draw_commands, &mut draw_buffer);
}

fn swap_draw_buffer(
    draw_commands: &Arc<(Mutex<DrawBuffer>, Condvar)>,
    draw_buffer: &mut DrawBuffer,
) {
    {
        let Ok(mut draw_commands) = draw_commands.0.lock() else {
            return;
        };
        std::mem::swap(draw_commands.deref_mut(), draw_buffer);
    }
    draw_commands.1.notify_all();
}
