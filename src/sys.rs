use std::{
    collections::{HashMap, HashSet},
    u64,
};

use rand::{
    RngCore,
    rngs::ThreadRng,
};

pub type UID = u64;
pub type Components<C> = HashMap<UID, C>;

#[derive(Debug, Clone)]
pub struct UIDs {
    entities: HashSet<UID>,
    rng: ThreadRng,
}
impl UIDs {
    pub fn new() -> Result<Self, rand::distr::uniform::Error> {
        Ok(Self {
            entities: Default::default(),
            rng: rand::rng(),
        })
    }

    pub fn new_uid(&mut self) -> UID {
        let uid = self.rng.next_u64();
        while !self.entities.insert(uid) {}
        uid
    }
}

pub mod tf;
pub mod tree;