// use std::{
//     sync::{
//         Arc,
//         mpsc::{Receiver, Sender, channel},
//     },
//     thread::{JoinHandle, Thread, spawn},
// };

// use parking_lot::Mutex;
// use winit::{
//     application::ApplicationHandler,
//     event::WindowEvent,
//     event_loop::ActiveEventLoop,
//     window::{Window, WindowAttributes},
// };

// use self::input::Input;

// pub mod input;
// pub mod window;

// #[derive(Debug)]
// #[non_exhaustive]
// pub enum UserEvent {
//     NewWindow(WindowAttributes),
//     Exit,
// }

// pub trait Game: Send + 'static {
//     fn new(event_loop: &ActiveEventLoop) -> Self;
//     fn start(self);
// }

// pub struct App<G: Game> {
//     input: Arc<Mutex<Input>>,
//     game: Option<JoinHandle<()>>,
// }

// impl<G: Game> App<G> {
//     pub fn new() -> Self {
//         let input = Arc::new(Mutex::new(Input::default()));
//         Self { input, game: None }
//     }
// }

// impl<G: Game> ApplicationHandler<UserEvent> for App<G> {
//     fn resumed(&mut self, event_loop: &ActiveEventLoop) {
//         let game = G::new(event_loop);
//         self.game = Some(spawn(|| game.start()))
//     }

//     fn window_event(
//         &mut self,
//         _event_loop: &ActiveEventLoop,
//         window_id: winit::window::WindowId,
//         event: winit::event::WindowEvent,
//     ) {
//         match event {
//             WindowEvent::CloseRequested => {
//                 let mut input = self.input.lock();
//                 let Some(uid) = self.window_ids.get_by_right(&window_id) else {
//                     return;
//                 };
//                 input.close.insert(*uid);
//             }
//             _ => {}
//         }
//     }

//     fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
//         match event {
//             UserEvent::NewWindow(attr) => {
//                 let window = event_loop
//                     .create_window(*attr)
//                     .expect("Window creation failed. See winit::event_loop::ActiveEventLoop.");
//                 self.window_ids.insert(uid, window.id());
//                 let _ = self.window_send.send((uid, window));
//             }
//             UserEvent::Exit => {
//                 event_loop.exit();
//             }
//         }
//     }

//     fn exiting(&mut self, event_loop: &ActiveEventLoop) {
//         if let Some(game) = self.game.take() {
//             _ = game.join();
//         }
//     }
// }
