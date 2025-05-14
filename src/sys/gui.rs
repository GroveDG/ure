use cgmath::Vector2;

pub mod layout;
// pub mod render;

type Pixels = u16;
type Precision = f32;
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pos: Vector2<Pixels>,
    size: Vector2<Pixels>,
}
impl Default for Rect {
    fn default() -> Self {
        Self {
            pos: Vector2 { x: 0, y: 0 },
            size: Vector2 { x: 0, y: 0 },
        }
    }
}