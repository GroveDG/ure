//! Implementation of [SwissTable](https://www.youtube.com/watch?v=ncHmEUmJZf4) without hashing.

use std::{
    alloc::{Allocator, Global, Layout}, arch::x86_64::{__m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8}, fmt::Debug, ptr::NonNull
};

use crate::sys::UID;

pub struct Table<A: Allocator = Global> {
    ptr: NonNull<[u8]>,
    capacity: usize,
    allocator: A,
}

impl Table<Global> {
    pub fn new(capacity: usize) -> Self {
        Self::new_in(Global, capacity)
    }
}

impl<A: Allocator> Table<A> {
    const GROUP_MASK: u16 = 0b11111111_11110000;
    const H2_MASK: i8 = 0b01111111;

    const EMPTY: i8 = 0b00000000;
    const DELETED: i8 = 0b01111111;

    #[inline]
    fn sections(&self) -> (&[i8], &[u16]) {
        unsafe {
            let slice = self.ptr.as_ref();
            let (control, keys) = slice.split_at_unchecked(self.capacity);
            (std::mem::transmute(control), std::mem::transmute(keys))
        }
    }

    #[inline]
    fn sections_mut(&mut self) -> (&mut [i8], &mut [u16]) {
        unsafe {
            let slice = self.ptr.as_mut();
            let (control, keys) = slice.split_at_mut_unchecked(self.capacity);
            (std::mem::transmute(control), std::mem::transmute(keys))
        }
    }

    #[inline]
    fn layout(capacity: usize) -> Layout {
        let capacity = capacity.next_power_of_two().max(16);
        Layout::from_size_align(capacity * 3, 2).unwrap()
    }

    #[inline]
    fn h1_index(&self, h1: u16) -> usize {
        (h1 & Self::GROUP_MASK) as usize & (self.capacity - 1)
    }

    pub fn new_in(allocator: A, capacity: usize) -> Self {
        let ptr = allocator.allocate_zeroed(Self::layout(capacity)).unwrap();
        Self {
            allocator,
            capacity,
            ptr,
        }
    }

    #[inline]
    fn control_group(control: &[i8]) -> __m128i {
        unsafe {
            _mm_loadu_si128(control.as_ptr().cast())
        }
    }

    #[inline]
    fn mask(control: __m128i, value: i8) -> u16 {
        unsafe {
            let idx8 = _mm_set1_epi8(value);
            _mm_movemask_epi8(_mm_cmpeq_epi8(idx8, control)) as u16
        }
    }

    #[inline]
    fn find(control: __m128i, keys: &[u16], h1: u16, h2: i8) -> Option<usize> {
        let mut bitmask = Self::mask(control, h2);
        while bitmask != 0 {
            let i = bitmask.trailing_zeros() as usize;
            if std::intrinsics::likely(keys[i] == h1) {
                return Some(i);
            }
            bitmask &= bitmask - 1;
        }
        None
    }

    #[inline]
    fn find_control(control: __m128i) -> Option<usize> {
        let bitmask = !unsafe {
            _mm_movemask_epi8(control) as u32
        };
        while bitmask != 0 {
            let i = bitmask.trailing_zeros() as usize;
            return Some(i);
        }
        None
    }

    pub fn get(&self, uid: UID) -> Option<usize> {
        let (h1, h2) = (uid.0, uid.1);
        let (control, keys) = self.sections();

        let mut i = (h1 as usize) & (self.capacity - 1);
        loop {
            let (control, keys) = (&control[i..i + 16], &keys[i..i + 16]);
            let control = Self::control_group(control);

            let result = Self::find(control, keys, h1, h2);
            if std::intrinsics::likely(result.is_some() || Self::mask(control, Self::EMPTY) != 0) {
                return result;
            }

            i = (i + 1) & self.capacity;
        }
    }

    pub fn insert(&mut self, uid: UID) -> Option<usize> {
        let (h1, h2) = (uid.0, uid.1);
        let capacity = self.capacity;
        let (control, keys) = self.sections_mut();
        let mut i = (h1 as usize) & (capacity - 1);

        loop {
            let (control, keys) = (&mut control[i..i + 16], &mut keys[i..i + 16]);
            let control_group = Self::control_group(control);

            let result = Self::find(control_group, keys, h1, h2);
            if std::intrinsics::likely(result.is_some()) {
                return result;
            }
            let result = Self::find_control(control_group);
            if std::intrinsics::likely(result.is_some()) {
                let i = unsafe {
                    result.unwrap_unchecked()
                };
                control[i] = h2;
                return result;
            }

            i = (i + 1) & capacity;
        }
    }
}

impl<A: Allocator> Drop for Table<A> {
    fn drop(&mut self) {
        unsafe {
            self.allocator
                .deallocate(self.ptr.cast(), Self::layout(self.capacity));
        }
    }
}

impl<A: Allocator> Debug for Table<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}