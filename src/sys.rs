use std::{
    collections::{HashMap, HashSet},
    fmt::Debug
};

use bimap::BiHashMap;
use nohash_hasher::BuildNoHashHasher;
use rand::{RngCore, rngs::ThreadRng};

use self::delete::Delete;

pub mod delete;

pub type Entities = HashSet<UID, BuildNoHashHasher<u64>>;
pub type Components<C> = HashMap<UID, C, BuildNoHashHasher<u64>>;
pub type BiComponents<C> = BiHashMap<UID, C, BuildNoHashHasher<u64>>;

pub type UID = u64;

#[derive(Debug, Clone)]
pub struct UIDs {
    ids: Entities,
    rng: ThreadRng,
}
impl UIDs {
    pub fn new() -> Self {
        Self {
            ids: Default::default(),
            rng: rand::rng(),
        }
    }

    pub fn add(&mut self) -> UID {
        let uid = self.rng.next_u64();
        while !self.ids.insert(uid) {}
        uid
    }
}

impl Delete for UIDs {
    fn delete(&mut self, uid: &UID) {
        self.ids.remove(uid);
    }
}
