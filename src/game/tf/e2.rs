use cgmath::Matrix3;

use super::{Space, Precision};

pub type Matrix2D = Matrix3<Precision>;
pub type Space2D = Space<Matrix2D>;