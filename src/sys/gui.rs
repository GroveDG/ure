//! Clay-inspired ui layout
//!
//!

use cgmath::Vector2;
use wgpu::Color;

use crate::sys::{
    Components, UID,
    tree::{DFSPost, Tree},
};

use super::{delete::Delete, gpu::{render2d::Rect, Pixels}, tf::Precision};

/// Layout is not Rendering, the two passes cannot happen at once.
///
#[derive(Debug, Default)]
pub struct Layout {
    tree: Tree,
    lay: Components<Lay>,
    out: Components<Rect>,
}

impl Layout {
    /* Clay layout pass order
    1. Fit Sizing Widths (DFS Post-Order)
    2. Grow & Shrink Sizing Widths
    3. Wrap Text
    4. Fit Sizing Heights
    5. Grow & Shrink Sizing Heights
    6. Positions
    7. Draw
    https://youtu.be/by9lQvpvMIc?t=2179 */

    pub fn run(&mut self) {
        for parent in self.tree.dfs_post() {
            let p_lay = self.lay.get(parent).unwrap();
            let children = self.tree.children(Some(parent)).unwrap();

            let (along_i, across_i) = match p_lay.direction {
                Direction::Right => (0, 1),
                Direction::Down => (1, 0),
            };
            let mut fit = [false, false];
            let mut out_size: Vector2<Pixels> = Vector2 { x: 0, y: 0 };

            match p_lay.size.w.sizing {
                Sizing::Fit => fit[0] = true,
                Sizing::Fill => todo!(),
                Sizing::Size(size) => match size {
                    Size::Fixed(x) => out_size.x = x,
                    _ => {}
                },
            }
            match p_lay.size.h.sizing {
                Sizing::Fit => fit[1] = true,
                Sizing::Fill => todo!(),
                Sizing::Size(size) => match size {
                    Size::Fixed(y) => out_size.y = y,
                    _ => {}
                },
            }

            if fit[0] || fit[1] {
                let out_size = &mut out_size;
                let mut sizing_fn: Box<dyn FnMut(&Rect)> = match (fit[along_i], fit[across_i]) {
                    (true, true) => Box::new(|child: &Rect| {
                        out_size[along_i] += child.size[along_i];
                        out_size[across_i] += child.size[across_i];
                    }),
                    (true, false) => {
                        Box::new(|child: &Rect| out_size[along_i] += child.size[along_i])
                    }
                    (false, true) => {
                        Box::new(|child: &Rect| out_size[across_i] += child.size[across_i])
                    }
                    (false, false) => unreachable!(),
                };
                for child in children.iter().map(|child| self.out.get(child).unwrap()) {
                    (sizing_fn)(child);
                }
            }

            let p_out = self.out.get_mut(parent).unwrap();

            out_size.x += p_lay.pad.left + p_lay.pad.right;
            out_size.y += p_lay.pad.up + p_lay.pad.down;

            p_out.size = out_size;
        }
    }

    pub fn get_rect(&self, uid: &UID) -> Option<&Rect> {
        self.out.get(uid)
    }

    pub fn render_order<'a>(&'a self) -> DFSPost<'a> {
        self.tree.dfs_post()
    }

    pub fn insert(&mut self, uid: UID, lay: Lay, parent: Option<UID>) {
        self.lay.insert(uid, lay);
        self.out.insert(uid, Default::default());
        self.tree.insert(uid, parent);
    }
}
impl Delete for Layout {
    fn delete(&mut self, uid: &UID) {
        self.lay.remove(uid);
        self.out.remove(uid);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Size {
    Fixed(Pixels),
    Ratio(Precision),
}

#[derive(Debug, Clone, Copy)]
pub enum Sizing {
    /// Fit all children
    Fit,
    /// Fill space
    Fill,
    /// Fixed or relative size,
    Size(Size),
}

#[derive(Debug, Clone, Copy)]
pub struct AxisSize {
    pub min: Option<Size>,
    pub max: Option<Size>,
    pub sizing: Sizing,
}

#[derive(Debug, Clone, Copy)]
pub struct BoxSize {
    pub w: AxisSize,
    pub h: AxisSize,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Pad {
    pub up: Pixels,
    pub down: Pixels,
    pub left: Pixels,
    pub right: Pixels,
}

#[derive(Debug, Clone, Copy)]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    /// Left to right
    Right,
    /// Top to bottom
    Down,
}

#[derive(Debug, Clone, Copy)]
pub struct Lay {
    pub size: BoxSize,
    pub pad: Pad,
    pub gap: Pixels,
    pub align: Align,
    pub direction: Direction,
}

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

impl Default for Lay {
    fn default() -> Self {
        Self {
            size: BoxSize {
                w: AxisSize {
                    min: None,
                    max: None,
                    sizing: Sizing::Size(Size::Ratio(1.)),
                },
                h: AxisSize {
                    min: None,
                    max: None,
                    sizing: Sizing::Size(Size::Ratio(1.)),
                },
            },
            pad: Default::default(),
            gap: 0,
            align: Align::Left,
            direction: Direction::Down,
        }
    }
}
impl Lay {
    pub fn fix_w(mut self, w: Pixels) -> Self {
        self.size.w.sizing = Sizing::Size(Size::Fixed(w));
        self
    }
    pub fn fix_h(mut self, h: Pixels) -> Self {
        self.size.h.sizing = Sizing::Size(Size::Fixed(h));
        self
    }
    pub fn fix_size(mut self, w: Pixels, h: Pixels) -> Self {
        self.size.w.sizing = Sizing::Size(Size::Fixed(w));
        self.size.h.sizing = Sizing::Size(Size::Fixed(h));
        self
    }
}