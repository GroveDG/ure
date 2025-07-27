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

fn main() {
    // // [VITAL] Initialize App
    // let event_loop = EventLoop::with_user_event()
    //     .build()
    //     .expect("EventLoop building failed. See winit::event_loop::EventLoopBuilder::build");

    // // [VITAL] Initialize Shared Systems
    // let (window_sndr, window_recv) = channel();
    // let input = Arc::new(Mutex::new(Input::default()));

    // // [VITAL] Initialize Game Thread
    // let game = {
    //     let event_proxy = event_loop.create_proxy();
    //     let input = Arc::clone(&input);
    //     let gpu = new_gpu().block();

    //     thread::Builder::new()
    //         .name("game".to_string())
    //         .spawn(move || {
    //             game::game(event_proxy, window_recv, input, &gpu);
    //             println!("GAME THREAD QUIT");
    //             // Seg fault on "terminate display" in drivers because... idk
    //             // forgetting solves this although at what cost I can't be sure.
    //             std::mem::forget(gpu.0);
    //         })
    //         .unwrap()
    // };

    // // [VITAL] Run App
    // let mut app = App {
    //     window_ids: Default::default(),
    //     window_sndr,
    //     input,
    // };
    // event_loop.run_app(&mut app).unwrap();

    // println!("APP THREAD QUIT");

    // // [VITAL] Prevent Dangling Threads
    // let _ = game.join();
}
