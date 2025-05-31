use wgpu::{
    Adapter, Device, Instance, InstanceDescriptor, Queue, RequestAdapterOptions,
    wgt::DeviceDescriptor,
};

// [NOTE] https://www.reddit.com/r/opengl/comments/v5w80e/instancing_how_to_account_for_new_data_after/

pub type Pixels = f32;
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}
impl Color {
    pub const WHITE: Self = Color {
        r: 1.,
        g: 1.,
        b: 1.,
        a: 1.,
    };
    pub const BLUE: Self = Color {
        r: 0.,
        g: 0.,
        b: 1.,
        a: 1.,
    };
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
