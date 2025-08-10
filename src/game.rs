use crate::declare_components;

declare_components! {
    #[cfg(feature = "2d")]
    color: crate::gpu::Color,
    #[cfg(feature = "2d")]
    transform_2d: glam::Affine2,
    #[cfg(feature = "2d")]
    visual_2d: crate::gpu::two::Instance2D,
    #[cfg(feature = "2d")]
    mesh: crate::gpu::two::MeshHandle2D,
    #[cfg(feature = "3d")]
    transform_3d: crate::tf::Transform3D,
}