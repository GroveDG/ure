use ordered_float::NotNan;
use wgpu::RenderPass;

use crate::store::Element;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct RenderPosition(NotNan<f32>);
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct RenderLayer(pub usize);

pub struct Rendering {
    pub layer: for<'a> fn(&'a Element) -> &'a [RenderLayer],
    pub render: fn(&Element, RenderLayer, &mut RenderPass),
}
