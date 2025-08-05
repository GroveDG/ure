use std::mem::MaybeUninit;

use wgpu::{Device, DeviceDescriptor, Instance, InstanceDescriptor, Queue, RequestAdapterOptions};

pub type Surface = wgpu::Surface<'static>;

pub fn init_surfaces(
    windows: &[crate::app::Window],
    surfaces: &mut [MaybeUninit<Surface>],
    gpu: &Gpu,
) {
    for (w, s) in windows.iter().zip(surfaces.iter_mut()) {
        let surface = gpu.instance.create_surface(w.clone()).unwrap();
        s.write(surface);
    }
}

pub struct Gpu {
    pub instance: Instance,
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
            device,
            queue,
        }
    }
}
