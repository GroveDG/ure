use sdl2::{Sdl, VideoSubsystem, render::Canvas, video::Window};

use crate::sys::{Components, UID, UIDs};

pub struct Windows {
    video: VideoSubsystem,
    canvases: Components<Canvas<Window>>,
}

impl Windows {
    pub fn new(sdl: &Sdl) -> Result<Self, String> {
        Ok(Self {
            video: sdl.video()?,
            canvases: Default::default(),
        })
    }
    pub fn new_window(&mut self, uids: &mut UIDs, title: &str, width: u32, height: u32) -> UID {
        let uid = uids.new_uid();
        let window = self
            .video
            .window(title, width, height)
            // Sensible Window defaults.
            .position_centered()
            .opengl() // In future, create Vulkan feature.
            .resizable()
            // Build window.
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();
        self.canvases.insert(uid, canvas);
        uid
    }
    /// See [Canvas::clear].
    pub fn clear(&mut self) {
        for canvas in self.canvases.values_mut() {
            canvas.clear();
        }
    }
    /// See [Canvas::present].
    pub fn present(&mut self) {
        for canvas in self.canvases.values_mut() {
            canvas.present();
        }
    }
}
