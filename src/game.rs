use crate::declare_components;

declare_components! {
    #[cfg(feature = "2d")]
    transform_2d: crate::tf::Transform2D,
    #[cfg(feature = "2d")]
    mesh: crate::render::MeshHandle,
    #[cfg(feature = "3d")]
    transform_3d: crate::tf::Transform3D,
}