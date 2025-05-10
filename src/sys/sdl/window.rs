use sdl2::{pixels::Color, render::Canvas, video::Window, Sdl, VideoSubsystem};

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
    pub fn get<'a>(&'a self, uid: &UID) -> Option<&'a Canvas<Window>> {
        self.canvases.get(uid)
    }
    pub fn get_mut<'a>(&'a mut self, uid: &UID) -> Option<&'a mut Canvas<Window>> {
        self.canvases.get_mut(uid)
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
            canvas.set_draw_color(Color::BLACK);
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
