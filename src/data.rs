use std::mem::MaybeUninit;

use crate::{
    app::{Surface, Window},
    game::tf::Transform2D,
};

#[derive(Debug, Default)]
pub struct Data {
    pub window: Grouper<Window>,
    pub surface: Grouper<Surface>,
    #[cfg(feature = "2D")]
    pub transform_2d: Grouper<Transform2D>,
    #[cfg(feature = "3D")]
    pub transform_3d: Grouper<Transform3D>,
}

#[derive(Debug)] // Default impl manually
pub struct Grouper<T> {
    elements: Vec<MaybeUninit<T>>,
    groups: Vec<Group>,
}
impl<T> Grouper<T> {
    pub fn new(capacities: &[usize]) -> Self {
        let mut groups = Vec::with_capacity(capacities.len());
        let mut total_capacity = 0;
        for capacity in capacities {
            groups.push(Group::new(*capacity, total_capacity));
            total_capacity += capacity;
        }
        Self {
            elements: Vec::with_capacity(total_capacity),
            groups,
        }
    }
    pub fn get_group(&self, group_index: usize) -> &[T] {
        let group = self.groups[group_index];
        (&self.elements[group.position .. group.position + group.length])
    }
    pub fn mut_group(&mut self, group_index: usize) -> &mut [T] {
        let group = self.groups[group_index];
        &mut self.elements[group.position .. group.position + group.length]
    }
    pub fn new_group(&mut self, capacity: usize) -> usize {
        self.groups.push(Group::new(capacity, self.elements.len()));
        self.elements.reserve(capacity);
        self.groups.len() - 1
    }
    pub fn grow_group(&mut self, group_index: usize, additional: usize) -> usize {
        self.elements.reserve(additional);
        self.elements.
        self.groups[group_index].capacity += additional;
        self.groups[]
        for i in group_index..self.groups.len() {
            self.groups[i].position += additional;
        }
        self.elements.reserve(capacity);
        self.groups.len() - 1
    }
}

#[derive(Debug, Clone, Copy)]
struct Group {
    length: usize,
    capacity: usize,
    position: usize,
}
impl Group {
    fn new(capacity: usize, position: usize) -> Self {
        Self {
            length: 0,
            capacity,
            position,
        }
    }
}





impl<T> Default for Grouper<T> {
    fn default() -> Self {
        Self {
            elements: Default::default(),
            groups: Default::default(),
        }
    }
}