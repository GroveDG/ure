use cgmath::Vector2;

use super::Pixels;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub pos: Vector2<Pixels>,
    pub size: Vector2<Pixels>,
}
impl Default for Rect {
    fn default() -> Self {
        Self {
            pos: Vector2 { x: 0, y: 0 },
            size: Vector2 { x: 0, y: 0 },
        }
    }
}