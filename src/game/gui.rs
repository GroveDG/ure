//! Clay-inspired ui layout
//!
//!

use fontdue::layout::Layout as TextLayout;
use glam::{Mat3, Vec2};

use crate::render::_2d::{Draw2DPass, Draw2DUpdate, Instance2D};
use crate::render::Color;
use crate::sys::UIDs;
use crate::sys::{Components, Uid, delete::Delete};

use super::{tf::Precision, tree::Tree};

pub struct Layout {
    tree: Tree,
    layout: Components<(Lay, Rect)>,
    style: Components<Style>,
    text: Components<Text>,
    quad: Uid,
    instances: Uid,
    changed: bool,
    text_layout: TextLayout,
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

    pub fn new(quad: Uid, uids: &mut UIDs) -> Self {
        Self {
            tree: Default::default(),
            layout: Default::default(),
            style: Default::default(),
            text: Default::default(),
            quad,
            instances: uids.add(),
            changed: true,
            text_layout: TextLayout::new(fontdue::layout::CoordinateSystem::PositiveYUp),
        }
    }

    fn fit_sizing(&mut self, axis: usize) {
        for parent in self.tree.dfs_post() {
            let (lay, _) = self.layout.get(parent).unwrap();
            let children = self.tree.get_children(Some(parent)).unwrap();

            let along = lay.direction.axis(axis);
            let mut out_size = 0.0;

            match lay.size.axis(axis).sizing {
                Sizing::Fit => {
                    if along {
                        for (_, out) in children.iter().map(|child| self.layout.get(child).unwrap())
                        {
                            out_size += out.size[axis];
                        }
                        out_size += children.len().saturating_sub(1) as f32 * lay.gap;
                    } else {
                        for (_, out) in children.iter().map(|child| self.layout.get(child).unwrap())
                        {
                            out_size += out_size.max(out.size[axis]);
                        }
                    }
                }
                Sizing::Fill => {} // Seperate pass
                Sizing::Size(size) => match size {
                    Size::Fixed(v) => out_size = v,
                    _ => {}
                },
            }

            let (lay, out) = self.layout.get_mut(parent).unwrap();

            out_size += lay.pad.size()[axis];

            out.size[axis] = out_size;
        }
    }

    fn fill_sizing(&mut self, axis: usize) {
        for parent in self.tree.bfs() {
            let (lay, out) = self.layout.get(&parent).unwrap();

            let Some(children) = self.tree.get_children(Some(&parent)) else {
                continue;
            };

            let along = lay.direction.axis(axis);
            let mut remaining = out.size[axis] - lay.pad.size()[axis];
            if along {
                remaining -= children.len().saturating_sub(1) as f32 * lay.gap;
            }
            let mut fill_children = Vec::with_capacity(children.len());
            for child in children {
                let (lay, out) = self.layout.get(child).unwrap();
                if lay.size.axis(axis).sizing == Sizing::Fill {
                    fill_children.push(child);
                }
                remaining -= out.size[axis];
            }

            if along && !fill_children.is_empty() {
                while remaining > 0.0 {
                    let mut smallest = f32::INFINITY;
                    let mut next_smallest = f32::INFINITY;
                    let mut add_width = remaining;
                    for child in fill_children.iter().copied() {
                        let (_, out) = self.layout.get_mut(child).unwrap();
                        if out.size[axis] < smallest {
                            next_smallest = smallest;
                            smallest = out.size[axis];
                        } else {
                            next_smallest = next_smallest.min(out.size[axis]);
                            add_width = next_smallest - smallest;
                        }
                        out.size[axis] = remaining / fill_children.len() as f32;
                    }
                    // Clamp added width
                    add_width = add_width.min(remaining / fill_children.len() as f32);
                    for child in fill_children.iter().copied() {
                        let (_, out) = self.layout.get_mut(child).unwrap();
                        if out.size[axis] == smallest {
                            out.size[axis] += add_width;
                            remaining -= add_width;
                        }
                    }
                }
            } else {
                for child in fill_children {
                    let (_, out) = self.layout.get_mut(child).unwrap();
                    out.size[axis] += (remaining - out.size[axis]).max(0.0);
                }
            }
        }
    }

    pub fn run(&mut self, draw_2d: &mut Draw2DUpdate) {
        if !self.changed {
            return;
        }

        // Fit Sizing Widths
        self.fit_sizing(0);

        // Fill Sizing Widths
        self.fill_sizing(0);

        // Fit Sizing Heights
        self.fit_sizing(1);

        // Fill Sizing Heights
        self.fill_sizing(1);

        let mut instances = Vec::new();
        for uid in self.tree.dfs_pre() {
            let Some(style) = self.style.get(uid) else {
                continue;
            };
            let (_, out) = self.layout.get(uid).unwrap();
            let tf = Mat3::from_scale_angle_translation(out.size, 0.0, out.pos);
            if let Some(color) = style.color {
                instances.push(Instance2D { tf, color });
            }
        }
        draw_2d.instances(self.instances, instances);

        self.changed = false;
    }

    pub fn draw(&self, draw_2d: &mut Draw2DPass) {
        draw_2d.mesh(self.quad);
        draw_2d.instances(self.instances);
        draw_2d.draw();
    }

    pub fn insert(
        &mut self,
        uid: Uid,
        lay: Lay,
        style: Option<Style>,
        text: Option<Text>,
        parent: Option<Uid>,
        index: Option<usize>,
    ) {
        self.layout.insert(uid, (lay, Default::default()));
        if let Some(style) = style {
            self.style.insert(uid, style);
        }
        if let Some(text) = text {
            self.text.insert(uid, text);
        }
        self.tree.parent(uid, parent, index);
        self.changed = true;
    }

    pub fn get_mut(&mut self, uid: &Uid) -> Option<&mut Lay> {
        self.changed = true;
        self.layout.get_mut(uid).map(|layout| &mut layout.0)
    }
}
impl Delete for Layout {
    fn delete(&mut self, uid: &Uid) {
        self.layout.remove(uid);
        self.style.remove(uid);
        self.text.remove(uid);
        self.tree.delete(uid);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    Fixed(f32),
    Percent(Precision),
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
impl BoxSize {
    pub fn axis(&self, axis: usize) -> AxisSize {
        match axis {
            0 => self.w,
            1 => self.h,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Pad {
    pub up: f32,
    pub down: f32,
    pub left: f32,
    pub right: f32,
}
impl Pad {
    pub fn size(&self) -> Vec2 {
        Vec2 {
            x: self.left + self.right,
            y: self.up + self.down,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub struct Text {
    pub align: Align,
    pub text: String,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    /// Left to right
    Right,
    /// Top to bottom
    Down,
}
impl Direction {
    pub fn axis(self, axis: usize) -> bool {
        match self {
            Self::Right => axis == 0,
            Self::Down => axis == 1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Lay {
    pub size: BoxSize,
    pub pad: Pad,
    pub gap: f32,
    pub direction: Direction,
}

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub color: Option<Color>,
    pub border: Option<Border>,
}

#[derive(Debug, Clone, Copy)]
pub struct Border {
    width: f32,
    color: Color,
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
            direction: Direction::Down,
        }
    }
}
impl Lay {
    pub fn fix_w(&mut self, w: f32) {
        self.size.w.sizing = Sizing::Size(Size::Fixed(w));
    }
    pub fn fix_h(&mut self, h: f32) {
        self.size.h.sizing = Sizing::Size(Size::Fixed(h));
    }
    pub fn fix_size(&mut self, w: f32, h: f32) {
        self.fix_w(w);
        self.fix_h(h);
    }
    pub fn fill_w(&mut self) {
        self.size.w.sizing = Sizing::Fill;
    }
    pub fn fill_h(&mut self) {
        self.size.h.sizing = Sizing::Fill;
    }
    pub fn fill(&mut self) {
        self.fill_w();
        self.fill_h();
    }
}
