use std::sync::{Arc, Weak};

use parking_lot::Mutex;

pub struct Resource<T, F = fn() -> T> {
    weak: Mutex<Weak<T>>,
    f: F,
}
impl<T, F: Fn() -> T> Resource<T, F> {
    pub const fn new(f: F) -> Self {
        Self {
            weak: Mutex::new(Weak::new()),
            f,
        }
    }
    pub fn load(&'static self) -> Arc<T> {
        let mut lock = self.weak.lock();
        if let Some(arc) = lock.upgrade() {
            return arc;
        }
        let arc = Arc::new((self.f)());
        *lock = Arc::downgrade(&arc);
        arc
    }
}

#[macro_export]
macro_rules! declare_components {
    ($($(#$attr:tt)? $component:ident : $t:ty,)+ $(,)?) => {
#[derive(Debug, Default)]
pub struct Data {
    pub spans: Vec<Span>,
$(
    $(#$attr)?
    pub $component : $crate::data::Slicer<$t>,
)+}

impl Data {
    pub fn new_span(&mut self, mask: SpanMask) -> usize {
        let length = self.spans.len();
        self.spans.push(Span {
            length: 0,
            $(
            $(#$attr)?
            $component: if mask.$component { Some(self.$component.init_slice()) } else { None },
            )+
        });
        length
    }
    pub fn grow_span(&mut self, span_index: usize, additional: usize) {
        let span = self.spans[span_index];
        $(
        $(#$attr)?
        if let Some(slice_index) = span.$component {
            self.$component.grow_slice(slice_index, additional);
        }
        )+
    }
    pub fn shrink_span(&mut self, span_index: usize, reduction: usize) {
        let span = self.spans[span_index];
        $(
        $(#$attr)?
        if let Some(slice_index) = span.$component {
            self.$component.shrink_slice(slice_index, reduction);
        }
        )+
    }
    pub fn get_span<'a>(&'a self, span_index: usize) -> SpanRef<'a> {
        let span = self.spans[span_index];
        SpanRef {
        $(
        $(#$attr)?
        $component: span.$component.map(|slice_index| {self.$component.get_slice(slice_index, span.length)}),
        )+
        }
    }
    pub fn get_mut_span<'a>(&'a mut self, span_index: usize) -> SpanMut<'a> {
        let span = self.spans[span_index];
        SpanMut {
        $(
        $(#$attr)?
        $component: span.$component.map(|slice_index| {self.$component.get_mut_slice(slice_index, span.length)}),
        )+
        }
    }
    pub fn extend_span<'a>(&'a mut self, span_index: usize, amount: usize) -> SpanInit<'a> {
        let span = self.spans[span_index];
        $(
        $(#$attr)?
        let $component = span.$component.map(|slice_index| {self.$component.extend_slice(slice_index, span.length, amount)});
        )+
        self.spans[span_index].length += amount;
        SpanInit {
            $(
            $(#$attr)?
            $component,
            )+
        }
    }
    pub fn init_span<'a>(&'a mut self, mask: SpanMask, amount: usize, init: impl FnOnce(SpanInit)) -> usize {
        let span_index = self.new_span(mask);
        self.grow_span(span_index, amount);
        let span = self.spans[span_index];
        $(
        $(#$attr)?
        let $component = span.$component.map(|slice_index| {self.$component.extend_slice(slice_index, span.length, amount)});
        )+
        self.spans[span_index].length += amount;
        init(SpanInit {
            $(
            $(#$attr)?
            $component,
            )+
        });
        span_index
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SpanMask {
    $(
    $(#$attr)?
    pub $component : bool,
    )+
}
impl SpanMask {
    pub const NONE: Self = Self {
        $(
        $(#$attr)?
        $component : false,
        )+
    };
    pub const fn merge(self, other: Self) -> Self {
        Self {
            $(
            $(#$attr)?
            $component : self.$component || other.$component,
            )+
        }
    }
}
#[derive(Debug)]
pub struct SpanRef<'a> {
    $(
    $(#$attr)?
    pub $component : Option<&'a [$t]>,
    )+
}
#[derive(Debug)]
pub struct SpanMut<'a> {
    $(
    $(#$attr)?
    pub $component : Option<&'a mut [$t]>,
    )+
}
#[derive(Debug)]
pub struct SpanInit<'a> {
    $(
    $(#$attr)?
    pub $component : Option<&'a mut [std::mem::MaybeUninit<$t>]>,
    )+
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Span {
    pub length: usize,
    $(
    $(#$attr)?
    pub $component : Option<usize>,
    )+
}

impl Drop for Data {
    fn drop(&mut self) {
        for span in self.spans.iter() {
            let length = span.length;
            $(
            $(#$attr)?
            if let Some(index) = span.$component {
                for component in self.$component.get_mut_slice(index, length) {
                    unsafe {
                        std::ptr::drop_in_place(component as *mut $t);
                    }
                }
            }
            )+
        }
    }
}
    };
}

#[derive(Debug)] // Default impl manually
pub struct Slicer<T> {
    pub elements: Vec<std::mem::MaybeUninit<T>>,
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
    ) -> &mut [std::mem::MaybeUninit<T>] {
        let position = self.positions[index] + length;
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
