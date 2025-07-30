#[cfg(feature = "2D")]
pub type Transform2D = glam::Affine2;

#[cfg(feature = "3D")]
pub type Transform3D = glam::Affine3A;
