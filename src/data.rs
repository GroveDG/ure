use std::mem::MaybeUninit;

use crate::{
    app::{Surface, Window},
    game::tf::Transform2D,
};

macro_rules! declare_components {
    ($($(#$attr:tt)? $component:ident : $t:ty,)+ $(,)?) => {
#[derive(Debug, Default)]
pub struct Data {
    pub lengths: Vec<usize>,
$(
    $(#$attr)?
    pub $component : Spanner<$t>,
)+}

impl Data {
    pub fn init_span(&mut self) -> usize {
        self.lengths.push(0);
        $(
        $(#$attr)?
        self.$component.init_span();
        )+
        self.lengths.len() - 1
    }
}

impl Drop for Data {
    fn drop(&mut self) {
        todo!()
    }
}
    };
}

#[macro_export]
macro_rules! group {
    [$($span_index:expr),+ $(,)?] => {
ure::data::Group(vec![$($span_index),+])
    };
}

declare_components! {
    window: Window,
    surface: Surface,
    #[cfg(feature = "2D")]
    transform_2d: Transform2D,
    #[cfg(feature = "3D")]
    transform_3d: Transform3D,
}

#[macro_export]
macro_rules! get_group {
    (get mut $component:ident, $data:expr, $group:expr) => {
$group.iter_mut(&mut $data.$component, &$data.lengths)
    };
    (get $component:ident, $data:expr, $group:expr) => {
$group.iter(&$data.$component, &$data.lengths)
    };
    ($data:expr, $group:expr, $(
        $component:ident $($mut:ident)?
    ),+) => {
$(
let mut $component = ure::get_group!(get $($mut)? $component, $data, $group);
)+
    };
}

#[macro_export]
macro_rules! new_span {
    ($data:expr, $length:expr, $($component:ident),+) => {
{
let span_index = $data.init_span();
ure::grow_span!($data, span_index, $length, $($component),+);
span_index
}
    };
}

#[macro_export]
macro_rules! grow_span {
    ($data:expr, $span_index:expr, $additional:expr, $($component:ident),+) => {
$(
let positions = &mut $data.$component.positions;
let start = positions[$span_index + 1];
for g in $span_index + 1 .. positions.len() {
    positions[g] += $additional;
}
ure::data::reserve(&mut $data.$component.elements, $additional);
$data.$component.elements[start..].rotate_right($additional);
)+
    };
}

#[macro_export]
macro_rules! shrink_span {
    ($data:expr, $span_index:expr, $($component:ident),+, $reduce:expr) => {
$(
let positions = &mut $data.$component.positions;
let start = positions[$span_index + 1];
for g in $span_index + 1 .. positions.len() {
    positions[g] -= $reduce;
}
$data.$component.elements[start..].rotate_left($reduce);
ure::data::shrink(&mut $data.$component.elements, $reduce);
)+
    };
}

pub fn reserve<T>(v: &mut SparseVec<T>, additional: usize) {
    v.reserve(additional);
    unsafe {
        v.set_len(v.len() + additional);
    }
}
pub fn shrink<T>(v: &mut SparseVec<T>, reduce: usize) {
    v.truncate(v.len() - reduce);
}

type SparseVec<T> = Vec<MaybeUninit<T>>;
#[derive(Debug)] // Default impl manually
pub struct Spanner<T> {
    pub elements: SparseVec<T>,
    pub positions: Vec<usize>,
}
impl<T> Default for Spanner<T> {
    fn default() -> Self {
        Self { elements: Default::default(), positions: Default::default() }
    }
}
impl<T> Spanner<T> {
    pub fn init_span(&mut self) {
        self.positions.push(self.elements.len());
    }
}

#[repr(transparent)]
pub struct Group(pub Vec<usize>); // A sequence of span indices
impl Group {
    pub fn iter<'a, T>(
        &'a self,
        components: &'a Spanner<T>,
        lengths: &'a Vec<usize>,
    ) -> GroupIter<'a, T> {
        GroupIter::new(self, components, lengths)
    }
    pub fn iter_mut<'a, T>(
        &'a self,
        components: &'a mut Spanner<T>,
        lengths: &'a Vec<usize>,
    ) -> GroupIterMut<'a, T> {
        GroupIterMut::new(self, components, lengths)
    }
}
pub struct GroupIter<'a, T> {
    group: &'a Group,
    components: &'a Spanner<T>,
    lengths: &'a Vec<usize>,
    g: usize,
}
impl<'a, T> GroupIter<'a, T> {
    fn new(group: &'a Group, components: &'a Spanner<T>, lengths: &'a Vec<usize>) -> Self {
        Self { group, components, lengths, g: 0 }
    }
}
impl<'a, T> Iterator for GroupIter<'a, T> {
    type Item = &'a [T];
    
    fn next(&mut self) -> Option<Self::Item> {
        let Some(span_index) = self.group.0.get(self.g).copied() else {
            return None;
        };
        let length = self.lengths[span_index];
        let position = self.components.positions[span_index];
        self.g += 1;
        unsafe {
            std::mem::transmute(&self.components.elements[position .. position + length])
        }
    }
}
pub struct GroupIterMut<'a, T> {
    group: &'a Group,
    components: &'a mut Spanner<T>,
    lengths: &'a Vec<usize>,
    g: usize,
}
impl<'a, T> GroupIterMut<'a, T> {
    fn new(group: &'a Group, components: &'a mut Spanner<T>, lengths: &'a Vec<usize>) -> Self {
        Self { group, components, lengths, g: 0 }
    }
}
impl<'a, T> Iterator for GroupIterMut<'a, T> {
    type Item = &'a [T];
    
    fn next(&mut self) -> Option<Self::Item> {
        let Some(span_index) = self.group.0.get(self.g).copied() else {
            return None;
        };
        let length = self.lengths[span_index];
        let position = self.components.positions[span_index];
        self.g += 1;
        unsafe {
            std::mem::transmute(&mut self.components.elements[position .. position + length])
        }
    }
}
