use super::{Components, UID};
use cgmath::SquareMatrix;

type P = f32;

#[derive(Debug, Clone)]
pub struct Space<M: SquareMatrix<Scalar = P>> {
    transforms: Components<M>,
}
impl<M: SquareMatrix<Scalar = P>> Default for Space<M> {
    fn default() -> Self {
        Self {
            transforms: Default::default(),
        }
    }
}
impl<M: SquareMatrix<Scalar = P>> Space<M> {
    pub fn insert(&mut self, uid: UID, matrix: M) {
        self.transforms.insert(uid, matrix);
    }
    pub fn delete(&mut self, uid: &UID) {
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
