use std::mem::MaybeUninit;

macro_rules! declare_components {
    ($($(#$attr:tt)? $component:ident : $t:ty,)+ $(,)?) => {
#[derive(Debug, Default)]
pub struct Data {
$(
    $(#$attr)?
    pub $component : Slicer<$t>,
)+}

#[derive(Debug, Default, Clone, Copy)]
pub struct Span {
    pub length: usize,
    $(
    $(#$attr)?
    pub $component : Option<usize>,
    )+
}
impl Span {
    pub fn grow_span(&self, data: &mut Data, additional: usize) {
        $(
        $(#$attr)?
        if let Some($component) = self.$component {
            data.$component.grow_slice($component, additional);
        }
        )+
    }
    pub fn shrink_span(&self, data: &mut Data, reduce: usize) {
        $(
        $(#$attr)?
        if let Some($component) = self.$component {
            data.$component.shrink_slice($component, reduce);
        }
        )+
    }
}

pub struct Group {
    pub lengths : Vec<usize>,
    $(
    $(#$attr)?
    pub $component : Option<Vec<usize>>,
    )+
}
impl Group {
    pub fn new(spans: &[Span]) -> Self {
        let mut lengths = Vec::new();
        $(
        $(#$attr)?
        let mut $component = Some(Vec::new());
        )+
        for span in spans {
            lengths.push(span.length);
            $(
            $(#$attr)?
            if let Some(ref mut grouped) = $component && let Some(index) = span.$component {
                grouped.push(index);
            } else {
                $component = None;
            }
            )+
        }
        Self {
            lengths,
            $(
            $(#$attr)?
            $component,
            )+
        }
    }
}

impl Drop for Data {
    fn drop(&mut self) {
        todo!()
    }
}
    };
}

declare_components! {
    window: crate::app::Window,
    surface: crate::gpu::Surface,
    #[cfg(feature = "2D")]
    transform_2d: crate::tf::Transform2D,
    #[cfg(feature = "3D")]
    transform_3d: crate::tf::Transform3D,
}

#[macro_export]
macro_rules! get_group {
    (get mut $component:ident, $data:expr, $group:expr) => {
$group.iter_mut($group.$component.as_ref().unwrap(), &mut $data.$component)
    };
    (get $component:ident, $data:expr, $group:expr) => {
$group.iter($group.$component.as_ref().unwrap(), &$data.$component)
    };
    ($data:expr, $group:expr, $(
        $component:ident $($mut:ident)?
    ),+) => {
$(
let mut $component = $crate::get_group!(get $($mut)? $component, $data, $group);
)+
    };
}

#[macro_export]
macro_rules! get_span {
    (get mut $component:ident, $data:expr, $span:expr) => {
$data.$component.get_mut_slice($span.$component.unwrap(), $span.length)
    };
    (get $component:ident, $data:expr, $span:expr) => {
$data.$component.get_slice($span.$component.unwrap(), $span.length)
    };
    ($data:expr, $span:expr, $(
        $component:ident $($mut:ident)?
    ),+) => {
$(
let mut $component = $crate::get_span!(get $($mut)? $component, $data, $span);
)+
    };
}

#[macro_export]
macro_rules! new_span {
    ($data:expr, $length:expr, $($component:ident),+) => {
$crate::data::Span {
    length: $length,
    $(
    $component : Some($data.$component.init_slice()),
    )+
    ..Default::default()
}
    };
}

#[macro_export]
macro_rules! extend_span {
    ($data:expr, $span:expr, $additional:expr, $($component:ident),+) => {
let length = $span.length;
$span.length += $additional;
$(
let $component = $data.$component.extend_slice($span.$component.unwrap(), length, $additional);
)+
    };
}

#[derive(Debug)] // Default impl manually
pub struct Slicer<T> {
    pub elements: Vec<MaybeUninit<T>>,
    pub positions: Vec<usize>,
}

impl<T> Slicer<T> {
    pub fn init_slice(&mut self) -> usize {
        self.positions.push(self.elements.len());
        self.positions.len() - 1
    }
    pub fn grow_slice(&mut self, index: usize, additional: usize) {
        self.elements.reserve(additional);
        unsafe {
            self.elements.set_len(self.elements.len() + additional);
        }

        let start = self.positions[index];
        self.elements[start..].rotate_right(additional);

        for position in &mut self.positions[index + 1..] {
            *position += additional;
        }
    }
    pub fn shrink_slice(&mut self, index: usize, reduce: usize) {
        self.elements.truncate(self.elements.len() - reduce);

        let start = self.positions[index];
        self.elements[start..].rotate_left(reduce);

        for position in &mut self.positions[index + 1..] {
            *position -= reduce;
        }
    }
    pub fn get_slice(&self, index: usize, length: usize) -> &[T] {
        let position = self.positions[index];
        unsafe { std::mem::transmute(&self.elements[position..position + length]) }
    }
    pub fn get_mut_slice(&mut self, index: usize, length: usize) -> &mut [T] {
        let position = self.positions[index];
        unsafe { std::mem::transmute(&mut self.elements[position..position + length]) }
    }
    pub fn extend_slice(
        &mut self,
        index: usize,
        length: usize,
        additional: usize,
    ) -> &mut [MaybeUninit<T>] {
        let position = self.positions[index] + length;
        let next_position = self
            .positions
            .get(index + 1)
            .copied()
            .unwrap_or(self.positions.len());
        if position + additional > next_position {
            self.grow_slice(index, next_position - position + additional);
        }
        &mut self.elements[position..position + additional]
    }
}
impl<T> Default for Slicer<T> {
    fn default() -> Self {
        Self {
            elements: Default::default(),
            positions: Default::default(),
        }
    }
}
impl Group {
    pub fn iter<'a, T>(
        &'a self,
        slice_indices: &'a [usize],
        components: &'a Slicer<T>,
    ) -> GroupIter<'a, T> {
        GroupIter::new(&self.lengths, slice_indices, components)
    }
    pub fn iter_mut<'a, T>(
        &'a self,
        slice_indices: &'a [usize],
        components: &'a mut Slicer<T>,
    ) -> GroupIterMut<'a, T> {
        GroupIterMut::new(&self.lengths, slice_indices, components)
    }
}
pub struct GroupIter<'a, T> {
    lengths: &'a [usize],
    slice_indices: &'a [usize],
    components: &'a Slicer<T>,
    g: usize,
}
impl<'a, T> GroupIter<'a, T> {
    fn new(lengths: &'a [usize], slice_indices: &'a [usize], components: &'a Slicer<T>) -> Self {
        Self {
            lengths,
            slice_indices,
            components,
            g: 0,
        }
    }
}
impl<'a, T> Iterator for GroupIter<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        let Some(slice_index) = self.slice_indices.get(self.g).copied() else {
            return None;
        };
        let length = self.lengths[slice_index];
        let position = self.components.positions[slice_index];
        self.g += 1;
        unsafe { std::mem::transmute(&self.components.elements[position..position + length]) }
    }
}
pub struct GroupIterMut<'a, T> {
    lengths: &'a [usize],
    slice_indices: &'a [usize],
    components: &'a mut Slicer<T>,
    g: usize,
}
impl<'a, T> GroupIterMut<'a, T> {
    fn new(
        lengths: &'a [usize],
        slice_indices: &'a [usize],
        components: &'a mut Slicer<T>,
    ) -> Self {
        Self {
            lengths,
            slice_indices,
            components,
            g: 0,
        }
    }
}
impl<'a, T> Iterator for GroupIterMut<'a, T> {
    type Item = &'a mut [T];

    fn next(&mut self) -> Option<Self::Item> {
        let Some(slice_index) = self.slice_indices.get(self.g).copied() else {
            return None;
        };
        let length = self.lengths[slice_index];
        let position = self.components.positions[slice_index];
        self.g += 1;
        unsafe { std::mem::transmute(&mut self.components.elements[position..position + length]) }
    }
}
