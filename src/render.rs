use std::sync::Arc;

pub type MeshHandle = Arc<wgpu::Buffer>;

#[cfg(feature = "2D")]
pub use two::*;
#[cfg(feature = "2D")]
mod two {
    pub type Mesh2D = wgpu::Buffer;
}