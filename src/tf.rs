#[cfg(feature = "2D")]
pub use two::*;
#[cfg(feature = "2D")]
mod two {
    pub type Transform2D = glam::Affine2;
}
#[cfg(feature = "3D")]
pub use three::*;
#[cfg(feature = "3D")]
mod three {
    pub type Transform3D = glam::Affine3A;
}