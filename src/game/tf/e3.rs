use cgmath::Matrix3;

use super::{Space, P};

pub type Matrix3D = Matrix4<P>;
pub type Space3D = Space<Matrix3D>;