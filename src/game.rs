use std::{
    sync::{Arc, mpsc::Receiver},
    time::{Duration, Instant},
};

use glam::Vec2;
use parking_lot::Mutex;
use spin_sleep::SpinSleeper;
use wgpu::{RenderPassColorAttachment, RenderPassDescriptor, wgt::CommandEncoderDescriptor};
use winit::{
    event_loop::EventLoopProxy,
    window::{Window, WindowAttributes},
};

use crate::{
    app::window::Windows,
    game::gui::{Style, Text},
    render::{Color, GpuColor},
    sys::{Components, Uid},
};
use crate::{
    app::{UserEvent, input::Input},
    render::_2d::Draw2D,
};
use crate::{
    game::gui::Lay,
    sys::{Uids, delete::DeleteQueue},
};

use self::gui::Layout;
use self::tf::Matrix2D;

pub mod assets;
pub mod gui;
pub mod tf;
pub mod tree;

pub const SURFACE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
pub const FRAME_TIME: Duration = Duration::new(0, 016_666_667);

pub fn game(
    event_proxy: EventLoopProxy<UserEvent>,
    window_recv: Receiver<(Uid, Window)>,
    input: Arc<Mutex<Input>>,
    gpu: &(wgpu::Instance, wgpu::Device, wgpu::Queue),
) {
    // [CORE] Initialize UID System
    let mut uids = Uids::new();

    // [VITAL] Initialize Delete System
    let mut delete = DeleteQueue::default();

    // [VITAL] Initialize Render Systems
    let (instance, device, queue) = gpu;
    let mut windows = Windows::new(event_proxy.clone(), window_recv);
    let mut draw_2d = Draw2D::new(device);
    let (quad,) = draw_2d.primitives(&mut uids, device, queue);

    // [USEFUL] Initialize UI Systems
    let mut layout = Layout::new(quad, &mut uids);

    // [USEFUL] Init Root
    let root = uids.add();
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
            color: Some(Color::WHITE),
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
    let sleeper = SpinSleeper::default();
    let mut last_start = Instant::now(); // Last frame start
    let mut render: Option<std::thread::JoinHandle<()>> = None;

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

        // [VITAL] Wait for Previous Frame to Render
        if let Some(render) = render.take() {
            let _ = render.join();
        }
        run!(windows, {
            // [VITAL] Receive New Windows
            windows.receive(&instance, &device);
            // [USEFUL] Delete Window on Close
            for uid in input_state.close {
                delete.delete(&mut windows, uid);
            }
            // [USEFUL] Quit when all windows are closed.
            if windows.is_empty() {
                break 'game;
            }
        });

        run!(draw_2d, {
            let mut draw_2d = draw_2d.update(device, queue);
            for (uid, (window, _)) in windows.windows.iter() {
                let size = window.inner_size();
                let width = size.width as f32;
                let height = size.height as f32;
                if let Some(lay) = layout.get_mut(uid) {
                    lay.fix_size(width, height);
                }
                draw_2d.camera(
                    *uid,
                    Matrix2D::from_scale(Vec2 {
                        x: width / 2.,
                        y: height / 2.,
                    })
                    .inverse(),
                );
            }

            // [USEFUL] GUI Layout
            #[cfg(feature = "GUI")]
            run!(layout, {
                layout.run(&mut draw_2d);
            });
        });

        // ================================================================================================================
        // RENDER
        // ================================================================================================================

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
        let mut surface_textures = Components::default();

        for (uid, (_, surface)) in windows.windows.iter() {
            let Ok(surface_texture) = surface.get_current_texture() else {
                continue;
            };
            surface_textures.insert(*uid, surface_texture);
        }

        for (uid, surface_texture) in surface_textures.iter() {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &surface_texture
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor {
                            format: Some(SURFACE_FORMAT),
                            ..Default::default()
                        }),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // [USEFUL] Clear Surface
                        load: wgpu::LoadOp::Clear(Color::BLACK.to_gpu()),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // [EXAMPLE] Render Example Quad
            run!(draw_2d, {
                let mut pass = draw_2d.pass(&mut pass);
                pass.camera(*uid);
                layout.draw(&mut pass);
            });
        }

        let commands = encoder.finish();
        queue.submit([commands]);
        for (_, surface_texture) in surface_textures {
            surface_texture.present();
        }
        // render = Some({
        //     let queue = queue.clone();
        //     let commands = encoder.finish();
        //     std::thread::Builder::new()
        //         .name("render".to_string())
        //         .spawn(move || {
        //             queue.submit([commands]);
        //             for (_, surface_texture) in surface_textures {
        //                 surface_texture.present();
        //             }
        //         })
        //         .unwrap()
        // });

        // ================================================================================================================
        // END OF FRAME
        // ================================================================================================================

        // [VITAL] Delay Next Frame
        sleeper.sleep(FRAME_TIME.saturating_sub(start.elapsed()));

        // [VITAL] Store Start of Last Frame
        last_start = start;
    }
    if let Some(render) = render {
        let _ = render.join();
    }
    event_proxy.send_event(UserEvent::Exit).unwrap();
}
