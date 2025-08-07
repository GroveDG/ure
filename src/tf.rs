#[cfg(feature = "2d")]
pub use two::*;
#[cfg(feature = "2d")]
mod two {
    pub type Transform2D = glam::Affine2;
}
#[cfg(feature = "3d")]
pub use three::*;
#[cfg(feature = "3d")]
mod three {
    pub type Transform3D = glam::Affine3A;
}