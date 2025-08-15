use std::{
    any::Any,
    fmt::Debug,
    mem::ManuallyDrop,
    sync::{Arc, Weak},
};

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

// The following is heavily based off of the slotmap crate.

pub type Compartment = Option<Box<dyn Any>>;
pub type Element = Vec<Compartment>;
pub type Generation = u32;
pub type Index = usize;
pub type Key = (Generation, Index);

/// We don't store both the previously freed index and an
/// element at the same time.
union SlotData {
    prev_freed: Index,
    element: ManuallyDrop<Element>,
}

struct Slot {
    generation: Generation,
    data: SlotData,
}
impl Slot {
    #[inline]
    fn occupied(&self) -> bool {
        // if the generation is event it's empty
        // if the generation is odd it's occupied
        self.generation & 1 == 0
    }
    #[inline]
    fn new(element: Element) -> Self {
        Self {
            generation: 0,
            data: SlotData {
                element: ManuallyDrop::new(element),
            },
        }
    }
    #[inline]
    unsafe fn increment(&mut self) -> Generation {
        self.generation = self.generation.wrapping_add(1);
        self.generation
    }
    #[inline]
    // Does NOT return Key. These Generation and Index are unrelated.
    unsafe fn set(&mut self, element: Element) -> (Generation, Index) {
        let prev_freed = unsafe { self.data.prev_freed };
        self.data.element = ManuallyDrop::new(element);
        (unsafe { self.increment() }, prev_freed)
    }
    #[inline]
    fn take(&mut self, generation: Generation, prev_freed: Index) -> Option<Element> {
        if self.generation == generation {
            unsafe { self.increment() };
            let element = unsafe { ManuallyDrop::take(&mut self.data.element) };
            self.data.prev_freed = prev_freed;
            Some(element)
        } else {
            None
        }
    }
}
impl Drop for Slot {
    fn drop(&mut self) {
        if self.occupied() {
            unsafe { ManuallyDrop::drop(&mut self.data.element) }
        }
    }
}
impl Debug for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Slot")
            .field("generation", &self.generation)
            .field("data", unsafe {
                if self.occupied() {
                    &self.data.element
                } else {
                    &self.data.prev_freed
                }
            })
            .finish()
    }
}

/// General purpose data structure compromising between Data and Object Orientation.
/// 
/// Data is grouped into [Elements](Element). These are elements in the sense that they
/// aren't just entities. Elements are composed of [Compartments](Compartment) which are
/// a collection of components. Elements are designed to be *groups* not single entities.
/// 
/// Compartments are [`dyn Any`](Any) internally. This means they need to be downcast
/// when accessed.
/// - In object orientation, it is assumed that you know what types the compartments are.
/// - In data orientation, our primary iterator iterates through [discriminants](CompartmentDiscriminant).
/// Discriminants allow us to quickly filter out all Elements that do not contain a
/// particular component and then downcast when necessary.
/// 
/// In order to allow highly-dynamic programs (like editors), the component-wise axis is
/// a [Vec]. This means components should be enumerated to be given indices. These indices
/// are implictly associated with an instance of [Data] and it is an error to use it
/// elsewhere.
#[derive(Debug)]
pub struct Data {
    elements: Vec<Slot>, //                                 Element then Component
    discriminants: Vec<Vec<CompartmentDiscriminant>>, //    Component then Element
    first_open: usize,
    total_open: usize,
}

impl Data {
    pub fn new(num_components: usize) -> Self {
        Self {
            elements: Default::default(),
            discriminants: vec![Default::default(); num_components],
            first_open: Default::default(),
            total_open: Default::default(),
        }
    }

    pub fn push(&mut self, element: Element, discriminants: &[CompartmentDiscriminant]) -> Key {
        debug_assert!(self.discriminants.len() == discriminants.len());
        debug_assert!(self.discriminants.len() == element.len());

        // If there are no empty slots to fill...
        if self.total_open == 0 {
            let index = self.elements.len(); //     New element is last
            self.elements.push(Slot::new(element)); //     Push the element
            // Push the discriminants
            for (c, discriminant) in discriminants.iter().enumerate() {
                self.discriminants[c].push(*discriminant);
            }
            (0, index) // New slots are generation 0
        } else {
            // If there are empty slots to fill...
            self.total_open -= 1; //                       Take an empty slot
            let index = self.first_open; //         Use the slot we know is empty
            // Fill the slot and take its information.
            let (generation, prev_freed) = unsafe { self.elements[index].set(element) };
            self.first_open = prev_freed; // Set the known empty slot to the one freed before this one
            // Set the discriminants
            for (c, discriminant) in discriminants.iter().enumerate() {
                self.discriminants[c][index] = *discriminant;
            }
            (generation, index)
        }
    }

    pub fn remove_element(&mut self, key: Key) -> Option<Element> {
        let (generation, index) = key;
        let element = self.elements[index].take(generation, self.first_open);
        if element.is_some() {
            // If the element was there...
            self.total_open += 1; //        Add an empty slot
            self.first_open = index; //     Remember this empty slot
        }
        element
    }
}

#[derive(Debug, Clone, Copy, Hash)]
#[repr(u8)]
pub enum CompartmentDiscriminant {
    None,
    One,
    Vec,
    Array,
}
