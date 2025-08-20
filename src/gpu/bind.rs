use std::num::NonZero;

use wgpu::{BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, ShaderStages};

macro_rules! bind_group {
    ($($binding:literal $t:ty),+ $(,)?) => {
        wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[$(
                $crate::gpu::bind::layout::<$t>($binding)
            ),+]
        }
    };
}

pub trait Bind {
    const VISIBILITY: ShaderStages;
    const TYPE: BindingType;
    const COUNT: Option<NonZero<u32>> = None;
}

pub const fn layout<T: Bind>(binding: u32) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: T::VISIBILITY,
        ty: T::TYPE,
        count: T::COUNT,
    }
}
