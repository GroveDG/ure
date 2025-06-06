//! Clay-inspired ui layout
//!
//!

use glam::{Mat3, Vec2};

use crate::render::_2d::{Draw2D, Instance2D};
use crate::render::gpu::{Color, Pixels};
use crate::sys::UIDs;
use crate::sys::{Components, UID, delete::Delete};

use super::{
    tf::Precision,
    tree::{DFSPost, Tree},
};

/// Layout is not Rendering, the two passes cannot happen at once.
///
#[derive(Debug)]
pub struct Layout {
    tree: Tree,
    layout: Components<(Lay, Rect)>,
    quad: UID,
    instances: UID,
    change: bool,
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
    https://youtu.be/by9lQvpvMIc */

    pub fn new(quad: UID, uids: &mut UIDs) -> Self {
        Self {
            tree: Default::default(),
            layout: Default::default(),
            quad,
            instances: uids.add(),
            change: true,
        }
    }

    pub fn run(&mut self, draw_2d: &Draw2D) {
        if !self.change {
            return;
        }

        for parent in self.tree.dfs_post() {
            let (lay, out) = self.layout.get(parent).unwrap();
            let children = self.tree.get_children(Some(parent)).unwrap();

            let (along_i, across_i) = match lay.direction {
                Direction::Right => (0, 1),
                Direction::Down => (1, 0),
            };
            let mut fit = [false, false];
            let mut out_size: Vec2 = Vec2::ZERO;

            match lay.size.w.sizing {
                Sizing::Fit => fit[0] = true,
                Sizing::Fill => todo!(),
                Sizing::Size(size) => match size {
                    Size::Fixed(x) => out_size.x = x,
                    _ => {}
                },
            }
            match lay.size.h.sizing {
                Sizing::Fit => fit[1] = true,
                Sizing::Fill => todo!(),
                Sizing::Size(size) => match size {
                    Size::Fixed(y) => out_size.y = y,
                    _ => {}
                },
            }

            if fit[0] || fit[1] {
                for (_, out) in children.iter().map(|child| self.layout.get(child).unwrap()) {
                    if fit[along_i] {
                        out_size[along_i] += out.size[along_i]
                    }
                    if fit[across_i] {
                        out_size[across_i] += out.size[across_i]
                    }
                }
            }

            let (lay, out) = self.layout.get_mut(parent).unwrap();

            out_size.x += lay.pad.left + lay.pad.right;
            out_size.y += lay.pad.up + lay.pad.down;

            out.size = out_size;
        }

        let instances: Vec<_> = self
            .tree
            .dfs_pre()
            .filter_map(|uid| self.layout.get(uid))
            .map(|(lay, out)| Instance2D {
                tf: Mat3::from_scale_angle_translation(out.size, 0.0, out.pos + out.size / 2.0),
                color: Color::WHITE,
            })
            .collect();
        draw_2d.update_instances(self.instances, instances);

        self.change = false;
    }

    pub fn draw(&self, draw_2d: &Draw2D) {
        draw_2d.mesh(self.quad);
        draw_2d.instances(self.instances);
        draw_2d.draw();
    }

    pub fn insert(&mut self, uid: UID, lay: Lay, parent: Option<UID>, index: Option<usize>) {
        self.layout.insert(uid, (lay, Default::default()));
        self.tree.parent(uid, parent, index);
        self.change = true;
    }

    pub fn get_mut(&mut self, uid: &UID) -> Option<&mut Lay> {
        self.change = true;
        self.layout.get_mut(uid).map(|layout| &mut layout.0)
    }
}
impl Delete for Layout {
    fn delete(&mut self, uid: &UID) {
        self.layout.remove(uid);
        self.tree.delete(uid);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Size {
    Fixed(Pixels),
    Percent(Precision),
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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub pos: Vec2,
    pub size: Vec2,
}

impl Default for Lay {
    fn default() -> Self {
        Self {
            size: BoxSize {
                w: AxisSize {
                    min: None,
                    max: None,
                    sizing: Sizing::Size(Size::Percent(1.)),
                },
                h: AxisSize {
                    min: None,
                    max: None,
                    sizing: Sizing::Size(Size::Percent(1.)),
                },
            },
            pad: Default::default(),
            gap: 0.0,
            align: Align::Left,
            direction: Direction::Down,
        }
    }
}
impl Lay {
    pub fn fix_w(&mut self, w: Pixels) {
        self.size.w.sizing = Sizing::Size(Size::Fixed(w));
    }
    pub fn fix_h(&mut self, h: Pixels) {
        self.size.h.sizing = Sizing::Size(Size::Fixed(h));
    }
    pub fn fix_size(&mut self, w: Pixels, h: Pixels) {
        self.fix_w(w);
        self.fix_h(h);
    }
}
