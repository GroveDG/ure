use std::{collections::VecDeque, sync::Mutex};

use crate::sys::UID;

pub struct Call<Args> {
    call: Box<dyn FnMut(&mut Vec<Args>)>,
    queue: Vec<Args>
}

impl<Args> Call<Args> {
    pub fn new(call: Box<dyn FnMut(&mut Vec<Args>)>) -> Self {
        Self {
            call,
            queue: Default::default(),
        }
    }
    pub fn add(&mut self, args: Args) {
        self.queue.push(args);
    }
    pub fn call(&mut self) {
        (self.call)(&mut self.queue);
    }
}

pub trait Delete {
    fn delete(&mut self, uid: &UID);
}