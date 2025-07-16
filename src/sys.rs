use std::{
    collections::{HashMap, HashSet},
    fmt::Debug
};

use bimap::BiHashMap;
use nohash_hasher::BuildNoHashHasher;
use rand::{RngCore, rngs::ThreadRng};

use self::delete::Delete;

pub mod delete;

pub type Entities = HashSet<Uid, BuildNoHashHasher<u64>>;
pub type Components<C> = HashMap<Uid, C, BuildNoHashHasher<u64>>;
pub type BiComponents<C> = BiHashMap<Uid, C, BuildNoHashHasher<u64>>;

pub type Uid = u64;

#[derive(Debug, Clone)]
pub struct Uids {
    ids: Entities,
    rng: ThreadRng,
}
impl Uids {
    pub fn new() -> Self {
        Self {
            ids: Default::default(),
            rng: rand::rng(),
        }
    }

    pub fn add(&mut self) -> Uid {
        let uid = self.rng.next_u64();
        while !self.ids.insert(uid) {}
        uid
    }
}

impl Delete for Uids {
    fn delete(&mut self, uid: &Uid) {
        self.ids.remove(uid);
    }
}
