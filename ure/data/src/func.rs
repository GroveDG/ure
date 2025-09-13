use std::{any::Any, collections::HashMap};

use anyhow::{Error, Result};
use nohash_hasher::BuildNoHashHasher;

use crate::Data;

pub type FuncId = u64;
pub type Implr = fn(&Data) -> Result<&'static dyn Any>;

pub type FuncAndImpl = (&'static Func, &'static dyn Any);

#[derive(Default)]
pub struct Functions {
    funcs: HashMap<FuncId, FuncAndImpl, BuildNoHashHasher<FuncId>>,
}

impl Functions {
    pub fn add(&mut self, data: &Data, func: &'static Func) -> Option<Error> {
        let i = match (func.implement)(data) {
            Ok(i) => i,
            Err(e) => return Some(e),
        };
        self.funcs.insert(func.id, (func, i));
        None
    }
    pub fn reimpl(&mut self, data: &Data) -> Option<Error> {
        for (func, i) in self.funcs.values_mut() {
            *i = match (func.implement)(data) {
                Ok(i) => i,
                Err(e) => return Some(e),
            };
        }
        None
    }
}

pub struct Func {
    pub(crate) name: &'static str,
    pub(crate) id: FuncId,
    pub(crate) implement: Implr,
}

impl Func {
    pub const fn new(name: &'static str, implr: Implr) -> Self {
        Self {
            name,
            id: const_fnv1a_hash::fnv1a_hash_str_64(name),
            implement: implr,
        }
    }
}