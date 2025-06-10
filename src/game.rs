use std::{
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
    thread,
    time::Instant,
};

use glam::Vec2;
use parking_lot::Mutex;
use winit::{
    event_loop::EventLoopProxy,
    window::{Window, WindowAttributes},
};

use crate::{
    app::window::Windows,
    game::gui::{Style, Text},
    render::gpu::Color,
    sys::UID,
};
use crate::{
    app::{UserEvent, input::Input},
    render::{_2d::Draw2D, RenderCommand},
};
use crate::{
    game::gui::Lay,
    sys::{UIDs, delete::DeleteQueue},
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

    // [VITAL] Initialize Render Systems
    let mut windows = Windows::new(event_proxy.clone(), window_recv);
    let mut draw_2d = Draw2D::new(render.clone());
    let (quad,) = draw_2d.primitives(&mut uids);

    // [USEFUL] Initialize UI Systems
    let mut layout = Layout::new(quad, &mut uids);

    // [USEFUL] Initialize Game Systems
    let mut space = Space2D::default();

    // [USEFUL] Init Root
    let root = uids.add();
    // space.insert(root, Matrix2D::IDENTITY, None, None);
    layout.insert(root, Lay::default(), None, None, None, None);
    let _ = windows.request(
        root,
        WindowAttributes::default().with_title("Untitled Rust Engine"),
    );

    //[EXAMPLE]
    let tray = uids.add();
    let mut lay = Lay::default();
    lay.fix_size(100.0, 100.0);
    layout.insert(
        tray,
        lay,
        Some(Style {
            color: Some(Color::BLUE),
            border: None,
        }),
        Some(Text {
            align: gui::Align::Left,
            text: "Bobos".to_string(),
        }),
        Some(root),
        None,
    );

    // [VITAL] Frame Timing
    let mut last_start = Instant::now(); // Last frame start

    // [VITAL] Game Loop
    'game: loop {
        // [USEFUL] Define General System Behavior
        macro_rules! run {
            ($system:ident, $run:block) => {
                delete.apply(&mut $system);
                $run
            };
        }

        // [VITAL] Time Frame
        let start = Instant::now();
        let _delta = last_start.elapsed();

        // ================================================================================================================
        // PRE-FRAME
        // ================================================================================================================

        // [VITAL] Clear Old Delete Requests
        delete.start_frame();

        // [VITAL] Delete UIDs
        run!(uids, {});

        // [VITAL] Acquire Input State
        let input_state = std::mem::take(&mut *input.lock());

        // ================================================================================================================
        // GAME LOGIC
        // ================================================================================================================
        // Only issue deletes in here.

        run!(windows, {
            // [VITAL] Receive New Windows
            windows.receive(&render);
            // [USEFUL] Delete Window on Close
            for uid in input_state.close {
                delete.delete(&mut windows, uid);
            }
            // [USEFUL] Quit when all windows are closed.
            if windows.is_empty() {
                break 'game;
            }
            for (uid, window) in windows.resized() {
                let size = window.inner_size();
                let width = size.width as f32;
                let height = size.height as f32;
                if let Some(lay) = layout.get_mut(uid) {
                    lay.fix_size(width, height);
                }
                draw_2d.update_camera(
                    *uid,
                    Matrix2D::from_scale(Vec2 {
                        x: width / 2.,
                        y: height / 2.,
                    })
                    .inverse(),
                );
            }
        });

        // [USEFUL] GUI Layout
        #[cfg(feature = "GUI")]
        run!(layout, {
            layout.run(&draw_2d);
        });

        // ================================================================================================================
        // RENDER
        // ================================================================================================================

        for (&window, _) in windows.windows.iter() {
            // [VITAL] Apply Delete to Render Thread
            for uid in delete.iter() {
                let _ = render.send(RenderCommand::Delete(*uid));
            }

            let _ = render.send(RenderCommand::Pass(window));

            // [EXAMPLE] Render Example Quad
            run!(draw_2d, {
                draw_2d.camera(window);
                draw_2d.start();
                layout.draw(&draw_2d);
            });
        }

        // [VITAL] Submit Rendering to GPU
        let _ = render.send(RenderCommand::Submit);

        // ================================================================================================================
        // END OF FRAME
        // ================================================================================================================

        // [VITAL] Store Start of Last Frame
        last_start = start;

        // [VITAL] Signal End of Frame
        let _ = parker.send(());
        // [VITAL] Wait for Next Frame
        thread::park();
    }
}
