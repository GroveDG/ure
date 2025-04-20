use super::Components;
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

#[cfg(feature = "2D")]
mod tf2d;
#[cfg(feature = "2D")]
pub use tf2d::*;

#[cfg(feature = "3D")]
mod tf3d;
#[cfg(feature = "3D")]
pub use tf3d::*;
