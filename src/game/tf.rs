use crate::{game::tree::Tree, sys::{delete::Delete, Components, Uid}};



pub type Precision = f32;

#[derive(Debug)]
pub struct Space<M> {
    pub transforms: Components<M>,
    pub tree: Tree,
}
impl<M> Default for Space<M> {
    fn default() -> Self {
        Self {
            transforms: Default::default(),
            tree: Default::default(),
        }
    }
}
impl<M> Space<M> {
    pub fn insert(&mut self, uid: Uid, matrix: M, parent: Option<Uid>, index: Option<usize>) {
        self.transforms.insert(uid, matrix);
        self.tree.parent(uid, parent, index);
    }
}
impl<M> Delete for Space<M> {
    fn delete(&mut self, uid: &Uid) {
        self.transforms.remove(uid);
    }
}

#[cfg(feature = "2D")]
mod e2;
#[cfg(feature = "2D")]
pub use e2::*;

#[cfg(feature = "3D")]
mod e3;
#[cfg(feature = "3D")]
pub use e3::*;
