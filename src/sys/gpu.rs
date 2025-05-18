use wgpu::{
    wgt::DeviceDescriptor, Adapter, Color, Device, Instance, InstanceDescriptor, Queue, RequestAdapterOptions
};

pub mod render2d;

pub type Pixels = u16;

pub struct GPU {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

impl GPU {
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