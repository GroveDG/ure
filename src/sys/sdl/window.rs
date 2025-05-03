use sdl2::{Sdl, VideoSubsystem, render::Canvas, video::Window};

use crate::sys::{Components, UID};

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
    /// Create a new window on the entity.
    pub fn insert(&mut self, uid: UID, title: &str, width: u32, height: u32) -> UID {
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
    pub fn delete(&mut self, uid: &UID) {
        self.canvases.remove(uid);
    }
    pub fn is_empty(&self) -> bool {
        self.canvases.is_empty()
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
