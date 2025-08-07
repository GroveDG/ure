use std::sync::Arc;

pub type MeshHandle = Arc<wgpu::Buffer>;

#[cfg(feature = "2d")]
pub use two::*;
#[cfg(feature = "2d")]
mod two {
    pub type Mesh2D = wgpu::Buffer;
}