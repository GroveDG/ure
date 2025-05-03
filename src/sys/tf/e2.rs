use cgmath::Matrix3;

use crate::sys::tree::Compose;

use super::{Space, P};
pub type Matrix2D = Matrix3<P>;
pub type Space2D = Space<Matrix2D>;

impl Compose for Matrix2D {
    fn compose(self, parent: Self) -> Self {
        parent * self
    }
}