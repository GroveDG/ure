use std::mem::MaybeUninit;

use crate::{
    app::{Surface, Window},
    game::tf::Transform2D,
};

#[derive(Debug, Default)]
pub struct Data {
    pub window: Spanner<Window>,
    pub surface: Spanner<Surface>,
    #[cfg(feature = "2D")]
    pub transform_2d: Spanner<Transform2D>,
    #[cfg(feature = "3D")]
    pub transform_3d: Spanner<Transform3D>,
}
#[macro_export]
macro_rules! new_group {
    ($start_span:expr, $end_span:expr, $($component:ident)+) => {
        ($(data.$component.spans[$start_span ..= $end_span]),+)
    };
}

#[derive(Debug)] // Default impl manually
pub struct Spanner<T> {
    elements: Vec<MaybeUninit<T>>,
    spans: Vec<Span>,
}
impl<T> Spanner<T> {
    pub fn new(capacities: &[usize]) -> Self {
        let mut total_capacity = 0;
        let mut spans = Vec::with_capacity(capacities.len());
        for capacity in capacities {
            spans.push(Span::new(total_capacity));
            total_capacity += capacity;
        }
        let mut elements = Vec::with_capacity(total_capacity);
        unsafe {
            elements.set_len(total_capacity);
        }
        Self { elements, spans }
    }
    pub fn get_span(&self, span_index: usize) -> &[T] {
        let span = self.spans[span_index];
        unsafe { std::mem::transmute(&self.elements[span.position..span.position + span.length]) }
    }
    pub fn mut_span(&mut self, span_index: usize) -> &mut [T] {
        let span = self.spans[span_index];
        unsafe {
            std::mem::transmute(&mut self.elements[span.position..span.position + span.length])
        }
    }
    fn reserve(&mut self, additional: usize) {
        self.elements.reserve(additional);
        unsafe {
            self.elements.set_len(self.elements.len() + additional);
        }
    }
    pub fn new_span(&mut self) -> usize {
        self.spans.push(Span::new(self.elements.len()));
        self.spans.len() - 1
    }
    pub fn grow_span(&mut self, span_index: usize, additional: usize) {
        self.reserve(additional);
        if let Some(next_span) = self.spans.get(span_index + 1) {
            self.elements[next_span.position..].rotate_right(additional);
            for i in span_index + 1..self.spans.len() {
                self.spans[i].position += additional;
            }
        }
    }
    fn shrink(&mut self, reduce: usize) {
        self.elements.truncate(self.elements.len() - reduce);
    }
    pub fn shrink_span(&mut self, span_index: usize, reduce: usize) {
        if let Some(next_span) = self.spans.get(span_index + 1) {
            self.elements[next_span.position..].rotate_left(reduce);
            for i in span_index + 1..self.spans.len() {
                self.spans[i].position -= reduce;
            }
        }
        self.shrink(reduce);
    }
    pub fn compress_span(&mut self, span_index: usize) {
        let span = self.spans[span_index];
        let span_end = self
            .spans
            .get(span_index)
            .map_or(self.elements.len(), |s| s.position);
        let capacity = span_end - span.position;
        self.shrink_span(span_index, capacity - span.length);
    }
}

impl<T> Drop for Spanner<T> {
    fn drop(&mut self) {
        for span in &self.spans {
            for element in &mut self.elements[span.position..span.position + span.length] {
                unsafe {
                    element.assume_init_drop();
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Span {
    length: usize,
    position: usize,
}
impl Span {
    fn new(position: usize) -> Self {
        Self {
            length: 0,
            position,
        }
    }
}

impl<T> Default for Spanner<T> {
    fn default() -> Self {
        Self {
            elements: Default::default(),
            spans: Default::default(),
        }
    }
}
