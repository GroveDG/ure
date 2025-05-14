use wgpu::{
    wgt::DeviceDescriptor, Adapter, Color, Device, Instance, InstanceDescriptor, Queue, RequestAdapterOptions
};

pub mod surfaces;

pub enum RenderCommand {
    Clear(Color),
    Submit,
}

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
