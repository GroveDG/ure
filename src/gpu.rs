use std::sync::OnceLock;

use ure_data::group::{Component, ComponentBox};
use wgpu::{
    Adapter, Device, DeviceDescriptor, Instance, InstanceDescriptor, Queue, RequestAdapterOptions,
    TextureFormat,
};

pub static GPU: std::sync::LazyLock<Gpu> =
    std::sync::LazyLock::new(|| futures::executor::block_on(Gpu::new()));
pub static SURFACE_FORMAT: OnceLock<TextureFormat> = OnceLock::new();

pub use color::Rgba8 as RgbaColor;

pub struct Color;
impl Component for Color {
    const IDENT: &'static str = "Color";

    type Container = Vec<RgbaColor>;
    type Dependencies = ();

    fn new(self) -> ure_data::group::ComponentBox {
        ComponentBox::new::<Color>(
            None,
            |c, range, d| {
                for i in range {
                    c.push(RgbaColor::from_u32(u32::MAX))
                }
            },
            |c, range| {
				for i in range.rev() {
					c.swap_remove(i);
				}
			},
        )
    }
}

pub struct Gpu {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}
impl Gpu {
    pub async fn new() -> Self {
        let instance = Instance::new(&InstanceDescriptor::from_env_or_default());
        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default())
            .await
            .unwrap();
        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }
}
