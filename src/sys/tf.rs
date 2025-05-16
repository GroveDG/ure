use super::{delete::Delete, Components, UID};
use cgmath::SquareMatrix;

pub type Precision = f32;

#[derive(Debug, Clone)]
pub struct Space<M: SquareMatrix<Scalar = Precision>> {
    transforms: Components<M>,
}
impl<M: SquareMatrix<Scalar = Precision>> Default for Space<M> {
    fn default() -> Self {
        Self {
            transforms: Default::default(),
        }
    }
}
impl<M: SquareMatrix<Scalar = Precision>> Space<M> {
    pub fn insert(&mut self, uid: UID, matrix: M) {
        self.transforms.insert(uid, matrix);
    }
}
impl<M: SquareMatrix<Scalar = Precision>> Delete for Space<M> {
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
