use color::{AlphaColor, Srgb};

use crate::sys::Components;

#[derive(Debug, Default)]
pub struct Colors {
    pub colors: Components<Color>
}

pub type Color = AlphaColor<Srgb>;
pub trait GpuColor {
    fn to_gpu(self) -> wgpu::Color;
}
impl GpuColor for Color {
    fn to_gpu(self) -> wgpu::Color {
        let components = self.components;
        wgpu::Color {
            r: components[0] as f64,
            g: components[1] as f64,
            b: components[2] as f64,
            a: components[3] as f64,
        }
    }
}