use cgmath::Matrix3;

use crate::sys::tree::Compose;

use super::{Space, P};
pub type Matrix3D = Matrix4<P>;
pub type Space3D = Space<Matrix3D>;

impl Compose for Matrix3D {
    fn compose(self, parent: Self) -> Self {
        parent * self
    }
}