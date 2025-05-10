use std::{
    fs::File,
    thread::sleep,
    time::{Duration, Instant},
};

use cgmath::SquareMatrix;
use ron::ser::PrettyConfig;
use sdl2::{event::Event, pixels::Color};
use sys::{
    gui::{
        layout::{Lay, Layout},
        render::{BoxRenderer, Style},
    }, sdl::{Events, Windows}, tf::{Matrix2D, Space2D}, tree::Tree, UIDs
};

mod call;
mod sys;

const FRAME_PERIOD: Duration = Duration::new(0, 0_016_666_667);

fn main() {
    // Initialize UID system
    let mut uids = UIDs::new().unwrap();

    // Initialize SDL systems
    let mut sdl = sdl2::init().unwrap();
    let mut windows = Windows::new(&sdl).unwrap();
    let mut events = Events::new(&sdl).unwrap();

    // Initialize game systems
    let mut tree = Tree::default();
    let mut space = Space2D::default();
    let mut layout = Layout::default();
    let mut box_renderer = BoxRenderer::default();

    // // Initialize mass calls
    // // If you call these functions while one of them is locked,
    // // the application will become deadlocked and freeze.
    // let mut delete = {
    //     let uids = Arc::clone(&uids);
    //     let tree = Arc::clone(&tree);
    //     let space = Arc::clone(&space);
    //     let windows = Arc::clone(&windows);
    //     Call::new(Box::new(move |batch: &mut Vec<UID>| {
    //         // Lock before call for performance.
    //         let mut uids = uids;
    //         let mut tree = tree;
    //         let mut space = space;
    //         let mut windows = windows;
    //         // Call
    //         for args in batch.drain(..) {
    //             uids.delete(&args);
    //             tree.delete(&args); // Let tree add children to batch
    //             space.delete(&args);
    //             windows.delete(&args);
    //         }
    //     }))
    // };

    // Init root
    let root = uids.add();
    tree.insert(root, None);
    tree.insert(uids.add(), Some(root));
    tree.insert(uids.add(), Some(root));
    tree.insert(uids.add(), None);
    space.insert(root, Matrix2D::identity());
    windows.insert(root, "Window", 640, 480);
    layout.insert(root, Lay::default().fix_size(240, 240), None);
    box_renderer.insert(root, Style {
        color: Color::WHITE,
        radius: None,
        border: None,
    });

    {
        let file = File::options()
            .create(true)
            .write(true)
            .open("tree.ure")
            .unwrap();
        ron::Options::default()
            .to_io_writer_pretty(
                file,
                &tree,
                PrettyConfig::default().indentor("\t").struct_names(true),
            )
            .unwrap();
    }

    let mut last_start = Instant::now(); // Last frame start
    'game: loop {
        // Time Frame
        let start = Instant::now();
        let delta = start - last_start;

        // Poll Events
        for event in events.poll() {
            match event {
                Event::Quit { .. } => break 'game,
                _ => {}
            }
        }

        // Clear Windows (& Quit)
        {
            // If no windows, quit.
            if windows.is_empty() {
                break 'game;
            }
            // Clear frame
            windows.clear();
        }

        // GUI Layout & Rendering
        {
            layout.run();
            if let Some(window) = windows.get_mut(&root) {
                for node in layout.render_order() {
                    box_renderer.render(node, window, &layout);
                }
            }
        }

        // Render and Present Frame
        windows.present();

        // Time Frame
        let end = Instant::now();
        let cpu_time = end - start;
        println!("{:?}, {:?}", cpu_time, delta);

        // Delay Update
        let remaining = FRAME_PERIOD.saturating_sub(cpu_time);
        sleep(remaining); // Sleep slightly overshoots frame period

        last_start = start;
    }
}
