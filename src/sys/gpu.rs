use wgpu::{
    wgt::DeviceDescriptor, Adapter, Color, Device, Instance, InstanceDescriptor, Queue, RequestAdapterOptions
};

pub mod render2d;

pub type Pixels = u16;

/// A queue-able abstract form of rendering.
/// 
/// Renderers receive these commands over threaded
/// channels. These commands are more abstract than
/// GPU render commands. Commands contain either
/// simple [Copy]-able structs or [UID][super::UID]s
/// to reference larger resources which may need to
/// be loaded in.
/// 
/// This allows a seperation of the nitty-gritty GPU
/// instructions and the developer's intentions and
/// systems. It also allows GPU communication to be
/// moved off thread without blocking or jeapordizing
/// frame-by-frame updates.
pub enum DrawCommand {
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
