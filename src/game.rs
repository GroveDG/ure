use crate::declare_components;

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

declare_components! {
    #[cfg(feature = "2d")]
    color: color::AlphaColor<color::Srgb>,
    #[cfg(feature = "2d")]
    transform_2d: Transform2D,
    #[cfg(feature = "2d")]
    visual_2d: crate::gpu::two::Instance2D,
    #[cfg(feature = "2d")]
    mesh: crate::gpu::two::MeshHandle2D,
    #[cfg(feature = "3d")]
    transform_3d: crate::tf::Transform3D,
}