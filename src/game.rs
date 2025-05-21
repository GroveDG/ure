use std::{
    ops::DerefMut,
    sync::{Arc, atomic::AtomicBool, mpsc::Sender},
    thread,
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
use parking_lot::{Condvar, Mutex, RwLock};
use winit::event_loop::EventLoopProxy;

pub fn game(
    event_proxy: EventLoopProxy<UserEvent>,
    windows: Arc<RwLock<Windows>>,
    input: Arc<Mutex<Input>>,
    draw: Arc<Mutex<DrawBuffer>>,
    quit: Arc<AtomicBool>,
    parker: Sender<()>,
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
        let mut windows = windows.write();
        if windows
            .request_new(root, Default::default(), &event_proxy)
            .is_err()
        {
            return;
        }
    }

    // [VITAL] Frame Timing
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
        let input_state = input.lock().clone();

        // ========================================================
        // GAME LOGIC
        // ========================================================

        // [USEFUL] Delete Window on Close
        {
            // [USEFUL] Prevent write lock when no deletes are needed.
            let needs_delete = {
                let windows = windows.read();
                windows.windows.keys().any(|uid| delete.contains(uid))
            };
            if needs_delete || !input_state.close.is_empty() {
                let mut windows = windows.write();
                delete.apply(windows.deref_mut());
                for window in input_state.close {
                    delete.delete(windows.deref_mut(), window);
                }
            }
        }
        // [USEFUL] Exit if No Windows
        {
            let windows = windows.read();
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

        {
            let draw = draw.lock();
        }

        // ========================================================
        // END OF FRAME
        // ========================================================

        // [VITAL] Time Frame
        let cpu_time = start.elapsed();
        println!("CPU {:?}", cpu_time);

        // [VITAL] Store Start of Last Frame
        last_start = start;

        let _ = parker.send(());
        thread::park();
    }
    let _ = parker.send(());
}
