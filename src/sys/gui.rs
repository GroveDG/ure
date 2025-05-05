//! Clay-inspired ui layout
//! 
//! 

/* Clay layout pass order
1. Fit Sizing Widths
2. Grow & Shrink Sizing Widths
3. Wrap Text
4. Fit Sizing Heights
5. Grow & Shrink Sizing Heights
6. Positions
7. Draw
https://youtu.be/by9lQvpvMIc?t=2179 */

use super::{tree::Tree, Components};

type Pixel = u16;
type Precision = f32;


pub enum Size {
    Fixed(Pixel),
    Percent(Precision)
}

/// 
pub enum Sizing {
    /// Fit all children
    Fit,
    /// Fill space 
    Fill,
    /// Fixed or relative size,
    Size(Size)
}

pub struct AxisSize {
    min: Option<Size>,
    max: Option<Size>,
    sizing: Sizing,
}

pub struct BoxSize {
    x: AxisSize,
    y: AxisSize,
}

pub struct Pad {
    up: Pixel,
    down: Pixel,
    left: Pixel,
    right: Pixel,
}

pub enum Align {
    Left,
    Center,
    Right,
}

pub enum Direction {
    /// Left to right
    Right,
    /// Top to bottom
    Down,
}

pub struct Lay {
    size: Size,
    pad: Pad,
    gap: Pixel,
    align: Align,
    direction: Direction,
}


/// Layout is not Rendering, the two passes cannot happen at once.
/// 
pub struct Layout {
    elements: Components<Lay>
}

// impl Layout {
//     pub fn run(&mut self, tree: &Tree) {
//         tree.lrn(tree.)
//     }
// }