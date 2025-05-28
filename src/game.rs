use std::{
    ops::DerefMut,
    sync::{Arc, atomic::AtomicBool, mpsc::Sender},
    thread,
    time::Instant,
};

use cgmath::SquareMatrix;
use parking_lot::{Mutex, RwLock};
use winit::event_loop::EventLoopProxy;

use crate::app::{UserEvent, input::Input, window::Windows};
use crate::render::{
    _2d::{Draw2D, Mesh2D, Vertex2D},
    RenderBuffer,
};
use crate::sys::{UIDs, delete::DeleteQueue};

use self::gui::Layout;
use self::tf::{Matrix2D, Space2D};
use self::tree::Tree;

pub mod assets;
pub mod gui;
pub mod tf;
pub mod tree;



pub fn game(
    event_proxy: EventLoopProxy<UserEvent>,
    windows: Arc<RwLock<Windows>>,
    input: Arc<Mutex<Input>>,
    draw: Arc<Mutex<RenderBuffer>>,
    quit: Arc<AtomicBool>,
    parker: Sender<()>,
) {
    // [CORE] Initialize UID System
    let mut uids = UIDs::new();

    // [VITAL] Initialize Delete System
    let mut delete = DeleteQueue::default();

    // [VITAL] Initialize Draw Buffer
    let mut draw2d = Draw2D::default();
    let quad = uids.add();
    {
        let quad_mesh = Mesh2D {
            vertex: vec![
                Vertex2D {
                    // Top Left
                    position: [-0.5, 0.5],
                    color: [1., 1., 1.],
                    uv: [0., 0.],
                },
                Vertex2D {
                    // Top Right
                    position: [0.5, 0.5],
                    color: [1., 1., 1.],
                    uv: [1., 0.],
                },
                Vertex2D {
                    // Bottom Left
                    position: [-0.5, -0.5],
                    color: [1., 1., 1.],
                    uv: [0., 1.],
                },
                Vertex2D {
                    // Bottom Right
                    position: [0.5, -0.5],
                    color: [1., 1., 1.],
                    uv: [1., 1.],
                },
            ],
            index: vec![0, 1, 2, 2, 1, 3],
        };
        draw2d.add_mesh(quad, quad_mesh);
    }

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
        let delta = last_start.elapsed();

        if quit.load(std::sync::atomic::Ordering::Relaxed) {
            break 'game;
        }

        // ========================================================
        // PRE-FRAME
        // ========================================================

        // [VITAL] Clear Old Delete Requests
        delete.start_frame();

        // Delete UIDs
        delete.apply(&mut uids);

        // [VITAL] Acquire Input State
        let input_state = input.lock().clone();

        // ========================================================
        // GAME LOGIC
        // ========================================================

        // [USEFUL] Prevent write lock when no deletes are needed.
        let needs_delete = {
            let windows = windows.read();
            windows.windows.keys().any(|uid| delete.contains(uid))
        };
        // [USEFUL] Delete Window on Close
        if needs_delete || !input_state.close.is_empty() {
            let mut windows = windows.write();
            delete.apply(windows.deref_mut());
            for window in input_state.close {
                delete.delete(windows.deref_mut(), window);
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
            let mut draw = draw.lock();
            (draw.updates._2d, draw.commands._2d) = draw2d.finish();
        }

        // ========================================================
        // END OF FRAME
        // ========================================================

        // [VITAL] Store Start of Last Frame
        last_start = start;

        // [VITAL] Signal End of Frame
        let _ = parker.send(());
        // [VITAL] Wait for Next Frame
        thread::park();
    }
    let _ = parker.send(());
}
