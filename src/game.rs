use std::{
    ops::DerefMut,
    sync::{mpsc::{Receiver, Sender}, Arc},
    thread,
    time::Instant,
};

use glam::Vec2;
use parking_lot::Mutex;
use winit::{
    event_loop::EventLoopProxy,
    window::{Window, WindowAttributes},
};

use crate::{app::window::Windows, render::_2d::{Mesh2D, Vertex2D}, sys::UID};
use crate::sys::{UIDs, delete::DeleteQueue};
use crate::{
    app::{UserEvent, input::Input},
    render::{
        _2d::{Draw2D, Instance2D},
        RenderCommand,
        gpu::Color,
    },
};

use self::gui::Layout;
use self::tf::{Matrix2D, Space2D};
use self::tree::Tree;

pub mod assets;
pub mod gui;
pub mod tf;
pub mod tree;

pub fn game(
    event_proxy: EventLoopProxy<UserEvent>,
    window_recv: Receiver<(UID, Window)>,
    input: Arc<Mutex<Input>>,
    render: Sender<RenderCommand>,
    parker: &Sender<()>,
) {
    // [CORE] Initialize UID System
    let mut uids = UIDs::new();

    // [VITAL] Initialize Delete System
    let mut delete = DeleteQueue::default();

    let mut windows = Windows::new(event_proxy.clone(), window_recv);
    let mut draw_2d = Draw2D::new(render.clone());

    // [VITAL] Initialize Draw Buffer
    let quad = uids.add();
    {
        let quad_mesh = Mesh2D {
            vertex: vec![
                Vertex2D {
                    // Top Left
                    position: Vec2 { x: -0.5, y: 0.5 },
                    color: Color::WHITE,
                    uv: Vec2::ZERO,
                },
                Vertex2D {
                    // Top Right
                    position: Vec2 { x: -0.5, y: -0.5 },
                    color: Color::WHITE,
                    uv: Vec2::X,
                },
                Vertex2D {
                    // Bottom Left
                    position: Vec2 { x: 0.5, y: 0.5 },
                    color: Color::WHITE,
                    uv: Vec2::Y,
                },
                Vertex2D {
                    // Bottom Right
                    position: Vec2 { x: 0.5, y: -0.5 },
                    color: Color::WHITE,
                    uv: Vec2::ONE,
                },
            ],
            index: vec![0, 1, 2, 2, 1, 3],
        };
        draw_2d.update_mesh(quad, &mut uids, quad_mesh);
    }
    let instance = uids.add();
    {
        draw_2d.update_instances(
            instance,
            vec![Instance2D {
                tf: Default::default(),
                color: Color::WHITE,
            }],
        );
    }

    // [USEFUL] Initialize UI Systems
    let mut layout = Layout::default();

    // [USEFUL] Initialize Game Systems
    let mut tree = Tree::default();
    let mut space = Space2D::default();

    // [EXAMPLE] Init Root
    let root = uids.add();
    tree.insert(root, None);
    space.insert(root, Matrix2D::IDENTITY);
    let _ = windows.request_new(
        root,
        WindowAttributes::default().with_title("Untitled Rust Engine"),
        &event_proxy,
    );

    // [VITAL] Frame Timing
    let mut last_start = Instant::now(); // Last frame start

    // [VITAL] Game Loop
    'game: loop {
        // [VITAL] Time Frame
        let start = Instant::now();
        let delta = last_start.elapsed();

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

        // [VITAL] Receive New Windows
        windows.receive();

        // [USEFUL] Delete Window on Close
        for uid in input_state.close {
            delete.delete(&mut windows, uid);
        }
        delete.apply(&mut windows);

        for uid in delete.iter().copied() {
            let _ = render.send(RenderCommand::Delete(uid));
        }
        delete.apply(&mut draw_2d);

        // [USEFUL] Quit when all windows are closed.
        if windows.is_empty() {
            break 'game;
        }

        for (uid, window) in windows.windows.iter() {
            let _ = render.send(RenderCommand::Window(window.clone(), *uid));
        }

        // [USEFUL] GUI Layout
        #[cfg(feature = "GUI")]
        {
            delete.apply(&mut layout);
            layout.run();
        }

        let _ = render.send(RenderCommand::Pass(root));
        draw_2d.start();
        draw_2d.mesh(&quad);
        draw_2d.instances(instance);
        draw_2d.draw();

        if render.send(RenderCommand::Submit).is_err() {
            break 'game;
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
}
