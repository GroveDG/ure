use wgpu::Color;

use crate::sys::{Components, UID};

use super::{layout::Layout, Pixels};

#[derive(Debug, Clone, Copy)]
pub struct Border {
    width: Pixels,
    color: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub color: Color,
    pub radius: Option<Pixels>,
    pub border: Option<Border>,
}
#[derive(Debug, Default)]
pub struct QuadRenderer {
    boxes: Components<Style>,
}

impl QuadRenderer {
    pub fn render(&self, uid: &UID, target: &mut Canvas<Window>, layout: &Layout) {
        let Some(rect) = layout.get_rect(uid).copied() else {
            return;
        };
        let Some(style) = self.boxes.get(uid) else {
            return;
        };
        target.set_draw_color(style.color);
        let _ = target.fill_rect(rect);
    }
    pub fn insert(&mut self, uid: UID, render_box: Style) {
        self.boxes.insert(uid, render_box);
    }
}
