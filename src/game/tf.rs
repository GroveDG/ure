use crate::sys::{Components, UID, delete::Delete};



pub type Precision = f32;

#[derive(Debug, Clone)]
pub struct Space<M> {
    transforms: Components<M>,
}
impl<M> Default for Space<M> {
    fn default() -> Self {
        Self {
            transforms: Default::default(),
        }
    }
}
impl<M> Space<M> {
    pub fn insert(&mut self, uid: UID, matrix: M) {
        self.transforms.insert(uid, matrix);
    }
}
impl<M> Delete for Space<M> {
    fn delete(&mut self, uid: &UID) {
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
